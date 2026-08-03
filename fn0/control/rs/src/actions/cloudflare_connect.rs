//! Records a project's connection to its owner's Cloudflare account.
//!
//! The provisioning itself — buckets, CORS, the CDN hostnames, the cache rule —
//! already happened, in the CLI, on the user's machine, with an account-wide
//! token fn0 is never given. What arrives here is the residue: three narrow
//! credentials and the names of what they reach.
//!
//! They are still proved before being stored. A credential that is wrong is
//! wrong whoever minted it, and the failure it would otherwise cause shows up
//! at request time or, for purge, as a stale object nobody notices for a year.
//!
//! First connection only. Reconnecting would have to decide whether to rotate
//! credentials and whether objects already written to the old account should
//! follow, and neither answer is obvious enough to pick silently.
//!
//! Nothing is written until every credential has been proved, and what is
//! written goes in one transaction. A half-written connection would leave
//! control holding a configuration the fleet never received, and `connect`
//! would refuse to run again to repair it.

use crate::common::auth;
use crate::common::aws_sign;
use crate::common::byoc;
use crate::common::cloudflare::{CachePurgeFile, CloudflareClient};
use crate::common::vault;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::Duration;

/// Cloudflare answers 401 on both cache purge and the R2 data plane for about
/// two seconds after `/user/tokens` hands back the credential: a token's
/// identity propagates ahead of its scoped permissions. Measured against the
/// live API, where `/user/tokens/verify` passes at 0.2s while the operations
/// the credential exists for are still refused at 2.3s — so verify cannot be
/// the readiness signal, and only the real call can.
///
/// Retried, not slept on, because the careful path's hand-made credentials are
/// minutes old and must not pay for the managed path's freshly minted ones.
const PROBE_ATTEMPTS: usize = 10;
const PROBE_RETRY_INTERVAL: Duration = Duration::from_millis(750);

