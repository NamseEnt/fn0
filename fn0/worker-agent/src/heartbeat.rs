//! Worker liveness heartbeat for zombie detection.
//!
//! Periodically writes a `WorkerHeartbeatDoc` to doc-db so the control plane's
//! `zombie_sweep` action can detect hosts that died ungracefully. When the
//! heartbeat goes stale, control treats the host as dead and removes the
//! orphaned Cloudflare DNS A record that still points at this host's public
//! IP.
//!
//! # Not to be confused with `host_status_reporter`
//!
//! `host_status_reporter` reports which worker container image is currently
//! active on this host — a rollout/deploy concern. It writes
//! `WorkerHostStatusDoc` and is unrelated to liveness or DNS cleanup.
//! Do not merge the two: a heartbeat needs to keep flowing even when the
//! worker container is not yet ready, while host status only makes sense
//! once an image is active. Different lifetimes, different consumers.
//!
//! # Lifecycle invariant
//!
//! The doc must exist before the self-DNS A record is registered, and must
//! be deleted only after the A record is deregistered. This guarantees
//! `DNS A record exists ⇒ doc exists`, so the sweep can decide an A record
//! is orphaned by looking at doc staleness alone (no separate "DNS without
//! doc" code path needed).
//!
//! `main.rs` enforces this ordering by calling [`write_initial`] before
//! `self_dns::register`, and by ensuring the periodic task only does its
//! final `Delete` after `self_dns::deregister` has run.

use crate::shutdown::Shutdown;
use fn0_shared_schema::{
    DbRequest, WorkerHeartbeatDoc, WorkerHeartbeatDocDelete, WorkerHeartbeatDocPut,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::*;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

pub async fn write_initial(host_id: &str, addr: &str) -> anyhow::Result<()> {
    let db = doc_db::turso();
    let doc = WorkerHeartbeatDoc {
        host_id: host_id.to_string(),
        addr: addr.to_string(),
        last_seen_at: now_epoch_sec(),
    };
    WorkerHeartbeatDocPut(doc).send_with(&db).await?;
    Ok(())
}

pub async fn run(shutdown: Shutdown, host_id: String, addr: String) {
    info!(%host_id, %addr, "heartbeat reporter started");
    let db = doc_db::turso();
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(HEARTBEAT_INTERVAL) => {}
        }
        let doc = WorkerHeartbeatDoc {
            host_id: host_id.clone(),
            addr: addr.clone(),
            last_seen_at: now_epoch_sec(),
        };
        if let Err(err) = WorkerHeartbeatDocPut(doc).send_with(&db).await {
            warn!(?err, %host_id, "heartbeat write failed");
        }
    }
    if let Err(err) = (WorkerHeartbeatDocDelete {
        host_id: host_id.clone(),
    })
    .send_with(&db)
    .await
    {
        warn!(?err, %host_id, "heartbeat doc delete on shutdown failed");
    }
    info!("heartbeat reporter stopped");
}

fn now_epoch_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
