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
}

#[derive(Serialize)]
struct DeployStartResponse {
    presigned_url: String,
    deploy_job_id: String,
}

#[derive(Deserialize)]
struct DeployFinishRequest {
    github_token: String,
    #[allow(dead_code)]
    deploy_job_id: String,
    code_id: u64,
    code_version: u64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
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

    if username != "namse" {
        return json_response(403, &ErrorResponse { error: "Unauthorized user".to_string() });
    }

    let deploy_job_id = uuid::Uuid::new_v4().to_string();
    let s3_key = format!("{}.wasm", deploy_job_id);

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

    let username = match verify_github_user(&request.github_token).await {
        Ok(u) => u,
        Err(e) => return json_response(401, &ErrorResponse { error: e }),
    };

    if username != "namse" {
        return json_response(403, &ErrorResponse { error: "Unauthorized user".to_string() });
    }

    if let Err(e) = ctx.doc_db.insert_deployment(request.code_id, request.code_version).await {
        return json_response(500, &ErrorResponse { error: format!("Failed to insert deployment: {}", e) });
    }

    json_response(200, &serde_json::json!({"ok": true}))
}
