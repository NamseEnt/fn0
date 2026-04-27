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

#[derive(Deserialize)]
struct DeployFinishResponse {
    job_id: String,
}

#[derive(Deserialize)]
struct DeployStatusResponse {
    delivered: bool,
    hosts_total: usize,
    hosts_at_target: usize,
    hosts_pending: Vec<String>,
    hosts_quarantined: Vec<String>,
    #[serde(default)]
    job: Option<DeployJobStatus>,
}

#[derive(Deserialize)]
struct DeployJobStatus {
    phase: String,
    #[serde(default)]
    #[allow(dead_code)]
    generation: Option<u64>,
    #[serde(default)]
    last_error: Option<String>,
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
        .form(&[("client_id", GITHUB_CLIENT_ID), ("scope", "read:user")])
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

pub async fn deploy(
    project_name: &str,
    bundle_tar_path: &Path,
    env_content: Option<String>,
) -> Result<()> {
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
    let finish_resp: DeployFinishResponse = client
        .post(format!("{}/deploy/finish", HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "deploy_job_id": start_resp.deploy_job_id,
            "subdomain": start_resp.subdomain,
            "code_id": start_resp.code_id,
            "env": env_content,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Deploy finish failed: {}", e))?
        .json()
        .await?;

    println!("Deploy job queued: {}", finish_resp.job_id);

    let poll_interval = std::time::Duration::from_secs(2);
    let timeout = std::time::Duration::from_secs(600);
    let start = std::time::Instant::now();
    let mut last_phase: Option<String> = None;
    let mut last_progress: Option<(usize, usize)> = None;

    loop {
        let status: DeployStatusResponse = client
            .get(format!(
                "{}/deploy/status?job_id={}",
                HQ_URL, finish_resp.job_id
            ))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow!("Deploy status failed: {}", e))?
            .json()
            .await?;

        if let Some(job) = status.job.as_ref() {
            if last_phase.as_deref() != Some(job.phase.as_str()) {
                println!("  phase: {}", job.phase);
                last_phase = Some(job.phase.clone());
            }
            if job.phase == "failed" {
                let msg = job
                    .last_error
                    .clone()
                    .unwrap_or_else(|| "unknown error".to_string());
                return Err(anyhow!("Deploy job failed: {}", msg));
            }
        }

        let progress = (status.hosts_at_target, status.hosts_total);
        if last_progress != Some(progress) {
            println!("  {}/{} hosts ready", progress.0, progress.1);
            last_progress = Some(progress);
        }

        let phase_done = status
            .job
            .as_ref()
            .map(|j| j.phase == "done")
            .unwrap_or(false);
        if phase_done && status.delivered {
            break;
        }

        if start.elapsed() > timeout {
            return Err(anyhow!(
                "Deploy timed out after {}s. phase={:?} pending={:?} quarantined={:?}",
                timeout.as_secs(),
                status.job.as_ref().map(|j| j.phase.clone()),
                status.hosts_pending,
                status.hosts_quarantined
            ));
        }

        tokio::time::sleep(poll_interval).await;
    }

    println!("Deploy complete!");

    Ok(())
}

#[derive(Deserialize)]
struct AdminGrantResponse {
    token: String,
    subdomain: String,
    #[allow(dead_code)]
    expires_at: i64,
}

pub struct AdminRunOutput {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

pub async fn admin_run(
    project_name: &str,
    task: &str,
    input_body: Vec<u8>,
    timeout_secs: u64,
) -> Result<AdminRunOutput> {
    let github_token = get_github_token().await?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()?;

    let grant: AdminGrantResponse = client
        .post(format!("{}/admin/grant", HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "project_name": project_name,
            "task": task,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Admin grant request failed: {}", e))?
        .json()
        .await?;

    let url = format!("https://{}.fn0.dev/__forte_admin/{}", grant.subdomain, task);
    let resp = client
        .post(&url)
        .header("Authorization", format!("FortoAdmin {}", grant.token))
        .header("Content-Type", "application/json")
        .body(input_body)
        .send()
        .await?;

    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = resp.bytes().await?.to_vec();

    Ok(AdminRunOutput {
        status,
        content_type,
        body,
    })
}

pub fn read_env_content(project_dir: &Path) -> Result<Option<String>> {
    let env_path = project_dir.join(".env");
    match std::fs::read_to_string(&env_path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow!("Failed to read {}: {}", env_path.display(), e)),
    }
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

    let manifest = br#"{"kind":"wasmjs"}"#;
    append_bytes(&mut builder, "manifest.json", manifest)?;

    let backend_wasm = dist_dir.join("backend.wasm");
    let wasm_bytes = std::fs::read(&backend_wasm)
        .map_err(|e| anyhow!("Failed to read {}: {}", backend_wasm.display(), e))?;
    append_bytes(&mut builder, "backend.wasm", &wasm_bytes)?;

    let server_js = dist_dir.join("server.js");
    let server_bytes = std::fs::read(&server_js)
        .map_err(|e| anyhow!("Failed to read {}: {}", server_js.display(), e))?;
    append_bytes(&mut builder, "entry.js", &server_bytes)?;

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
