use crate::route_generated::Redirect;
use anyhow::Result;
use forte_sdk::*;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok {},
    NotLoggedIn { oauth_nonce: String },
}

pub async fn handler(req: ForteRequest<'_>) -> Result<Props> {
    if crate::common::auth::get_me(req.jar).is_some() {
        return Ok(Props::Ok {});
    }

    let oauth_nonce = crate::common::auth::prepare_github_login(req.jar, Redirect::Write);
    Ok(Props::NotLoggedIn { oauth_nonce })
}
