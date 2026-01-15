pub mod cookie_sign;
mod generate_env;
mod generate_routes;

pub use anyhow;
pub use generate_env::generate_env;
pub use cookie::{self, Cookie, CookieBuilder, CookieJar};
pub use forte_db;
pub use forte_json;
pub use generate_routes::*;
pub use serde;
pub use serde_json;
pub use sha2;
pub type DateTime = chrono::DateTime<chrono::Utc>;
pub use futures;
pub use hex;
pub use time;
pub mod http_header {
    pub use http::header::*;
}
pub use wstd::{self, future, http, io, iter, net, rand, runtime, task};

pub fn now() -> DateTime {
    chrono::Utc::now()
}

pub struct ForteRequest<'a, Body = ()> {
    pub uri_authority: &'a str,
    pub headers: &'a http::HeaderMap,
    pub jar: &'a mut CookieJar,
    pub body: Body,
}
