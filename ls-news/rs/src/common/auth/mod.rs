mod oauth;

use forte_sdk::*;
pub use oauth::*;
use serde::{Deserialize, Serialize};

const USER_COOKIE: &str = "USER";
const ADMIN_ID: &str = "3580430";

#[derive(Serialize, Deserialize)]
pub struct CookieUser {
    pub user_id: String,
    pub username: String,
    pub avatar_url: String,
}

pub fn is_admin(jar: &CookieJar) -> bool {
    let Some(user) = forte_sdk::cookie_sign::unsign_cookie::<CookieUser>(jar, USER_COOKIE) else {
        return false;
    };
    user.user_id == ADMIN_ID
}

pub fn get_me(jar: &mut CookieJar) -> Option<CookieUser> {
    forte_sdk::cookie_sign::unsign_cookie::<CookieUser>(jar, USER_COOKIE)
}
