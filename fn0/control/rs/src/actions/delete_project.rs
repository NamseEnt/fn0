//! Owner-initiated project deletion. Verifies ownership, records an access
//! revoke tombstone, then enqueues the `project_teardown` queue task. Deletion
//! is asynchronous: `Ok` means teardown is enqueued, not finished. The task
//! does not change Signy's retention policy or purge Signy telemetry.

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

    let db = doc_db::turso();
    let deletion = match (ProjectDeletionDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(deletion) => deletion,
        Err(error) => {
            tracing::error!(project_id = %req.body.project_id, %error, "delete_project: failed to read deletion tombstone");
            return Output::InternalError;
        }
    };
    let deletion_now = now();
    if deletion.is_none()
        && let Err(error) = (ProjectDeletionDocPut(ProjectDeletionDoc {
            project_id: req.body.project_id.clone(),
            fence: 1,
            state: ProjectDeletionState::RevokePending,
            revoke_pending_since: Some(deletion_now),
            updated_at: deletion_now,
            last_error: None,
        }))
        .send_with(&db)
        .await
    {
        tracing::error!(project_id = %req.body.project_id, %error, "delete_project: tombstone write failed");
        return Output::InternalError;
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
