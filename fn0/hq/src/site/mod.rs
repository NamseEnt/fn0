mod deployment_pusher;
mod graceful_coordinator;
mod host_discovery;
mod host_inspector;
mod scaler;
mod terminate;
mod worker_update;

pub use deployment_pusher::push_to_all;

use crate::doc_db::DocDb;
use crate::{
    deployment_cache::DeploymentCache, host_provider::oci_compute_vm::OciComputeVmHostProvider,
    ssh_pool::SshPool, telemetry,
};
use std::num::NonZeroUsize;
use tokio::time::MissedTickBehavior;
use tracing::warn;

#[derive(Clone)]
pub struct Site {
    name: String,
    host_provider: OciComputeVmHostProvider,
    ssh_pool: SshPool,
    pub deployment_cache: DeploymentCache,
    host_cpu_cores: NonZeroUsize,
    host_memory_in_gb: NonZeroUsize,
    doc_db: DocDb,
}

impl Site {
    pub fn new(
        name: String,
        host_provider: OciComputeVmHostProvider,
        deployment_cache: DeploymentCache,
        host_cpu_cores: NonZeroUsize,
        host_memory_in_gb: NonZeroUsize,
        doc_db: DocDb,
    ) -> Self {
        let ssh_pool = SshPool::new(
            "opc".to_string(),
            host_provider.ssh_private_key_pem().to_string(),
        );
        Site {
            name,
            host_provider,
            ssh_pool,
            deployment_cache,
            host_cpu_cores,
            host_memory_in_gb,
            doc_db,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ssh_pool(&self) -> &SshPool {
        &self.ssh_pool
    }

    #[tracing::instrument(skip_all, fields(site = %self.name))]
    pub async fn run(self) {
        tokio::join!(
            self.run_host_discovery_loop(),
            self.run_host_inspector_loop(),
            self.run_scaler(),
            self.run_graceful_coordinator_loop(),
            self.run_worker_update_loop(),
            self.run_metrics_reporter(),
        );
    }

    #[tracing::instrument(skip_all)]
    async fn run_metrics_reporter(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            interval.tick().await;

            match self.doc_db.list_host_statuses(&self.name).await {
                Ok(list) => {
                    let total = list.len();
                    let healthy = list.iter().filter(|s| s.healthy).count();
                    telemetry::known_hosts(total);
                    telemetry::dead_hosts(total - healthy);
                }
                Err(err) => {
                    warn!(%err, "Metrics reporter: failed to list host-status");
                }
            }
        }
    }
}
