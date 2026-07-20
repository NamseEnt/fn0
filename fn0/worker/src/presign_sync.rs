//! Presign gate sync against the control DB: pulls `PresignBlockedDoc` into
//! the in-process gate every second, and reports per-(project, hour) mint
//! counts as `PresignMintCountDoc` rows once a minute. Rows are keyed by a
//! per-process writer id so concurrent hosts write disjoint rows and the
//! control plane sums them; re-reporting the running total for an hour is an
//! idempotent overwrite of this process's own row.

use doc_db::{Database, DbRequest};
use fn0::PresignGate;
use fn0_shared_schema::{PresignBlockedDocGet, PresignMintCountDoc, PresignMintCountDocPut};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const REPORT_EVERY_TICKS: u64 = 60;

fn epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub async fn run(db: Database, gate: Arc<PresignGate>) {
    let writer_id = format!("{}-{}", epoch_secs(), std::process::id());
    let mut tick: u64 = 0;

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;
        tick += 1;

        match (PresignBlockedDocGet {}).send_with(&db).await {
            Ok(doc) => {
                let blocked: HashSet<String> = doc
                    .map(|d| d.blocked_project_ids.into_iter().collect())
                    .unwrap_or_default();
                gate.set_blocked(blocked);
            }
            Err(err) => tracing::warn!(%err, "presign blocked doc fetch failed"),
        }

        if !tick.is_multiple_of(REPORT_EVERY_TICKS) {
            continue;
        }
        let current_epoch_hour = epoch_secs() / 3600;
        for (project_id, window_epoch_hour, minted) in gate.snapshot_minted(current_epoch_hour) {
            let result = PresignMintCountDocPut(PresignMintCountDoc {
                project_id: project_id.clone(),
                window_epoch_hour,
                writer_id: writer_id.clone(),
                minted,
            })
            .send_with(&db)
            .await;
            if let Err(err) = result {
                tracing::warn!(%err, %project_id, window_epoch_hour, "presign mint count report failed");
            }
        }
    }
}
