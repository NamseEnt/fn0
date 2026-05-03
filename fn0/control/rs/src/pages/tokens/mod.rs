use crate::common::auth;
use crate::route_generated::Redirect;
use forte_sdk::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Props {
    pub github_id: i64,
    pub github_login: String,
}

pub async fn handler(req: ForteRequest<'_>) -> anyhow::Result<Props> {
    let Some(user) = auth::current_user(req.jar).await else {
        return Err(Redirect::Login.into());
    };
    Ok(Props {
        github_id: user.github_id,
        github_login: user.github_login,
    })
}
