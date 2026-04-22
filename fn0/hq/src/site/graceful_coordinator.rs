use super::*;
use crate::doc_db::{GracefulPurpose, HostStatus};
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::{info, warn};

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
            Ok((0, stdout)) => match serde_json::from_str::<serde_json::Value>(stdout.trim()) {
                Ok(v) => v.get("instances").and_then(|x| x.as_u64()) == Some(0),
                Err(_) => false,
            },
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
            Some(GracefulPurpose::ImageSwap) => {
                self.swap_image_on(&s.host_id, &s.addr).await;
            }
            None => {
                warn!(host_id = %s.host_id, "Graceful without purpose");
            }
        }
    }

    async fn swap_image_on(&self, host_id: &str, addr: &str) {
        let target = match self.doc_db.get_worker_target(&self.name).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                warn!(%host_id, "swap_image: no worker-target; clearing graceful");
                let _ = self
                    .doc_db
                    .clear_host_graceful(host_id)
                    .await
                    .inspect_err(|err| warn!(%err, %host_id, "clear_graceful failed"));
                return;
            }
            Err(err) => {
                warn!(%err, %host_id, "swap_image: failed to read worker-target");
                return;
            }
        };

        let envs = self.host_provider.envs().clone();
        let image = target.image_ref();
        let script = super::worker_update::build_deploy_script(&image, &envs, &target);

        match self
            .ssh_pool
            .exec_with_timeout(addr, &script, crate::ssh_pool::DEPLOY_TIMEOUT)
            .await
        {
            Ok((0, _)) => {
                info!(%host_id, new_image = %image, "image swap complete");
                if let Err(err) = self.doc_db.clear_host_graceful(host_id).await {
                    warn!(%err, %host_id, "clear_graceful failed after swap");
                }
            }
            Ok((code, output)) => {
                warn!(
                    %host_id,
                    exit = code,
                    output = %output.trim(),
                    "image swap script non-zero; will retry"
                );
            }
            Err(err) => {
                warn!(%err, %host_id, "image swap ssh exec failed; will retry");
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
