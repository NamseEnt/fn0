use crate::common::dodb_write_health::{self, Failure};
use crate::common::known_values::ProbeFailure;
use anyhow::Result;
use forte_sdk::http::Response;
use forte_sdk::{ForteRequest, ForteResponse};
use std::fmt::Write;

pub type Props = ForteResponse;

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    let now_unix_ms = match dodb_write_health::unix_time_millis() {
        Ok(now_unix_ms) => now_unix_ms,
        Err(error) => {
            eprintln!("dodb write canary clock failed: {error:?}");
            return response(Err(ProbeFailure::Unavailable));
        }
    };
    let nonce = nonce()?;
    match dodb_write_health::probe(&doc_db::database(), now_unix_ms, nonce).await {
        Ok(success) => response(Ok(success)),
        Err(failure) => {
            eprintln!("dodb write canary failed: {failure:?}");
            response(Err(match failure {
                Failure::WriteFailed => ProbeFailure::WriteFailed,
                Failure::ReadFailed => ProbeFailure::ReadFailed,
                Failure::Mismatch => ProbeFailure::Mismatch,
            }))
        }
    }
}

fn nonce() -> Result<String> {
    let random_bytes = forte_sdk::rand::get_random_bytes(16);
    let mut nonce = String::with_capacity(random_bytes.len() * 2);
    for random_byte in random_bytes {
        write!(&mut nonce, "{random_byte:02x}")?;
    }
    Ok(nonce)
}

fn response(
    result: std::result::Result<dodb_write_health::Success, ProbeFailure>,
) -> Result<Props> {
    let (status, body) = match result {
        Ok(success) => (
            200,
            serde_json::json!({
                "ok": true,
                "write_performed": success.write_performed,
                "write_age_seconds": success.write_age_seconds,
            })
            .to_string(),
        ),
        Err(failure) => (
            503,
            serde_json::json!({
                "ok": false,
                "failure": failure.as_str(),
            })
            .to_string(),
        ),
    };
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("cache-control", "no-store")
        .body(body.into())?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn generated_route_uses_the_public_hyphenated_path() {
        let generated_routes = include_str!("../route_generated.rs");
        assert!(generated_routes.contains("if path == \"/api/dodb-write\""));
        assert!(!generated_routes.contains("if path == \"/api/dodb_write\""));
    }
}
