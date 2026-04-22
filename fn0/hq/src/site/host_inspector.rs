use super::*;
use crate::doc_db::HostStatus;
use std::time::Duration;
use tokio::time::MissedTickBehavior;

const HOST_INSPECTOR_INTERVAL: Duration = Duration::from_secs(2);
const SSH_FAILURE_TERMINATE_THRESHOLD: u32 = 150;
const PER_TICK_BACKOFFS: [Duration; 3] = [
    Duration::from_millis(200),
    Duration::from_millis(500),
    Duration::from_millis(1000),
];

#[derive(serde::Deserialize)]
struct WorkerStatus {
    generation: u64,
    instances: u64,
}

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_host_inspector_loop(&self) {
        let mut interval = tokio::time::interval(HOST_INSPECTOR_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let statuses = match self.doc_db.list_host_statuses(&self.name).await {
                Ok(list) => list,
                Err(err) => {
                    warn!(%err, "Inspector: failed to list host-status");
                    continue;
                }
            };

            let latest_generation = self.deployment_cache.last_deployment_id();

            let mut tasks = Vec::with_capacity(statuses.len());
            for status in statuses {
                let site = self.clone();
                tasks.push(tokio::spawn(async move {
                    site.inspect_one(status, latest_generation).await;
                }));
            }
            for t in tasks {
                let _ = t.await;
            }
        }
    }

    async fn inspect_one(&self, status: HostStatus, latest_generation: u64) {
        let host_id = status.host_id.clone();
        let addr = status.addr.clone();

        match attempt_status(&self.ssh_pool, &addr).await {
            Ok(ws) => {
                let now = chrono::Utc::now().to_rfc3339();
                if let Err(err) = self
                    .doc_db
                    .mark_host_verified(&host_id, ws.generation, ws.instances, &now)
                    .await
                {
                    warn!(%err, %host_id, "Failed to mark host verified");
                }
                if ws.generation < latest_generation {
                    let pool = self.ssh_pool.clone();
                    let cache = self.deployment_cache.clone();
                    let addr = addr.clone();
                    tokio::spawn(async move {
                        if let Err(err) =
                            super::deployment_pusher::push_to_addr(&pool, &cache, &addr).await
                        {
                            warn!(%err, %addr, "Deferred push failed");
                        }
                    });
                }
            }
            Err(err) => {
                warn!(%err, %host_id, "SSH status check failed");
                match self.doc_db.mark_host_failed(&host_id).await {
                    Ok(failures) => {
                        if failures >= SSH_FAILURE_TERMINATE_THRESHOLD {
                            let provider = self.host_provider.clone();
                            let db = self.doc_db.clone();
                            let pool = self.ssh_pool.clone();
                            let id = host_id.clone();
                            let a = addr.clone();
                            tokio::spawn(async move {
                                super::terminate::terminate_host(&provider, &db, &pool, &id, &a)
                                    .await;
                            });
                        }
                    }
                    Err(err) => {
                        warn!(%err, %host_id, "Failed to mark host failed");
                    }
                }
            }
        }
    }
}

async fn attempt_status(
    ssh_pool: &crate::ssh_pool::SshPool,
    addr: &str,
) -> color_eyre::eyre::Result<WorkerStatus> {
    let cmd = "curl -sk --max-time 3 https://localhost/status";
    let mut last_err: Option<color_eyre::eyre::Error> = None;
    for (idx, backoff) in PER_TICK_BACKOFFS.iter().enumerate() {
        if idx > 0 {
            tokio::time::sleep(*backoff).await;
        }
        match ssh_pool.exec(addr, cmd).await {
            Ok((0, stdout)) => match serde_json::from_str::<WorkerStatus>(stdout.trim()) {
                Ok(ws) => return Ok(ws),
                Err(err) => {
                    last_err = Some(color_eyre::eyre::eyre!(
                        "Invalid /status JSON: {err}; body={}",
                        stdout.trim()
                    ));
                }
            },
            Ok((code, output)) => {
                last_err = Some(color_eyre::eyre::eyre!(
                    "curl /status exit={code}: {}",
                    output.trim()
                ));
            }
            Err(err) => {
                last_err = Some(err);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| color_eyre::eyre::eyre!("unknown ssh failure")))
}
