use std::sync::Arc;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::{Request, Response, body::Bytes};
use serde::{Deserialize, Serialize};

use crate::args_parse::DeployContext;

#[derive(Deserialize)]
struct DeployStartRequest {
    github_token: String,
    project_name: String,
}

#[derive(Serialize)]
struct DeployStartResponse {
    presigned_url: String,
    deploy_job_id: String,
    subdomain: String,
    code_id: u64,
    build_id: String,
    static_base_url: String,
}

#[derive(Deserialize)]
struct DeployFinishRequest {
    github_token: String,
    subdomain: String,
    code_id: u64,
    build_id: Option<String>,
    env: Option<String>,
}

#[derive(Deserialize)]
struct DeployR2SignRequest {
    github_token: String,
    subdomain: String,
    build_id: String,
    files: Vec<DeployR2SignFile>,
}

#[derive(Deserialize)]
struct DeployR2SignFile {
    path: String,
    content_type: Option<String>,
}

#[derive(Serialize)]
struct DeployR2SignResponse {
    uploads: Vec<DeployR2SignUpload>,
}

#[derive(Serialize)]
struct DeployR2SignUpload {
    path: String,
    presigned_url: String,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize)]
struct DeployDestroyRequest {
    github_token: String,
    project_name: String,
}

#[derive(Serialize)]
struct DeployStatusResponse {
    latest_generation: u64,
    target_generation: u64,
    delivered: bool,
    hosts_total: usize,
    hosts_at_target: usize,
    hosts_pending: Vec<String>,
    hosts_quarantined: Vec<String>,
    sites: Vec<SiteStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    job: Option<DeployJobStatus>,
}

#[derive(Serialize)]
struct DeployJobStatus {
    job_id: String,
    phase: crate::doc_db::DeployJobPhase,
    code_version: Option<u64>,
    generation: Option<u64>,
    attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error: Option<String>,
}

#[derive(Serialize)]
struct SiteStatus {
    name: String,
    target_fn0_wasmtime_version: Option<String>,
    ready_fn0_wasmtime_version: Option<String>,
    wasmtime_synced: bool,
}

pub(crate) fn json_response<T: Serialize>(status: u16, body: &T) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(body).unwrap();
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

pub(crate) async fn verify_github_user(token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "fn0-hq")
        .send()
        .await
        .map_err(|e| format!("GitHub API request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err("Invalid GitHub token".to_string());
    }

    #[derive(Deserialize)]
    struct GithubUser {
        login: String,
    }

    let user: GithubUser = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    Ok(user.login)
}

pub async fn handle_deploy_start(
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

    let request: DeployStartRequest = match serde_json::from_slice(&body) {
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
        .get_or_create_project(&username, &request.project_name)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to get project: {}", e),
                },
            );
        }
    };

    let deploy_job_id = uuid::Uuid::new_v4().to_string();
    let s3_key = format!("bundles/{}.raw.tar", project.subdomain);

    let presigned = match ctx
        .aws_s3
        .presign_write(&s3_key, std::time::Duration::from_secs(300))
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to generate presigned URL: {}", e),
                },
            );
        }
    };

    let build_id = uuid::Uuid::new_v4().to_string();
    let static_base_url = ctx.forte_r2.static_base_url(&project.subdomain, &build_id);

    json_response(
        200,
        &DeployStartResponse {
            presigned_url: presigned.uri().to_string(),
            deploy_job_id,
            subdomain: project.subdomain,
            code_id: project.code_id,
            build_id,
            static_base_url,
        },
    )
}

pub async fn handle_deploy_r2_sign(
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

    let request: DeployR2SignRequest = match serde_json::from_slice(&body) {
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

    let project_name = match request.subdomain.strip_prefix(&format!("{username}-")) {
        Some(name) => name,
        None => {
            return json_response(
                403,
                &ErrorResponse {
                    error: "subdomain does not match authenticated user".to_string(),
                },
            );
        }
    };

    let project = match ctx.doc_db.get_project(&username, project_name).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return json_response(
                404,
                &ErrorResponse {
                    error: "Project not found".to_string(),
                },
            );
        }
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to get project: {e}"),
                },
            );
        }
    };
    if project.subdomain != request.subdomain {
        return json_response(
            403,
            &ErrorResponse {
                error: "subdomain mismatch".to_string(),
            },
        );
    }

    let expires = std::time::Duration::from_secs(600);
    let mut uploads = Vec::with_capacity(request.files.len());
    for file in &request.files {
        match ctx
            .forte_r2
            .presign_put(
                &request.subdomain,
                &request.build_id,
                &file.path,
                file.content_type.as_deref(),
                expires,
            )
            .await
        {
            Ok(url) => uploads.push(DeployR2SignUpload {
                path: file.path.clone(),
                presigned_url: url,
            }),
            Err(e) => {
                return json_response(
                    500,
                    &ErrorResponse {
                        error: format!("presign failed: {e}"),
                    },
                );
            }
        }
    }

    json_response(200, &DeployR2SignResponse { uploads })
}

