//! Connecting a project to the owner's own Cloudflare account.
//!
//! The account-wide token is used here and only here. It provisions the
//! account, mints two narrow credentials, and goes out of scope; fn0 receives
//! the narrow credentials and never the token that made them.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::cloudflare_provision::{
    ASSET_BUCKET, DataPlaneCredentials, PAGE_BUCKET, ProvisionedResources, Provisioner,
    object_bucket_name,
};

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
    CredentialRejected {
        reason: String,
    },
    AlreadyConnected {
        account_id: String,
        zone_name: String,
    },
    NotLoggedIn,
    NotFound,
    InternalError {
        reason: String,
    },
}

/// One `API Tokens -> Edit` token; the CLI provisions and mints from it.
pub async fn cloudflare_connect_managed(
    project_id: &str,
    account_id: &str,
    zone_id: &str,
    api_token: &str,
) -> Result<()> {
    println!("provisioning your Cloudflare account (this runs locally)...");
    let provisioner = Provisioner::new(
        api_token.to_string(),
        account_id.to_string(),
        zone_id.to_string(),
    );
    let (resources, credentials, minted) = provisioner.run_managed(project_id).await?;
    print_resources(&resources);
    println!("  minted a bucket-scoped R2 token and a purge-only token");

    match send_connect(project_id, account_id, zone_id, &resources, &credentials).await {
        Ok(()) => Ok(()),
        // fn0 answered, and its answer proves it stored nothing. These two
        // credentials never expire, so leaving them would hand the account a
        // live R2 read-write pair for every failed attempt.
        Err(ConnectFailure::Rejected(error)) => {
            provisioner.revoke_minted_credentials(&minted).await;
            Err(error)
        }
        // No answer arrived, so whether fn0 stored them is unknown. Revoking
        // could break a connection that did succeed; naming them lets the user
        // decide.
        Err(ConnectFailure::Indeterminate(error)) => {
            eprintln!(
                "warning: could not tell whether fn0 stored the credentials. If it did not, \
                 revoke these in the Cloudflare dashboard: data plane {}, cache purge {}.",
                minted.dataplane, minted.purge
            );
            Err(error)
        }
    }
}

/// A token that can provision but cannot create tokens. Provisions and stops;
/// the user makes the two long-lived credentials themselves.
pub async fn cloudflare_provision(
    project_id: &str,
    account_id: &str,
    zone_id: &str,
    api_token: &str,
) -> Result<()> {
    println!("provisioning your Cloudflare account (this runs locally)...");
    let provisioner = Provisioner::new(
        api_token.to_string(),
        account_id.to_string(),
        zone_id.to_string(),
    );
    let resources = provisioner.run_manual(project_id).await?;
    print_resources(&resources);

    println!();
    println!("Now create the two credentials fn0 will keep. Neither can be minted for");
    println!("you here, because this token deliberately cannot create tokens.");
    println!();
    println!("1. R2 -> Manage API Tokens -> Create API token");
    println!("     permission: Object Read & Write");
    println!("     apply to specific buckets:");
    println!("       {}", resources.object_bucket);
    println!("       {}", resources.asset_bucket);
    println!("       {}", resources.page_bucket);
    println!("     the screen shows an Access Key ID and a Secret Access Key; keep both");
    println!();
    println!("2. My Profile -> API Tokens -> Create Custom Token");
    println!("     permission: Zone -> Cache Purge -> Purge");
    println!("     zone resources: {} only", resources.zone_name);
    println!();
    println!("Then finish with:");
    println!();
    println!("    forte cloudflare connect \\");
    println!("      --account-id {account_id} \\");
    println!("      --zone-id {zone_id} \\");
    println!("      --zone-name {} \\", resources.zone_name);
    println!("      --dataplane-access-key-id <Access Key ID> \\");
    println!("      --dataplane-secret <Secret Access Key> \\");
    println!("      --purge-token <purge token>");
    println!();
    println!("You can delete the token you just used once you have done that.");
    Ok(())
}

