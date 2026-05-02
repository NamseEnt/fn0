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

#[derive(Deserialize)]
struct RenameStartResponse {
    job_id: String,
    #[serde(default)]
    already_running: bool,
}

#[derive(Deserialize)]
struct RenameStatusResponse {
    job_id: String,
    phase: String,
    attempts: u32,
    last_error: Option<String>,
    is_terminal: bool,
}

pub async fn rename(project_name: &str, new_project_name: &str) -> Result<()> {
    let github_token = get_github_token().await?;

    let client = reqwest::Client::new();

    println!("Requesting rename start...");
    let start_resp: RenameStartResponse = client
        .post(format!("{}/rename/start", HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "project_name": project_name,
            "new_project_name": new_project_name,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Rename start failed: {}", e))?
        .json()
        .await?;

    if start_resp.already_running {
        println!("Following existing rename job: {}", start_resp.job_id);
    } else {
        println!("Rename job queued: {}", start_resp.job_id);
    }

    let poll_interval = std::time::Duration::from_secs(2);
    let timeout = std::time::Duration::from_secs(900);
    let start = std::time::Instant::now();
    let mut last_phase: Option<String> = None;

    loop {
        let status: RenameStatusResponse = client
            .get(format!(
                "{}/rename/status?job_id={}",
                HQ_URL, start_resp.job_id
            ))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow!("Rename status failed: {}", e))?
            .json()
            .await?;

        if last_phase.as_deref() != Some(status.phase.as_str()) {
            println!("  phase: {} (attempts={})", status.phase, status.attempts);
            last_phase = Some(status.phase.clone());
        }

        if status.phase == "failed" {
            let msg = status
                .last_error
                .unwrap_or_else(|| "unknown error".to_string());
            return Err(anyhow!("Rename job failed: {}", msg));
        }

        if status.is_terminal && status.phase == "done" {
            break;
        }

        if start.elapsed() > timeout {
            return Err(anyhow!(
                "Rename timed out after {}s. phase={:?}",
                timeout.as_secs(),
                status.phase
            ));
        }

        let _ = status.job_id;
        tokio::time::sleep(poll_interval).await;
    }

    println!("Rename complete!");
    Ok(())
}

#[derive(Deserialize)]
struct DomainAddResponse {
    job_id: String,
    already_running: bool,
    subdomain: String,
    domain: String,
}

#[derive(Deserialize)]
struct DomainRemoveResponse {
    job_id: String,
    already_running: bool,
}

#[derive(Deserialize)]
struct DomainStatusResponse {
    subdomain: String,
    current_domain: Option<String>,
    add_job: Option<DomainJobStatus>,
    remove_job: Option<DomainJobStatus>,
}

#[derive(Deserialize, Clone)]
struct DomainJobStatus {
    job_id: String,
    phase: String,
    domain: String,
    attempts: u32,
    last_error: Option<String>,
    is_terminal: bool,
}

pub async fn domain_add(project_name: &str, domain: &str) -> Result<()> {
    let github_token = get_github_token().await?;
    let client = reqwest::Client::new();

    println!("Requesting domain add...");
    let resp: DomainAddResponse = client
        .post(format!("{HQ_URL}/domain/add"))
        .json(&serde_json::json!({
            "github_token": github_token,
            "project_name": project_name,
            "domain": domain,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("domain add failed: {e}"))?
        .json()
        .await?;

    if resp.already_running {
        println!("Following existing add job: {}", resp.job_id);
    } else {
        println!("Add job queued: {}", resp.job_id);
    }
    println!(
        "\nPoint a CNAME for {} to: {}.fn0.dev",
        resp.domain, resp.subdomain
    );
    println!("(Cloudflare for SaaS will validate and issue a TLS cert.)\n");

    poll_domain_until(project_name, &github_token, |s| {
        s.add_job.as_ref().map(|j| j.is_terminal).unwrap_or(false)
            && s.current_domain.as_deref() == Some(resp.domain.as_str())
    })
    .await
}

pub async fn domain_remove(project_name: &str) -> Result<()> {
    let github_token = get_github_token().await?;
    let client = reqwest::Client::new();

    println!("Requesting domain remove...");
    let resp: DomainRemoveResponse = client
        .post(format!("{HQ_URL}/domain/remove"))
        .json(&serde_json::json!({
            "github_token": github_token,
            "project_name": project_name,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("domain remove failed: {e}"))?
        .json()
        .await?;

    if resp.already_running {
        println!("Following existing remove job: {}", resp.job_id);
    } else {
        println!("Remove job queued: {}", resp.job_id);
    }

    poll_domain_until(project_name, &github_token, |s| {
        s.remove_job.as_ref().map(|j| j.is_terminal).unwrap_or(false)
            && s.current_domain.is_none()
    })
    .await
}

pub async fn domain_status(project_name: &str) -> Result<()> {
    let github_token = get_github_token().await?;
    let client = reqwest::Client::new();

    let url = format!(
        "{HQ_URL}/domain/status?github_token={}&project_name={}",
        urlencoding::encode(&github_token),
        urlencoding::encode(project_name)
    );
    let status: DomainStatusResponse = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("domain status failed: {e}"))?
        .json()
        .await?;

    println!("Subdomain: {}.fn0.dev", status.subdomain);
    match &status.current_domain {
        Some(d) => println!("Current domain: {d}"),
        None => println!("Current domain: (none)"),
    }
    if let Some(j) = &status.add_job {
        println!(
            "Add job: id={} phase={} domain={} attempts={} terminal={}",
            j.job_id, j.phase, j.domain, j.attempts, j.is_terminal
        );
        if let Some(e) = &j.last_error {
            println!("  last_error: {e}");
        }
    }
    if let Some(j) = &status.remove_job {
        println!(
            "Remove job: id={} phase={} domain={} attempts={} terminal={}",
            j.job_id, j.phase, j.domain, j.attempts, j.is_terminal
        );
        if let Some(e) = &j.last_error {
            println!("  last_error: {e}");
        }
    }
    Ok(())
}

async fn poll_domain_until<F>(project_name: &str, github_token: &str, done: F) -> Result<()>
where
    F: Fn(&DomainStatusResponse) -> bool,
{
    let client = reqwest::Client::new();
    let poll_interval = std::time::Duration::from_secs(3);
    let timeout = std::time::Duration::from_secs(900);
    let start = std::time::Instant::now();
    let mut last_phase: Option<String> = None;

    loop {
        let url = format!(
            "{HQ_URL}/domain/status?github_token={}&project_name={}",
            urlencoding::encode(github_token),
            urlencoding::encode(project_name)
        );
        let status: DomainStatusResponse = client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow!("domain status failed: {e}"))?
            .json()
            .await?;

        let active_phase = status
            .add_job
            .as_ref()
            .map(|j| j.phase.clone())
            .or_else(|| status.remove_job.as_ref().map(|j| j.phase.clone()));
        if last_phase != active_phase
            && let Some(p) = active_phase.clone()
        {
            println!("  phase: {p}");
            last_phase = active_phase;
        }

        if let Some(j) = status.add_job.as_ref()
            && j.phase == "failed"
        {
            let msg = j
                .last_error
                .clone()
                .unwrap_or_else(|| "unknown error".to_string());
            return Err(anyhow!("Add job failed: {msg}"));
        }
        if let Some(j) = status.remove_job.as_ref()
            && j.phase == "failed"
        {
            let msg = j
                .last_error
                .clone()
                .unwrap_or_else(|| "unknown error".to_string());
            return Err(anyhow!("Remove job failed: {msg}"));
        }

        if done(&status) {
            println!("Done!");
            return Ok(());
        }

        if start.elapsed() > timeout {
            return Err(anyhow!(
                "Domain operation timed out after {}s",
                timeout.as_secs()
            ));
        }
        tokio::time::sleep(poll_interval).await;
    }
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