pub async fn handle_deploy_finish(
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

    let request: DeployFinishRequest = match serde_json::from_slice(&body) {
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

    let _username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

    let env_ciphertext_b64 = match &request.env {
        Some(content) => {
            let ciphertext =
                match crate::env_crypto::encrypt(&ctx.env_encryption_key, content.as_bytes()) {
                    Ok(c) => c,
                    Err(e) => {
                        return json_response(
                            500,
                            &ErrorResponse {
                                error: format!("env encryption failed: {}", e),
                            },
                        );
                    }
                };
            use base64::{Engine, engine::general_purpose::STANDARD};
            Some(STANDARD.encode(ciphertext))
        }
        None => None,
    };

    let now = chrono::Utc::now().to_rfc3339();
    let job_id = uuid::Uuid::new_v4().to_string();
    let job = crate::doc_db::DeployJob {
        job_id: job_id.clone(),
        subdomain: request.subdomain.clone(),
        code_id: request.code_id,
        build_id: request.build_id.clone(),
        env_ciphertext: env_ciphertext_b64,
        phase: crate::doc_db::DeployJobPhase::Queued,
        code_version: None,
        old_build_ids: None,
        generation: None,
        attempts: 0,
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
        heartbeat_at: None,
    };

    if let Err(e) = ctx.doc_db.insert_deploy_job(job.clone()).await {
        return json_response(
            500,
            &ErrorResponse {
                error: format!("Failed to enqueue deploy job: {}", e),
            },
        );
    }

    json_response(
        200,
        &serde_json::json!({
            "ok": true,
            "job_id": job_id,
        }),
    )
}

pub async fn handle_deploy_destroy(
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

    let request: DeployDestroyRequest = match serde_json::from_slice(&body) {
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
                    error: "Project not found".to_string(),
                },
            );
        }
        Err(e) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to get project: {}", e),
                },
            );
        }
    };

    let job_id = uuid::Uuid::new_v4().to_string();
    let s3_key = format!("bundles/{}.tar.zst", project.subdomain);
    let job_payload = serde_json::json!({
        "s3_key": s3_key,
        "bucket": ctx.cwasm_bucket,
    })
    .to_string();

    if let Err(e) = ctx
        .doc_db
        .insert_undeployment_with_job(&project.subdomain, &job_id, &job_payload)
        .await
    {
        return json_response(
            500,
            &ErrorResponse {
                error: format!("Failed to destroy deployment: {}", e),
            },
        );
    }

    if let Err(e) = ctx.doc_db.clear_build(&project.subdomain).await {
        tracing::warn!(%e, subdomain = %project.subdomain, "Failed to clear build registry");
    }
    if let Err(e) = ctx
        .doc_db
        .enqueue_r2_subdomain_delete(&project.subdomain)
        .await
    {
        tracing::warn!(%e, subdomain = %project.subdomain, "Failed to enqueue r2 subdomain delete");
    }

    spawn_immediate_push(&ctx).await;

    json_response(
        200,
        &serde_json::json!({"ok": true, "subdomain": project.subdomain}),
    )
}

