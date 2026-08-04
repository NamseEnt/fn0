use crate::common::byoc::ProjectStorage;
use crate::common::cloudflare::{CachePurgeFile, CachePurgeRequest};
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

/// Cloudflare's per-request ceiling for by-URL purge.
const MAX_URLS_PER_REQUEST: usize = 100;

#[derive(Serialize, Deserialize)]
pub struct Input {
    /// Which project's zone to purge on. Absent from messages enqueued before
    /// projects could own their own zone; those are platform-zone purges.
    #[serde(default)]
    pub project_id: String,
    pub urls: Vec<CachePurgeFile>,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    if input.urls.is_empty() {
        return Ok(());
    }

    let db = doc_db::turso();
    let manifest = (WorkerManifestDocGet {}).send_with(&db).await?;
    let files = expand_purge_files(
        &requested_urls(&input.urls),
        &readable_origins(manifest.as_ref(), &input.project_id),
    );

    let cloudflare = ProjectStorage::resolve(&db, &input.project_id)
        .await?
        .purge_client();
    for chunk in files.chunks(MAX_URLS_PER_REQUEST) {
        let started = std::time::Instant::now();
        // A failure must propagate: the queue retry is the only thing standing
        // between a replaced object and a year of `s-maxage` serving the old
        // bytes.
        cloudflare
            .purge_cache_urls(chunk)
            .await
            .inspect_err(|error| {
                tracing::warn!(
                    files = chunk.len(),
                    duration_ms = started.elapsed().as_millis() as u64,
                    %error,
                    "public object purge failed"
                )
            })?;
        tracing::info!(
            files = chunk.len(),
            duration_ms = started.elapsed().as_millis() as u64,
            "public object purge completed"
        );
    }
    Ok(())
}

/// Cloudflare keys a separate cache entry per `Origin` on an R2 custom domain,
/// so one stored object is several cache entries and a purge has to name each.
/// Expanding here rather than at the enqueue site is what puts the app's own
/// `put`/`delete` — which reaches this queue through the worker and carries no
/// domain knowledge — on the same footing as the `public_purge` action.
///
/// A message enqueued before this moved already carries the expanded entries;
/// collapsing back to the requested URLs first keeps re-expansion from
/// multiplying them.
fn requested_urls(files: &[CachePurgeFile]) -> Vec<String> {
    let mut urls: Vec<String> = Vec::with_capacity(files.len());
    for file in files {
        let url = match file {
            CachePurgeFile::Url(url) => url,
            CachePurgeFile::Request(CachePurgeRequest { url, .. }) => url,
        };
        if !urls.iter().any(|seen| seen == url) {
            urls.push(url.clone());
        }
    }
    urls
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
    use super::{CachePurgeFile, Input, expand_purge_files, readable_origins, requested_urls};
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
    fn input_accepts_legacy_string_files() {
        let input: Input = serde_json::from_value(json!({
            "project_id": "project",
            "urls": ["https://cdn.example/asset.js"]
        }))
        .unwrap();

        assert!(matches!(
            input.urls.as_slice(),
            [CachePurgeFile::Url(url)] if url == "https://cdn.example/asset.js"
        ));
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

    #[test]
    fn a_worker_enqueued_url_expands_the_same_as_an_action_enqueued_one() {
        let manifest = manifest_with_domain(Some("app.example"));
        let origins = readable_origins(Some(&manifest), "project");
        let from_worker: Input = serde_json::from_value(json!({
            "project_id": "project",
            "urls": ["https://cdn.example/clips/intro.mp4"]
        }))
        .unwrap();

        assert_eq!(
            serde_json::to_value(expand_purge_files(
                &requested_urls(&from_worker.urls),
                &origins
            ))
            .unwrap(),
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
    fn an_already_expanded_message_does_not_multiply() {
        let already_expanded: Input = serde_json::from_value(json!({
            "project_id": "project",
            "urls": [
                { "url": "https://cdn.example/clips/intro.mp4" },
                {
                    "url": "https://cdn.example/clips/intro.mp4",
                    "headers": { "Origin": "https://app.example" }
                }
            ]
        }))
        .unwrap();

        assert_eq!(
            requested_urls(&already_expanded.urls),
            vec!["https://cdn.example/clips/intro.mp4".to_string()]
        );
    }
}
