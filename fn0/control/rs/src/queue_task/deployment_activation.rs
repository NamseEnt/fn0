use crate::common::byoc::ProjectStorage;
use crate::docs::*;
use fn0_shared_schema::{
    STATIC_CACHE_STATE_ACTIVATING, STATIC_CACHE_STATE_ACTIVE, STATIC_CACHE_STATE_PRE_PURGE,
};
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const RUNTIME_QUERY_LIMIT: usize = 1_000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActivationStage {
    AlreadyActive,
    Pending,
    Activating,
    Obsolete,
}

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
    let activation_stage = activation_stage(
        entry.code_version,
        entry.pending_code_version,
        &entry.static_cache_state,
        input.code_version,
    );
    if activation_stage == ActivationStage::Obsolete {
        return Ok(());
    }

    if activation_stage != ActivationStage::AlreadyActive {
        let user_zone = if entry.domain.is_empty() {
            None
        } else {
            Some((
                ProjectStorage::resolve(&db, &input.project_id).await?,
                entry.domain.clone(),
            ))
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
    }

    crate::enqueue::deploy_artifact_prune(crate::queue_task::deploy_artifact_prune::Input {
        project_id: input.project_id.clone(),
    })
    .await?;
    enqueue_singletons(&db, &input).await
}

fn activation_stage(
    code_version: u64,
    pending_code_version: Option<u64>,
    static_cache_state: &str,
    target_code_version: u64,
) -> ActivationStage {
    if code_version == target_code_version && static_cache_state == STATIC_CACHE_STATE_ACTIVE {
        ActivationStage::AlreadyActive
    } else if pending_code_version == Some(target_code_version)
        && static_cache_state == STATIC_CACHE_STATE_PRE_PURGE
    {
        ActivationStage::Pending
    } else if code_version == target_code_version
        && static_cache_state == STATIC_CACHE_STATE_ACTIVATING
    {
        ActivationStage::Activating
    } else {
        ActivationStage::Obsolete
    }
}

async fn enqueue_singletons(db: &doc_db::Database, input: &Input) -> anyhow::Result<()> {
    let declarations = (WebSocketSingletonConfigDocGet {
        project_id: input.project_id.as_str(),
        code_version: input.code_version,
    })
    .send_with(db)
    .await?
    .map(|config| config.declarations)
    .unwrap_or_default();
    remove_stale_runtime_records(db, &input.project_id, input.code_version, &declarations).await?;
    for declaration in declarations {
        crate::enqueue::websocket_singleton_reconcile(
            crate::queue_task::websocket_singleton_reconcile::Input {
                project_id: input.project_id.clone(),
                code_version: input.code_version,
                singleton_id: declaration.singleton_id,
            },
        )
        .await?;
    }
    Ok(())
}

async fn remove_stale_runtime_records(
    db: &doc_db::Database,
    project_id: &str,
    code_version: u64,
    declarations: &[WebSocketSingletonDeclaration],
) -> anyhow::Result<()> {
    let declared: HashSet<&str> = declarations
        .iter()
        .map(|declaration| declaration.singleton_id.as_str())
        .collect();
    let runtime_records: Vec<WebSocketSingletonRuntimeDoc> = (WebSocketSingletonRuntimeDocQuery {
        project_id,
        singleton_id: None,
        limit: Some(RUNTIME_QUERY_LIMIT),
    })
    .send_with(db)
    .await?;
    for runtime_record in runtime_records {
        if runtime_is_stale(&runtime_record, code_version, &declared) {
            crate::actions::cron_on_tick::delete_runtime_if_unchanged(db, &runtime_record).await?;
        }
    }
    Ok(())
}

fn runtime_is_stale(
    runtime: &WebSocketSingletonRuntimeDoc,
    code_version: u64,
    declared: &HashSet<&str>,
) -> bool {
    runtime.code_version != code_version || !declared.contains(runtime.singleton_id.as_str())
}

async fn purge_user_zone(
    user_zone: Option<&(ProjectStorage, String)>,
    input: &Input,
    phase: &str,
) -> anyhow::Result<()> {
    let Some((storage, domain)) = user_zone else {
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
        domain,
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

#[cfg(test)]
mod tests {
    use super::{ActivationStage, activation_stage, runtime_is_stale};
    use crate::docs::WebSocketSingletonRuntimeDoc;
    use fn0_shared_schema::{STATIC_CACHE_STATE_ACTIVE, STATIC_CACHE_STATE_PRE_PURGE};
    use forte_sdk::{chrono, now};
    use std::collections::HashSet;

    #[test]
    fn active_redelivery_still_enters_singleton_initialization() {
        assert!(matches!(
            activation_stage(42, None, STATIC_CACHE_STATE_ACTIVE, 42),
            ActivationStage::AlreadyActive
        ));
    }

    #[test]
    fn replaced_activation_is_ignored() {
        assert!(matches!(
            activation_stage(42, Some(43), STATIC_CACHE_STATE_PRE_PURGE, 41),
            ActivationStage::Obsolete
        ));
    }

    #[test]
    fn deleted_singleton_runtime_is_removed() {
        let runtime = WebSocketSingletonRuntimeDoc {
            project_id: "project".to_string(),
            singleton_id: "deleted".to_string(),
            code_version: 42,
            claim_token: "claim".to_string(),
            connection_id: "connection".to_string(),
            lease_expires_at: now() + chrono::Duration::seconds(60),
        };
        assert!(runtime_is_stale(&runtime, 42, &HashSet::new()));
        let declared = HashSet::from(["deleted"]);
        assert!(!runtime_is_stale(&runtime, 42, &declared));
    }
}
