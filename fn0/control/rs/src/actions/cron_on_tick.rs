use crate::actions::bundle_gc;
use crate::actions::zombie_sweep;
use crate::common::admin;
use crate::common::websocket_directory_gc;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub scheduled_time: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        fired_count: u64,
        scanned_projects_count: u64,
        scanned_jobs_count: u64,
    },
    Unauthorized,
    Error {
        message: String,
    },
}

const QUERY_PAGE_LIMIT: usize = 256;

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }

    let scheduled = match chrono::DateTime::parse_from_rfc3339(&req.body.scheduled_time) {
        Ok(dt) => dt.timestamp(),
        Err(e) => {
            return Output::Error {
                message: format!("scheduled_time parse: {e}"),
            };
        }
    };
    let epoch_minute: i64 = scheduled / 60;

    let invoke_queue_url = match std::env::var("FN0_CROSS_PROJECT_ENQUEUE_URL") {
        Ok(u) => u,
        Err(_) => {
            return Output::Error {
                message: "FN0_CROSS_PROJECT_ENQUEUE_URL not set".to_string(),
            };
        }
    };

    let db = doc_db::turso();
    let client = http::Client::new();

    let mut after: Option<String> = None;
    let mut scanned_projects: u64 = 0;
    let mut scanned_jobs: u64 = 0;
    let mut fired: u64 = 0;

    loop {
        let docs: Vec<CronConfigDoc> = match (CronConfigDocQuery {
            project_id: after.clone(),
            limit: Some(QUERY_PAGE_LIMIT),
        })
        .send_with(&db)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(?e, "cron query failed");
                return Output::Error {
                    message: format!("query: {e}"),
                };
            }
        };
        if docs.is_empty() {
            break;
        }
        let last = docs.last().map(|d| d.project_id.clone());
        let page_len = docs.len();

        for doc in docs {
            scanned_projects += 1;
            for job in &doc.jobs {
                scanned_jobs += 1;
                if job.every_minutes == 0 {
                    continue;
                }
                if epoch_minute % (job.every_minutes as i64) != 0 {
                    continue;
                }
                if let Err(e) = enqueue_other_project_cron_task(
                    &client,
                    &invoke_queue_url,
                    &doc.project_id,
                    &job.function,
                )
                .await
                {
                    tracing::error!(
                        project_id = %doc.project_id,
                        function = %job.function,
                        error = %e,
                        "cron enqueue failed"
                    );
                    continue;
                }
                fired += 1;
            }
        }

        if page_len < QUERY_PAGE_LIMIT {
            break;
        }
        after = last;
    }

    tracing::info!(
        fired_count = fired,
        scanned_projects_count = scanned_projects,
        scanned_jobs_count = scanned_jobs,
        epoch_minute,
        "cron_on_tick dispatch completed"
    );

    if let Err(err) = zombie_sweep::run_sweep().await {
        tracing::error!(?err, "zombie_sweep within cron_on_tick failed");
    }

    match websocket_directory_gc::run_gc().await {
        Ok(stats) => tracing::info!(
            scanned_connections_count = stats.scanned_connections,
            deleted_connections_count = stats.deleted_connections,
            unreachable_workers_count = stats.unreachable_workers,
            "websocket directory GC completed",
        ),
        Err(err) => tracing::error!(?err, "websocket directory GC within cron_on_tick failed"),
    }

    if let Err(err) = recover_expired_websocket_singletons().await {
        tracing::error!(?err, "websocket singleton lease recovery failed");
    }

    if epoch_minute % 60 == 0 {
        match bundle_gc::run_gc().await {
            Ok(stats) => tracing::info!(
                deleted_versions_count = stats.deleted_versions,
                deleted_orphans_count = stats.deleted_orphans,
                deleted_static_prefixes_count = stats.deleted_static_prefixes,
                "bundle_gc completed",
            ),
            Err(err) => tracing::error!(?err, "bundle_gc within cron_on_tick failed"),
        }
    }

    Output::Ok {
        fired_count: fired,
        scanned_projects_count: scanned_projects,
        scanned_jobs_count: scanned_jobs,
    }
}

