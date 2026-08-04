//! Invalidates individual `cache_static` pages an app asked to replace.
//!
//! The deploy-time counterpart is [`crate::queue_task::static_cache_purge`],
//! which drops the whole project by cache tag. This one takes route paths and
//! purges by URL, because tag purge is 5/min account-wide against 800/s for
//! URLs — a budget an app-callable path would exhaust for everyone.
//!
//! Retry policy matches the deploy purge: a failure returns `Err`, the message
//! is redelivered, and the page stays stale rather than silently missing its
//! invalidation.

use crate::common::byoc::ProjectStorage;
use crate::common::cloudflare::CachePurgeFile;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

/// Cloudflare's per-request ceiling for by-URL purge.
const MAX_URLS_PER_REQUEST: usize = 100;

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub project_id: String,
    /// Route paths as a visitor requests them, already normalized by the
    /// worker: `/episode/1`, never a full URL.
    pub paths: Vec<String>,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    if input.paths.is_empty() {
        return Ok(());
    }

    let db = doc_db::turso();
    let Some(manifest) = (WorkerManifestDocGet {}).send_with(&db).await? else {
        return Ok(());
    };
    let Some(entry) = manifest.project_manifests.get(&input.project_id) else {
        return Ok(());
    };
    // No domain means no zone of the project's own, so nothing cached the page
    // in the first place.
    let Some(domain) = entry.custom_domain.as_deref() else {
        return Ok(());
    };

    let files: Vec<CachePurgeFile> = input
        .paths
        .iter()
        .map(|path| CachePurgeFile::from_url(page_url(domain, path), None))
        .collect();

    let cloudflare = ProjectStorage::resolve(&db, &input.project_id)
        .await?
        .purge_client();
    for chunk in files.chunks(MAX_URLS_PER_REQUEST) {
        let started = std::time::Instant::now();
        cloudflare
            .purge_cache_urls(chunk)
            .await
            .inspect_err(|error| {
                tracing::warn!(
                    project_id = %input.project_id,
                    files = chunk.len(),
                    duration_ms = started.elapsed().as_millis() as u64,
                    %error,
                    "static page purge failed"
                )
            })?;
        tracing::info!(
            project_id = %input.project_id,
            files = chunk.len(),
            duration_ms = started.elapsed().as_millis() as u64,
            "static page purge completed"
        );
    }
    Ok(())
}

/// The URL the edge holds this page at.
///
/// One entry, with no `Origin`: a page is fetched by top-level navigation,
/// which sends none, so the entry a visitor reads is the one purged here.
/// Public objects need the `Origin` expansion
/// ([`crate::queue_task::public_object_purge`]) because a browser reaches them
/// through a cross-origin `fetch`; a page has no such reader, and one that
/// arrives with an `Origin` cannot read the response anyway — the page carries
/// no CORS headers.
fn page_url(domain: &str, path: &str) -> String {
    format!("https://{domain}{path}")
}

#[cfg(test)]
mod tests {
    use super::page_url;

    #[test]
    fn builds_the_url_a_visitor_requests() {
        assert_eq!(
            page_url("reading-party.example", "/episode/1"),
            "https://reading-party.example/episode/1"
        );
        assert_eq!(page_url("app.example", "/"), "https://app.example/");
    }
}
