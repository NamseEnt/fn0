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
use crate::common::byoc::{ProjectPurge, R2KeySource};
use crate::common::cloudflare::CachePurgeFile;
use crate::common::vault;
use crate::docs::*;
use forte_sdk::*;
use futures::future::{Either, select};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::pin;
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
const PROBE_BUDGET: Duration = Duration::from_secs(10);

async fn prove<Probe, Fut>(remaining: Duration, attempts: usize, probe: Probe) -> anyhow::Result<()>
where
    Probe: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    anyhow::ensure!(!remaining.is_zero(), "credential verification timed out");
    let verify = async {
        for _ in 1..attempts {
            if probe().await.is_ok() {
                return Ok(());
            }
            time_wasi::sleep(PROBE_RETRY_INTERVAL).await;
        }
        probe().await
    };
    match select(pin!(verify), pin!(time_wasi::sleep(remaining))).await {
        Either::Left((result, _)) => result,
        Either::Right(_) => anyhow::bail!("credential verification timed out"),
    }
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
    #[serde(default)]
    pub runtime_broker_url: Option<String>,
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
    let started_at = time_wasi::Instant::now();
    let remaining_probe_time = || PROBE_BUDGET.saturating_sub(started_at.elapsed());
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let db = doc_db::database();
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
    let broker_url = req.body.runtime_broker_url.as_deref();
    let project_id = &req.body.project_id;
    if broker_url.is_some() {
        let is_cloudflare_id =
            |value: &str| value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit());
        let valid_buckets = [
            (
                &req.body.private_object_storage_bucket,
                "private-object-storage",
            ),
            (
                &req.body.public_object_storage_bucket,
                "public-object-storage",
            ),
            (&req.body.frontend_asset_bucket, "frontend-asset"),
        ]
        .into_iter()
        .all(|(name, suffix)| *name == format!("fn0-{project_id}-{suffix}"));
        if !valid_buckets
            || !is_cloudflare_id(&req.body.account_id)
            || !is_cloudflare_id(&req.body.zone_id)
        {
            return Output::CredentialRejected {
                reason:
                    "runtime project bucket names and Cloudflare IDs must match broker provisioning"
                        .to_string(),
            };
        }
    }
    let (purge_client, worker_keys, asset_keys) = match (
        ProjectPurge::new(
            broker_url,
            project_id,
            req.body.purge_token.clone(),
            req.body.account_id.clone(),
            req.body.zone_id.clone(),
        ),
        R2KeySource::new(
            broker_url,
            project_id,
            req.body.worker_access_key_id.clone(),
            req.body.worker_secret.clone(),
        ),
        R2KeySource::new(
            broker_url,
            project_id,
            req.body.frontend_asset_access_key_id.clone(),
            req.body.frontend_asset_secret.clone(),
        ),
    ) {
        (Ok(purge), Ok(worker), Ok(assets)) => (purge, worker, assets),
        (Err(error), ..) | (_, Err(error), _) | (.., Err(error)) => {
            return Output::CredentialRejected {
                reason: error.to_string(),
            };
        }
    };
    let probe_url = format!(
        "https://{}/__fn0_connect_probe",
        req.body.frontend_asset_hostname
    );
    let probe_file = CachePurgeFile::from_url(probe_url, None);
    let (purge_attempts, purge_credential) = if broker_url.is_some() {
        (1, "the broker's FN0_R2_PARENT_TOKEN")
    } else {
        (PROBE_ATTEMPTS, "the project purge token")
    };
    if let Err(error) = prove(remaining_probe_time(), purge_attempts, || {
        purge_client.purge_cache_urls(std::slice::from_ref(&probe_file))
    })
    .await
    {
        tracing::warn!(%error, "cloudflare_connect purge probe failed");
        let zone_name = &req.body.zone_name;
        let zone_id = &req.body.zone_id;
        return Output::CredentialRejected {
            reason: format!(
                "Could not verify cache purge for Cloudflare zone {zone_name} ({zone_id}): {error}. \
                 Ensure {purge_credential} has Cache Purge permission for this zone, then retry forte cloud init."
            ),
        };
    }

    // Probed with each credential as sent, before anything is written. Every
    // bucket, because on the path where the user scopes the tokens by hand a
    // missed bucket is an easy mistake and a confusing failure later. Each
    // credential is proved only against the buckets it is meant to open, so a
    // pair swapped between the two tokens is caught here rather than becoming a
    // worker that can rewrite a deployed frontend.
    let probes = [
        (&req.body.private_object_storage_bucket, &worker_keys),
        (&req.body.public_object_storage_bucket, &worker_keys),
        (&req.body.frontend_asset_bucket, &asset_keys),
    ];
    for (bucket, source) in probes {
        if let Err(error) = prove(remaining_probe_time(), PROBE_ATTEMPTS, || async {
            let keys = source.for_bucket(bucket, 0).await?;
            aws_sign::r2_list_objects(aws_sign::R2ListArgs {
                account_id: &req.body.account_id,
                bucket,
                region: "auto",
                prefix: "",
                continuation_token: None,
                access_key_id: &keys.access_key_id,
                secret_access_key: &keys.secret_access_key,
                session_token: keys.session_token.as_deref(),
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
        runtime_broker_url: req.body.runtime_broker_url.clone(),
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

#[cfg(test)]
mod tests {
    use super::prove;
    use forte_sdk::runtime::block_on;
    use std::cell::Cell;
    use std::future::ready;
    use std::time::Duration;

    #[test]
    fn a_single_attempt_returns_the_rejection() {
        block_on(async {
            let calls = Cell::new(0);
            let error = prove(Duration::from_secs(1), 1, || {
                calls.set(calls.get() + 1);
                ready(Err(anyhow::anyhow!("purge denied")))
            })
            .await
            .unwrap_err();
            assert_eq!(calls.get(), 1);
            assert_eq!(error.to_string(), "purge denied");
        });
    }

    #[test]
    fn an_exhausted_budget_skips_the_probe() {
        block_on(async {
            let called = Cell::new(false);
            let error = prove(Duration::ZERO, 1, || {
                called.set(true);
                ready(Ok(()))
            })
            .await
            .unwrap_err();
            assert!(!called.get());
            assert_eq!(error.to_string(), "credential verification timed out");
        });
    }
}