async fn recover_expired_websocket_singletons() -> anyhow::Result<()> {
    let db = doc_db::turso();
    recover_expired_websocket_singletons_with(&db, now()).await
}

const WEBSOCKET_RECONCILE_PROJECT_LIMIT: usize = 64;
const WEBSOCKET_RECONCILE_DECLARATION_LIMIT: usize = 256;

async fn recover_expired_websocket_singletons_with(
    db: &doc_db::Database,
    current_time: DateTime,
) -> anyhow::Result<()> {
    let Some(manifest) = (WorkerManifestDocGet {}).send_with(db).await? else {
        return Ok(());
    };
    let cursor = (WebSocketSingletonReconcileCursorDocGet {})
        .send_with(db)
        .await?
        .unwrap_or(WebSocketSingletonReconcileCursorDoc {
            after_project_id: None,
            after_singleton_id: None,
        });
    let mut project_ids: Vec<String> = manifest.project_manifests.keys().cloned().collect();
    project_ids.sort();
    let project_ids = project_ids_after_cursor(
        &project_ids,
        cursor.after_project_id.as_deref(),
        cursor.after_singleton_id.as_deref(),
    );
    if project_ids.is_empty() {
        save_websocket_reconcile_cursor(db, None, None).await?;
        return Ok(());
    }
    let mut scanned_declarations = 0_usize;
    let mut checkpoint_project = cursor.after_project_id.clone();
    let mut checkpoint_singleton = cursor.after_singleton_id.clone();
    for (project_index, project_id) in project_ids.into_iter().enumerate() {
        if project_index >= WEBSOCKET_RECONCILE_PROJECT_LIMIT
            || scanned_declarations >= WEBSOCKET_RECONCILE_DECLARATION_LIMIT
        {
            save_websocket_reconcile_cursor(
                db,
                checkpoint_project.clone(),
                checkpoint_singleton.clone(),
            )
            .await?;
            return Ok(());
        }
        let Some(entry) = manifest.project_manifests.get(project_id.as_str()) else {
            continue;
        };
        if entry.static_cache_state != fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE {
            checkpoint_project = Some(project_id);
            checkpoint_singleton = None;
            continue;
        }
        let config = match (WebSocketSingletonConfigDocGet {
            project_id: project_id.as_str(),
            code_version: entry.code_version,
        })
        .send_with(db)
        .await
        {
            Ok(config) => config,
            Err(error) => {
                tracing::error!(%project_id, %error, "websocket singleton config scan failed");
                checkpoint_project = Some(project_id);
                checkpoint_singleton = None;
                continue;
            }
        };
        let mut declarations = config.map(|config| config.declarations).unwrap_or_default();
        declarations.sort_by(|left, right| left.singleton_id.cmp(&right.singleton_id));
        let runtime_records: Vec<WebSocketSingletonRuntimeDoc> =
            match (WebSocketSingletonRuntimeDocQuery {
                project_id: project_id.as_str(),
                singleton_id: None,
                limit: Some(fn0_shared_schema::MAX_WEBSOCKET_SINGLETONS_PER_PROJECT + 1),
            })
            .send_with(db)
            .await
            {
                Ok(runtime_records) => runtime_records,
                Err(error) => {
                    tracing::error!(%project_id, %error, "websocket singleton runtime scan failed");
                    checkpoint_project = Some(project_id);
                    checkpoint_singleton = None;
                    continue;
                }
            };
        let declared_ids: std::collections::HashSet<&str> = declarations
            .iter()
            .map(|declaration| declaration.singleton_id.as_str())
            .collect();
        for runtime in &runtime_records {
            if (runtime.code_version != entry.code_version
                || !declared_ids.contains(runtime.singleton_id.as_str()))
                && let Err(error) = delete_runtime_if_unchanged(db, runtime).await
            {
                tracing::error!(
                    %project_id,
                    singleton_id = %runtime.singleton_id,
                    %error,
                    "stale websocket singleton runtime cleanup failed"
                );
            }
        }
        let runtime_by_id: std::collections::HashMap<&str, &WebSocketSingletonRuntimeDoc> =
            runtime_records
                .iter()
                .map(|runtime| (runtime.singleton_id.as_str(), runtime))
                .collect();
        let resume_singleton = if cursor.after_project_id.as_deref() == Some(project_id.as_str()) {
            cursor.after_singleton_id.as_deref()
        } else {
            None
        };
        let mut finished_project = true;
        for declaration in declarations_after_cursor(&declarations, resume_singleton) {
            if scanned_declarations >= WEBSOCKET_RECONCILE_DECLARATION_LIMIT {
                finished_project = false;
                break;
            }
            scanned_declarations += 1;
            if runtime_needs_reconnect(
                runtime_by_id
                    .get(declaration.singleton_id.as_str())
                    .copied(),
                entry.code_version,
                current_time,
            ) && let Err(error) = crate::enqueue::websocket_singleton_reconcile(
                crate::queue_task::websocket_singleton_reconcile::Input {
                    project_id: project_id.clone(),
                    code_version: entry.code_version,
                    singleton_id: declaration.singleton_id.clone(),
                },
            )
            .await
            {
                tracing::error!(
                    %project_id,
                    singleton_id = %declaration.singleton_id,
                    %error,
                    "websocket singleton reconcile enqueue failed"
                );
            }
            checkpoint_project = Some(project_id.clone());
            checkpoint_singleton = Some(declaration.singleton_id.clone());
        }
        if finished_project {
            checkpoint_project = Some(project_id);
            checkpoint_singleton = None;
        } else {
            save_websocket_reconcile_cursor(db, checkpoint_project, checkpoint_singleton).await?;
            return Ok(());
        }
    }
    save_websocket_reconcile_cursor(db, None, None).await?;
    Ok(())
}

