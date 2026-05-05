use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod credentials;

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

pub async fn ensure_project_id(
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
        NewProjectRaw { ok: Some(ok), .. } => ok.project_id,
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
    build_id: &'a str,
    files: Vec<DeployFile>,
    jobs: &'a [CronJob],
    cron_updated_at: &'a str,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CronJob {
    pub function: String,
    pub every_minutes: u32,
}

#[derive(Serialize)]
struct DeployFile {
    path: String,
    size: u64,
}

#[derive(Deserialize)]
struct DeployRaw {
    #[serde(rename = "Ok")]
    ok: Option<DeployOk>,
    #[serde(rename = "QuotaExceeded")]
    quota_exceeded: Option<QuotaExceededBody>,
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
    static_uploads: Vec<StaticUpload>,
}

#[derive(Deserialize)]
struct StaticUpload {
    path: String,
    presigned_url: String,
}

#[derive(Deserialize)]
struct QuotaExceededBody {
    reason: String,
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

#[allow(clippy::too_many_arguments)]
pub async fn deploy_wasm(
    control_url: &str,
    token: &str,
    project_name: &str,
    project_id: &mut Option<String>,
    build_id: &str,
    bundle_tar_path: &Path,
    jobs: &[CronJob],
    cron_updated_at: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let project_id_resolved =
        ensure_project_id(&client, control_url, token, project_name, project_id).await?;
    println!("project_id: {project_id_resolved}");

    let DeployOk {
        presigned_put_url,
        object_key,
        static_uploads: _,
    } = request_deploy(
        &client,
        control_url,
        token,
        &project_id_resolved,
        build_id,
        Vec::new(),
        jobs,
        cron_updated_at,
    )
    .await?;

    println!("uploading bundle to {object_key}...");
    let last_modified = upload_bundle(&client, &presigned_put_url, bundle_tar_path).await?;
    println!("uploaded. last_modified={last_modified}");

    poll_deploy_status(
        &client,
        control_url,
        token,
        &project_id_resolved,
        &last_modified,
    )
    .await?;
    println!("Deploy complete!");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn deploy_forte(
    control_url: &str,
    token: &str,
    project_name: &str,
    project_id: &mut Option<String>,
    build_id: &str,
    fe_dist_dir: &Path,
    bundle_tar_path: &Path,
    jobs: &[CronJob],
    cron_updated_at: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let project_id_resolved =
        ensure_project_id(&client, control_url, token, project_name, project_id).await?;
    println!("project_id: {project_id_resolved}");

    let static_files = collect_static_files(fe_dist_dir)?;
    let deploy_files: Vec<DeployFile> = static_files
        .iter()
        .map(|f| DeployFile {
            path: f.relative_path.clone(),
            size: f.size,
        })
        .collect();
    println!(
        "Requesting deploy ({} static asset(s))...",
        deploy_files.len()
    );

    let DeployOk {
        presigned_put_url,
        object_key,
        static_uploads,
    } = request_deploy(
        &client,
        control_url,
        token,
        &project_id_resolved,
        build_id,
        deploy_files,
        jobs,
        cron_updated_at,
    )
    .await?;

    if !static_files.is_empty() {
        println!("Uploading {} static asset(s)...", static_files.len());
        upload_static_assets(&client, &static_files, static_uploads).await?;
    }

    println!("uploading bundle to {object_key}...");
    let last_modified = upload_bundle(&client, &presigned_put_url, bundle_tar_path).await?;
    println!("uploaded. last_modified={last_modified}");

    poll_deploy_status(
        &client,
        control_url,
        token,
        &project_id_resolved,
        &last_modified,
    )
    .await?;
    println!("Deploy complete!");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn request_deploy(
    client: &reqwest::Client,
    control_url: &str,
    token: &str,
    project_id: &str,
    build_id: &str,
    files: Vec<DeployFile>,
    jobs: &[CronJob],
    cron_updated_at: &str,
) -> Result<DeployOk> {
    let deploy_url = format!(
        "{}/__forte_action/deploy",
        control_url.trim_end_matches('/')
    );
    let raw: DeployRaw = client
        .post(&deploy_url)
        .bearer_auth(token)
        .json(&DeployInput {
            project_id,
            build_id,
            files,
            jobs,
            cron_updated_at,
        })
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("deploy failed: {e}"))?
        .json()
        .await?;
    match raw {
        DeployRaw { ok: Some(ok), .. } => Ok(ok),
        DeployRaw {
            quota_exceeded: Some(q),
            ..
        } => Err(anyhow!("deploy quota exceeded: {}", q.reason)),
        DeployRaw {
            not_logged_in: Some(_),
            ..
        } => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DeployRaw {
            not_found: Some(_), ..
        } => Err(anyhow!("project not found")),
        DeployRaw {
            forbidden: Some(_), ..
        } => Err(anyhow!("not the owner of this project")),
        DeployRaw {
            error: Some(err), ..
        } => Err(anyhow!("deploy: {}", err.message)),
        _ => Err(anyhow!("unexpected deploy response")),
    }
}

async fn upload_bundle(
    client: &reqwest::Client,
    presigned_put_url: &str,
    bundle_tar_path: &Path,
) -> Result<String> {
    let bundle_bytes = std::fs::read(bundle_tar_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", bundle_tar_path.display(), e))?;
    let put_resp = client
        .put(presigned_put_url)
        .body(bundle_bytes)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("bundle upload failed: {e}"))?;
    extract_last_modified(&put_resp)
}

async fn upload_static_assets(
    client: &reqwest::Client,
    files: &[StaticFile],
    uploads: Vec<StaticUpload>,
) -> Result<()> {
    use futures::StreamExt;
    use std::collections::HashMap;

    let mut url_for_path: HashMap<String, String> = HashMap::new();
    for u in uploads {
        url_for_path.insert(u.path, u.presigned_url);
    }

    let mut tasks = futures::stream::FuturesUnordered::new();
    for file in files {
        let url = url_for_path.remove(&file.relative_path).ok_or_else(|| {
            anyhow!(
                "control did not return presigned URL for {}",
                file.relative_path
            )
        })?;
        let bytes = std::fs::read(&file.absolute_path)
            .map_err(|e| anyhow!("read {}: {}", file.absolute_path.display(), e))?;
        let client = client.clone();
        let content_type = file.content_type;
        let path = file.relative_path.clone();
        tasks.push(async move {
            let resp = client
                .put(&url)
                .header("content-type", content_type)
                .body(bytes)
                .send()
                .await
                .map_err(|e| anyhow!("R2 PUT {}: {}", path, e))?;
            resp.error_for_status()
                .map_err(|e| anyhow!("R2 PUT {} HTTP error: {}", path, e))?;
            Ok::<_, anyhow::Error>(())
        });
    }
    while let Some(result) = tasks.next().await {
        result?;
    }
    Ok(())
}

pub struct StaticFile {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub size: u64,
    pub content_type: &'static str,
}

pub fn collect_static_files(dir: &Path) -> Result<Vec<StaticFile>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    walk_collect(dir, dir, &mut out)?;
    Ok(out)
}

fn walk_collect(base: &Path, dir: &Path, out: &mut Vec<StaticFile>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|s| s.to_str()) == Some("ssr")
                && path.parent() == Some(base)
            {
                continue;
            }
            walk_collect(base, &path, out)?;
            continue;
        }
        let metadata = entry.metadata()?;
        let rel = path
            .strip_prefix(base)
            .map_err(|e| anyhow!("strip_prefix: {e}"))?
            .to_string_lossy()
            .replace('\\', "/");
        out.push(StaticFile {
            relative_path: rel,
            absolute_path: path.clone(),
            size: metadata.len(),
            content_type: content_type_for(&path),
        });
    }
    Ok(())
}

pub fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") | Some("cjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("pdf") => "application/pdf",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        _ => "application/octet-stream",
    }
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
    Err(anyhow!("domain commands are not yet migrated to control."))
}

pub async fn domain_remove(_project_name: &str) -> Result<()> {
    Err(anyhow!("domain commands are not yet migrated to control."))
}

pub async fn domain_status(_project_name: &str) -> Result<()> {
    Err(anyhow!("domain commands are not yet migrated to control."))
}
