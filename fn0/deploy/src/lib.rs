use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod credentials;

pub const MAX_PROJECT_NAME_LEN: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong { len: usize },
    InvalidChar { ch: char },
}

impl std::fmt::Display for NameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameError::Empty => write!(f, "name cannot be empty"),
            NameError::TooLong { len } => write!(
                f,
                "name too long: {len} chars (max {MAX_PROJECT_NAME_LEN})"
            ),
            NameError::InvalidChar { ch } => write!(
                f,
                "name contains invalid character {ch:?}; allowed: letters, digits, '.', '_', '-'"
            ),
        }
    }
}

impl std::error::Error for NameError {}

pub fn validate_project_name(s: &str) -> std::result::Result<(), NameError> {
    let len = s.chars().count();
    if len == 0 {
        return Err(NameError::Empty);
    }
    if len > MAX_PROJECT_NAME_LEN {
        return Err(NameError::TooLong { len });
    }
    if let Some(ch) = s
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '.' && *c != '_' && *c != '-')
    {
        return Err(NameError::InvalidChar { ch });
    }
    Ok(())
}

pub fn is_valid_project_name(s: &str) -> bool {
    validate_project_name(s).is_ok()
}

#[derive(Serialize)]
struct NewProjectInput<'a> {
    name: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum NewProject {
    Ok { project_id: String },
    NotLoggedIn,
    InvalidName,
    InternalError,
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
    let raw: NewProject = resp.json().await?;
    let id = match raw {
        NewProject::Ok { project_id } => project_id,
        NewProject::NotLoggedIn => {
            return Err(anyhow!("control rejected token; run `fn0 login` again."));
        }
        NewProject::InvalidName => {
            return Err(anyhow!(
                "control rejected project name '{project_name}': must be 1-{MAX_PROJECT_NAME_LEN} chars of letters, digits, '.', '_', '-'"
            ));
        }
        NewProject::InternalError => {
            return Err(anyhow!(
                "new_project: server error; check fn0-control logs"
            ));
        }
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
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum Deploy {
    Ok {
        presigned_put_url: String,
        object_key: String,
        static_uploads: Vec<StaticUpload>,
        code_version: u64,
    },
    QuotaExceeded {
        reason: String,
    },
    NotLoggedIn,
    NotFound,
    InternalError,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StaticUpload {
    path: String,
    presigned_url: String,
}

#[derive(Serialize)]
struct DeployStatusInput<'a> {
    project_id: &'a str,
    code_version: u64,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DeployStatus {
    Done {
        active_version: String,
        pending_version: Option<String>,
        pending_compiled: bool,
        compiled_versions: Vec<String>,
    },
    Pending {
        active_version: String,
        pending_version: Option<String>,
        pending_compiled: bool,
        compiled_versions: Vec<String>,
    },
    NoActiveVersion,
    NotLoggedIn,
    NotFound,
    InternalError,
}

#[allow(clippy::too_many_arguments)]
pub async fn deploy_wasm(
    control_url: &str,
    token: &str,
    project_id: &str,
    build_id: &str,
    bundle_tar_path: &Path,
    jobs: &[CronJob],
    cron_updated_at: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    println!("project_id: {project_id}");

    let DeployOk {
        presigned_put_url,
        object_key,
        static_uploads: _,
        code_version,
    } = request_deploy(
        &client,
        control_url,
        token,
        project_id,
        build_id,
        Vec::new(),
        jobs,
        cron_updated_at,
    )
    .await?;

    println!("uploading bundle to {object_key} (code_version={code_version})...");
    upload_bundle(&client, &presigned_put_url, bundle_tar_path, code_version).await?;

    poll_deploy_status(&client, control_url, token, project_id, code_version).await?;
    println!("Deploy complete!");
    Ok(())
}

struct DeployOk {
    presigned_put_url: String,
    object_key: String,
    static_uploads: Vec<StaticUpload>,
    code_version: u64,
}

#[allow(clippy::too_many_arguments)]
pub async fn deploy_forte(
    control_url: &str,
    token: &str,
    project_id: &str,
    build_id: &str,
    fe_dist_dir: &Path,
    bundle_tar_path: &Path,
    jobs: &[CronJob],
    cron_updated_at: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    println!("project_id: {project_id}");

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
        code_version,
    } = request_deploy(
        &client,
        control_url,
        token,
        project_id,
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

    println!("uploading bundle to {object_key} (code_version={code_version})...");
    upload_bundle(&client, &presigned_put_url, bundle_tar_path, code_version).await?;

    poll_deploy_status(&client, control_url, token, project_id, code_version).await?;
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
    let raw: Deploy = client
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
        Deploy::Ok {
            presigned_put_url,
            object_key,
            static_uploads,
            code_version,
        } => Ok(DeployOk {
            presigned_put_url,
            object_key,
            static_uploads,
            code_version,
        }),
        Deploy::QuotaExceeded { reason } => Err(anyhow!("deploy quota exceeded: {reason}")),
        Deploy::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        Deploy::NotFound => Err(anyhow!("project '{project_id}' not found or not owned by you.")),
        Deploy::InternalError => Err(anyhow!("deploy: server error; check fn0-control logs")),
    }
}

async fn upload_bundle(
    client: &reqwest::Client,
    presigned_put_url: &str,
    bundle_tar_path: &Path,
    code_version: u64,
) -> Result<()> {
    let bundle_bytes = std::fs::read(bundle_tar_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", bundle_tar_path.display(), e))?;
    let _ = code_version;
    client
        .put(presigned_put_url)
        .body(bundle_bytes)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("bundle upload failed: {e}"))?;
    Ok(())
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

async fn poll_deploy_status(
    client: &reqwest::Client,
    control_url: &str,
    token: &str,
    project_id: &str,
    code_version: u64,
) -> Result<()> {
    let url = format!(
        "{}/__forte_action/deploy_status",
        control_url.trim_end_matches('/')
    );
    let timeout = std::time::Duration::from_secs(600);
    let start = std::time::Instant::now();
    let mut last_state: Option<String> = None;

    loop {
        let raw: DeployStatus = client
            .post(&url)
            .bearer_auth(token)
            .json(&DeployStatusInput {
                project_id,
                code_version,
            })
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow!("deploy_status failed: {e}"))?
            .json()
            .await?;

        match raw {
            DeployStatus::Done {
                active_version,
                pending_version,
                pending_compiled,
                compiled_versions,
            } => {
                log_status_line(
                    &active_version,
                    &compiled_versions,
                    &pending_version,
                    pending_compiled,
                    &mut last_state,
                );
                return Ok(());
            }
            DeployStatus::Pending {
                active_version,
                pending_version,
                pending_compiled,
                compiled_versions,
            } => {
                log_status_line(
                    &active_version,
                    &compiled_versions,
                    &pending_version,
                    pending_compiled,
                    &mut last_state,
                );
                if start.elapsed() > timeout {
                    return Err(anyhow!(
                        "deploy_status timed out after {}s",
                        timeout.as_secs()
                    ));
                }
            }
            DeployStatus::NoActiveVersion => {
                return Err(anyhow!("control has no active fn0-wasmtime version yet"));
            }
            DeployStatus::NotLoggedIn => {
                return Err(anyhow!("control rejected token; run `fn0 login` again."));
            }
            DeployStatus::NotFound => {
                return Err(anyhow!(
                    "project '{project_id}' not found or not owned by you."
                ));
            }
            DeployStatus::InternalError => {
                return Err(anyhow!(
                    "deploy_status: server error; check fn0-control logs"
                ));
            }
        }
    }
}

fn log_status_line(
    active_version: &str,
    compiled_versions: &[String],
    pending_version: &Option<String>,
    pending_compiled: bool,
    last_state: &mut Option<String>,
) {
    let state = format!(
        "active={active_version} compiled={compiled_versions:?} pending={pending_version:?} pending_compiled={pending_compiled}",
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
    _project_id: &str,
    _task: &str,
    _input_body: Vec<u8>,
    _timeout_secs: u64,
) -> Result<AdminRunOutput> {
    Err(anyhow!(
        "admin run is not yet migrated to control. See GitHub issue #4."
    ))
}

#[derive(Serialize)]
struct DomainAddInput<'a> {
    project_id: &'a str,
    domain: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DomainAdd {
    Ok,
    NotLoggedIn,
    NotFound,
    InvalidDomain { message: String },
    DomainTaken { existing_project_id: String },
    AlreadyHasDomain { current_domain: String },
    InternalError,
}

pub async fn domain_add(project_id: &str, domain: &str) -> Result<()> {
    let creds = credentials::require()?;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/__forte_action/domain_add",
        creds.control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainAddInput { project_id, domain })
        .send()
        .await?
        .error_for_status()?;
    let raw: DomainAdd = resp.json().await?;
    match raw {
        DomainAdd::Ok => {
            println!("domain '{domain}' attached to project '{project_id}'");
            println!(
                "Cloudflare hostname registration is queued; run `fn0 domain status` to check."
            );
            Ok(())
        }
        DomainAdd::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DomainAdd::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        DomainAdd::InvalidDomain { message } => Err(anyhow!("invalid domain: {message}")),
        DomainAdd::DomainTaken {
            existing_project_id,
        } => Err(anyhow!(
            "domain '{domain}' already in use by project '{existing_project_id}'"
        )),
        DomainAdd::AlreadyHasDomain { current_domain } => Err(anyhow!(
            "project '{project_id}' already has domain '{current_domain}'; remove it first"
        )),
        DomainAdd::InternalError => Err(anyhow!("domain_add: server error; check fn0-control logs")),
    }
}

#[derive(Serialize)]
struct DomainProjectInput<'a> {
    project_id: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DomainRemove {
    Ok { removed_domain: String },
    NotLoggedIn,
    NotFound,
    NoDomain,
    InternalError,
}

pub async fn domain_remove(project_id: &str) -> Result<()> {
    let creds = credentials::require()?;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/__forte_action/domain_remove",
        creds.control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainProjectInput { project_id })
        .send()
        .await?
        .error_for_status()?;
    let raw: DomainRemove = resp.json().await?;
    match raw {
        DomainRemove::Ok { removed_domain } => {
            println!("domain '{removed_domain}' detached from project '{project_id}'");
            println!("Cloudflare hostname removal is queued.");
            Ok(())
        }
        DomainRemove::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DomainRemove::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        DomainRemove::NoDomain => Err(anyhow!(
            "no custom domain attached to project '{project_id}'."
        )),
        DomainRemove::InternalError => Err(anyhow!(
            "domain_remove: server error; check fn0-control logs"
        )),
    }
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DomainStatus {
    NotConfigured,
    Configured {
        domain: String,
        cloudflare_status: CloudflareStatus,
    },
    NotLoggedIn,
    NotFound,
    InternalError,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum CloudflareStatus {
    Active,
    Pending,
    Missing,
    Other { value: String },
}

pub async fn domain_status(project_id: &str) -> Result<()> {
    let creds = credentials::require()?;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/__forte_action/domain_status",
        creds.control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainProjectInput { project_id })
        .send()
        .await?
        .error_for_status()?;
    let raw: DomainStatus = resp.json().await?;
    match raw {
        DomainStatus::NotConfigured => {
            println!("project '{project_id}' has no custom domain configured.");
            Ok(())
        }
        DomainStatus::Configured {
            domain,
            cloudflare_status,
        } => {
            println!("project '{project_id}' custom domain: {domain}");
            println!(
                "cloudflare status: {}",
                format_cloudflare_status(&cloudflare_status)
            );
            Ok(())
        }
        DomainStatus::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DomainStatus::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        DomainStatus::InternalError => Err(anyhow!(
            "domain_status: server error; check fn0-control logs"
        )),
    }
}

fn format_cloudflare_status(status: &CloudflareStatus) -> String {
    match status {
        CloudflareStatus::Active => "active".to_string(),
        CloudflareStatus::Pending => "pending (waiting for DV verification)".to_string(),
        CloudflareStatus::Missing => {
            "missing on Cloudflare (registration may still be in progress)".to_string()
        }
        CloudflareStatus::Other { value } => format!("other: {value}"),
    }
}

#[derive(Serialize)]
struct RenameProjectInput<'a> {
    project_id: &'a str,
    new_name: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum RenameProject {
    Ok,
    NotLoggedIn,
    NotFound,
    InvalidName,
    InternalError,
}

pub async fn rename_project(project_id: &str, new_name: &str) -> Result<()> {
    let creds = credentials::require()?;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/__forte_action/rename_project",
        creds.control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&RenameProjectInput {
            project_id,
            new_name,
        })
        .send()
        .await?
        .error_for_status()?;
    let raw: RenameProject = resp.json().await?;
    match raw {
        RenameProject::Ok => Ok(()),
        RenameProject::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        RenameProject::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        RenameProject::InvalidName => Err(anyhow!(
            "control rejected name '{new_name}': must be 1-{MAX_PROJECT_NAME_LEN} chars of letters, digits, '.', '_', '-'"
        )),
        RenameProject::InternalError => Err(anyhow!(
            "rename_project: server error; check fn0-control logs"
        )),
    }
}
