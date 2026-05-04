use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize)]
struct NewProjectInput<'a> {
    name: &'a str,
}

#[derive(Deserialize)]
struct NewProjectRaw {
    #[serde(rename = "Ok")]
    ok: Option<NewProjectOk>,
    #[serde(rename = "NotLoggedIn")]
    not_logged_in: Option<()>,
    #[serde(rename = "Error")]
    error: Option<MessageErr>,
}

#[derive(Deserialize)]
struct NewProjectOk {
    project_id: String,
}

#[derive(Deserialize)]
struct MessageErr {
    message: String,
}

async fn ensure_project_id(
    client: &reqwest::Client,
    control_url: &str,
    token: &str,
    project_name: &str,
    project_id: &mut Option<String>,
) -> Result<String> {
    if let Some(id) = project_id.as_ref() {
        return Ok(id.clone());
    }
    let url = format!(
        "{}/__forte_action/new_project",
        control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(token)
        .json(&NewProjectInput { name: project_name })
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("new_project failed: {e}"))?;
    let raw: NewProjectRaw = resp.json().await?;
    let id = match raw {
        NewProjectRaw {
            ok: Some(ok), ..
        } => ok.project_id,
        NewProjectRaw {
            not_logged_in: Some(_),
            ..
        } => return Err(anyhow!("control rejected token; run `fn0 login` again.")),
        NewProjectRaw {
            error: Some(err), ..
        } => return Err(anyhow!("new_project: {}", err.message)),
        _ => return Err(anyhow!("unexpected new_project response")),
    };
    *project_id = Some(id.clone());
    Ok(id)
}

#[derive(Serialize)]
struct DeployInput<'a> {
    project_id: &'a str,
}

#[derive(Deserialize)]
struct DeployRaw {
    #[serde(rename = "Ok")]
    ok: Option<DeployOk>,
    #[serde(rename = "NotLoggedIn")]
    not_logged_in: Option<()>,
    #[serde(rename = "NotFound")]
    not_found: Option<()>,
    #[serde(rename = "Forbidden")]
    forbidden: Option<()>,
    #[serde(rename = "Error")]
    error: Option<MessageErr>,
}

#[derive(Deserialize)]
struct DeployOk {
    presigned_put_url: String,
    object_key: String,
}

#[derive(Serialize)]
struct DeployStatusInput<'a> {
    project_id: &'a str,
    last_modified: &'a str,
}

#[derive(Deserialize)]
struct DeployStatusRaw {
    #[serde(rename = "Done")]
    done: Option<DeployStatusBody>,
    #[serde(rename = "Pending")]
    pending: Option<DeployStatusBody>,
    #[serde(rename = "NoActiveVersion")]
    no_active_version: Option<()>,
    #[serde(rename = "NotLoggedIn")]
    not_logged_in: Option<()>,
    #[serde(rename = "NotFound")]
    not_found: Option<()>,
    #[serde(rename = "Forbidden")]
    forbidden: Option<()>,
    #[serde(rename = "Error")]
    error: Option<MessageErr>,
}

#[derive(Deserialize)]
struct DeployStatusBody {
    active_version: String,
    pending_version: Option<String>,
    pending_compiled: bool,
    compiled_versions: Vec<String>,
}

pub async fn deploy(
    control_url: &str,
    token: &str,
    project_name: &str,
    project_id: &mut Option<String>,
    bundle_tar_path: &Path,
) -> Result<()> {
    let client = reqwest::Client::new();

    let project_id_resolved =
        ensure_project_id(&client, control_url, token, project_name, project_id).await?;
    println!("project_id: {project_id_resolved}");

    println!("Requesting deploy...");
    let deploy_url = format!(
        "{}/__forte_action/deploy",
        control_url.trim_end_matches('/')
    );
    let raw: DeployRaw = client
        .post(&deploy_url)
        .bearer_auth(token)
        .json(&DeployInput {
            project_id: &project_id_resolved,
        })
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("deploy failed: {e}"))?
        .json()
        .await?;
    let DeployOk {
        presigned_put_url,
        object_key,
    } = match raw {
        DeployRaw { ok: Some(ok), .. } => ok,
        DeployRaw {
            not_logged_in: Some(_),
            ..
        } => return Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DeployRaw {
            not_found: Some(_), ..
        } => return Err(anyhow!("project not found")),
        DeployRaw {
            forbidden: Some(_), ..
        } => return Err(anyhow!("not the owner of this project")),
        DeployRaw {
            error: Some(err), ..
        } => return Err(anyhow!("deploy: {}", err.message)),
        _ => return Err(anyhow!("unexpected deploy response")),
    };
    println!("uploading bundle to {object_key}...");

    let bundle_bytes = std::fs::read(bundle_tar_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", bundle_tar_path.display(), e))?;

    let put_resp = client
        .put(&presigned_put_url)
        .body(bundle_bytes)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("bundle upload failed: {e}"))?;
    let last_modified = extract_last_modified(&put_resp)?;
    println!("uploaded. last_modified={last_modified}");

    poll_deploy_status(&client, control_url, token, &project_id_resolved, &last_modified).await?;

    println!("Deploy complete!");
    Ok(())
}

fn extract_last_modified(resp: &reqwest::Response) -> Result<String> {
    let hv = resp
        .headers()
        .get(reqwest::header::LAST_MODIFIED)
        .ok_or_else(|| anyhow!("R2 PUT response missing Last-Modified header"))?
        .to_str()
        .map_err(|e| anyhow!("Last-Modified not utf-8: {e}"))?;
    let dt = chrono::DateTime::parse_from_rfc2822(hv)
        .map_err(|e| anyhow!("Last-Modified parse: {e}; raw={hv}"))?;
    Ok(dt
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string())
}

