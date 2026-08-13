//! Connecting a project to the owner's own Cloudflare account.
//!
//! The account-wide token is used here and only here. It provisions the
//! account, mints two narrow credentials, and goes out of scope; fn0 receives
//! the narrow credentials and never the token that made them.
//!
//! Everything in this module is a step of `forte cloud init`. The command owns
//! the prompting and the printing; these functions own the Cloudflare calls, so
//! the CLI never handles a token or decides what to mint.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::cloudflare_provision::{ConnectCredentials, ProvisionedResources, Provisioner};

/// What the user chose and typed, carried between the steps of one setup.
pub struct CloudSetup<'a> {
    pub project_id: &'a str,
    pub account_id: &'a str,
    pub zone_id: &'a str,
    /// Discarded when the command exits. fn0 never receives it.
    pub api_token: &'a str,
    pub domain: &'a str,
}

impl CloudSetup<'_> {
    /// The one origin the project's own pages are served from, and so the only
    /// origin allowed to read its buckets from a browser.
    pub fn app_origin(&self) -> String {
        format!("https://{}", self.domain)
    }

    fn provisioner(&self) -> Provisioner {
        Provisioner::new(
            self.api_token.to_string(),
            self.account_id.to_string(),
            self.zone_id.to_string(),
        )
    }

    pub async fn ensure_websockets(&self) -> Result<()> {
        self.provisioner()
            .ensure_websockets(self.mint_from_setup_token)
            .await
    }
}

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

/// The convenient path: one `API Tokens -> Edit` token. Provisions the account,
/// mints the three credentials fn0 keeps, and hands them over.
pub async fn provision_and_connect(setup: &CloudSetup<'_>) -> Result<ProvisionedResources> {
    let provisioner = setup.provisioner();
    let (resources, credentials, minted) = provisioner
        .run_managed(setup.project_id, &setup.app_origin(), setup.domain)
        .await?;

    match send_connect(setup, &resources, &credentials).await {
        Ok(()) => Ok(resources),
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

async fn send_connect(
    setup: &CloudSetup<'_>,
    provisioned: &ProvisionedResources,
    credentials: &ConnectCredentials,
) -> std::result::Result<(), ConnectFailure> {
    let creds = crate::credentials::require().map_err(ConnectFailure::Indeterminate)?;
    let url = format!(
        "{}/__forte_action/cloudflare_connect",
        creds.control_url.trim_end_matches('/')
    );
    let project_id = setup.project_id;
    let response = async {
        reqwest::Client::new()
            .post(&url)
            .bearer_auth(&creds.token)
            .json(&ConnectInput {
                project_id,
                account_id: setup.account_id,
                zone_id: setup.zone_id,
                zone_name: &provisioned.zone_name,
                frontend_asset_hostname: &provisioned.frontend_asset_hostname,
                public_object_storage_hostname: &provisioned.public_object_storage_hostname,
                private_object_storage_bucket: &provisioned.private_object_storage_bucket,
                public_object_storage_bucket: &provisioned.public_object_storage_bucket,
                frontend_asset_bucket: &provisioned.frontend_asset_bucket,
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
        Connect::Ok => Ok(()),
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
    Connected { zone_name: String },
    NotLoggedIn,
    NotFound,
    InternalError { reason: String },
}

/// Whether a project has an account behind it. `forte deploy` refuses on this,
/// because a project without one has nowhere to put its bundle or its assets.
pub enum CloudflareConnection {
    Connected { zone_name: String },
    NotConnected,
    NotFound,
}

pub async fn fetch_cloudflare_connection(project_id: &str) -> Result<CloudflareConnection> {
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
        Status::Connected { zone_name } => Ok(CloudflareConnection::Connected { zone_name }),
        Status::NotConnected => Ok(CloudflareConnection::NotConnected),
        Status::NotFound => Ok(CloudflareConnection::NotFound),
        Status::NotLoggedIn => Err(anyhow!("control rejected token; sign in again.")),
        Status::InternalError { reason } => Err(anyhow!("cloudflare_status: {reason}")),
    }
}
