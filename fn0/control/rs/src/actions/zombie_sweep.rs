use crate::common::admin;
use fn0_shared_schema::{
    DbRequest, WorkerHostStatusDoc, WorkerHostStatusDocDelete, WorkerHostStatusDocQuery,
};
use forte_sdk::*;
use serde::{Deserialize, Serialize};

pub const ZOMBIE_TIMEOUT_SECS: i64 = 60;

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    Ok {
        scanned_instances_count: u64,
        reaped_docs_count: u64,
    },
    Unauthorized,
    Error {
        message: String,
    },
}

pub struct SweepStats {
    pub scanned_instances: u64,
    pub reaped_docs: u64,
}

pub async fn run_sweep() -> anyhow::Result<SweepStats> {
    let db = doc_db::turso();
    let docs: Vec<WorkerHostStatusDoc> = WorkerHostStatusDocQuery {
        host_id: None,
        limit: None,
    }
    .send_with(&db)
    .await?;

    let scanned_instances = docs.len() as u64;
    let now = chrono::Utc::now().timestamp();
    let stale: Vec<WorkerHostStatusDoc> = docs
        .into_iter()
        .filter(|d| now - d.reported_at > ZOMBIE_TIMEOUT_SECS)
        .collect();

    let mut reaped_docs: u64 = 0;
    for d in stale {
        if let Err(e) = (WorkerHostStatusDocDelete {
            host_id: d.host_id.clone(),
        })
        .send_with(&db)
        .await
        {
            tracing::error!(
                host_id = %d.host_id,
                error = %e,
                "zombie_sweep doc delete failed",
            );
            continue;
        }
        reaped_docs += 1;
        tracing::info!(
            host_id = %d.host_id,
            addr = %d.addr,
            "zombie_sweep reaped host doc",
        );
    }

    Ok(SweepStats {
        scanned_instances,
        reaped_docs,
    })
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }
    match run_sweep().await {
        Ok(stats) => Output::Ok {
            scanned_instances_count: stats.scanned_instances,
            reaped_docs_count: stats.reaped_docs,
        },
        Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