fn project_ids_after_cursor(
    project_ids: &[String],
    after_project_id: Option<&str>,
    after_singleton_id: Option<&str>,
) -> Vec<String> {
    project_ids
        .iter()
        .filter(|project_id| match after_project_id {
            Some(after_project_id) if after_singleton_id.is_some() => {
                project_id.as_str() >= after_project_id
            }
            Some(after_project_id) => project_id.as_str() > after_project_id,
            None => true,
        })
        .cloned()
        .collect()
}

fn declarations_after_cursor<'a>(
    declarations: &'a [WebSocketSingletonDeclaration],
    after_singleton_id: Option<&str>,
) -> Vec<&'a WebSocketSingletonDeclaration> {
    declarations
        .iter()
        .filter(|declaration| {
            after_singleton_id
                .is_none_or(|singleton_id| declaration.singleton_id.as_str() > singleton_id)
        })
        .collect()
}

async fn save_websocket_reconcile_cursor(
    db: &doc_db::Database,
    after_project_id: Option<String>,
    after_singleton_id: Option<String>,
) -> anyhow::Result<()> {
    WebSocketSingletonReconcileCursorDocPut(WebSocketSingletonReconcileCursorDoc {
        after_project_id,
        after_singleton_id,
    })
    .send_with(db)
    .await
}

