//! Resolves where a project's frontend assets will be served from.
//!
//! Called before the build, not after: the base URL is compiled into the
//! bundle, so by the time `deploy` hands back upload URLs it is already too
//! late to learn it.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Input<'a> {
    project_id: &'a str,
    code_version: u64,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum Response {
    Ok { static_base_url: String },
    NotLoggedIn,
    NotFound,
    InternalError { reason: String },
}

pub async fn resolve_asset_base_url(project_id: &str, code_version: u64) -> Result<String> {
    let creds = crate::credentials::require()?;
    let url = format!(
        "{}/__forte_action/project_asset_origin",
        creds.control_url.trim_end_matches('/')
    );
    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&Input {
            project_id,
            code_version,
        })
        .send()
        .await?
        .error_for_status()?;
    match response.json::<Response>().await? {
        Response::Ok { static_base_url } => Ok(static_base_url),
        Response::NotLoggedIn => Err(anyhow!("control rejected token; sign in again.")),
        Response::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        Response::InternalError { reason } => {
            Err(anyhow!("could not resolve the asset origin: {reason}"))
        }
    }
}
