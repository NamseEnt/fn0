use crate::{common::auth::GitHubUser, docs::UserDoc, route_generated::Redirect};
use anyhow::Result;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

pub struct SearchParams {
    pub code: String,
    pub state: String,
}

#[derive(Serialize)]
pub enum Props {}

pub async fn handler(req: ForteRequest<'_>, search_params: SearchParams) -> Result<Props> {
    let client_id = crate::env_generated::public_github_client_id();
    let client_secret = crate::env_generated::github_client_secret();

    let response = http::Client::new()
        .send(
            http::Request::builder()
                .uri("https://github.com/login/oauth/access_token")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .header("User-Agent", "ls-news")
                .body(
                    serde_json::json!({
                        "client_id": client_id,
                        "client_secret": client_secret,
                        "code": search_params.code,
                    })
                    .to_string(),
                )?,
        )
        .await?;

    if !response.status().is_success() {
        eprintln!("Failed to exchange code for token");
        return Err(Redirect::Index.into());
    }

    #[derive(Deserialize)]
    struct TokenData {
        access_token: String,
    }

    let TokenData { access_token } = response.into_body().json().await?;

    let response = http::Client::new()
        .send(
            http::Request::builder()
                .uri("https://api.github.com/user")
                .method("GET")
                .header("Authorization", format!("Bearer {access_token}"))
                .header("Accept", "application/json")
                .header("User-Agent", "ls-news")
                .body(())?,
        )
        .await?;

    if !response.status().is_success() {
        eprintln!("Failed to fetch user info");
        return Err(Redirect::Index.into());
    }

    let github_user: GitHubUser = response.into_body().json().await?;

    UserDoc::put(
        github_user.id.to_string(),
        &UserDoc {
            id: github_user.id.to_string(),
            username: github_user.login.clone(),
            avatar_url: github_user.avatar_url.clone(),
            created_at: now(),
            updated_at: now(),
        },
    )
    .await?;

    crate::common::auth::set_auth_cookies(req.jar, &access_token, &github_user);

    match crate::common::auth::verify_oauth_state(req.jar, &search_params.state) {
        Ok(redirect) => Err(redirect.into()),
        Err(err) => {
            eprintln!("GitHub OAuth error: {err}");
            Err(Redirect::Index.into())
        }
    }
}
