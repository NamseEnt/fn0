use crate::credentials;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct StaticPagePurgeInput<'a> {
    project_id: &'a str,
    paths: &'a [String],
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum StaticPagePurgeResponse {
    Ok { paths: Vec<String> },
    NotLoggedIn,
    NotFound,
    Forbidden,
    NoDomain,
    TooManyPaths { max: usize },
    UnusablePath { path: String, reason: String },
    InternalError { reason: String },
}

/// Queues an edge invalidation for each `cache_static` page path. Returns the
/// paths that were queued.
pub async fn static_page_purge(project_id: &str, paths: &[String]) -> Result<Vec<String>> {
    let creds = credentials::require()?;
    let url = format!(
        "{}/__forte_action/static_page_purge",
        creds.control_url.trim_end_matches('/')
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let raw: StaticPagePurgeResponse = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&StaticPagePurgeInput { project_id, paths })
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("static page purge control call failed: {e}"))?
        .json()
        .await?;

    match raw {
        StaticPagePurgeResponse::Ok { paths } => Ok(paths),
        StaticPagePurgeResponse::NotLoggedIn => {
            Err(anyhow!("control rejected token; run `fn0 login` again."))
        }
        StaticPagePurgeResponse::NotFound => Err(anyhow!("project '{project_id}' not found.")),
        StaticPagePurgeResponse::Forbidden => Err(anyhow!(
            "project '{project_id}' is not owned by the signed-in user."
        )),
        StaticPagePurgeResponse::NoDomain => Err(anyhow!(
            "project '{project_id}' has no domain, so no edge holds its pages."
        )),
        StaticPagePurgeResponse::TooManyPaths { max } => {
            Err(anyhow!("too many paths in one call; the limit is {max}."))
        }
        StaticPagePurgeResponse::UnusablePath { path, reason } => {
            Err(anyhow!("'{path}' is not a page path: {reason}."))
        }
        StaticPagePurgeResponse::InternalError { reason } => Err(anyhow!(
            "control static_page_purge internal error: {reason}"
        )),
    }
}
