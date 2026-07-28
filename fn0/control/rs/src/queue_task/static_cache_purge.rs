use crate::common::cloudflare::CloudflareClient;
use crate::docs::*;
use forte_sdk::*;
use fn0_shared_schema::{
    STATIC_CACHE_STATE_ACTIVE, STATIC_CACHE_STATE_ACTIVATING, STATIC_CACHE_STATE_PRE_PURGE,
};
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

    let cloudflare = CloudflareClient::from_env()?;
    let cache_tag = format!("fn0-project-{}", input.project_id);
    if entry.static_cache_state == STATIC_CACHE_STATE_PRE_PURGE {
        purge(&cloudflare, &cache_tag, &input, "pre_purge").await?;
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

    purge(&cloudflare, &cache_tag, &input, "post_purge").await?;
    set_active(&db, &input).await?;

    crate::enqueue::deploy_artifact_prune(crate::queue_task::deploy_artifact_prune::Input {
        project_id: input.project_id,
    })
    .await
}

async fn purge(
    cloudflare: &CloudflareClient,
    cache_tag: &str,
    input: &Input,
    phase: &str,
) -> anyhow::Result<()> {
    let started = std::time::Instant::now();
    let result = cloudflare.purge_cache_tags(&[cache_tag]).await;
    match &result {
        Ok(()) => tracing::info!(
            project_id = %input.project_id,
            code_version = input.code_version,
            phase,
            duration_ms = started.elapsed().as_millis() as u64,
            "static cache purge completed"
        ),
        Err(error) => tracing::warn!(
            project_id = %input.project_id,
            code_version = input.code_version,
            phase,
            duration_ms = started.elapsed().as_millis() as u64,
            %error,
            "static cache purge failed"
        ),
    }
    result
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
