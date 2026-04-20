use std::sync::Arc;

use aws_sdk_s3::presigning::PresigningConfig;
use http_body_util::BodyExt;
use hyper::{Request, Response, body::Bytes};
use http_body_util::Full;
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
struct ErrorResponse {
    error: String,
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
}

#[derive(Serialize)]
struct SiteStatus {
    name: String,
    target_fn0_wasmtime_version: Option<String>,
    ready_fn0_wasmtime_version: Option<String>,
    wasmtime_synced: bool,
}

fn json_response<T: Serialize>(status: u16, body: &T) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(body).unwrap();
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

async fn verify_github_user(token: &str) -> Result<String, String> {
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
        Err(_) => return json_response(400, &ErrorResponse { error: "Failed to read body".to_string() }),
    };

    let request: DeployStartRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return json_response(400, &ErrorResponse { error: "Invalid request body".to_string() }),
    };

    let username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

    let project = match ctx.doc_db.get_or_create_project(&username, &request.project_name).await {
        Ok(p) => p,
        Err(e) => return json_response(500, &ErrorResponse { error: format!("Failed to get project: {}", e) }),
    };

    let deploy_job_id = uuid::Uuid::new_v4().to_string();
    let s3_key = format!("bundles/{}.raw.tar", project.subdomain);

    let presigning_config = match PresigningConfig::expires_in(std::time::Duration::from_secs(300)) {
        Ok(c) => c,
        Err(_) => return json_response(500, &ErrorResponse { error: "Failed to create presigning config".to_string() }),
    };

    let presigned = match ctx
        .aws_s3_client
        .put_object()
        .bucket(&ctx.wasm_bucket)
        .key(&s3_key)
        .presigned(presigning_config)
        .await
    {
        Ok(p) => p,
        Err(e) => return json_response(500, &ErrorResponse { error: format!("Failed to generate presigned URL: {}", e) }),
    };

    let build_id = uuid::Uuid::new_v4().to_string();
    let static_base_url = ctx.forte_r2.static_base_url(&project.subdomain, &build_id);

    json_response(200, &DeployStartResponse {
        presigned_url: presigned.uri().to_string(),
        deploy_job_id,
        subdomain: project.subdomain,
        code_id: project.code_id,
        build_id,
        static_base_url,
    })
}

pub async fn handle_deploy_r2_sign(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Response<Full<Bytes>> {
    let body = match req.collect().await {
        Ok(b) => b.to_bytes(),
        Err(_) => return json_response(400, &ErrorResponse { error: "Failed to read body".to_string() }),
    };

    let request: DeployR2SignRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return json_response(400, &ErrorResponse { error: "Invalid request body".to_string() }),
    };

    let username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

    let project_name = match request.subdomain.strip_prefix(&format!("{username}-")) {
        Some(name) => name,
        None => return json_response(403, &ErrorResponse { error: "subdomain does not match authenticated user".to_string() }),
    };

    let project = match ctx.doc_db.get_project(&username, project_name).await {
        Ok(Some(p)) => p,
        Ok(None) => return json_response(404, &ErrorResponse { error: "Project not found".to_string() }),
        Err(e) => return json_response(500, &ErrorResponse { error: format!("Failed to get project: {e}") }),
    };
    if project.subdomain != request.subdomain {
        return json_response(403, &ErrorResponse { error: "subdomain mismatch".to_string() });
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
                return json_response(500, &ErrorResponse {
                    error: format!("presign failed: {e}"),
                });
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
        Err(_) => return json_response(400, &ErrorResponse { error: "Failed to read body".to_string() }),
    };

    let request: DeployFinishRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return json_response(400, &ErrorResponse { error: "Invalid request body".to_string() }),
    };

    let _username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

    let env_key = crate::cwasm_compile::env_key(&request.subdomain);
    let env_present = match &request.env {
        Some(content) => {
            let ciphertext = match crate::env_crypto::encrypt(
                &ctx.env_encryption_key,
                content.as_bytes(),
            ) {
                Ok(c) => c,
                Err(e) => return json_response(500, &ErrorResponse {
                    error: format!("env encryption failed: {}", e),
                }),
            };
            if let Err(e) = ctx
                .aws_s3_client
                .put_object()
                .bucket(&ctx.wasm_bucket)
                .key(&env_key)
                .body(aws_sdk_s3::primitives::ByteStream::from(ciphertext))
                .send()
                .await
            {
                return json_response(500, &ErrorResponse {
                    error: format!("env upload failed: {}", e),
                });
            }
            true
        }
        None => {
            let _ = ctx
                .aws_s3_client
                .delete_object()
                .bucket(&ctx.wasm_bucket)
                .key(&env_key)
                .send()
                .await;
            false
        }
    };

    let target_versions: std::collections::BTreeSet<String> = {
        let mut versions = std::collections::BTreeSet::new();
        for site in &ctx.sites {
            match ctx.doc_db.get_worker_target(site.name()).await {
                Ok(Some(t)) => {
                    versions.insert(t.fn0_wasmtime_version);
                }
                Ok(None) => {
                    return json_response(409, &ErrorResponse {
                        error: format!("worker-target not set for site '{}'", site.name()),
                    });
                }
                Err(e) => {
                    return json_response(500, &ErrorResponse {
                        error: format!("Failed to read worker-target: {}", e),
                    });
                }
            }
            match ctx.doc_db.get_worker_last_stable(site.name()).await {
                Ok(Some(l)) => {
                    versions.insert(l.fn0_wasmtime_version);
                }
                Ok(None) => {}
                Err(e) => {
                    return json_response(500, &ErrorResponse {
                        error: format!("Failed to read worker-last-stable: {}", e),
                    });
                }
            }
        }
        versions
    };

    for version in &target_versions {
        if let Err(e) = crate::cwasm_compile::compile_and_publish(
            &ctx.lambda_client,
            &ctx.aws_s3_client,
            &ctx.cwasm_s3_client,
            &ctx.wasm_bucket,
            &ctx.cwasm_bucket,
            version,
            &request.subdomain,
            env_present,
        )
        .await
        {
            return json_response(500, &ErrorResponse {
                error: format!("Compile failed for fn0-wasmtime {}: {}", version, e),
            });
        }
    }

    if let Err(e) = crate::turso_admin::ensure_database(&ctx.forte_db, &request.subdomain).await {
        return json_response(500, &ErrorResponse {
            error: format!("Failed to ensure forte-db database: {}", e),
        });
    }

    let code_version = match ctx.doc_db.next_code_version(request.code_id).await {
        Ok(v) => v,
        Err(e) => return json_response(500, &ErrorResponse { error: format!("Failed to get next version: {}", e) }),
    };

    if let Err(e) = ctx.doc_db.insert_deployment(&request.subdomain, request.code_id, code_version).await {
        return json_response(500, &ErrorResponse { error: format!("Failed to insert deployment: {}", e) });
    }

    if let Some(build_id) = &request.build_id {
        let old_build_ids = match ctx
            .doc_db
            .register_build(&request.subdomain, build_id)
            .await
        {
            Ok(ids) => ids,
            Err(e) => return json_response(500, &ErrorResponse {
                error: format!("Failed to register build: {}", e),
            }),
        };
        for old in old_build_ids {
            if let Err(e) = ctx.doc_db.enqueue_r2_prefix_delete(&request.subdomain, &old).await {
                tracing::warn!(%e, %old, "Failed to enqueue r2 prefix delete");
            }
        }
    }

    ctx.deployment_cache.refresh().await;
    let generation = ctx.deployment_cache.last_deployment_id();

    spawn_immediate_push(&ctx).await;

    json_response(200, &serde_json::json!({
        "ok": true,
        "code_version": code_version,
        "generation": generation,
    }))
}

