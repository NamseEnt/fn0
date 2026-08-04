use crate::common::auth;
use crate::common::byoc::ProjectStorage;
use crate::common::cloudflare::CachePurgeFile;
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

    let urls: Vec<String> = req
        .body
        .keys
        .iter()
        .map(|key| served_url(&cdn_origin, key))
        .collect();

    if urls.is_empty() {
        return Output::Ok { urls };
    }

    if let Err(error) =
        enqueue::public_object_purge(crate::queue_task::public_object_purge::Input {
            project_id: req.body.project_id.clone(),
            urls: urls.iter().cloned().map(CachePurgeFile::Url).collect(),
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

/// The URL the CDN serves this key at, which is the only URL a purge can
/// invalidate.
///
/// The key maps straight onto the path: the project owns the bucket, so there
/// is no project segment. This has to stay identical to `public_url_for` in the
/// `fn0` crate, which builds the URL the guest hands out. The two live in
/// crates that share no dependency, so nothing but this note couples them.
fn served_url(cdn_origin: &str, key: &str) -> String {
    format!("{cdn_origin}/{}", key.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::served_url;

    #[test]
    fn served_url_carries_no_project_segment() {
        assert_eq!(
            served_url(
                "https://fn0-proj1-public-object-storage.example",
                "captures/8/captures.json"
            ),
            "https://fn0-proj1-public-object-storage.example/captures/8/captures.json"
        );
    }

    #[test]
    fn a_leading_slash_does_not_double_up() {
        assert_eq!(
            served_url("https://cdn.example", "/clips/intro.mp4"),
            "https://cdn.example/clips/intro.mp4"
        );
    }
}
