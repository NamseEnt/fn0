use std::sync::Arc;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::{Request, Response, body::Bytes};
use serde::{Deserialize, Serialize};

use crate::args_parse::DeployContext;
use crate::deploy::{json_response, verify_github_user};
use crate::doc_db::{
    CustomDomainAddJob, CustomDomainAddPhase, CustomDomainRemoveJob, CustomDomainRemovePhase,
};

#[derive(Deserialize)]
struct DomainAddRequest {
    github_token: String,
    project_name: String,
    domain: String,
}

#[derive(Deserialize)]
struct DomainRemoveRequest {
    github_token: String,
    project_name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct DomainAddResponse {
    job_id: String,
    already_running: bool,
    subdomain: String,
    domain: String,
}

#[derive(Serialize)]
struct DomainRemoveResponse {
    job_id: String,
    already_running: bool,
    subdomain: String,
    domain: String,
}

#[derive(Serialize)]
struct DomainStatusResponse {
    subdomain: String,
    current_domain: Option<String>,
    add_job: Option<AddJobStatus>,
    remove_job: Option<RemoveJobStatus>,
}

#[derive(Serialize)]
struct AddJobStatus {
    job_id: String,
    phase: String,
    domain: String,
    attempts: u32,
    last_error: Option<String>,
    is_terminal: bool,
}

#[derive(Serialize)]
struct RemoveJobStatus {
    job_id: String,
    phase: String,
    domain: String,
    attempts: u32,
    last_error: Option<String>,
    is_terminal: bool,
}

pub async fn handle_domain_add(
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

    let request: DomainAddRequest = match serde_json::from_slice(&body) {
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

    let domain = request.domain.trim().to_ascii_lowercase();
    if !is_valid_domain(&domain) {
        return json_response(
            400,
            &ErrorResponse {
                error: "Invalid domain".to_string(),
            },
        );
    }

    let username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

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
                    error: format!("Failed to read project: {e}"),
                },
            );
        }
    };

    match ctx
        .doc_db
        .get_custom_domain_by_subdomain(&project.subdomain)
        .await
    {
        Ok(Some(existing)) => {
            if existing == domain {
            } else {
                return json_response(
                    409,
                    &ErrorResponse {
                        error: format!(
                            "project already has custom domain '{existing}'; remove it first",
                        ),
                    },
                );
            }
        }
        Ok(None) => {}
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to read custom domain: {e}"),
                },
            );
        }
    }

    match ctx.doc_db.get_custom_domain_by_domain(&domain).await {
        Ok(Some((existing_sub, _))) if existing_sub != project.subdomain => {
            return json_response(
                409,
                &ErrorResponse {
                    error: format!("domain '{domain}' is in use by another project"),
                },
            );
        }
        Ok(_) => {}
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to read custom domain: {e}"),
                },
            );
        }
    }

    if let Ok(Some(existing)) = ctx
        .doc_db
        .find_active_custom_domain_add_job_for(&project.subdomain)
        .await
    {
        return json_response(
            200,
            &DomainAddResponse {
                job_id: existing.job_id,
                already_running: true,
                subdomain: project.subdomain,
                domain: existing.domain,
            },
        );
    }

    let now = chrono::Utc::now().to_rfc3339();
    let job_id = uuid::Uuid::new_v4().to_string();
    let job = CustomDomainAddJob {
        job_id: job_id.clone(),
        github_username: username.clone(),
        project_name: request.project_name.clone(),
        subdomain: project.subdomain.clone(),
        domain: domain.clone(),
        cloudflare_hostname_id: None,
        phase: CustomDomainAddPhase::Queued,
        attempts: 0,
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
        heartbeat_at: None,
    };
    if let Err(e) = ctx.doc_db.insert_custom_domain_add_job(job).await {
        return json_response(
            500,
            &ErrorResponse {
                error: format!("Failed to enqueue add job: {e}"),
            },
        );
    }

    json_response(
        200,
        &DomainAddResponse {
            job_id,
            already_running: false,
            subdomain: project.subdomain,
            domain,
        },
    )
}

pub async fn handle_domain_remove(
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

    let request: DomainRemoveRequest = match serde_json::from_slice(&body) {
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

    let username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

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
                    error: format!("Failed to read project: {e}"),
                },
            );
        }
    };

    let domain = match ctx
        .doc_db
        .get_custom_domain_by_subdomain(&project.subdomain)
        .await
    {
        Ok(Some(d)) => d,
        Ok(None) => {
            return json_response(
                404,
                &ErrorResponse {
                    error: "No custom domain attached to this project".to_string(),
                },
            );
        }
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to read custom domain: {e}"),
                },
            );
        }
    };

    let cloudflare_hostname_id = ctx
        .doc_db
        .get_custom_domain_by_domain(&domain)
        .await
        .ok()
        .flatten()
        .map(|(_, id)| id);

    if let Ok(Some(existing)) = ctx
        .doc_db
        .find_active_custom_domain_remove_job_for(&project.subdomain)
        .await
    {
        return json_response(
            200,
            &DomainRemoveResponse {
                job_id: existing.job_id,
                already_running: true,
                subdomain: project.subdomain,
                domain: existing.domain,
            },
        );
    }

    let now = chrono::Utc::now().to_rfc3339();
    let job_id = uuid::Uuid::new_v4().to_string();
    let job = CustomDomainRemoveJob {
        job_id: job_id.clone(),
        github_username: username.clone(),
        project_name: request.project_name.clone(),
        subdomain: project.subdomain.clone(),
        domain: domain.clone(),
        cloudflare_hostname_id,
        phase: CustomDomainRemovePhase::Queued,
        attempts: 0,
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
        heartbeat_at: None,
    };
    if let Err(e) = ctx.doc_db.insert_custom_domain_remove_job(job).await {
        return json_response(
            500,
            &ErrorResponse {
                error: format!("Failed to enqueue remove job: {e}"),
            },
        );
    }

    json_response(
        200,
        &DomainRemoveResponse {
            job_id,
            already_running: false,
            subdomain: project.subdomain,
            domain,
        },
    )
}

