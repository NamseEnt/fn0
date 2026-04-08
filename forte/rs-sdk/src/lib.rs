pub mod cookie_sign;
mod generate_env;
mod generate_routes;

pub use anyhow;
pub use cookie::{self, Cookie, CookieBuilder, CookieJar};
pub use forte_db;
pub use forte_db::DbRequest;
pub use forte_json;
pub use generate_env::generate_env;
pub use generate_routes::*;
pub use serde;
pub use serde_json;
pub use sha2;
pub use forte_macros::forte_doc;
pub type DateTime = chrono::DateTime<chrono::Utc>;
pub use form_urlencoded;
pub use futures;
pub use hex;
pub use time;
pub mod http_header {
    pub use http::header::*;
}
pub use uuid::Uuid;
pub use wstd::{self, future, http, io, iter, net, rand, runtime, task};

pub fn now() -> DateTime {
    chrono::Utc::now()
}

pub struct ForteRequest<'a, Body = ()> {
    pub uri_authority: &'a str,
    pub method: &'a http::Method,
    pub headers: &'a http::HeaderMap,
    pub jar: &'a mut CookieJar,
    pub raw_body: &'a [u8],
    pub body: Body,
}
