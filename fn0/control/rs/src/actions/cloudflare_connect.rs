//! Attaches a project to the owner's own Cloudflare account.
//!
//! Credentials are proved against the live API before they are stored: a token
//! that is missing one permission fails at deploy time or, worse, at purge
//! time, where the symptom is a year of stale bytes rather than an error. Each
//! probe here maps to the permission it needs, so the answer names what to fix
//! instead of returning a Cloudflare error code.

use crate::common::auth;
use crate::common::byoc::{
    ProjectStorage, USER_ASSET_BUCKET, USER_PAGE_BUCKET, dataplane_secret_from_token,
    static_hostname_for,
};
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
    pub api_token: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        static_hostname: String,
        asset_bucket: String,
        page_bucket: String,
    },
    /// The token reached Cloudflare but cannot do everything fn0 needs. Nothing
    /// was stored.
    MissingPermissions {
        missing: Vec<String>,
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

    let cloudflare = CloudflareClient::for_account(
        req.body.api_token.clone(),
        req.body.account_id.clone(),
        req.body.zone_id.clone(),
    );

    let token = match cloudflare.verify_token().await {
        Ok(token) => token,
        Err(error) => {
            tracing::warn!(%error, "cloudflare_connect token verify failed");
            return Output::MissingPermissions {
                missing: vec!["a valid account API token".to_string()],
            };
        }
    };

    let zone_name = match cloudflare.zone_name().await {
        Ok(name) => name,
        Err(error) => {
            tracing::warn!(%error, "cloudflare_connect zone lookup failed");
            return Output::MissingPermissions {
                missing: vec!["Zone -> Zone -> Read on the zone".to_string()],
            };
        }
    };
    let static_hostname = static_hostname_for(&zone_name);

    let mut missing = Vec::new();
    if let Err(error) = cloudflare.create_r2_bucket(USER_ASSET_BUCKET, None).await {
        tracing::warn!(%error, "cloudflare_connect bucket probe failed");
        missing.push("Account -> Workers R2 Storage -> Edit".to_string());
    }
    // Purge is probed for real: an unpurgeable zone is the one failure that is
    // silent at write time and only shows up as a stale object an
    // `s-maxage=31536000` will hold for a year.
    if let Err(error) = cloudflare
        .purge_cache_urls(&[format!("https://{static_hostname}/__fn0_connect_probe")])
        .await
    {
        tracing::warn!(%error, "cloudflare_connect purge probe failed");
        missing.push("Zone -> Cache Purge -> Purge".to_string());
    }
    if !missing.is_empty() {
        return Output::MissingPermissions { missing };
    }

    let secret = dataplane_secret_from_token(&req.body.api_token);
    let (bootstrap_token_ciphertext, dataplane_secret_ciphertext) = match (
        vault::encrypt(req.body.api_token.as_bytes()).await,
        vault::encrypt(secret.as_bytes()).await,
    ) {
        (Ok(token_ciphertext), Ok(secret_ciphertext)) => (token_ciphertext, secret_ciphertext),
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

    let config = ProjectCloudflareConfigDoc {
        project_id: req.body.project_id.clone(),
        account_id: req.body.account_id.clone(),
        zone_id: req.body.zone_id.clone(),
        zone_name,
        static_hostname: static_hostname.clone(),
        asset_bucket: USER_ASSET_BUCKET.to_string(),
        page_bucket: USER_PAGE_BUCKET.to_string(),
        bootstrap_token_ciphertext,
        dataplane_access_key_id: token.id,
        dataplane_secret_ciphertext,
        state: CloudflareConnectionState::Migrating,
        checked_at: forte_sdk::now(),
        config_version: existing.map(|doc| doc.config_version + 1).unwrap_or(1),
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

    let storage = match ProjectStorage::resolve_connected(&db, &req.body.project_id).await {
        Ok(Some(storage)) => storage,
        Ok(None) => {
            return Output::InternalError {
                reason: "config vanished between write and read".to_string(),
            };
        }
        Err(error) => {
            tracing::error!("cloudflare_connect ProjectStorage::resolve_connected: {error}");
            return Output::InternalError {
                reason: format!("resolve: {error}"),
            };
        }
    };
    if let Err(error) = storage.ensure_resources(&req.body.project_id).await {
        tracing::error!("cloudflare_connect ensure_resources: {error}");
        return Output::InternalError {
            reason: format!("provisioning: {error}"),
        };
    }

    // The manifest is not published here: workers must keep reading the
    // platform account until the migration has copied everything across.
    // `byoc_migrate` flips both when it finishes.
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

    Output::Ok {
        static_hostname,
        asset_bucket: USER_ASSET_BUCKET.to_string(),
        page_bucket: USER_PAGE_BUCKET.to_string(),
    }
}
