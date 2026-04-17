use crate::common::auth::{get_me, prepare_github_login};
use crate::route_generated::Redirect;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    LoggedIn { user: User },
    NotLoggedIn { oauth_nonce: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if let Some(u) = get_me(req.jar) {
        Output::LoggedIn {
            user: User {
                id: u.user_id,
                username: u.username,
                avatar_url: u.avatar_url,
            },
        }
    } else {
        let oauth_nonce = prepare_github_login(req.jar, Redirect::Index);
        Output::NotLoggedIn { oauth_nonce }
    }
}

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub avatar_url: String,
}
