//! Whether a project's connected Cloudflare account still works.
//!
//! Revocation is a live-traffic failure: a deleted token breaks the project's
//! object storage at request time, with no signal anywhere else. This re-probes
//! the credentials on demand and records the verdict, so the answer is a
//! current fact rather than what was true at connect time.

use crate::common::auth;
use crate::common::byoc::ProjectStorage;
use crate::common::r2_store::ProjectR2Store;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
}

#[derive(Serialize)]
pub enum Output {
    /// No Cloudflare account is connected, so the project has nowhere to store
    /// and cannot serve a frontend.
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
        Ok(None) => return Output::NotConnected,
        Err(error) => {
            return Output::InternalError {
                reason: format!("config read: {error}"),
            };
        }
    };

    let storage = match ProjectStorage::resolve(&db, &req.body.project_id).await {
        Ok(storage) => storage,
        Err(error) => {
            return Output::InternalError {
                reason: format!("resolve: {error}"),
            };
        }
    };

    // Probed the way traffic reaches it: an S3 list with the credential itself.
    // The REST bucket endpoint would answer 403 for every healthy connection,
    // because the credentials fn0 stores deliberately cannot call the
    // Cloudflare API at all — which is also what makes a real revocation
    // indistinguishable there. Both R2 credentials are probed, since either can
    // be revoked on its own and each covers buckets the other cannot reach.
    let problem = match storage.purge_client().verify_token().await {
        Err(error) => Some(format!("purge token is no longer valid: {error}")),
        Ok(_) => {
            let probes = [
                (
                    ProjectR2Store::private_objects(&storage),
                    &storage.private_object_storage_bucket,
                ),
                (
                    ProjectR2Store::frontend_assets(&storage),
                    &storage.frontend_asset_bucket,
                ),
            ];
            let mut found = None;
            for (store, bucket) in probes {
                if let Err(error) = store.list_all("", forte_sdk::now()).await {
                    found = Some(format!("{bucket} is unreachable: {error}"));
                    break;
                }
            }
            found
        }
    };

    let state = match &problem {
        None => CloudflareConnectionState::Ok,
        Some(problem) => CloudflareConnectionState::Degraded {
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
        frontend_asset_hostname: config.frontend_asset_hostname,
        public_object_storage_hostname: config.public_object_storage_hostname,
        frontend_asset_bucket: config.frontend_asset_bucket,
        public_object_storage_bucket: config.public_object_storage_bucket,
        private_object_storage_bucket: config.private_object_storage_bucket,
        rendered_html_cache_bucket: config.rendered_html_cache_bucket,
        healthy: problem.is_none(),
        problem,
    }
}
