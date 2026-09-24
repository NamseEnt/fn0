use crate::common::auth;
use crate::docs::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const CODE_TTL_SECS: i64 = 300;

#[derive(Deserialize)]
pub struct Input {
    pub response_mode: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub state: Option<String>,
    pub label: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        code: String,
        redirect_to: Option<String>,
    },
    NotLoggedIn,
    InvalidRequest {
        message: String,
    },
    Error {
        message: String,
    },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::current_user(req.jar).await else {
        return Output::NotLoggedIn;
    };

    let manual_code = match req.body.response_mode.as_deref() {
        None => false,
        Some("code") => true,
        Some(_) => {
            return Output::InvalidRequest {
                message: "unsupported response_mode".to_string(),
            };
        }
    };
    if req.body.code_challenge_method != "S256" {
        return Output::InvalidRequest {
            message: "code_challenge_method must be S256".to_string(),
        };
    }
    if req.body.code_challenge.is_empty() {
        return Output::InvalidRequest {
            message: "code_challenge is required".to_string(),
        };
    }
    if manual_code {
        if req.body.redirect_uri.is_some() || req.body.state.is_some() {
            return Output::InvalidRequest {
                message: "manual code mode cannot include a redirect".to_string(),
            };
        }
    } else {
        let Some(redirect_uri) = req.body.redirect_uri.as_deref() else {
            return Output::InvalidRequest {
                message: "redirect_uri is required".to_string(),
            };
        };
        if !auth::is_loopback_redirect(redirect_uri) {
            return Output::InvalidRequest {
                message: "redirect_uri must be http loopback (127.0.0.1, localhost, or [::1])"
                    .to_string(),
            };
        }
        if req.body.state.as_deref().is_none_or(str::is_empty) {
            return Output::InvalidRequest {
                message: "state is required".to_string(),
            };
        }
    }

    let label = req.body.label.trim().to_string();
    if label.is_empty() {
        return Output::InvalidRequest {
            message: "label cannot be empty".to_string(),
        };
    }

    let code_bytes = rand::get_random_bytes(32);
    let code = URL_SAFE_NO_PAD.encode(&code_bytes);

    let now = forte_sdk::now();
    let expires_at = now + forte_sdk::chrono::Duration::seconds(CODE_TTL_SECS);

    let db = doc_db::database();
    let put_result = CliAuthorizationCodeDocPut(CliAuthorizationCodeDoc {
        code: code.clone(),
        github_id: user.github_id,
        code_challenge: req.body.code_challenge.clone(),
        redirect_uri: req.body.redirect_uri.clone(),
        label,
        expires_at,
    })
    .send_with(&db)
    .await;
    if let Err(e) = put_result {
        return Output::Error {
            message: e.to_string(),
        };
    }

    let redirect_to = if manual_code {
        None
    } else {
        let redirect_uri = req.body.redirect_uri.as_deref().unwrap();
        let state = req.body.state.as_deref().unwrap();
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("code", &code)
            .append_pair("state", state)
            .finish();
        let separator = if redirect_uri.contains('?') { '&' } else { '?' };
        Some(format!("{}{}{}", redirect_uri, separator, query))
    };

    Output::Ok { code, redirect_to }
}
