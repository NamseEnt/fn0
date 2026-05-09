use crate::shutdown::Shutdown;
use doc_db::DbRequest;
use forte_macros::forte_doc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::watch;
use tracing::*;

const REPORT_INTERVAL: Duration = Duration::from_secs(30);

#[forte_doc]
pub struct WorkerHostStatusDoc {
    #[sk]
    pub host_id: String,
    pub active_image_ref: Option<String>,
    pub reported_at: i64,
}

pub async fn run(
    shutdown: Shutdown,
    active_rx: watch::Receiver<Option<String>>,
    host_id: String,
) {
    info!(%host_id, "host status reporter started");
    let db = doc_db::turso();
    loop {
        let active_image_ref = active_rx.borrow().clone();
        let doc = WorkerHostStatusDoc {
            host_id: host_id.clone(),
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
    info!("host status reporter stopped");
}

fn now_epoch_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
