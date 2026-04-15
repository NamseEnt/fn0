use super::*;
use crate::doc_db::{GracefulPurpose, HostStatus};
use std::time::Duration;
use tokio::time::MissedTickBehavior;

const DNS_GRACEFUL_REMOVAL_SECS: i64 = 20;
const DRAIN_POLL_INTERVAL: Duration = Duration::from_secs(1);

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_graceful_coordinator_loop(&self) {
        let mut interval = tokio::time::interval(DRAIN_POLL_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let statuses = match self.doc_db.list_host_statuses(&self.name).await {
                Ok(list) => list,
                Err(err) => {
                    warn!(%err, "Graceful coordinator: failed to list host-status");
                    continue;
                }
            };

            for s in statuses.into_iter().filter(|s| s.graceful) {
                if !should_act(&s) {
                    continue;
                }
                self.try_finish_graceful(&s).await;
            }
        }
    }

    async fn try_finish_graceful(&self, s: &HostStatus) {
        let instances_cmd = "curl -sk --max-time 3 https://localhost/status";
        let drained = match self.ssh_pool.exec(&s.addr, instances_cmd).await {
            Ok((0, stdout)) => {
                match serde_json::from_str::<serde_json::Value>(stdout.trim()) {
                    Ok(v) => v.get("instances").and_then(|x| x.as_u64()) == Some(0),
                    Err(_) => false,
                }
            }
            _ => false,
        };

        if !drained {
            return;
        }

        match s.graceful_purpose {
            Some(GracefulPurpose::Terminate) => {
                super::terminate::terminate_host(
                    &self.host_provider,
                    &self.doc_db,
                    &self.ssh_pool,
                    &s.host_id,
                    &s.addr,
                )
                .await;
            }
            None => {
                warn!(host_id = %s.host_id, "Graceful without purpose");
            }
        }
    }
}

fn should_act(s: &HostStatus) -> bool {
    let Some(since) = s.graceful_since_at.as_deref() else {
        return false;
    };
    let Ok(since) = chrono::DateTime::parse_from_rfc3339(since) else {
        return false;
    };
    let elapsed = chrono::Utc::now().signed_duration_since(since.with_timezone(&chrono::Utc));
    elapsed.num_seconds() >= DNS_GRACEFUL_REMOVAL_SECS
}
