//! Whether a project's connected Cloudflare account still works.
//!
//! Revocation is a live-traffic failure: a deleted token breaks the project's
//! object storage at request time, with no signal anywhere else. This re-probes
//! the credentials on demand and records the verdict, so the answer is a
//! current fact rather than what was true at connect time.

use crate::common::auth;
use crate::common::byoc::ProjectStorage;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
}

#[derive(Serialize)]
pub enum Output {
    /// The project runs on the platform's own Cloudflare account.
    Platform,
    Connected {
        account_id: String,
        zone_name: String,
        static_hostname: String,
        asset_bucket: String,
        page_bucket: String,
        healthy: bool,
        problem: Option<String>,
        /// Existing objects are still being copied across; the project is
        /// still served from the platform account until they are.
        migrating: bool,
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
            return Output::InternalError {
                reason: format!("ProjectDocGet: {error}"),
            };
        }
    };
    if project.owner_github_id != user.github_id {
        return Output::NotFound;
    }

    let config = match (ProjectCloudflareConfigDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(Some(config)) => config,
        Ok(None) => return Output::Platform,
        Err(error) => {
            return Output::InternalError {
                reason: format!("config read: {error}"),
            };
        }
    };

    let storage = match ProjectStorage::resolve_connected(&db, &req.body.project_id).await {
        Ok(Some(storage)) => storage,
        Ok(None) => return Output::Platform,
        Err(error) => {
            return Output::InternalError {
                reason: format!("resolve: {error}"),
            };
        }
    };
    let migrating = config.state == CloudflareConnectionState::Migrating;
    let cloudflare = match storage.purge_client() {
        Ok(cloudflare) => cloudflare,
        Err(error) => {
            return Output::InternalError {
                reason: format!("client: {error}"),
            };
        }
    };

    let problem = match cloudflare.verify_token().await {
        Ok(_) => match cloudflare.r2_bucket_exists(&storage.asset_bucket).await {
            Ok(true) => None,
            Ok(false) => Some(format!("bucket {} is gone", storage.asset_bucket)),
            Err(error) => Some(format!("bucket check failed: {error}")),
        },
        Err(error) => Some(format!("token is no longer valid: {error}")),
    };

    // A migration in flight is not a health verdict, so it is left alone: the
    // queue task owns that transition and overwriting it here would switch the
    // project over before its objects had arrived.
    let state = match (&problem, migrating) {
        (_, true) => CloudflareConnectionState::Migrating,
        (None, false) => CloudflareConnectionState::Ok,
        (Some(problem), false) => CloudflareConnectionState::Degraded {
            missing: vec![problem.clone()],
        },
    };
    if state != config.state
        && let Err(error) = (ProjectCloudflareConfigDocPut(ProjectCloudflareConfigDoc {
            state,
            checked_at: forte_sdk::now(),
            ..config.clone()
        }))
        .send_with(&db)
        .await
    {
        tracing::warn!(%error, "cloudflare_status could not record the new state");
    }

    Output::Connected {
        account_id: config.account_id,
        zone_name: config.zone_name,
        static_hostname: config.static_hostname,
        asset_bucket: config.asset_bucket,
        page_bucket: config.page_bucket,
        healthy: problem.is_none(),
        problem,
        migrating,
    }
}
