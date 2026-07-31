//! The CDN origin a project's frontend assets will be fetched from.
//!
//! The CLI needs this *before* it builds: the base URL is compiled into the
//! bundle, and the `deploy` action that hands back upload URLs only runs after
//! the build, with the file list. Guessing it from a fixed platform domain is
//! what made the origin process-global in the first place.

use crate::common::auth;
use crate::common::byoc::ProjectStorage;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub code_version: u64,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        /// Ends with a slash: it is a base for relative asset paths.
        static_base_url: String,
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

    match ProjectStorage::resolve(&db, &req.body.project_id).await {
        Ok(storage) => Output::Ok {
            static_base_url: storage.asset_base_url(req.body.code_version),
        },
        Err(error) => Output::InternalError {
            reason: format!("resolve: {error}"),
        },
    }
}
