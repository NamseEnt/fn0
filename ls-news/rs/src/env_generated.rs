#![allow(dead_code)]
pub fn public_github_client_id() -> &'static str {
    static PUBLIC_GITHUB_CLIENT_ID: std::sync::LazyLock<String> = std::sync::LazyLock::new(||
    { std::env::var("PUBLIC_GITHUB_CLIENT_ID").unwrap() });
    &PUBLIC_GITHUB_CLIENT_ID
}
pub fn github_client_secret() -> &'static str {
    static GITHUB_CLIENT_SECRET: std::sync::LazyLock<String> = std::sync::LazyLock::new(||
    { std::env::var("GITHUB_CLIENT_SECRET").unwrap() });
    &GITHUB_CLIENT_SECRET
}
pub fn turso_url() -> &'static str {
    static TURSO_URL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        std::env::var("TURSO_URL").unwrap()
    });
    &TURSO_URL
}
pub fn turso_auth_token() -> &'static str {
    static TURSO_AUTH_TOKEN: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        std::env::var("TURSO_AUTH_TOKEN").unwrap()
    });
    &TURSO_AUTH_TOKEN
}
pub fn cookie_secret() -> &'static str {
    static COOKIE_SECRET: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        std::env::var("COOKIE_SECRET").unwrap()
    });
    &COOKIE_SECRET
}
