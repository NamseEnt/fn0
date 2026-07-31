use crate::common::auth;
use crate::common::byoc::ProjectStorage;
use crate::docs::*;
use crate::route_generated::enqueue;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

/// Bounded so one call cannot hand Cloudflare an unbounded batch; the queue
/// task chunks what it receives, but the enqueue itself has a message size
/// limit.
const MAX_KEYS_PER_CALL: usize = 100;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    /// Keys inside the project's public namespace, e.g. `captures/1/0.mp4`.
    pub keys: Vec<String>,
}

#[derive(Serialize)]
pub enum Output {
    Ok { urls: Vec<String> },
    NotLoggedIn,
    NotFound,
    Forbidden,
    TooManyKeys { max: usize },
    InternalError { reason: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    if req.body.keys.len() > MAX_KEYS_PER_CALL {
        return Output::TooManyKeys {
            max: MAX_KEYS_PER_CALL,
        };
    }

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
            tracing::error!(?error, "public_purge ProjectDocGet");
            return Output::InternalError {
                reason: format!("ProjectDocGet: {error}"),
            };
        }
    };

    if project.owner_github_id != user.github_id {
        return Output::Forbidden;
    }

    let storage = match ProjectStorage::resolve(&db, &req.body.project_id).await {
        Ok(storage) => storage,
        Err(error) => {
            tracing::error!(?error, "public_purge ProjectStorage::resolve");
            return Output::InternalError {
                reason: format!("resolve: {error}"),
            };
        }
    };
    let cdn_origin = format!(
        "https://{}",
        storage.connection.public_object_storage_hostname
    );
    let cdn_origin = cdn_origin.as_str();

    let urls: Vec<String> = req
        .body
        .keys
        .iter()
        .map(|key| {
            format!(
                "{cdn_origin}/{}/public/{}",
                req.body.project_id,
                key.trim_start_matches('/')
            )
        })
        .collect();

    if urls.is_empty() {
        return Output::Ok { urls };
    }

    if let Err(error) =
        enqueue::public_object_purge(crate::queue_task::public_object_purge::Input {
            project_id: req.body.project_id.clone(),
            urls: urls.clone(),
        })
        .await
    {
        tracing::error!(?error, "public_purge enqueue");
        return Output::InternalError {
            reason: format!("enqueue: {error}"),
        };
    }

    Output::Ok { urls }
}
