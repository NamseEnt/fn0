use crate::static_files::{StaticFile, collect_static_files};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize)]
struct DeployInput<'a> {
    project_id: &'a str,
    code_version: u64,
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
    code_version: u64,
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
    } = request_deploy(
        &client,
        control_url,
        token,
        project_id,
        code_version,
        Vec::new(),
        jobs,
        cron_updated_at,
    )
    .await?;

    println!("uploading bundle to {object_key} (code_version={code_version})...");
    upload_bundle(&client, &presigned_put_url, bundle_tar_path).await?;

    poll_deploy_status(&client, control_url, token, project_id, code_version).await?;
    println!("Deploy complete!");
    Ok(())
}

struct DeployOk {
    presigned_put_url: String,
    object_key: String,
    static_uploads: Vec<StaticUpload>,
}

#[allow(clippy::too_many_arguments)]
pub async fn deploy_forte(
    control_url: &str,
    token: &str,
    project_id: &str,
    code_version: u64,
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
    } = request_deploy(
        &client,
        control_url,
        token,
        project_id,
        code_version,
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
    upload_bundle(&client, &presigned_put_url, bundle_tar_path).await?;

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
    code_version: u64,
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
            code_version,
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
        } => Ok(DeployOk {
            presigned_put_url,
            object_key,
            static_uploads,
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
) -> Result<()> {
    let bundle_bytes = std::fs::read(bundle_tar_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", bundle_tar_path.display(), e))?;
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
