use crate::route_generated::Redirect;
use anyhow::Result;
use cookie::CookieJar;
use forte_sdk::*;
use http::HeaderMap;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok {},
    NotLoggedIn { oauth_nonce: String },
}

pub async fn handler(_headers: HeaderMap, jar: &mut CookieJar) -> Result<Props> {
    if crate::common::auth::get_me(jar).is_some() {
        return Ok(Props::Ok {});
    }

    let oauth_nonce = crate::common::auth::prepare_github_login(jar, Redirect::Write);
    Ok(Props::NotLoggedIn { oauth_nonce })
}
