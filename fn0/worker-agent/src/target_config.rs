use crate::shutdown::Shutdown;
use doc_db::DbRequest;
use forte_macros::forte_doc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::*;

const POLL_INTERVAL: Duration = Duration::from_secs(30);

#[forte_doc]
pub struct TargetFn0WorkerConfigDoc {
    pub image_ref: String,
}

pub async fn run(shutdown: Shutdown, target_tx: watch::Sender<Option<String>>) {
    info!("worker target poller started");
    let db = crate::db::build();
    loop {
        match (TargetFn0WorkerConfigDocGet {}).send_with(&db).await {
            Ok(Some(doc)) => {
                target_tx.send_if_modified(|cur| {
                    if cur.as_deref() == Some(doc.image_ref.as_str()) {
                        false
                    } else {
                        *cur = Some(doc.image_ref.clone());
                        true
                    }
                });
            }
            Ok(None) => {
                debug!("TargetFn0WorkerConfigDoc not present");
            }
            Err(err) => {
                warn!(?err, "polling TargetFn0WorkerConfigDoc failed");
            }
        }

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
        }
    }
    info!("worker target poller stopped");
}
