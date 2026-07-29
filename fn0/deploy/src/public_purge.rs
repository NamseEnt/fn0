use crate::credentials;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct PublicPurgeInput<'a> {
    project_id: &'a str,
    keys: &'a [String],
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum PublicPurgeResponse {
    Ok { urls: Vec<String> },
    NotLoggedIn,
    NotFound,
    Forbidden,
    TooManyKeys { max: usize },
    InternalError { reason: String },
}

/// Queues an edge invalidation for each key in the project's public namespace.
/// Returns the URLs that were queued.
pub async fn public_purge(project_id: &str, keys: &[String]) -> Result<Vec<String>> {
    let creds = credentials::require()?;
    let url = format!(
        "{}/__forte_action/public_purge",
        creds.control_url.trim_end_matches('/')
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let raw: PublicPurgeResponse = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&PublicPurgeInput { project_id, keys })
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("public purge control call failed: {e}"))?
        .json()
        .await?;

    match raw {
        PublicPurgeResponse::Ok { urls } => Ok(urls),
        PublicPurgeResponse::NotLoggedIn => {
            Err(anyhow!("control rejected token; run `fn0 login` again."))
        }
        PublicPurgeResponse::NotFound => Err(anyhow!("project '{project_id}' not found.")),
        PublicPurgeResponse::Forbidden => Err(anyhow!(
            "project '{project_id}' is not owned by the signed-in user."
        )),
        PublicPurgeResponse::TooManyKeys { max } => {
            Err(anyhow!("too many keys in one call; the limit is {max}."))
        }
        PublicPurgeResponse::InternalError { reason } => {
            Err(anyhow!("control public_purge internal error: {reason}"))
        }
    }
}