async fn prove<Probe, Fut>(probe: Probe) -> anyhow::Result<()>
where
    Probe: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    for _ in 1..PROBE_ATTEMPTS {
        if probe().await.is_ok() {
            return Ok(());
        }
        time_wasi::sleep(PROBE_RETRY_INTERVAL).await;
    }
    probe().await
}

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub account_id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub frontend_asset_hostname: String,
    pub public_object_storage_hostname: String,
    pub private_object_storage_bucket: String,
    pub public_object_storage_bucket: String,
    pub frontend_asset_bucket: String,
    /// The worker token's id, which is also its S3 access key id.
    pub worker_access_key_id: String,
    /// SHA-256 of the worker token, already derived by the CLI. The token value
    /// itself is never sent: this hash is the only form fn0 needs, and a hash
    /// cannot be replayed against the REST API.
    pub worker_secret: String,
    pub frontend_asset_access_key_id: String,
    pub frontend_asset_secret: String,
    /// Sent whole, because purging needs a bearer token rather than a hash.
    /// Scoped to cache purge on `zone_id` and nothing else.
    pub purge_token: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok,
    /// Connecting a second time would need decisions this action does not make:
    /// whether to rotate credentials, and whether the objects already in the
    /// old account should follow. Refused rather than guessed.
    AlreadyConnected {
        account_id: String,
        zone_name: String,
    },
    /// A credential does not do what it is supposed to. Nothing was stored.
    CredentialRejected {
        reason: String,
    },
    NotLoggedIn,
    NotFound,
    InternalError {
        reason: String,
    },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let db = doc_db::turso();
    let project = match (ProjectDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(Some(project)) => project,
        Ok(None) => return Output::NotFound,
        Err(error) => {
            tracing::error!("cloudflare_connect ProjectDocGet: {error}");
            return Output::InternalError {
                reason: format!("ProjectDocGet: {error}"),
            };
        }
    };
    if project.owner_github_id != user.github_id {
        return Output::NotFound;
    }

    for (what, secret) in [
        ("worker", &req.body.worker_secret),
        ("frontend asset", &req.body.frontend_asset_secret),
    ] {
        if secret.len() != 64 || !secret.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Output::CredentialRejected {
                reason: format!("{what} secret is not a SHA-256 hex digest"),
            };
        }
    }

    // Purge is probed for real: an unpurgeable zone is the one failure that is
    // silent at write time and only surfaces as an object an
    // `s-maxage=31536000` holds for a year.
    let purge_client = CloudflareClient::for_zone(
        req.body.purge_token.clone(),
        req.body.account_id.clone(),
        req.body.zone_id.clone(),
    );
    let probe_url = format!(
        "https://{}/__fn0_connect_probe",
        req.body.frontend_asset_hostname
    );
    let probe_file = CachePurgeFile::from_url(probe_url, None);
    if let Err(error) =
        prove(|| purge_client.purge_cache_urls(std::slice::from_ref(&probe_file))).await
    {
        tracing::warn!(%error, "cloudflare_connect purge probe failed");
        return Output::CredentialRejected {
            reason: format!("the purge token cannot purge this zone: {error}"),
        };
    }

    // Probed with each credential as sent, before anything is written. Every
    // bucket, because on the path where the user scopes the tokens by hand a
    // missed bucket is an easy mistake and a confusing failure later. Each
    // credential is proved only against the buckets it is meant to open, so a
    // pair swapped between the two tokens is caught here rather than becoming a
    // worker that can rewrite a deployed frontend.
    let probes: [(&str, &str, &str); 3] = [
        (
            &req.body.private_object_storage_bucket,
            &req.body.worker_access_key_id,
            &req.body.worker_secret,
        ),
        (
            &req.body.public_object_storage_bucket,
            &req.body.worker_access_key_id,
            &req.body.worker_secret,
        ),
        (
            &req.body.frontend_asset_bucket,
            &req.body.frontend_asset_access_key_id,
            &req.body.frontend_asset_secret,
        ),
    ];
    for (bucket, access_key_id, secret_access_key) in probes {
        if let Err(error) = prove(|| async {
            aws_sign::r2_list_objects(aws_sign::R2ListArgs {
                account_id: &req.body.account_id,
                bucket,
                region: "auto",
                prefix: "",
                continuation_token: None,
                access_key_id,
                secret_access_key,
                now: forte_sdk::now(),
            })
            .await
            .map(|_| ())
        })
        .await
        {
            tracing::warn!(%error, %bucket, "cloudflare_connect R2 probe failed");
            return Output::CredentialRejected {
                reason: format!("the stored credential cannot read {bucket}: {error}"),
            };
        }
    }

    let (worker_secret_ciphertext, frontend_asset_secret_ciphertext, purge_token_ciphertext) =
        match (
            vault::encrypt(req.body.worker_secret.as_bytes()).await,
            vault::encrypt(req.body.frontend_asset_secret.as_bytes()).await,
            vault::encrypt(req.body.purge_token.as_bytes()).await,
        ) {
            (Ok(worker), Ok(asset), Ok(purge)) => (worker, asset, purge),
            (Err(error), ..) | (_, Err(error), _) | (.., Err(error)) => {
                tracing::error!("cloudflare_connect vault encrypt: {error}");
                return Output::InternalError {
                    reason: format!("vault encrypt: {error}"),
                };
            }
        };

    let config = ProjectCloudflareConfigDoc {
        project_id: req.body.project_id.clone(),
        account_id: req.body.account_id.clone(),
        zone_id: req.body.zone_id.clone(),
        zone_name: req.body.zone_name.clone(),
        frontend_asset_hostname: req.body.frontend_asset_hostname.clone(),
        public_object_storage_hostname: req.body.public_object_storage_hostname.clone(),
        private_object_storage_bucket: req.body.private_object_storage_bucket.clone(),
        public_object_storage_bucket: req.body.public_object_storage_bucket.clone(),
        frontend_asset_bucket: req.body.frontend_asset_bucket.clone(),
        worker_access_key_id: req.body.worker_access_key_id.clone(),
        worker_secret_ciphertext,
        frontend_asset_access_key_id: req.body.frontend_asset_access_key_id.clone(),
        frontend_asset_secret_ciphertext,
        purge_token_ciphertext,
        state: CloudflareConnectionState::Ok,
        checked_at: forte_sdk::now(),
        config_version: 1,
    };

    match byoc::connect_project(&db, config).await {
        Ok(byoc::ConnectOutcome::Connected) => {}
        Ok(byoc::ConnectOutcome::AlreadyConnected {
            account_id,
            zone_name,
        }) => {
            return Output::AlreadyConnected {
                account_id,
                zone_name,
            };
        }
        Err(error) => {
            tracing::error!("cloudflare_connect connect_project: {error}");
            return Output::InternalError {
                reason: format!("connect: {error}"),
            };
        }
    }

    Output::Ok
}
