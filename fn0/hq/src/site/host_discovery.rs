use super::*;
use crate::host_provider::HostProvide;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::MissedTickBehavior;

const HOST_DISCOVERY_INTERVAL: Duration = Duration::from_secs(5);

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_host_discovery_loop(&self) {
        let mut interval = tokio::time::interval(HOST_DISCOVERY_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let hosts = match self.host_provider.list_hosts().await {
                Ok(hosts) => {
                    telemetry::list_hosts_status(true);
                    hosts
                }
                Err(err) => {
                    warn!(%err, "Failed to list hosts");
                    telemetry::list_hosts_status(false);
                    continue;
                }
            };

            let current_ids: HashSet<String> = hosts.iter().map(|h| h.id.to_string()).collect();

            for host in &hosts {
                if let Err(err) = self
                    .doc_db
                    .upsert_host_status_if_absent(
                        host.id.as_ref(),
                        &self.name,
                        &host.addr,
                        host.dns_addr.as_deref(),
                    )
                    .await
                {
                    warn!(%err, host_id = %host.id, "Failed to upsert host-status");
                }
            }

            let existing = match self.doc_db.list_host_statuses(&self.name).await {
                Ok(list) => list,
                Err(err) => {
                    warn!(%err, "Failed to list host-statuses for cleanup");
                    continue;
                }
            };
            for record in existing {
                if !current_ids.contains(&record.host_id) {
                    if let Err(err) = self.doc_db.delete_host_status(&record.site_name, &record.host_id).await {
                        warn!(%err, host_id = %record.host_id, "Failed to delete stale host-status");
                    }
                    self.ssh_pool.invalidate(&record.addr);
                }
            }
        }
    }
}
