use std::sync::Arc;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, body::Bytes};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::args_parse::DeployContext;
use crate::deploy::{ErrorResponse, json_response, verify_github_user};

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_TTL_SECONDS: u64 = 300;
const MAX_TTL_SECONDS: u64 = 600;

#[derive(Deserialize)]
struct AdminGrantRequest {
    github_token: String,
    project_name: String,
    task: String,
    #[serde(default)]
    ttl_seconds: Option<u64>,
}

#[derive(Serialize)]
struct AdminGrantResponse {
    token: String,
    subdomain: String,
    expires_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AdminTokenClaims {
    pub subdomain: String,
    pub task: String,
    pub github_login: String,
    pub iat: i64,
    pub exp: i64,
}

pub fn sign_token(claims: &AdminTokenClaims, key: &[u8; 32]) -> String {
    let payload_json = serde_json::to_vec(claims).expect("claims serializable");
    let payload_b64 = URL_SAFE_NO_PAD.encode(&payload_json);

    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
    mac.update(payload_b64.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig);

    format!("{payload_b64}.{sig_b64}")
}

pub async fn handle_admin_grant(
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

    let request: AdminGrantRequest = match serde_json::from_slice(&body) {
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
                    error: format!("Failed to get project: {e}"),
                },
            );
        }
    };

    let now = chrono::Utc::now().timestamp();
    let ttl = request
        .ttl_seconds
        .unwrap_or(DEFAULT_TTL_SECONDS)
        .min(MAX_TTL_SECONDS);
    let exp = now + ttl as i64;

    let claims = AdminTokenClaims {
        subdomain: project.subdomain.clone(),
        task: request.task,
        github_login: username,
        iat: now,
        exp,
    };

    let token = sign_token(&claims, &ctx.admin_signing_key);

    json_response(
        200,
        &AdminGrantResponse {
            token,
            subdomain: project.subdomain,
            expires_at: exp,
        },
    )
}