/// The second half of the careful path: credentials the user made by hand.
pub async fn cloudflare_connect_manual(
    project_id: &str,
    account_id: &str,
    zone_id: &str,
    zone_name: &str,
    dataplane_access_key_id: &str,
    dataplane_secret: &str,
    purge_token: &str,
) -> Result<()> {
    let resources = ProvisionedResources {
        zone_name: zone_name.to_string(),
        static_hostname: format!("static.{zone_name}"),
        object_bucket: object_bucket_name(project_id),
        asset_bucket: ASSET_BUCKET.to_string(),
        page_bucket: PAGE_BUCKET.to_string(),
    };
    let credentials = DataPlaneCredentials {
        dataplane_access_key_id: dataplane_access_key_id.to_string(),
        dataplane_secret: dataplane_secret.to_string(),
        purge_token: purge_token.to_string(),
    };
    // Nothing to revoke on failure: these credentials are the user's own, and
    // fn0 holds no token that could revoke them anyway.
    send_connect(project_id, account_id, zone_id, &resources, &credentials)
        .await
        .map_err(ConnectFailure::into_error)
}

fn print_resources(resources: &ProvisionedResources) {
    println!("  zone:    {}", resources.zone_name);
    println!(
        "  buckets: {}, {}, {}",
        resources.object_bucket, resources.asset_bucket, resources.page_bucket
    );
    println!("  assets:  https://{}", resources.static_hostname);
}

/// Why a connect did not succeed, split by what it says about fn0's state — the
/// caller has credentials to clean up and may only do so when nothing was
/// stored.
enum ConnectFailure {
    /// fn0 answered. Every answer other than `Ok` is returned before anything
    /// is written, so the credentials sent are certainly unused.
    Rejected(anyhow::Error),
    /// The request or its answer did not complete. fn0 may or may not have
    /// stored the credentials.
    Indeterminate(anyhow::Error),
}

impl ConnectFailure {
    fn into_error(self) -> anyhow::Error {
        match self {
            Self::Rejected(error) | Self::Indeterminate(error) => error,
        }
    }
}

async fn send_connect(
    project_id: &str,
    account_id: &str,
    zone_id: &str,
    provisioned: &ProvisionedResources,
    credentials: &DataPlaneCredentials,
) -> std::result::Result<(), ConnectFailure> {
    let creds = crate::credentials::require().map_err(ConnectFailure::Indeterminate)?;
    let url = format!(
        "{}/__forte_action/cloudflare_connect",
        creds.control_url.trim_end_matches('/')
    );
    let response = async {
        reqwest::Client::new()
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
                dataplane_access_key_id: &credentials.dataplane_access_key_id,
                dataplane_secret: &credentials.dataplane_secret,
                purge_token: &credentials.purge_token,
            })
            .send()
            .await?
            .error_for_status()?
            .json::<Connect>()
            .await
    }
    .await
    .map_err(|error| ConnectFailure::Indeterminate(error.into()))?;

    match response {
        Connect::Ok => {
            println!();
            println!("connected. the token you used was not sent to fn0 and is no longer");
            println!("needed — you can delete it in the Cloudflare dashboard.");
            println!("workers pick this up within a second; no redeploy needed.");
            Ok(())
        }
        Connect::CredentialRejected { reason } => Err(ConnectFailure::Rejected(anyhow!(
            "fn0 rejected the credentials: {reason}"
        ))),
        Connect::AlreadyConnected {
            account_id,
            zone_name,
        } => Err(ConnectFailure::Rejected(anyhow!(
            "project '{project_id}' is already connected to account {account_id} ({zone_name}). \
             Reconnecting is not supported yet — it would have to decide whether to rotate \
             credentials and whether to move objects already written to that account."
        ))),
        Connect::NotLoggedIn => Err(ConnectFailure::Rejected(anyhow!(
            "control rejected token; sign in again."
        ))),
        Connect::NotFound => Err(ConnectFailure::Rejected(anyhow!(
            "project '{project_id}' not found or not owned by you."
        ))),
        Connect::InternalError { reason } => Err(ConnectFailure::Rejected(anyhow!(
            "cloudflare_connect: {reason}"
        ))),
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
        } => {
            println!("account: {account_id}");
            println!("zone:    {zone_name}");
            println!("assets:  {asset_bucket} -> https://{static_hostname}");
            println!("pages:   {page_bucket}");
            match (healthy, problem) {
                (true, _) => println!("status:  ok"),
                (false, Some(problem)) => println!("status:  degraded - {problem}"),
                (false, None) => println!("status:  degraded"),
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
