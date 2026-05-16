//! Per-host status reporter.
//!
//! Periodically writes `WorkerHostStatusDoc` to doc-db. Carries two
//! responsibilities on a single row keyed by `host_id`:
//!
//! 1. Liveness signal — `reported_at` lets the control plane's
//!    `zombie_sweep` action detect hosts that died ungracefully.
//! 2. Rollout signal — `active_image_ref` lets the deploy script's
//!    convergence gate confirm every live host has switched to the new
//!    fn0-worker image before promoting cwasm-compiler pending → active.

use crate::shutdown::Shutdown;
use fn0_shared_schema::{
    DbRequest, WorkerHostStatusDoc, WorkerHostStatusDocDelete, WorkerHostStatusDocPut,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::watch;
use tracing::*;

const REPORT_INTERVAL: Duration = Duration::from_secs(10);

pub async fn write_initial(host_id: &str, addr: &str) -> anyhow::Result<()> {
    let db = crate::db::build();
    let doc = WorkerHostStatusDoc {
        host_id: host_id.to_string(),
        addr: addr.to_string(),
        active_image_ref: None,
        reported_at: now_epoch_sec(),
    };
    WorkerHostStatusDocPut(doc).send_with(&db).await?;
    Ok(())
}

pub async fn run(
    shutdown: Shutdown,
    active_rx: watch::Receiver<Option<String>>,
    host_id: String,
    addr: String,
) {
    info!(%host_id, %addr, "host status reporter started");
    let db = crate::db::build();
    loop {
        let active_image_ref = active_rx.borrow().clone();
        let doc = WorkerHostStatusDoc {
            host_id: host_id.clone(),
            addr: addr.clone(),
            active_image_ref,
            reported_at: now_epoch_sec(),
        };
        if let Err(err) = WorkerHostStatusDocPut(doc).send_with(&db).await {
            warn!(?err, %host_id, "host status report failed");
        }
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(REPORT_INTERVAL) => {}
        }
    }
    if shutdown.is_soft() {
        info!(
            %host_id,
            "host status reporter: soft shutdown; keeping doc so the next agent inherits liveness without a sweep gap"
        );
        info!("host status reporter stopped");
        return;
    }
    if let Err(err) = (WorkerHostStatusDocDelete {
        host_id: host_id.clone(),
    })
    .send_with(&db)
    .await
    {
        warn!(?err, %host_id, "host status doc delete on shutdown failed");
    }
    info!("host status reporter stopped");
}

fn now_epoch_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