pub async fn handle_deploy_status(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Response<Full<Bytes>> {
    ctx.deployment_cache.refresh().await;
    let latest_generation = ctx.deployment_cache.last_deployment_id();

    let mut parsed_generation: Option<u64> = None;
    let mut job_id: Option<String> = None;
    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            if let Some(v) = pair.strip_prefix("generation=") {
                match v.parse() {
                    Ok(n) => parsed_generation = Some(n),
                    Err(_) => {
                        return json_response(
                            400,
                            &ErrorResponse {
                                error: format!("invalid generation '{}'", v),
                            },
                        );
                    }
                }
            } else if let Some(v) = pair.strip_prefix("job_id=") {
                job_id = Some(v.to_string());
            }
        }
    }

    let job_info = if let Some(id) = job_id.as_deref() {
        match ctx.doc_db.get_deploy_job(id).await {
            Ok(Some(job)) => Some(job),
            Ok(None) => {
                return json_response(
                    404,
                    &ErrorResponse {
                        error: format!("deploy job '{}' not found", id),
                    },
                );
            }
            Err(e) => {
                return json_response(
                    500,
                    &ErrorResponse {
                        error: format!("Failed to read deploy job: {}", e),
                    },
                );
            }
        }
    } else {
        None
    };

    let target_generation = parsed_generation
        .or_else(|| job_info.as_ref().and_then(|j| j.generation))
        .unwrap_or(latest_generation);

    let mut hosts_total = 0usize;
    let mut hosts_at_target = 0usize;
    let mut hosts_pending: Vec<String> = Vec::new();
    let mut hosts_quarantined: Vec<String> = Vec::new();
    let mut sites: Vec<SiteStatus> = Vec::new();
    let mut all_sites_synced = true;

    for site in &ctx.sites {
        let target = match ctx.doc_db.get_worker_target(site.name()).await {
            Ok(t) => t,
            Err(e) => {
                return json_response(
                    500,
                    &ErrorResponse {
                        error: format!("Failed to read worker-target: {}", e),
                    },
                );
            }
        };
        let ready = match ctx.doc_db.get_cwasm_ready(site.name()).await {
            Ok(r) => r,
            Err(e) => {
                return json_response(
                    500,
                    &ErrorResponse {
                        error: format!("Failed to read cwasm-ready: {}", e),
                    },
                );
            }
        };

        let target_ver = target.as_ref().map(|t| t.fn0_wasmtime_version.clone());
        let ready_ver = ready.as_ref().map(|r| r.fn0_wasmtime_version.clone());
        let wasmtime_synced = match (&target_ver, &ready_ver) {
            (Some(t), Some(r)) => t == r,
            (None, _) => true,
            (Some(_), None) => false,
        };
        if !wasmtime_synced {
            all_sites_synced = false;
        }

        sites.push(SiteStatus {
            name: site.name().to_string(),
            target_fn0_wasmtime_version: target_ver,
            ready_fn0_wasmtime_version: ready_ver,
            wasmtime_synced,
        });

        match ctx.doc_db.list_host_statuses(site.name()).await {
            Ok(statuses) => {
                for s in statuses {
                    hosts_total += 1;
                    if s.consecutive_failures > 0 {
                        hosts_quarantined.push(s.host_id.clone());
                    }
                    if s.generation.unwrap_or(0) >= target_generation {
                        hosts_at_target += 1;
                    } else {
                        hosts_pending.push(s.host_id.clone());
                    }
                }
            }
            Err(e) => {
                return json_response(
                    500,
                    &ErrorResponse {
                        error: format!("Failed to list host-status: {}", e),
                    },
                );
            }
        }
    }

    let delivered = hosts_total > 0
        && hosts_total == hosts_at_target
        && hosts_quarantined.is_empty()
        && all_sites_synced;

    let job = job_info.map(|j| DeployJobStatus {
        job_id: j.job_id,
        phase: j.phase,
        code_version: j.code_version,
        generation: j.generation,
        attempts: j.attempts,
        last_error: j.last_error,
    });

    json_response(
        200,
        &DeployStatusResponse {
            latest_generation,
            target_generation,
            delivered,
            hosts_total,
            hosts_at_target,
            hosts_pending,
            hosts_quarantined,
            sites,
            job,
        },
    )
}

pub(crate) async fn spawn_immediate_push(ctx: &Arc<DeployContext>) {
    ctx.deployment_cache.refresh().await;
    ctx.custom_domain_cache.refresh().await;
    for site in &ctx.sites {
        let pool = site.ssh_pool().clone();
        let cache = ctx.deployment_cache.clone();
        let dom_cache = ctx.custom_domain_cache.clone();
        let db = ctx.doc_db.clone();
        let site_name = site.name().to_string();
        tokio::spawn(async move {
            let statuses = match db.list_host_statuses(&site_name).await {
                Ok(list) => list,
                Err(err) => {
                    tracing::warn!(%err, "push: list_host_statuses failed");
                    return;
                }
            };
            let addrs: Vec<String> = statuses.into_iter().map(|s| s.addr).collect();
            crate::site::push_to_all(&pool, &cache, &dom_cache, addrs).await;
        });
    }
}
