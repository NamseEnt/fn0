//! Purges a project's cache tag around a code-version activation.
//!
//! Retry policy: a failure returns `Err` and the queue message is not acked,
//! so OCI Queue redelivers it after its 60s visibility timeout and keeps doing
//! so. The project degrades to uncached SSR until a purge lands, never to
//! stale HTML, and no failure mode pins it there permanently. Redelivery is
//! also what absorbs Cloudflare's tag-purge rate limit, which is per account:
//! an owner deploying several projects at once may see a 429, and the message
//! comes back a minute later.

use crate::common::byoc::ProjectStorage;
use crate::docs::*;
use fn0_shared_schema::{
    STATIC_CACHE_STATE_ACTIVATING, STATIC_CACHE_STATE_ACTIVE, STATIC_CACHE_STATE_PRE_PURGE,
};
use forte_sdk::*;
use serde::{Deserialize, Serialize};

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

    // The edge holding this project's cached pages is the owner's own, reached
    // through the custom domain on their zone, and purged with their own token
    // off their own budget.
    let user_zone = match entry.custom_domain.clone() {
        Some(domain) => Some((
            ProjectStorage::resolve(&db, &input.project_id).await?,
            domain,
        )),
        None => None,
    };
    if entry.static_cache_state == STATIC_CACHE_STATE_PRE_PURGE {
        purge_user_zone(user_zone.as_ref(), &input, "pre_purge").await?;
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

    purge_user_zone(user_zone.as_ref(), &input, "post_purge").await?;
    set_active(&db, &input).await?;

    crate::enqueue::deploy_artifact_prune(crate::queue_task::deploy_artifact_prune::Input {
        project_id: input.project_id,
    })
    .await
}

/// Invalidates the copy held by the owner's own zone, which exists only while
/// the project has a custom domain there.
async fn purge_user_zone(
    user_zone: Option<&(ProjectStorage, String)>,
    input: &Input,
    phase: &str,
) -> anyhow::Result<()> {
    let Some((storage, custom_domain)) = user_zone else {
        return Ok(());
    };
    let tag = format!("fn0-project-{}", input.project_id);
    storage
        .purge_client()
        .purge_cache_tags(&[tag.as_str()])
        .await?;
    tracing::info!(
        project_id = %input.project_id,
        code_version = input.code_version,
        custom_domain,
        phase,
        "static cache purge completed on the project owner's zone"
    );
    Ok(())
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