pub(crate) async fn delete_runtime_if_unchanged(
    db: &doc_db::Database,
    expected: &WebSocketSingletonRuntimeDoc,
) -> anyhow::Result<bool> {
    let project_id = expected.project_id.clone();
    let singleton_id = expected.singleton_id.clone();
    let code_version = expected.code_version;
    let claim_token = expected.claim_token.clone();
    let connection_id = expected.connection_id.clone();
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            let singleton_id = singleton_id.clone();
            let claim_token = claim_token.clone();
            let connection_id = connection_id.clone();
            async move {
                let Some(runtime) = trx
                    .get(WebSocketSingletonRuntimeDocGet {
                        project_id: project_id.as_str(),
                        singleton_id: singleton_id.as_str(),
                    })
                    .await?
                else {
                    return trx.commit::<_, ()>(false);
                };
                if runtime.code_version != code_version
                    || runtime.claim_token != claim_token
                    || runtime.connection_id != connection_id
                {
                    return trx.commit::<_, ()>(false);
                }
                runtime.delete();
                trx.commit::<_, ()>(true)
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(deleted) => Ok(deleted),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("websocket singleton stale cleanup conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

fn runtime_needs_reconnect(
    runtime: Option<&WebSocketSingletonRuntimeDoc>,
    code_version: u64,
    current_time: DateTime,
) -> bool {
    runtime.is_none_or(|runtime| {
        runtime.code_version != code_version || runtime.lease_expires_at <= current_time
    })
}

#[derive(Serialize)]
struct CrossProjectCronInvokeBody<'a> {
    project_id: &'a str,
    task_name: &'a str,
    payload: serde_json::Value,
}

async fn enqueue_other_project_cron_task(
    client: &http::Client,
    invoke_queue_url: &str,
    project_id: &str,
    task_name: &str,
) -> anyhow::Result<()> {
    let body = serde_json::to_vec(&CrossProjectCronInvokeBody {
        project_id,
        task_name,
        payload: serde_json::Value::Null,
    })?;
    let resp = client
        .send(
            http::Request::builder()
                .method("POST")
                .uri(invoke_queue_url)
                .header("content-type", "application/json")
                .body(body)?,
        )
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("invoke queue status {}", resp.status());
    }
    Ok(())
}

#[cfg(test)]
mod websocket_singleton_lease_tests {
    use super::{declarations_after_cursor, project_ids_after_cursor, runtime_needs_reconnect};
    use crate::docs::{WebSocketSingletonDeclaration, WebSocketSingletonRuntimeDoc};
    use forte_sdk::{chrono, now};

    #[test]
    fn missing_expired_and_old_version_leases_reconnect() {
        let current_time = now();
        assert!(runtime_needs_reconnect(None, 42, current_time));
        let expired = WebSocketSingletonRuntimeDoc {
            project_id: "project".to_string(),
            singleton_id: "feed".to_string(),
            code_version: 42,
            claim_token: "expired-claim".to_string(),
            connection_id: "expired".to_string(),
            lease_expires_at: current_time - chrono::Duration::seconds(1),
        };
        assert!(runtime_needs_reconnect(Some(&expired), 42, current_time));
        let old_version = WebSocketSingletonRuntimeDoc {
            lease_expires_at: current_time + chrono::Duration::seconds(60),
            code_version: 41,
            ..expired
        };
        assert!(runtime_needs_reconnect(
            Some(&old_version),
            42,
            current_time
        ));
        let live = WebSocketSingletonRuntimeDoc {
            code_version: 42,
            ..old_version
        };
        assert!(!runtime_needs_reconnect(Some(&live), 42, current_time));
    }

    #[test]
    fn project_cursor_resumes_project_only_for_singleton_checkpoint() {
        let project_ids = vec![
            "alpha".to_string(),
            "bravo".to_string(),
            "charlie".to_string(),
        ];
        assert_eq!(
            project_ids_after_cursor(&project_ids, Some("bravo"), None),
            vec!["charlie"]
        );
        assert_eq!(
            project_ids_after_cursor(&project_ids, Some("bravo"), Some("feed")),
            vec!["bravo", "charlie"]
        );
        assert_eq!(
            project_ids_after_cursor(&project_ids, None, None),
            project_ids
        );
    }

    #[test]
    fn singleton_cursor_resumes_after_last_processed_declaration() {
        let declarations: Vec<WebSocketSingletonDeclaration> = ["alpha", "bravo", "charlie"]
            .into_iter()
            .map(|singleton_id| WebSocketSingletonDeclaration {
                singleton_id: singleton_id.to_string(),
                route_path: format!("/ws_singleton/{singleton_id}"),
            })
            .collect();
        let remaining_ids: Vec<&str> = declarations_after_cursor(&declarations, Some("bravo"))
            .into_iter()
            .map(|declaration| declaration.singleton_id.as_str())
            .collect();
        assert_eq!(remaining_ids, vec!["charlie"]);
    }
}
