use crate::credentials;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

pub struct AdminRunOutput {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Serialize)]
struct AdminRunInput<'a> {
    project_id: &'a str,
    task: &'a str,
    input: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum AdminRunResponse {
    Ok {
        status: u16,
        #[serde(default)]
        content_type: Option<String>,
        #[serde(default)]
        body: String,
    },
    NotLoggedIn,
    NotFound,
    Forbidden,
    NotDeployed,
    UpstreamError {
        status: u16,
        #[serde(default)]
        content_type: Option<String>,
        #[serde(default)]
        body: String,
    },
    InternalError {
        reason: String,
    },
}

pub async fn admin_run(
    project_id: &str,
    task: &str,
    input_body: Vec<u8>,
    timeout_secs: u64,
) -> Result<AdminRunOutput> {
    let creds = credentials::require()?;

    let input: serde_json::Value = if input_body.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&input_body)
            .map_err(|e| anyhow!("admin run input is not valid JSON: {e}"))?
    };

    let url = format!(
        "{}/__forte_action/admin_run",
        creds.control_url.trim_end_matches('/')
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()?;

    let raw: AdminRunResponse = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&AdminRunInput {
            project_id,
            task,
            input,
        })
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("admin run control call failed: {e}"))?
        .json()
        .await?;

    match raw {
        AdminRunResponse::Ok {
            status,
            content_type,
            body,
        } => Ok(AdminRunOutput {
            status,
            content_type,
            body: body.into_bytes(),
        }),
        AdminRunResponse::UpstreamError {
            status,
            content_type,
            body,
        } => Ok(AdminRunOutput {
            status,
            content_type,
            body: body.into_bytes(),
        }),
        AdminRunResponse::NotLoggedIn => {
            Err(anyhow!("control rejected token; run `fn0 login` again."))
        }
        AdminRunResponse::NotFound => Err(anyhow!("project '{project_id}' not found.")),
        AdminRunResponse::Forbidden => Err(anyhow!(
            "project '{project_id}' is not owned by the signed-in user."
        )),
        AdminRunResponse::NotDeployed => Err(anyhow!(
            "project '{project_id}' has no deployed version yet."
        )),
        AdminRunResponse::InternalError { reason } => {
            Err(anyhow!("control admin_run internal error: {reason}"))
        }
    }
}