async fn poll_deploy_status(
    client: &reqwest::Client,
    control_url: &str,
    token: &str,
    project_id: &str,
    last_modified: &str,
) -> Result<()> {
    let url = format!(
        "{}/__forte_action/deploy_status",
        control_url.trim_end_matches('/')
    );
    let timeout = std::time::Duration::from_secs(600);
    let start = std::time::Instant::now();
    let mut last_state: Option<String> = None;

    loop {
        let raw: DeployStatusRaw = client
            .post(&url)
            .bearer_auth(token)
            .json(&DeployStatusInput {
                project_id,
                last_modified,
            })
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow!("deploy_status failed: {e}"))?
            .json()
            .await?;

        if let Some(body) = raw.done {
            log_status_line(&body, &mut last_state);
            return Ok(());
        }
        if raw.no_active_version.is_some() {
            return Err(anyhow!("control has no active fn0-wasmtime version yet"));
        }
        if raw.not_logged_in.is_some() {
            return Err(anyhow!("control rejected token; run `fn0 login` again."));
        }
        if raw.not_found.is_some() {
            return Err(anyhow!("project not found"));
        }
        if raw.forbidden.is_some() {
            return Err(anyhow!("not the owner of this project"));
        }
        if let Some(err) = raw.error {
            return Err(anyhow!("deploy_status: {}", err.message));
        }
        if let Some(body) = raw.pending {
            log_status_line(&body, &mut last_state);
            if start.elapsed() > timeout {
                return Err(anyhow!(
                    "deploy_status timed out after {}s",
                    timeout.as_secs()
                ));
            }
            continue;
        }
        return Err(anyhow!("unexpected deploy_status response"));
    }
}

fn log_status_line(body: &DeployStatusBody, last_state: &mut Option<String>) {
    let state = format!(
        "active={} compiled={:?} pending={:?} pending_compiled={}",
        body.active_version, body.compiled_versions, body.pending_version, body.pending_compiled,
    );
    if last_state.as_deref() != Some(&state) {
        println!("  {state}");
        *last_state = Some(state);
    }
}

pub fn read_env_yaml(project_dir: &Path) -> Result<Option<Vec<u8>>> {
    let p = project_dir.join("env.yaml");
    match std::fs::read(&p) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow!("Failed to read {}: {}", p.display(), e)),
    }
}

pub fn create_raw_bundle_wasm(
    wasm_path: &Path,
    env_yaml: Option<&[u8]>,
    output_path: &Path,
) -> Result<()> {
    let file = std::fs::File::create(output_path)
        .map_err(|e| anyhow!("Failed to create {}: {}", output_path.display(), e))?;
    let mut builder = tar::Builder::new(file);
    append_bytes(&mut builder, "manifest.json", br#"{"kind":"wasm"}"#)?;
    let wasm_bytes = std::fs::read(wasm_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", wasm_path.display(), e))?;
    append_bytes(&mut builder, "backend.wasm", &wasm_bytes)?;
    if let Some(env) = env_yaml {
        append_bytes(&mut builder, "env.yaml", env)?;
    }
    builder.finish()?;
    Ok(())
}

pub fn create_raw_bundle_forte(
    dist_dir: &Path,
    env_yaml: Option<&[u8]>,
    output_path: &Path,
) -> Result<()> {
    let file = std::fs::File::create(output_path)
        .map_err(|e| anyhow!("Failed to create {}: {}", output_path.display(), e))?;
    let mut builder = tar::Builder::new(file);
    append_bytes(&mut builder, "manifest.json", br#"{"kind":"wasmjs"}"#)?;

    let backend_wasm = dist_dir.join("backend.wasm");
    let wasm_bytes = std::fs::read(&backend_wasm)
        .map_err(|e| anyhow!("Failed to read {}: {}", backend_wasm.display(), e))?;
    append_bytes(&mut builder, "backend.wasm", &wasm_bytes)?;

    let server_js = dist_dir.join("server.js");
    let server_bytes = std::fs::read(&server_js)
        .map_err(|e| anyhow!("Failed to read {}: {}", server_js.display(), e))?;
    append_bytes(&mut builder, "entry.js", &server_bytes)?;

    if let Some(env) = env_yaml {
        append_bytes(&mut builder, "env.yaml", env)?;
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

pub struct AdminRunOutput {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

pub async fn admin_run(
    _project_name: &str,
    _task: &str,
    _input_body: Vec<u8>,
    _timeout_secs: u64,
) -> Result<AdminRunOutput> {
    Err(anyhow!(
        "admin run is not yet migrated to control. See GitHub issue #4."
    ))
}

pub async fn rename(_project_name: &str, _new_project_name: &str) -> Result<()> {
    Err(anyhow!(
        "rename is not yet migrated to control. See GitHub issue #5."
    ))
}

pub async fn domain_add(_project_name: &str, _domain: &str) -> Result<()> {
    Err(anyhow!(
        "domain commands are not yet migrated to control."
    ))
}

pub async fn domain_remove(_project_name: &str) -> Result<()> {
    Err(anyhow!(
        "domain commands are not yet migrated to control."
    ))
}

pub async fn domain_status(_project_name: &str) -> Result<()> {
    Err(anyhow!(
        "domain commands are not yet migrated to control."
    ))
}

pub fn read_env_content(_project_dir: &Path) -> Result<Option<String>> {
    Err(anyhow!(
        "read_env_content has been replaced by read_env_yaml; env.yaml is now bundled directly."
    ))
}
