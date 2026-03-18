use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const HQ_URL: &str = "http://fn0-hq.fn0.dev:8080";
const GITHUB_CLIENT_ID: &str = "Ov23liGi0QXBR8oEcBaI";

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
}

fn credentials_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| eyre!("Cannot find home directory"))?;
    Ok(home.join(".fn0").join("credentials"))
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
            Some(e) => return Err(eyre!("GitHub OAuth error: {}", e)),
            None => continue,
        }
    }
}

async fn get_github_token() -> Result<String> {
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

pub async fn execute(code_id: u64, code_version: u64) -> Result<()> {
    let github_token = get_github_token().await?;

    println!("Starting build...");
    crate::commands::build::execute().await?;

    let client = reqwest::Client::new();

    println!("Requesting deploy start...");
    let start_resp: DeployStartResponse = client
        .post(format!("{}/deploy/start", HQ_URL))
        .json(&serde_json::json!({ "github_token": github_token }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| eyre!("Deploy start failed: {}", e))?
        .json()
        .await?;

    println!("Uploading WASM...");
    let wasm_bytes = std::fs::read("dist/component.wasm")
        .map_err(|e| eyre!("Failed to read dist/component.wasm: {}", e))?;

    client
        .put(&start_resp.presigned_url)
        .body(wasm_bytes)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| eyre!("WASM upload failed: {}", e))?;

    println!("Requesting deploy finish...");
    client
        .post(format!("{}/deploy/finish", HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "deploy_job_id": start_resp.deploy_job_id,
            "code_id": code_id,
            "code_version": code_version,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| eyre!("Deploy finish failed: {}", e))?;

    println!("Deploy complete!");

    Ok(())
}
