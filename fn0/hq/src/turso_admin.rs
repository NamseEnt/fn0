use color_eyre::eyre::{Result, eyre};
use reqwest::StatusCode;

use crate::args::ForteDbArgs;

/// Ensure a Turso database exists in the doc-db group. Idempotent: a 409
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

pub async fn create_database_from(
    args: &ForteDbArgs,
    new_name: &str,
    source_name: &str,
) -> Result<()> {
    let url = format!(
        "https://api.turso.tech/v1/organizations/{}/databases",
        args.organization_slug
    );
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .bearer_auth(&args.api_token)
        .json(&serde_json::json!({
            "name": new_name,
            "group": args.group_name,
            "seed": {
                "type": "database",
                "name": source_name,
            },
        }))
        .send()
        .await
        .map_err(|e| eyre!("turso create-from request failed: {e}"))?;

    match resp.status() {
        StatusCode::OK | StatusCode::CREATED => Ok(()),
        StatusCode::CONFLICT => Ok(()),
        status => {
            let body = resp.text().await.unwrap_or_default();
            Err(eyre!(
                "turso create-from '{new_name}' (seed='{source_name}') failed: {status} {body}"
            ))
        }
    }
}

pub async fn set_block_writes(args: &ForteDbArgs, name: &str, blocked: bool) -> Result<()> {
    let url = format!(
        "https://api.turso.tech/v1/organizations/{}/databases/{}/configuration",
        args.organization_slug, name
    );
    let client = reqwest::Client::new();
    let resp = client
        .patch(&url)
        .bearer_auth(&args.api_token)
        .json(&serde_json::json!({ "block_writes": blocked }))
        .send()
        .await
        .map_err(|e| eyre!("turso configuration patch request failed: {e}"))?;

    match resp.status() {
        StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
        status => {
            let body = resp.text().await.unwrap_or_default();
            Err(eyre!(
                "turso set_block_writes '{name}' (blocked={blocked}) failed: {status} {body}"
            ))
        }
    }
}

pub async fn destroy_database(args: &ForteDbArgs, name: &str) -> Result<()> {
    let url = format!(
        "https://api.turso.tech/v1/organizations/{}/databases/{}",
        args.organization_slug, name
    );
    let client = reqwest::Client::new();
    let resp = client
        .delete(&url)
        .bearer_auth(&args.api_token)
        .send()
        .await
        .map_err(|e| eyre!("turso destroy database request failed: {e}"))?;

    match resp.status() {
        StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
        StatusCode::NOT_FOUND => Ok(()),
        status => {
            let body = resp.text().await.unwrap_or_default();
            Err(eyre!(
                "turso destroy database '{name}' failed: {status} {body}"
            ))
        }
    }
}
