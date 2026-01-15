use super::*;
use crate::route_generated::Redirect;
use forte_sdk::{
    anyhow::{Result, bail},
    *,
};
use serde::{Deserialize, Serialize};

const OAUTH_STATE_COOKIE: &str = "OAUTH_STATE";
const GITHUB_TOKEN_COOKIE: &str = "GITHUB_TOKEN";

#[derive(Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
}

pub fn set_auth_cookies(jar: &mut CookieJar, token: &str, user: &GitHubUser) {
    let max_age = time::Duration::seconds(60 * 60 * 24 * 365);

    jar.add(
        CookieBuilder::new(GITHUB_TOKEN_COOKIE, token.to_string())
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Lax)
            .path("/")
            .max_age(max_age),
    );

    let user_info = CookieUser {
        user_id: user.id.to_string(),
        username: user.login.clone(),
        avatar_url: user.avatar_url.clone(),
    };

    forte_sdk::cookie_sign::sign_cookie(jar, USER_COOKIE, &user_info);
}

pub fn clear_auth_cookies(jar: &mut CookieJar) {
    jar.remove(CookieBuilder::new(GITHUB_TOKEN_COOKIE, "").path("/"));
    jar.remove(CookieBuilder::new(USER_COOKIE, "").path("/"));
}

fn is_allowed_redirect_on_oauth(redirect: &Redirect) -> bool {
    match redirect {
        Redirect::External { .. } | Redirect::ApiAuthCallbackGithub | Redirect::Signout => false,
        Redirect::Index | Redirect::Post_id_ { .. } | Redirect::Write => true,
    }
}

#[derive(Serialize, Deserialize)]
struct OauthState {
    nonce: String,
    redirect: Redirect,
    timestamp: i64,
}

pub fn verify_oauth_state(jar: &mut CookieJar, state_from_url: &str) -> Result<Redirect> {
    let state_cookie = jar
        .get(OAUTH_STATE_COOKIE)
        .ok_or(anyhow::anyhow!("State cookie not found"))?;

    let OauthState {
        nonce,
        redirect,
        timestamp,
    } = serde_json::from_str(state_cookie.value())
        .ok()
        .ok_or(anyhow::anyhow!("Invalid state cookie"))?;

    if nonce != state_from_url {
        bail!("Invalid state cookie");
    }

    let ten_minutes = 60 * 10;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    if now - timestamp > ten_minutes {
        bail!("Expired state cookie");
    }

    if !is_allowed_redirect_on_oauth(&redirect) {
        bail!("Invalid redirect path");
    }

    jar.remove(OAUTH_STATE_COOKIE);
    Ok(redirect)
}

pub fn prepare_github_login(jar: &mut CookieJar, mut redirect: Redirect) -> String {
    if !is_allowed_redirect_on_oauth(&redirect) {
        redirect = Redirect::Index;
    }

    let mut buf = [0u8; 32];
    rand::get_random_bytes(&mut buf);
    let nonce = hex::encode(buf);

    let state_data = serde_json::to_string(&OauthState {
        nonce: nonce.clone(),
        redirect,
        timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
    })
    .expect("Failed to serialize state data");

    jar.add(
        CookieBuilder::new(OAUTH_STATE_COOKIE, state_data)
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Lax)
            .path("/")
            .max_age(time::Duration::seconds(60 * 10)),
    );

    nonce
}
