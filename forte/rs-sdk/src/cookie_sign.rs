use cookie::{Cookie, CookieJar};
use hmac::{Hmac, Mac};
use serde::{Serialize, de::DeserializeOwned};
use sha2::Sha256;
use std::sync::OnceLock;

type HmacSha256 = Hmac<Sha256>;

fn get_secret() -> &'static [u8] {
    static SECRET: OnceLock<String> = OnceLock::new();
    SECRET
        .get_or_init(|| std::env::var("COOKIE_SECRET").expect("COOKIE_SECRET NOT SET"))
        .as_bytes()
}

pub fn sign_cookie<T: Serialize>(jar: &mut CookieJar, name: &str, value: &T) {
    let value = serde_json::to_string(value).expect("Fail to serialize value");

    let mut mac = HmacSha256::new_from_slice(get_secret()).expect("HMAC can take key of any size");
    mac.update(value.as_bytes());

    let cookie_value = format!("{value}.{}", hex::encode(mac.finalize().into_bytes()));

    let cookie = Cookie::build((name.to_string(), cookie_value))
        .http_only(true)
        .secure(true)
        .path("/")
        .build();

    jar.add(cookie);
}

pub fn unsign_cookie<T: DeserializeOwned>(jar: &CookieJar, name: &str) -> Option<T> {
    let cookie = jar.get(name)?;
    let (value, signature_hex) = cookie.value().rsplit_once(".")?;

    let mut mac = HmacSha256::new_from_slice(get_secret()).ok()?;
    mac.update(value.as_bytes());

    let signature_bytes = hex::decode(signature_hex).ok()?;

    if mac.verify_slice(&signature_bytes).is_err() {
        return None;
    }

    serde_json::from_str(value).ok()
}
