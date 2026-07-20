use crate::actions::bundle_gc;
use crate::actions::presign_quota;
use crate::actions::usage_metering;
use crate::actions::zombie_sweep;
use crate::common::admin;
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

        match usage_metering::run_metering().await {
            Ok(stats) => {
                tracing::info!(
                    projects_count = stats.projects,
                    operations_docs_count = stats.operations_docs,
                    snapshot_docs_count = stats.snapshot_docs,
                    "usage_metering completed",
                );
                match presign_quota::run_enforcement(&stats.project_ids).await {
                    Ok(enforcement) => tracing::info!(
                        evaluated_count = enforcement.evaluated,
                        blocked_count = enforcement.blocked,
                        "presign_quota completed",
                    ),
                    Err(err) => {
                        tracing::error!(?err, "presign_quota within cron_on_tick failed")
                    }
                }
            }
            Err(err) => tracing::error!(?err, "usage_metering within cron_on_tick failed"),
        }
    }

    Output::Ok {
        fired_count: fired,
        scanned_projects_count: scanned_projects,
        scanned_jobs_count: scanned_jobs,
    }
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