pub async fn handle_deploy_destroy(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Response<Full<Bytes>> {
    let body = match req.collect().await {
        Ok(b) => b.to_bytes(),
        Err(_) => return json_response(400, &ErrorResponse { error: "Failed to read body".to_string() }),
    };

    let request: DeployDestroyRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return json_response(400, &ErrorResponse { error: "Invalid request body".to_string() }),
    };

    let username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

    let project = match ctx.doc_db.get_project(&username, &request.project_name).await {
        Ok(Some(p)) => p,
        Ok(None) => return json_response(404, &ErrorResponse { error: "Project not found".to_string() }),
        Err(e) => return json_response(500, &ErrorResponse { error: format!("Failed to get project: {}", e) }),
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
        return json_response(500, &ErrorResponse {
            error: format!("Failed to destroy deployment: {}", e),
        });
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

    json_response(200, &serde_json::json!({"ok": true, "subdomain": project.subdomain}))
}

pub async fn handle_deploy_status(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Response<Full<Bytes>> {
    ctx.deployment_cache.refresh().await;
    let latest_generation = ctx.deployment_cache.last_deployment_id();

    let target_generation = match req.uri().query() {
        Some(q) => {
            let mut parsed: Option<u64> = None;
            for pair in q.split('&') {
                if let Some(v) = pair.strip_prefix("generation=") {
                    match v.parse() {
                        Ok(n) => parsed = Some(n),
                        Err(_) => {
                            return json_response(400, &ErrorResponse {
                                error: format!("invalid generation '{}'", v),
                            });
                        }
                    }
                }
            }
            parsed.unwrap_or(latest_generation)
        }
        None => latest_generation,
    };

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
                return json_response(500, &ErrorResponse {
                    error: format!("Failed to read worker-target: {}", e),
                });
            }
        };
        let ready = match ctx.doc_db.get_cwasm_ready(site.name()).await {
            Ok(r) => r,
            Err(e) => {
                return json_response(500, &ErrorResponse {
                    error: format!("Failed to read cwasm-ready: {}", e),
                });
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
                return json_response(500, &ErrorResponse {
                    error: format!("Failed to list host-status: {}", e),
                });
            }
        }
    }

    let delivered = hosts_total > 0
        && hosts_total == hosts_at_target
        && hosts_quarantined.is_empty()
        && all_sites_synced;

    json_response(200, &DeployStatusResponse {
        latest_generation,
        target_generation,
        delivered,
        hosts_total,
        hosts_at_target,
        hosts_pending,
        hosts_quarantined,
        sites,
    })
}

async fn spawn_immediate_push(ctx: &Arc<DeployContext>) {
    ctx.deployment_cache.refresh().await;
    for site in &ctx.sites {
        let pool = site.ssh_pool().clone();
        let cache = ctx.deployment_cache.clone();
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
            crate::site::push_to_all(&pool, &cache, addrs).await;
        });
    }
}
