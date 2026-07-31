use crate::common::byoc::ProjectStorage;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

/// Cloudflare's per-request ceiling for by-URL purge.
const MAX_URLS_PER_REQUEST: usize = 100;

#[derive(Serialize, Deserialize)]
pub struct Input {
    /// Which project's zone to purge on. Absent from messages enqueued before
    /// projects could own their own zone; those are platform-zone purges.
    #[serde(default)]
    pub project_id: Option<String>,
    pub urls: Vec<String>,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    if input.urls.is_empty() {
        return Ok(());
    }

    let cloudflare = match &input.project_id {
        Some(project_id) => ProjectStorage::resolve(&doc_db::turso(), project_id)
            .await?
            .purge_client(),
        None => crate::common::cloudflare::CloudflareClient::from_env()?,
    };
    for chunk in input.urls.chunks(MAX_URLS_PER_REQUEST) {
        let started = std::time::Instant::now();
        // A failure must propagate: the queue retry is the only thing standing
        // between a replaced object and a year of `s-maxage` serving the old
        // bytes.
        cloudflare
            .purge_cache_urls(chunk)
            .await
            .inspect_err(|error| {
                tracing::warn!(
                    urls = chunk.len(),
                    duration_ms = started.elapsed().as_millis() as u64,
                    %error,
                    "public object purge failed"
                )
            })?;
        tracing::info!(
            urls = chunk.len(),
            duration_ms = started.elapsed().as_millis() as u64,
            "public object purge completed"
        );
    }
    Ok(())
}
