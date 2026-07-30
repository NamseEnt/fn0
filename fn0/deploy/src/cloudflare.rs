//! Connecting a project to the owner's own Cloudflare account.
//!
//! The account-wide token is used here and only here. It provisions the
//! account, mints two narrow credentials, and goes out of scope; fn0 receives
//! the narrow credentials and never the token that made them.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::cloudflare_provision::Provisioner;

#[derive(Serialize)]
struct ConnectInput<'a> {
    project_id: &'a str,
    account_id: &'a str,
    zone_id: &'a str,
    zone_name: &'a str,
    static_hostname: &'a str,
    object_bucket: &'a str,
    asset_bucket: &'a str,
    page_bucket: &'a str,
    dataplane_access_key_id: &'a str,
    dataplane_secret: &'a str,
    purge_token: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum Connect {
    Ok,
    CredentialRejected { reason: String },
    NotLoggedIn,
    NotFound,
    InternalError { reason: String },
}

pub async fn cloudflare_connect(
    project_id: &str,
    account_id: &str,
    zone_id: &str,
    api_token: &str,
) -> Result<()> {
    let creds = crate::credentials::require()?;

    println!("provisioning your Cloudflare account (this runs locally)...");
    let provisioner = Provisioner::new(
        api_token.to_string(),
        account_id.to_string(),
        zone_id.to_string(),
    );
    let provisioned = provisioner.run(project_id).await?;
    println!("  zone:    {}", provisioned.zone_name);
    println!(
        "  buckets: {}, {}, {}",
        provisioned.object_bucket, provisioned.asset_bucket, provisioned.page_bucket
    );
    println!("  assets:  https://{}", provisioned.static_hostname);
    println!("  minted a bucket-scoped R2 token and a purge-only token");

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
            zone_name: &provisioned.zone_name,
            static_hostname: &provisioned.static_hostname,
            object_bucket: &provisioned.object_bucket,
            asset_bucket: &provisioned.asset_bucket,
            page_bucket: &provisioned.page_bucket,
            dataplane_access_key_id: &provisioned.dataplane_access_key_id,
            dataplane_secret: &provisioned.dataplane_secret,
            purge_token: &provisioned.purge_token,
        })
        .send()
        .await?
        .error_for_status()?;

    match response.json::<Connect>().await? {
        Connect::Ok => {
            println!();
            println!("connected. your account-wide token was not sent to fn0 and is no longer");
            println!("needed — you can delete it in the Cloudflare dashboard.");
            println!(
                "existing objects are being copied across; the project keeps using the fn0 \
                 platform account until that finishes."
            );
            Ok(())
        }
        Connect::CredentialRejected { reason } => Err(anyhow!(
            "fn0 rejected the credentials this command minted: {reason}"
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
                    "status:  migrating - still served from the fn0 platform account until \
                     existing objects finish copying"
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
