//! Records a project's connection to its owner's Cloudflare account.
//!
//! The provisioning itself — buckets, CORS, the CDN hostname, the cache rule —
//! already happened, in the CLI, on the user's machine, with an account-wide
//! token fn0 is never given. What arrives here is the residue: two narrow
//! credentials and the names of what they reach.
//!
//! They are still proved before being stored. A credential that is wrong is
//! wrong whoever minted it, and the failure it would otherwise cause shows up
//! at request time or, for purge, as a stale object nobody notices for a year.

use crate::common::auth;
use crate::common::byoc::{self, ProjectStorage};
use crate::common::cloudflare::CloudflareClient;
use crate::common::vault;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub account_id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub static_hostname: String,
    pub object_bucket: String,
    pub asset_bucket: String,
    pub page_bucket: String,
    /// The data-plane token's id, which is also its S3 access key id.
    pub dataplane_access_key_id: String,
    /// SHA-256 of the data-plane token, already derived by the CLI. The token
    /// value itself is never sent: this hash is the only form fn0 needs, and a
    /// hash cannot be replayed against the REST API.
    pub dataplane_secret: String,
    /// Sent whole, because purging needs a bearer token rather than a hash.
    /// Scoped to cache purge on `zone_id` and nothing else.
    pub purge_token: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok,
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

    if req.body.dataplane_secret.len() != 64
        || !req
            .body
            .dataplane_secret
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Output::CredentialRejected {
            reason: "data-plane secret is not a SHA-256 hex digest".to_string(),
        };
    }

    // Purge is probed for real: an unpurgeable zone is the one failure that is
    // silent at write time and only surfaces as an object an
    // `s-maxage=31536000` holds for a year.
    let purge_client = CloudflareClient::for_zone(
        req.body.purge_token.clone(),
        req.body.account_id.clone(),
        req.body.zone_id.clone(),
    );
    if let Err(error) = purge_client
        .purge_cache_urls(&[format!(
            "https://{}/__fn0_connect_probe",
            req.body.static_hostname
        )])
        .await
    {
        tracing::warn!(%error, "cloudflare_connect purge probe failed");
        return Output::CredentialRejected {
            reason: format!("the purge token cannot purge this zone: {error}"),
        };
    }

    let (dataplane_secret_ciphertext, purge_token_ciphertext) = match (
        vault::encrypt(req.body.dataplane_secret.as_bytes()).await,
        vault::encrypt(req.body.purge_token.as_bytes()).await,
    ) {
        (Ok(secret), Ok(purge)) => (secret, purge),
        (Err(error), _) | (_, Err(error)) => {
            tracing::error!("cloudflare_connect vault encrypt: {error}");
            return Output::InternalError {
                reason: format!("vault encrypt: {error}"),
            };
        }
    };

    let existing = match (ProjectCloudflareConfigDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(existing) => existing,
        Err(error) => {
            tracing::error!("cloudflare_connect ProjectCloudflareConfigDocGet: {error}");
            return Output::InternalError {
                reason: format!("config read: {error}"),
            };
        }
    };

    // A project that already finished migrating is not connecting, it is
    // rotating: its objects are in the user's account and the platform's copies
    // are a frozen snapshot from the first migration. Running the migration
    // again would copy that snapshot back over live data.
    let rotating = existing
        .as_ref()
        .is_some_and(|doc| doc.state != CloudflareConnectionState::Migrating);

    let config = ProjectCloudflareConfigDoc {
        project_id: req.body.project_id.clone(),
        account_id: req.body.account_id.clone(),
        zone_id: req.body.zone_id.clone(),
        zone_name: req.body.zone_name.clone(),
        static_hostname: req.body.static_hostname.clone(),
        object_bucket: req.body.object_bucket.clone(),
        asset_bucket: req.body.asset_bucket.clone(),
        page_bucket: req.body.page_bucket.clone(),
        dataplane_access_key_id: req.body.dataplane_access_key_id.clone(),
        dataplane_secret_ciphertext,
        purge_token_ciphertext,
        state: if rotating {
            CloudflareConnectionState::Ok
        } else {
            CloudflareConnectionState::Migrating
        },
        checked_at: forte_sdk::now(),
        config_version: existing
            .as_ref()
            .map(|doc| doc.config_version + 1)
            .unwrap_or(1),
    };

    if let Err(error) = (ProjectCloudflareConfigDocPut(config.clone()))
        .send_with(&db)
        .await
    {
        tracing::error!("cloudflare_connect config put: {error}");
        return Output::InternalError {
            reason: format!("config write: {error}"),
        };
    }

    // Proves the data-plane credential against the bucket it will actually be
    // used on, now that it is decryptable through the stored config. A rotation
    // that fails here has already overwritten a working credential, so the old
    // one goes back.
    let probe = match ProjectStorage::resolve_connected(&db, &req.body.project_id).await {
        Ok(Some(storage)) => crate::common::r2_store::ProjectR2Store::assets(&storage)
            .list_all(&format!("{}/", req.body.project_id), forte_sdk::now())
            .await
            .map_err(|error| {
                format!(
                    "the data-plane token cannot read {}: {error}",
                    storage.asset_bucket
                )
            }),
        Ok(None) => Err("config vanished between write and read".to_string()),
        Err(error) => Err(format!("resolve: {error}")),
    };
    if let Err(reason) = probe {
        tracing::warn!(%reason, "cloudflare_connect data-plane probe failed");
        if let Some(previous) = existing
            && let Err(error) = (ProjectCloudflareConfigDocPut(previous))
                .send_with(&db)
                .await
        {
            tracing::error!("cloudflare_connect could not restore the previous config: {error}");
        }
        return Output::CredentialRejected { reason };
    }

    if rotating {
        // Straight to the manifest: nothing is moving, only the credential the
        // workers sign with. They pick it up on their next poll.
        if let Err(error) =
            byoc::publish_storage_to_manifest(&db, &req.body.project_id, Some(&config)).await
        {
            tracing::error!("cloudflare_connect publish_storage_to_manifest: {error}");
            return Output::InternalError {
                reason: format!("manifest publish: {error}"),
            };
        }
        return Output::Ok;
    }

    // First connect. The manifest is not published here: workers must keep
    // reading the platform account until the migration has copied everything
    // across. `byoc_migrate` flips both when it finishes.
    if let Err(error) = crate::enqueue::byoc_migrate(crate::queue_task::byoc_migrate::Input {
        project_id: req.body.project_id.clone(),
    })
    .await
    {
        tracing::error!("cloudflare_connect enqueue byoc_migrate: {error}");
        return Output::InternalError {
            reason: format!("migration enqueue: {error}"),
        };
    }

    Output::Ok
}