pub async fn handle_domain_status(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Response<Full<Bytes>> {
    let mut github_token: Option<String> = None;
    let mut project_name: Option<String> = None;
    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            if let Some(v) = pair.strip_prefix("github_token=") {
                github_token = Some(percent_decode(v));
            } else if let Some(v) = pair.strip_prefix("project_name=") {
                project_name = Some(percent_decode(v));
            }
        }
    }
    let github_token = match github_token {
        Some(t) => t,
        None => {
            return json_response(
                400,
                &ErrorResponse {
                    error: "missing github_token".to_string(),
                },
            );
        }
    };
    let project_name = match project_name {
        Some(p) => p,
        None => {
            return json_response(
                400,
                &ErrorResponse {
                    error: "missing project_name".to_string(),
                },
            );
        }
    };

    let username = match verify_github_user(&github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

    let project = match ctx.doc_db.get_project(&username, &project_name).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return json_response(
                404,
                &ErrorResponse {
                    error: format!("project '{project_name}' not found for user '{username}'"),
                },
            );
        }
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to read project: {e}"),
                },
            );
        }
    };

    let current_domain = match ctx
        .doc_db
        .get_custom_domain_by_subdomain(&project.subdomain)
        .await
    {
        Ok(d) => d,
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to read custom domain: {e}"),
                },
            );
        }
    };

    let add_job = ctx
        .doc_db
        .find_active_custom_domain_add_job_for(&project.subdomain)
        .await
        .ok()
        .flatten()
        .map(|j| AddJobStatus {
            job_id: j.job_id.clone(),
            phase: add_phase_to_str(&j.phase).to_string(),
            domain: j.domain.clone(),
            attempts: j.attempts,
            last_error: j.last_error.clone(),
            is_terminal: j.is_terminal(),
        });

    let remove_job = ctx
        .doc_db
        .find_active_custom_domain_remove_job_for(&project.subdomain)
        .await
        .ok()
        .flatten()
        .map(|j| RemoveJobStatus {
            job_id: j.job_id.clone(),
            phase: remove_phase_to_str(&j.phase).to_string(),
            domain: j.domain.clone(),
            attempts: j.attempts,
            last_error: j.last_error.clone(),
            is_terminal: j.is_terminal(),
        });

    json_response(
        200,
        &DomainStatusResponse {
            subdomain: project.subdomain,
            current_domain,
            add_job,
            remove_job,
        },
    )
}

fn percent_decode(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'+' {
            out.push(b' ');
            i += 1;
        } else if b == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            match (hi, lo) {
                (Some(h), Some(l)) => {
                    out.push((h * 16 + l) as u8);
                    i += 3;
                }
                _ => {
                    out.push(b);
                    i += 1;
                }
            }
        } else {
            out.push(b);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

fn is_valid_domain(d: &str) -> bool {
    if d.is_empty() || d.len() > 253 {
        return false;
    }
    if !d.contains('.') {
        return false;
    }
    for label in d.split('.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        if !label
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return false;
        }
        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }
    }
    true
}

fn add_phase_to_str(p: &CustomDomainAddPhase) -> &'static str {
    match p {
        CustomDomainAddPhase::Queued => "queued",
        CustomDomainAddPhase::CloudflareHostnameCreated => "cloudflare_hostname_created",
        CustomDomainAddPhase::DocPersisted => "doc_persisted",
        CustomDomainAddPhase::Pushed => "pushed",
        CustomDomainAddPhase::Verified => "verified",
        CustomDomainAddPhase::Done => "done",
        CustomDomainAddPhase::Failed => "failed",
    }
}

fn remove_phase_to_str(p: &CustomDomainRemovePhase) -> &'static str {
    match p {
        CustomDomainRemovePhase::Queued => "queued",
        CustomDomainRemovePhase::DocDeleted => "doc_deleted",
        CustomDomainRemovePhase::Pushed => "pushed",
        CustomDomainRemovePhase::CloudflareHostnameDeleted => "cloudflare_hostname_deleted",
        CustomDomainRemovePhase::Done => "done",
        CustomDomainRemovePhase::Failed => "failed",
    }
}
