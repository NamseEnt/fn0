//! Connecting a project to the owner's own Cloudflare account.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ConnectInput<'a> {
    project_id: &'a str,
    account_id: &'a str,
    zone_id: &'a str,
    api_token: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum Connect {
    Ok {
        static_hostname: String,
        asset_bucket: String,
        page_bucket: String,
    },
    MissingPermissions {
        missing: Vec<String>,
    },
    NotLoggedIn,
    NotFound,
    InternalError {
        reason: String,
    },
}

pub async fn cloudflare_connect(
    project_id: &str,
    account_id: &str,
    zone_id: &str,
    api_token: &str,
) -> Result<()> {
    let creds = crate::credentials::require()?;
    let url = format!(
        "{}/__forte_action/cloudflare_connect",
        creds.control_url.trim_end_matches('/')
    );
    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&ConnectInput {
            project_id,
            account_id,
            zone_id,
            api_token,
        })
        .send()
        .await?
        .error_for_status()?;

    match response.json::<Connect>().await? {
        Connect::Ok {
            static_hostname,
            asset_bucket,
            page_bucket,
        } => {
            println!("connected project '{project_id}' to Cloudflare account {account_id}");
            println!("  assets and public objects: {asset_bucket} -> https://{static_hostname}");
            println!("  cached pages: {page_bucket} (private)");
            println!(
                "existing objects are being copied across; the project keeps using the fn0 platform account until that finishes."
            );
            Ok(())
        }
        Connect::MissingPermissions { missing } => Err(anyhow!(
            "the API token cannot do everything fn0 needs:\n  - {}",
            missing.join("\n  - ")
        )),
        Connect::NotLoggedIn => Err(anyhow!("control rejected token; sign in again.")),
        Connect::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        Connect::InternalError { reason } => Err(anyhow!("cloudflare_connect: {reason}")),
    }
}

#[derive(Serialize)]
struct StatusInput<'a> {
    project_id: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum Status {
    Platform,
    Connected {
        account_id: String,
        zone_name: String,
        static_hostname: String,
        asset_bucket: String,
        page_bucket: String,
        healthy: bool,
        problem: Option<String>,
        migrating: bool,
    },
    NotLoggedIn,
    NotFound,
    InternalError {
        reason: String,
    },
}

pub async fn cloudflare_status(project_id: &str) -> Result<()> {
    let creds = crate::credentials::require()?;
    let url = format!(
        "{}/__forte_action/cloudflare_status",
        creds.control_url.trim_end_matches('/')
    );
    let response = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&StatusInput { project_id })
        .send()
        .await?
        .error_for_status()?;

    match response.json::<Status>().await? {
        Status::Platform => {
            println!("project '{project_id}' runs on the fn0 platform Cloudflare account.");
            println!("run `forte cloudflare connect` to move it onto your own.");
            Ok(())
        }
        Status::Connected {
            account_id,
            zone_name,
            static_hostname,
            asset_bucket,
            page_bucket,
            healthy,
            problem,
            migrating,
        } => {
            println!("account: {account_id}");
            println!("zone:    {zone_name}");
            println!("assets:  {asset_bucket} -> https://{static_hostname}");
            println!("pages:   {page_bucket}");
            match (migrating, healthy, problem) {
                (true, _, _) => println!(
                    "status:  migrating - still served from the fn0 platform account until existing objects finish copying"
                ),
                (false, true, _) => println!("status:  ok"),
                (false, false, Some(problem)) => println!("status:  degraded - {problem}"),
                (false, false, None) => println!("status:  degraded"),
            }
            Ok(())
        }
        Status::NotLoggedIn => Err(anyhow!("control rejected token; sign in again.")),
        Status::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        Status::InternalError { reason } => Err(anyhow!("cloudflare_status: {reason}")),
    }
}
