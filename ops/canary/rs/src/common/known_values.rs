//! The values the canary probes compare against. `seed_known_values` is the
//! only writer; a probe that finds them missing or different reports it and
//! leaves them as they are, so a lost or corrupted dependency stays visible.

use forte_sdk::ForteResponse;
use forte_sdk::http::Response;

pub const DOC_DB_PARTITION_KEY: &str = "fn0-ops-canary/known-value";
pub const DOC_DB_SORT_KEY: &str = "health";
pub const DOC_DB_VALUE: &[u8] = b"fn0-canary-v1";

pub const PRIVATE_OBJECT_KEY: &str = "canary/known-object-v1.txt";
pub const PRIVATE_OBJECT_CONTENT_TYPE: &str = "text/plain";
pub const PRIVATE_OBJECT_BODY: &[u8] = b"fn0-canary-v1\n";

pub enum ProbeFailure {
    Missing,
    Mismatch,
    Unavailable,
}

impl ProbeFailure {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Mismatch => "mismatch",
            Self::Unavailable => "unavailable",
        }
    }
}

pub fn probe_response(result: Result<(), ProbeFailure>) -> anyhow::Result<ForteResponse> {
    let (status, body) = match result {
        Ok(()) => (200, r#"{"ok":true}"#.to_string()),
        Err(failure) => (
            503,
            format!(r#"{{"ok":false,"failure":"{}"}}"#, failure.as_str()),
        ),
    };
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("cache-control", "no-store")
        .body(body.into())?)
}
