use std::sync::Arc;

use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::MetadataDirective;
use http_body_util::BodyExt;
use hyper::{Request, Response, body::Bytes};
use http_body_util::Full;
use serde::{Deserialize, Serialize};

use crate::args_parse::DeployContext;
use crate::wasmtime;

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
}

#[derive(Deserialize)]
struct DeployFinishRequest {
    github_token: String,
    deploy_job_id: String,
    subdomain: String,
    code_id: u64,
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
    let s3_key = format!("uploads/{deploy_job_id}/bundle.raw.tar");

    if let Err(e) = ctx
        .doc_db
        .insert_deploy_upload_session(
            &deploy_job_id,
            project.code_id,
            &project.subdomain,
            &s3_key,
        )
        .await
    {
        return json_response(500, &ErrorResponse {
            error: format!("Failed to store deploy session: {}", e),
        });
    }

    let presigning_config = match PresigningConfig::expires_in(std::time::Duration::from_secs(300)) {
        Ok(c) => c,
        Err(_) => return json_response(500, &ErrorResponse { error: "Failed to create presigning config".to_string() }),
    };

    let presigned = match ctx
        .s3_client
        .put_object()
        .bucket(&ctx.wasm_bucket)
        .key(&s3_key)
        .presigned(presigning_config)
        .await
    {
        Ok(p) => p,
        Err(e) => return json_response(500, &ErrorResponse { error: format!("Failed to generate presigned URL: {}", e) }),
    };

    json_response(200, &DeployStartResponse {
        presigned_url: presigned.uri().to_string(),
        deploy_job_id,
        subdomain: project.subdomain,
        code_id: project.code_id,
    })
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

    let code_version = match ctx.doc_db.next_code_version(request.code_id).await {
        Ok(v) => v,
        Err(e) => return json_response(500, &ErrorResponse { error: format!("Failed to get next version: {}", e) }),
    };

    let session = match ctx
        .doc_db
        .get_deploy_upload_session(&request.deploy_job_id)
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) => {
            return json_response(400, &ErrorResponse {
                error: "Unknown deploy_job_id".to_string(),
            });
        }
        Err(e) => {
            return json_response(500, &ErrorResponse {
                error: format!("Failed to read deploy session: {}", e),
            });
        }
    };

    if session.code_id != request.code_id || session.subdomain != request.subdomain {
        return json_response(400, &ErrorResponse {
            error: "deploy_job_id does not match code_id/subdomain".to_string(),
        });
    }

    let source_bundle_key = format!("sources/{}/{}/bundle.raw.tar", request.code_id, code_version);
    if let Err(e) = ctx
        .s3_client
        .copy_object()
        .bucket(&ctx.wasm_bucket)
        .copy_source(format!("{}/{}", ctx.wasm_bucket, session.source_bundle_key))
        .key(&source_bundle_key)
        .metadata_directive(MetadataDirective::Copy)
        .send()
        .await
    {
        return json_response(500, &ErrorResponse {
            error: format!("Failed to persist source bundle: {}", e),
        });
    }

    let compile_targets = match wasmtime::compile_targets_for_deploy(
        &ctx,
        request.code_id,
        &request.subdomain,
        code_version,
        &source_bundle_key,
    )
    .await
    {
        Ok(targets) => targets,
        Err(e) => {
            return json_response(500, &ErrorResponse {
                error: format!("Failed to determine compile targets: {}", e),
            });
        }
    };

    if let Err(e) = ctx
        .doc_db
        .finalize_deploy(
            &request.deploy_job_id,
            &request.subdomain,
            request.code_id,
            code_version,
            &source_bundle_key,
            &compile_targets,
        )
        .await
    {
        return json_response(500, &ErrorResponse {
            error: format!("Failed to finalize deployment: {}", e),
        });
    }

    json_response(200, &serde_json::json!({"ok": true, "code_version": code_version}))
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

    json_response(200, &serde_json::json!({"ok": true, "subdomain": project.subdomain}))
}
