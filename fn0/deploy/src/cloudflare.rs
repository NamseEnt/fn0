//! Connecting a project to the owner's own Cloudflare account.
//!
//! The account-wide token is used here and only here. It provisions the
//! account, mints two narrow credentials, and goes out of scope; fn0 receives
//! the narrow credentials and never the token that made them.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::cloudflare_provision::{
    ConnectCredentials, ProvisionedResources, Provisioner, frontend_asset_bucket_name,
    private_object_storage_bucket_name, public_object_storage_bucket_name,
    rendered_html_cache_bucket_name,
};

#[derive(Serialize)]
struct ConnectInput<'a> {
    project_id: &'a str,
    account_id: &'a str,
    zone_id: &'a str,
    zone_name: &'a str,
    frontend_asset_hostname: &'a str,
    public_object_storage_hostname: &'a str,
    private_object_storage_bucket: &'a str,
    public_object_storage_bucket: &'a str,
    frontend_asset_bucket: &'a str,
    rendered_html_cache_bucket: &'a str,
    worker_access_key_id: &'a str,
    worker_secret: &'a str,
    frontend_asset_access_key_id: &'a str,
    frontend_asset_secret: &'a str,
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
    println!("  minted two bucket-scoped R2 tokens and a purge-only token");

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
                 revoke these in the Cloudflare dashboard: worker {}, frontend assets {}, \
                 cache purge {}.",
                minted.worker, minted.frontend_asset, minted.purge
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
    println!("Now create the three credentials fn0 will keep. None can be minted for");
    println!("you here, because this token deliberately cannot create tokens.");
    println!();
    println!("The first two are separate on purpose: only the worker one is handed to");
    println!("the fleet, and only the frontend-asset one is used by the GC that deletes.");
    println!("Scoping them apart is what keeps either from reaching the other's bucket.");
    println!();
    println!("1. R2 -> Manage API Tokens -> Create API token");
    println!("     permission: Object Read & Write");
    println!("     apply to specific buckets:");
    println!("       {}", resources.private_object_storage_bucket);
    println!("       {}", resources.public_object_storage_bucket);
    println!("       {}", resources.rendered_html_cache_bucket);
    println!("     the screen shows an Access Key ID and a Secret Access Key; keep both");
    println!();
    println!("2. R2 -> Manage API Tokens -> Create API token");
    println!("     permission: Object Read & Write");
    println!("     apply to specific buckets:");
    println!("       {}", resources.frontend_asset_bucket);
    println!();
    println!("3. My Profile -> API Tokens -> Create Custom Token");
    println!("     permission: Zone -> Cache Purge -> Purge");
    println!("     zone resources: {} only", resources.zone_name);
    println!();
    println!("Then finish with:");
    println!();
    println!("    forte cloudflare connect \\");
    println!("      --account-id {account_id} \\");
    println!("      --zone-id {zone_id} \\");
    println!("      --zone-name {} \\", resources.zone_name);
    println!("      --worker-access-key-id <Access Key ID from 1> \\");
    println!("      --worker-secret <Secret Access Key from 1> \\");
    println!("      --frontend-asset-access-key-id <Access Key ID from 2> \\");
    println!("      --frontend-asset-secret <Secret Access Key from 2> \\");
    println!("      --purge-token <purge token from 3>");
    println!();
    println!("You can delete the token you just used once you have done that.");
    Ok(())
}

/// The second half of the careful path: credentials the user made by hand.
#[allow(clippy::too_many_arguments)]
pub async fn cloudflare_connect_manual(
    project_id: &str,
    account_id: &str,
    zone_id: &str,
    zone_name: &str,
    worker_access_key_id: &str,
    worker_secret: &str,
    frontend_asset_access_key_id: &str,
    frontend_asset_secret: &str,
    purge_token: &str,
) -> Result<()> {
    let frontend_asset_bucket = frontend_asset_bucket_name(project_id);
    let public_object_storage_bucket = public_object_storage_bucket_name(project_id);
    let resources = ProvisionedResources {
        frontend_asset_hostname: format!("{frontend_asset_bucket}.{zone_name}"),
        public_object_storage_hostname: format!("{public_object_storage_bucket}.{zone_name}"),
        zone_name: zone_name.to_string(),
        private_object_storage_bucket: private_object_storage_bucket_name(project_id),
        public_object_storage_bucket,
        frontend_asset_bucket,
        rendered_html_cache_bucket: rendered_html_cache_bucket_name(project_id),
    };
    let credentials = ConnectCredentials {
        worker_access_key_id: worker_access_key_id.to_string(),
        worker_secret: worker_secret.to_string(),
        frontend_asset_access_key_id: frontend_asset_access_key_id.to_string(),
        frontend_asset_secret: frontend_asset_secret.to_string(),
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
    println!("  buckets: {}", resources.private_object_storage_bucket);
    println!("           {}", resources.public_object_storage_bucket);
    println!("           {}", resources.frontend_asset_bucket);
    println!("           {}", resources.rendered_html_cache_bucket);
    println!("  assets:  https://{}", resources.frontend_asset_hostname);
    println!(
        "  public:  https://{}",
        resources.public_object_storage_hostname
    );
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
    credentials: &ConnectCredentials,
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
                frontend_asset_hostname: &provisioned.frontend_asset_hostname,
                public_object_storage_hostname: &provisioned.public_object_storage_hostname,
                private_object_storage_bucket: &provisioned.private_object_storage_bucket,
                public_object_storage_bucket: &provisioned.public_object_storage_bucket,
                frontend_asset_bucket: &provisioned.frontend_asset_bucket,
                rendered_html_cache_bucket: &provisioned.rendered_html_cache_bucket,
                worker_access_key_id: &credentials.worker_access_key_id,
                worker_secret: &credentials.worker_secret,
                frontend_asset_access_key_id: &credentials.frontend_asset_access_key_id,
                frontend_asset_secret: &credentials.frontend_asset_secret,
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
    NotConnected,
    Connected {
        account_id: String,
        zone_name: String,
        frontend_asset_hostname: String,
        public_object_storage_hostname: String,
        frontend_asset_bucket: String,
        public_object_storage_bucket: String,
        private_object_storage_bucket: String,
        rendered_html_cache_bucket: String,
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
        Status::NotConnected => {
            println!("project '{project_id}' has no Cloudflare account connected.");
            println!("run `forte cloudflare connect` before deploying it.");
            Ok(())
        }
        Status::Connected {
            account_id,
            zone_name,
            frontend_asset_hostname,
            public_object_storage_hostname,
            frontend_asset_bucket,
            public_object_storage_bucket,
            private_object_storage_bucket,
            rendered_html_cache_bucket,
            healthy,
            problem,
        } => {
            println!("account: {account_id}");
            println!("zone:    {zone_name}");
            println!("assets:  {frontend_asset_bucket} -> https://{frontend_asset_hostname}");
            println!(
                "public:  {public_object_storage_bucket} -> https://{public_object_storage_hostname}"
            );
            println!("private: {private_object_storage_bucket}");
            println!("cache:   {rendered_html_cache_bucket}");
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
