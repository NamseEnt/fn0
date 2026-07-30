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
//!
//! First connection only. Reconnecting would have to decide whether to rotate
//! credentials and whether objects already written to the old account should
//! follow, and neither answer is obvious enough to pick silently.

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

    if let Some(existing) = existing {
        return Output::AlreadyConnected {
            account_id: existing.account_id,
            zone_name: existing.zone_name,
        };
    }

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
        state: CloudflareConnectionState::Ok,
        checked_at: forte_sdk::now(),
        config_version: 1,
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
    // used on, now that it is decryptable through the stored config. Deleting
    // the config on failure leaves the project exactly as it was.
    let probe = match ProjectStorage::resolve(&db, &req.body.project_id).await {
        Ok(storage) => crate::common::r2_store::ProjectR2Store::assets(&storage)
            .list_all(&format!("{}/", req.body.project_id), forte_sdk::now())
            .await
            .map_err(|error| {
                format!(
                    "the data-plane token cannot read {}: {error}",
                    storage.asset_bucket
                )
            }),
        Err(error) => Err(format!("resolve: {error}")),
    };
    if let Err(reason) = probe {
        tracing::warn!(%reason, "cloudflare_connect data-plane probe failed");
        if let Err(error) = (ProjectCloudflareConfigDocDelete {
            project_id: &req.body.project_id,
        })
        .send_with(&db)
        .await
        {
            tracing::error!("cloudflare_connect could not undo the config write: {error}");
        }
        return Output::CredentialRejected { reason };
    }

    // Workers pick this up on their next poll, about a second later. There is
    // nothing to move first: a project connects before it has objects, and one
    // that already has them on the platform account keeps them there.
    if let Err(error) =
        byoc::publish_storage_to_manifest(&db, &req.body.project_id, Some(&config)).await
    {
        tracing::error!("cloudflare_connect publish_storage_to_manifest: {error}");
        return Output::InternalError {
            reason: format!("manifest publish: {error}"),
        };
    }

    Output::Ok
}
