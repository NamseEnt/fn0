//! Purges a project's cache tag around a code-version activation.
//!
//! Retry policy: a failure returns `Err` and the queue message is not acked,
//! so OCI Queue redelivers it after its 60s visibility timeout and keeps doing
//! so. The project degrades to uncached SSR until a purge lands, never to
//! stale HTML, and no failure mode pins it there permanently.

use crate::common::cloudflare::CloudflareClient;
use crate::docs::*;
use fn0_shared_schema::{
    STATIC_CACHE_STATE_ACTIVATING, STATIC_CACHE_STATE_ACTIVE, STATIC_CACHE_STATE_PRE_PURGE,
};
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// Cloudflare's tag-purge budget on the Free plan: 5 requests per minute
// account-wide into a bucket of 25, with up to 100 tags per request.
// https://developers.cloudflare.com/cache/how-to/purge-cache/#availability-and-limits
const PURGE_BUCKET_CAPACITY: f64 = 25.0;
const PURGE_REFILL_PER_SECOND: f64 = 5.0 / 60.0;
const PURGE_TAGS_PER_REQUEST: usize = 100;
// Long enough to cover a redelivery cycle that observes a purge another task
// performed, short enough that the doc does not grow without bound.
const PURGE_COMPLETION_RETENTION_SECONDS: i64 = 3600;

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub project_id: String,
    pub code_version: u64,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    let db = doc_db::turso();
    let Some(manifest) = (WorkerManifestDocGet {}).send_with(&db).await? else {
        return Ok(());
    };
    let Some(entry) = manifest.project_manifests.get(&input.project_id) else {
        return Ok(());
    };
    if entry.static_cache_state == STATIC_CACHE_STATE_ACTIVE {
        return Ok(());
    }
    let pending = entry.pending_code_version == Some(input.code_version)
        && entry.static_cache_state == STATIC_CACHE_STATE_PRE_PURGE;
    let activating = entry.code_version == input.code_version
        && entry.static_cache_state == STATIC_CACHE_STATE_ACTIVATING;
    if !pending && !activating {
        return Ok(());
    }

    let cloudflare = CloudflareClient::from_env()?;
    if entry.static_cache_state == STATIC_CACHE_STATE_PRE_PURGE {
        purge(&db, &cloudflare, &input, "pre_purge").await?;
        set_activating(&db, &input).await?;
    }

    let manifest = (WorkerManifestDocGet {}).send_with(&db).await?;
    let Some(entry) = manifest
        .as_ref()
        .and_then(|value| value.project_manifests.get(&input.project_id))
    else {
        return Ok(());
    };
    if entry.code_version != input.code_version
        || entry.static_cache_state != STATIC_CACHE_STATE_ACTIVATING
    {
        return Ok(());
    }

    purge(&db, &cloudflare, &input, "post_purge").await?;
    set_active(&db, &input).await?;

    crate::enqueue::deploy_artifact_prune(crate::queue_task::deploy_artifact_prune::Input {
        project_id: input.project_id,
    })
    .await
}

/// Spends one token of the shared budget on as many pending purges as
/// Cloudflare accepts per request, so a burst of deploys costs one request
/// instead of one each. Returns once this project's purge for this phase is
/// known to have happened, whether this call performed it or another task's
/// batch already covered it.
async fn purge(
    db: &doc_db::Database,
    cloudflare: &CloudflareClient,
    input: &Input,
    phase: &str,
) -> anyhow::Result<()> {
    let request = CachePurgeRequest {
        project_id: input.project_id.clone(),
        code_version: input.code_version,
        phase: phase.to_string(),
    };
    let batch = match claim_batch(db, &request).await? {
        Claim::AlreadyPurged => {
            tracing::info!(
                project_id = %input.project_id,
                code_version = input.code_version,
                phase,
                "static cache purge already covered by another batch"
            );
            return Ok(());
        }
        Claim::OutOfBudget => anyhow::bail!(
            "cache purge budget exhausted for {} {phase}; waiting for redelivery",
            input.project_id
        ),
        Claim::Batch(batch) => batch,
    };

    let tags: BTreeSet<String> = batch
        .iter()
        .map(|request| format!("fn0-project-{}", request.project_id))
        .collect();
    let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();

    let started = std::time::Instant::now();
    match cloudflare.purge_cache_tags(&tag_refs).await {
        Ok(()) => {
            record_purged(db, &batch).await?;
            tracing::info!(
                project_id = %input.project_id,
                code_version = input.code_version,
                phase,
                batched_tags = tag_refs.len(),
                duration_ms = started.elapsed().as_millis() as u64,
                "static cache purge completed"
            );
            Ok(())
        }
        Err(error) => {
            // The batch left `pending` when it was claimed, so without this the
            // other projects in it would wait for their own redelivery to
            // re-add themselves.
            restore_pending(db, &batch).await?;
            tracing::warn!(
                project_id = %input.project_id,
                code_version = input.code_version,
                phase,
                batched_tags = tag_refs.len(),
                duration_ms = started.elapsed().as_millis() as u64,
                %error,
                "static cache purge failed"
            );
            Err(error)
        }
    }
}

