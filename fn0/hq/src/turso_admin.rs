use color_eyre::eyre::{Result, eyre};
use reqwest::StatusCode;

use crate::args::ForteDbArgs;

/// Ensure a Turso database exists in the forte-db group. Idempotent: a 409
/// response from the Turso API (database already exists) is treated as success.
pub async fn ensure_database(args: &ForteDbArgs, name: &str) -> Result<()> {
    let url = format!(
        "https://api.turso.tech/v1/organizations/{}/databases",
        args.organization_slug
    );
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .bearer_auth(&args.api_token)
        .json(&serde_json::json!({
            "name": name,
            "group": args.group_name,
        }))
        .send()
        .await
        .map_err(|e| eyre!("turso create database request failed: {e}"))?;

    match resp.status() {
        StatusCode::OK | StatusCode::CREATED => Ok(()),
        StatusCode::CONFLICT => Ok(()),
        status => {
            let body = resp.text().await.unwrap_or_default();
            Err(eyre!(
                "turso create database '{name}' failed: {status} {body}"
            ))
        }
    }
}
