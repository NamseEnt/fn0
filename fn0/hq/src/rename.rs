use std::sync::Arc;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::{Request, Response, body::Bytes};
use serde::{Deserialize, Serialize};

use crate::args_parse::DeployContext;
use crate::deploy::{json_response, verify_github_user};

#[derive(Deserialize)]
struct RenameStartRequest {
    github_token: String,
    project_name: String,
    new_project_name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct RenameStartResponse {
    ok: bool,
    job_id: String,
    already_running: bool,
}

#[derive(Serialize)]
struct RenameStatusResponse {
    job_id: String,
    phase: String,
    attempts: u32,
    last_error: Option<String>,
    is_terminal: bool,
}

pub async fn handle_rename_start(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Response<Full<Bytes>> {
    let body = match req.collect().await {
        Ok(b) => b.to_bytes(),
        Err(_) => {
            return json_response(
                400,
                &ErrorResponse {
                    error: "Failed to read body".to_string(),
                },
            );
        }
    };

    let request: RenameStartRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            return json_response(
                400,
                &ErrorResponse {
                    error: "Invalid request body".to_string(),
                },
            );
        }
    };

    if request.project_name == request.new_project_name {
        return json_response(
            400,
            &ErrorResponse {
                error: "project_name and new_project_name are identical".to_string(),
            },
        );
    }

    if !is_valid_project_name(&request.new_project_name) {
        return json_response(
            400,
            &ErrorResponse {
                error: "new_project_name has invalid characters".to_string(),
            },
        );
    }

    let username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

    let old_subdomain = format!("{username}-{}", request.project_name);
    let new_subdomain = format!("{username}-{}", request.new_project_name);

    let project = match ctx
        .doc_db
        .get_project(&username, &request.project_name)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return json_response(
                404,
                &ErrorResponse {
                    error: format!(
                        "project '{}' not found for user '{}'",
                        request.project_name, username
                    ),
                },
            );
        }
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to read project: {}", e),
                },
            );
        }
    };

    match ctx
        .doc_db
        .get_project(&username, &request.new_project_name)
        .await
    {
        Ok(Some(_)) => {
            return json_response(
                409,
                &ErrorResponse {
                    error: format!(
                        "project '{}' already exists for user '{}'",
                        request.new_project_name, username
                    ),
                },
            );
        }
        Ok(None) => {}
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to check new project: {}", e),
                },
            );
        }
    }

    if let Ok(Some(existing)) = ctx.doc_db.find_active_rename_job_for(&old_subdomain).await {
        return json_response(
            200,
            &RenameStartResponse {
                ok: true,
                job_id: existing.job_id,
                already_running: true,
            },
        );
    }

    let now = chrono::Utc::now().to_rfc3339();
    let job_id = uuid::Uuid::new_v4().to_string();
    let job = crate::doc_db::RenameJob {
        job_id: job_id.clone(),
        github_username: username.clone(),
        old_project_name: request.project_name.clone(),
        new_project_name: request.new_project_name.clone(),
        old_subdomain,
        new_subdomain,
        code_id: project.code_id,
        code_version: None,
        build_id: None,
        env_ciphertext: None,
        old_build_ids: None,
        generation: None,
        phase: crate::doc_db::RenameJobPhase::Queued,
        attempts: 0,
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
        heartbeat_at: None,
    };

    if let Err(e) = ctx.doc_db.insert_rename_job(job).await {
        return json_response(
            500,
            &ErrorResponse {
                error: format!("Failed to enqueue rename job: {}", e),
            },
        );
    }

    json_response(
        200,
        &RenameStartResponse {
            ok: true,
            job_id,
            already_running: false,
        },
    )
}

pub async fn handle_rename_status(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Response<Full<Bytes>> {
    let mut job_id: Option<String> = None;
    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            if let Some(v) = pair.strip_prefix("job_id=") {
                job_id = Some(v.to_string());
            }
        }
    }

    let job_id = match job_id {
        Some(id) => id,
        None => {
            return json_response(
                400,
                &ErrorResponse {
                    error: "missing job_id".to_string(),
                },
            );
        }
    };

    match ctx.doc_db.get_rename_job(&job_id).await {
        Ok(Some(job)) => json_response(
            200,
            &RenameStatusResponse {
                job_id: job.job_id.clone(),
                phase: phase_to_str(&job.phase).to_string(),
                attempts: job.attempts,
                last_error: job.last_error.clone(),
                is_terminal: job.is_terminal(),
            },
        ),
        Ok(None) => json_response(
            404,
            &ErrorResponse {
                error: format!("rename job '{}' not found", job_id),
            },
        ),
        Err(e) => json_response(
            500,
            &ErrorResponse {
                error: format!("Failed to read rename job: {}", e),
            },
        ),
    }
}

fn is_valid_project_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 63 {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn phase_to_str(phase: &crate::doc_db::RenameJobPhase) -> &'static str {
    use crate::doc_db::RenameJobPhase;
    match phase {
        RenameJobPhase::Queued => "queued",
        RenameJobPhase::OldWritesBlocked => "old_writes_blocked",
        RenameJobPhase::NewDbSeeded => "new_db_seeded",
        RenameJobPhase::NewEnvUploaded => "new_env_uploaded",
        RenameJobPhase::NewCwasmCompiled => "new_cwasm_compiled",
        RenameJobPhase::NewVersioned => "new_versioned",
        RenameJobPhase::NewDeploymentInserted => "new_deployment_inserted",
        RenameJobPhase::NewBuildRegistered => "new_build_registered",
        RenameJobPhase::NewDeploymentPushed => "new_deployment_pushed",
        RenameJobPhase::NewDeploymentVerified => "new_deployment_verified",
        RenameJobPhase::ProjectRenamed => "project_renamed",
        RenameJobPhase::OldUndeployed => "old_undeployed",
        RenameJobPhase::OldEnvDeleted => "old_env_deleted",
        RenameJobPhase::OldDbDestroyed => "old_db_destroyed",
        RenameJobPhase::Done => "done",
        RenameJobPhase::Failed => "failed",
    }
}
