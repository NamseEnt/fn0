use crate::common::auth;
use crate::route_generated::Redirect;
use forte_sdk::*;
use serde::Serialize;

pub struct SearchParams {
    pub response_mode: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub state: Option<String>,
    pub label: String,
}

#[derive(Serialize)]
pub struct Props {
    pub github_login: String,
    pub manual_code: bool,
    pub redirect_uri: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub state: Option<String>,
    pub default_label: String,
}

pub async fn handler(req: ForteRequest<'_>, search_params: SearchParams) -> anyhow::Result<Props> {
    let manual_code = match search_params.response_mode.as_deref() {
        None => false,
        Some("code") => true,
        Some(_) => anyhow::bail!("unsupported response_mode"),
    };
    if search_params.code_challenge_method != "S256" {
        anyhow::bail!("code_challenge_method must be S256");
    }
    if search_params.code_challenge.is_empty() {
        anyhow::bail!("code_challenge is required");
    }
    if manual_code {
        if search_params.redirect_uri.is_some() || search_params.state.is_some() {
            anyhow::bail!("manual code mode cannot include a redirect");
        }
    } else {
        let redirect_uri = search_params
            .redirect_uri
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("redirect_uri is required"))?;
        if !auth::is_loopback_redirect(redirect_uri) {
            anyhow::bail!("redirect_uri must be http loopback (127.0.0.1, localhost, or [::1])");
        }
        if search_params.state.as_deref().is_none_or(str::is_empty) {
            anyhow::bail!("state is required");
        }
    }

    let Some(user) = auth::current_user(req.jar).await else {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        if let Some(response_mode) = search_params.response_mode.as_deref() {
            serializer.append_pair("response_mode", response_mode);
        }
        if let Some(redirect_uri) = search_params.redirect_uri.as_deref() {
            serializer.append_pair("redirect_uri", redirect_uri);
        }
        serializer
            .append_pair("code_challenge", &search_params.code_challenge)
            .append_pair(
                "code_challenge_method",
                &search_params.code_challenge_method,
            );
        if let Some(state) = search_params.state.as_deref() {
            serializer.append_pair("state", state);
        }
        let query = serializer
            .append_pair("label", &search_params.label)
            .finish();
        let url = format!("/oauth/cli/authorize?{query}");
        auth::stash_pending_cli_consent(req.jar, &url);
        return Err(Redirect::Login.into());
    };

    Ok(Props {
        github_login: user.github_login,
        manual_code,
        redirect_uri: search_params.redirect_uri,
        code_challenge: search_params.code_challenge,
        code_challenge_method: search_params.code_challenge_method,
        state: search_params.state,
        default_label: search_params.label,
    })
}
