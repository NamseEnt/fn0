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

    let manifest = match (WorkerManifestDocGet {}).send_with(&db).await {
        Ok(manifest) => manifest,
        Err(error) => {
            tracing::error!(?error, "public_purge WorkerManifestDocGet");
            return Output::InternalError {
                reason: format!("WorkerManifestDocGet: {error}"),
            };
        }
    };
    let origins = readable_origins(manifest.as_ref(), &req.body.project_id);

    let urls: Vec<String> = req
        .body
        .keys
        .iter()
        .map(|key| served_url(&cdn_origin, key))
        .collect();
    let files = expand_purge_files(&urls, &origins);

    if urls.is_empty() {
        return Output::Ok { urls };
    }

    if let Err(error) =
        enqueue::public_object_purge(crate::queue_task::public_object_purge::Input {
            project_id: req.body.project_id.clone(),
            urls: files,
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

fn readable_origins(manifest: Option<&WorkerManifestDoc>, project_id: &str) -> Vec<Option<String>> {
    let custom_domain = manifest
        .and_then(|manifest| manifest.project_manifests.get(project_id))
        .and_then(|entry| entry.custom_domain.as_deref());
    match custom_domain {
        Some(custom_domain) => vec![None, Some(format!("https://{custom_domain}"))],
        None => vec![None],
    }
}

fn expand_purge_files(urls: &[String], origins: &[Option<String>]) -> Vec<CachePurgeFile> {
    urls.iter()
        .flat_map(|url| {
            origins
                .iter()
                .map(|origin| CachePurgeFile::from_url(url.clone(), origin.as_deref()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{expand_purge_files, readable_origins, served_url};
    use crate::docs::{WorkerManifestDoc, WorkerProjectManifest};
    use fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE;
    use serde_json::json;
    use std::collections::HashMap;

    fn manifest_with_domain(custom_domain: Option<&str>) -> WorkerManifestDoc {
        let mut project_manifests = HashMap::new();
        project_manifests.insert(
            "project".to_string(),
            WorkerProjectManifest {
                code_version: 0,
                custom_domain: custom_domain.map(str::to_string),
                static_cache_state: STATIC_CACHE_STATE_ACTIVE.to_string(),
                pending_code_version: None,
                storage: None,
            },
        );
        WorkerManifestDoc {
            manifest_version: 1,
            project_manifests,
        }
    }

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

    #[test]
    fn purge_files_include_the_no_origin_and_project_origin_entries() {
        let manifest = manifest_with_domain(Some("app.example"));
        let origins = readable_origins(Some(&manifest), "project");
        let urls = vec!["https://cdn.example/clips/intro.mp4".to_string()];

        assert_eq!(
            serde_json::to_value(expand_purge_files(&urls, &origins)).unwrap(),
            json!([
                { "url": "https://cdn.example/clips/intro.mp4" },
                {
                    "url": "https://cdn.example/clips/intro.mp4",
                    "headers": { "Origin": "https://app.example" }
                }
            ])
        );
    }

    #[test]
    fn purge_files_include_only_the_no_origin_entry_without_a_project_domain() {
        let manifest = manifest_with_domain(None);
        let origins = readable_origins(Some(&manifest), "project");
        let urls = vec!["https://cdn.example/clips/intro.mp4".to_string()];

        assert_eq!(
            serde_json::to_value(expand_purge_files(&urls, &origins)).unwrap(),
            json!([{ "url": "https://cdn.example/clips/intro.mp4" }])
        );
    }
}
