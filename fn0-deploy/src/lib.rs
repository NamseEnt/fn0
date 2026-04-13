use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const HQ_URL: &str = "http://fn0-hq.fn0.dev:8080";
const GITHUB_CLIENT_ID: &str = "Ov23liRuIJf1NSe9ccP8";

#[derive(Serialize, Deserialize)]
struct Credentials {
    github_token: String,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct DeployStartResponse {
    presigned_url: String,
    deploy_job_id: String,
    subdomain: String,
    code_id: u64,
}

fn credentials_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow!("Cannot find HOME directory"))?;
    Ok(PathBuf::from(home).join(".fn0").join("credentials"))
}

fn load_credentials() -> Result<Option<Credentials>> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let creds: Credentials = serde_json::from_str(&content)?;
    Ok(Some(creds))
}

fn save_credentials(creds: &Credentials) -> Result<()> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(creds)?)?;
    Ok(())
}

async fn github_device_flow() -> Result<String> {
    let client = reqwest::Client::new();

    let resp: DeviceCodeResponse = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", GITHUB_CLIENT_ID),
            ("scope", "read:user"),
        ])
        .send()
        .await?
        .json()
        .await?;

    println!("\nGitHub authentication required.");
    println!("Open {} in your browser", resp.verification_uri);
    println!("and enter the code: {}\n", resp.user_code);

    let interval = std::time::Duration::from_secs(resp.interval.max(5));

    loop {
        tokio::time::sleep(interval).await;

        let token_resp: TokenResponse = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", GITHUB_CLIENT_ID),
                ("device_code", resp.device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?
            .json()
            .await?;

        if let Some(token) = token_resp.access_token {
            return Ok(token);
        }

        match token_resp.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            Some(e) => return Err(anyhow!("GitHub OAuth error: {}", e)),
            None => continue,
        }
    }
}

pub async fn get_github_token() -> Result<String> {
    if let Some(creds) = load_credentials()? {
        return Ok(creds.github_token);
    }

    let token = github_device_flow().await?;
    save_credentials(&Credentials {
        github_token: token.clone(),
    })?;
    println!("Authentication complete! Token saved.\n");

    Ok(token)
}

pub async fn deploy(project_name: &str, bundle_tar_path: &Path) -> Result<()> {
    let github_token = get_github_token().await?;

    let client = reqwest::Client::new();

    println!("Requesting deploy start...");
    let start_resp: DeployStartResponse = client
        .post(format!("{}/deploy/start", HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "project_name": project_name,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Deploy start failed: {}", e))?
        .json()
        .await?;

    println!("Subdomain: {}.fn0.dev", start_resp.subdomain);

    println!("Uploading bundle...");
    let bundle_bytes = std::fs::read(bundle_tar_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", bundle_tar_path.display(), e))?;

    client
        .put(&start_resp.presigned_url)
        .header("content-type", "application/x-tar")
        .body(bundle_bytes)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Bundle upload failed: {}", e))?;

    println!("Requesting deploy finish...");
    client
        .post(format!("{}/deploy/finish", HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "deploy_job_id": start_resp.deploy_job_id,
            "subdomain": start_resp.subdomain,
            "code_id": start_resp.code_id,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Deploy finish failed: {}", e))?;

    println!("Deploy complete!");

    Ok(())
}

pub fn create_raw_bundle_wasm(wasm_path: &Path, output_path: &Path) -> Result<()> {
    let file = std::fs::File::create(output_path)
        .map_err(|e| anyhow!("Failed to create {}: {}", output_path.display(), e))?;
    let mut builder = tar::Builder::new(file);

    let manifest = br#"{"kind":"wasm"}"#;
    append_bytes(&mut builder, "manifest.json", manifest)?;

    let wasm_bytes = std::fs::read(wasm_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", wasm_path.display(), e))?;
    append_bytes(&mut builder, "backend.wasm", &wasm_bytes)?;

    builder.finish()?;
    Ok(())
}

pub fn create_raw_bundle_forte(dist_dir: &Path, output_path: &Path) -> Result<()> {
    let file = std::fs::File::create(output_path)
        .map_err(|e| anyhow!("Failed to create {}: {}", output_path.display(), e))?;
    let mut builder = tar::Builder::new(file);

    let manifest = br#"{"kind":"forte","frontend_script_path":"/frontend.js"}"#;
    append_bytes(&mut builder, "manifest.json", manifest)?;

    let backend_wasm = dist_dir.join("backend.wasm");
    let wasm_bytes = std::fs::read(&backend_wasm)
        .map_err(|e| anyhow!("Failed to read {}: {}", backend_wasm.display(), e))?;
    append_bytes(&mut builder, "backend.wasm", &wasm_bytes)?;

    let server_js = dist_dir.join("server.js");
    let server_bytes = std::fs::read(&server_js)
        .map_err(|e| anyhow!("Failed to read {}: {}", server_js.display(), e))?;
    append_bytes(&mut builder, "frontend.js", &server_bytes)?;

    let public_dir = dist_dir.join("public");
    if public_dir.exists() {
        for entry in walkdir::WalkDir::new(&public_dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(&public_dir)
                .map_err(|e| anyhow!("strip_prefix failed: {}", e))?;
            let tar_path = format!("public/{}", rel.to_string_lossy().replace('\\', "/"));
            let bytes = std::fs::read(entry.path())
                .map_err(|e| anyhow!("Failed to read {}: {}", entry.path().display(), e))?;
            append_bytes(&mut builder, &tar_path, &bytes)?;
        }
    }

    builder.finish()?;
    Ok(())
}

fn append_bytes<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    path: &str,
    data: &[u8],
) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, data)
        .map_err(|e| anyhow!("tar append failed for {}: {}", path, e))?;
    Ok(())
}
