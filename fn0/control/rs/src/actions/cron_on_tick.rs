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
    let Some(manifest) = (WorkerManifestDocGet {}).send_with(&db).await? else {
        return Ok(());
    };
    let current_time = now();
    for (project_id, entry) in manifest.project_manifests {
        let declarations = (WebSocketSingletonConfigDocGet {
            project_id: project_id.as_str(),
            code_version: entry.code_version,
        })
        .send_with(&db)
        .await?
        .map(|config| config.declarations)
        .unwrap_or_default();
        let mut reconnect = false;
        for declaration in declarations {
            let runtime = (WebSocketSingletonRuntimeDocGet {
                project_id: project_id.as_str(),
                singleton_id: declaration.singleton_id.as_str(),
            })
            .send_with(&db)
            .await?;
            if runtime_needs_reconnect(runtime.as_ref(), entry.code_version, current_time) {
                reconnect = true;
                break;
            }
        }
        if reconnect {
            crate::enqueue::deployment_activation(
                crate::queue_task::deployment_activation::Input {
                    project_id,
                    code_version: entry.code_version,
                },
            )
            .await?;
        }
    }
    Ok(())
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
    use super::runtime_needs_reconnect;
    use crate::docs::WebSocketSingletonRuntimeDoc;
    use forte_sdk::{chrono, now};

    #[test]
    fn missing_expired_and_old_version_leases_reconnect() {
        let current_time = now();
        assert!(runtime_needs_reconnect(None, 42, current_time));
        let expired = WebSocketSingletonRuntimeDoc {
            project_id: "project".to_string(),
            singleton_id: "feed".to_string(),
            code_version: 42,
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
}
