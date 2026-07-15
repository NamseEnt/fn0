//! Owner-initiated project deletion. Verifies ownership, then enqueues the
//! `project_teardown` queue task which removes every resource the project
//! accretes (routing, cron, bundles, static assets, object storage, turso
//! database, docs). Deletion is asynchronous: `Ok` means teardown is
//! enqueued, not finished.

use crate::common::auth;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok,
    NotLoggedIn,
    NotFound,
    InternalError,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let project = match (ProjectDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&doc_db::turso())
    .await
    {
        Ok(Some(project)) => project,
        Ok(None) => return Output::NotFound,
        Err(e) => {
            tracing::error!("delete_project get ProjectDoc: {e}");
            return Output::InternalError;
        }
    };
    if project.owner_github_id != user.github_id {
        return Output::NotFound;
    }

    if let Err(e) = crate::enqueue::project_teardown(crate::queue_task::project_teardown::Input {
        project_id: req.body.project_id.clone(),
    })
    .await
    {
        tracing::error!("delete_project enqueue project_teardown: {e}");
        return Output::InternalError;
    }

    tracing::info!(project_id = %req.body.project_id, "delete_project: teardown enqueued");
    Output::Ok
}