enum Claim {
    AlreadyPurged,
    OutOfBudget,
    Batch(Vec<CachePurgeRequest>),
}

async fn claim_batch(db: &doc_db::Database, request: &CachePurgeRequest) -> anyhow::Result<Claim> {
    let now = forte_sdk::now();
    let result = db
        .trx(|trx| {
            let request = request.clone();
            async move {
                let mut doc = match trx.get(CachePurgeDocGet {}).await? {
                    Some(doc) => doc,
                    None => trx.create(CachePurgeDoc {
                        tokens: PURGE_BUCKET_CAPACITY,
                        refilled_at: now,
                        pending: Vec::new(),
                        completed: Vec::new(),
                    })?,
                };

                doc.completed.retain(|completed| {
                    (now - completed.purged_at).num_seconds() < PURGE_COMPLETION_RETENTION_SECONDS
                });
                if doc
                    .completed
                    .iter()
                    .any(|completed| completed.request == request)
                {
                    return trx.commit::<_, ()>(Claim::AlreadyPurged);
                }

                let elapsed_seconds =
                    (now - doc.refilled_at).num_milliseconds().max(0) as f64 / 1000.0;
                doc.tokens = (doc.tokens + elapsed_seconds * PURGE_REFILL_PER_SECOND)
                    .min(PURGE_BUCKET_CAPACITY);
                doc.refilled_at = now;

                if !doc.pending.contains(&request) {
                    doc.pending.push(request);
                }
                if doc.tokens < 1.0 {
                    return trx.commit::<_, ()>(Claim::OutOfBudget);
                }
                doc.tokens -= 1.0;
                let batch_size = doc.pending.len().min(PURGE_TAGS_PER_REQUEST);
                let batch: Vec<CachePurgeRequest> = doc.pending.drain(..batch_size).collect();
                trx.commit::<_, ()>(Claim::Batch(batch))
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(claim) => Ok(claim),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => anyhow::bail!("claim_batch conflict: {error:?}"),
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

async fn record_purged(db: &doc_db::Database, batch: &[CachePurgeRequest]) -> anyhow::Result<()> {
    update_purge_doc(db, batch, |doc, batch, now| {
        for request in batch {
            doc.completed.push(CompletedCachePurge {
                request: request.clone(),
                purged_at: now,
            });
        }
    })
    .await
}

async fn restore_pending(db: &doc_db::Database, batch: &[CachePurgeRequest]) -> anyhow::Result<()> {
    update_purge_doc(db, batch, |doc, batch, _now| {
        for request in batch {
            if !doc.pending.contains(request) {
                doc.pending.push(request.clone());
            }
        }
    })
    .await
}

async fn update_purge_doc(
    db: &doc_db::Database,
    batch: &[CachePurgeRequest],
    apply: fn(&mut CachePurgeDoc, &[CachePurgeRequest], DateTime),
) -> anyhow::Result<()> {
    let now = forte_sdk::now();
    let result = db
        .trx(|trx| {
            let batch = batch.to_vec();
            async move {
                let Some(mut doc) = trx.get(CachePurgeDocGet {}).await? else {
                    return trx.commit::<_, ()>(());
                };
                apply(&mut doc, &batch, now);
                trx.commit::<_, ()>(())
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(()) => Ok(()),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("update_purge_doc conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

async fn set_activating(db: &doc_db::Database, input: &Input) -> anyhow::Result<()> {
    let project_id = input.project_id.clone();
    let code_version = input.code_version;
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            async move {
                let Some(mut manifest) = trx.get(WorkerManifestDocGet {}).await? else {
                    return trx.commit::<_, ()>(());
                };
                let Some(entry) = manifest.project_manifests.get_mut(&project_id) else {
                    return trx.commit::<_, ()>(());
                };
                if entry.pending_code_version == Some(code_version)
                    && entry.static_cache_state == STATIC_CACHE_STATE_PRE_PURGE
                {
                    entry.code_version = code_version;
                    entry.pending_code_version = None;
                    entry.static_cache_state = STATIC_CACHE_STATE_ACTIVATING.to_string();
                    manifest.manifest_version += 1;
                }
                trx.commit::<_, ()>(())
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(()) => Ok(()),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => anyhow::bail!("set_activating conflict: {error:?}"),
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

async fn set_active(db: &doc_db::Database, input: &Input) -> anyhow::Result<()> {
    let project_id = input.project_id.clone();
    let code_version = input.code_version;
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            async move {
                let Some(mut manifest) = trx.get(WorkerManifestDocGet {}).await? else {
                    return trx.commit::<_, ()>(());
                };
                let Some(entry) = manifest.project_manifests.get_mut(&project_id) else {
                    return trx.commit::<_, ()>(());
                };
                if entry.code_version == code_version
                    && entry.static_cache_state == STATIC_CACHE_STATE_ACTIVATING
                {
                    entry.static_cache_state = STATIC_CACHE_STATE_ACTIVE.to_string();
                    manifest.manifest_version += 1;
                }
                trx.commit::<_, ()>(())
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(()) => Ok(()),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => anyhow::bail!("set_active conflict: {error:?}"),
        doc_db::TrxResult::Err(error) => Err(error),
    }
}
