mod list_host;
mod reaper;
mod recv_pong;
mod scaler;
mod send_ping;
mod worker_update;

use crate::{
    deployment_cache::DeploymentCache, host_connection::HostConnection,
    host_provider::oci_compute_vm::OciComputeVmHostProvider,
    telemetry, *,
};
use dashmap::{DashMap, DashSet};
use doc_db::DocDb;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

pub struct Site {
    name: String,
    host_provider: OciComputeVmHostProvider,
    pub host_connections: Arc<DashMap<Host, HostConnection>>,
    hosts_status: Arc<DashMap<Host, HostStatus>>,
    ca_cert_pem: String,
    pub deployment_cache: DeploymentCache,
    known_hosts: Arc<DashSet<Host>>,
    dead_hosts: Arc<DashMap<Host, Instant>>,
    graceful_shutdown_hosts: Arc<DashMap<Host, Instant>>,
    host_cpu_cores: NonZeroUsize,
    host_memory_in_gb: NonZeroUsize,
    doc_db: DocDb,
    ws_secret: String,
}

impl Site {
    pub fn new(
        name: String,
        host_provider: OciComputeVmHostProvider,
        ca_cert_pem: String,
        deployment_cache: DeploymentCache,
        host_cpu_cores: NonZeroUsize,
        host_memory_in_gb: NonZeroUsize,
        doc_db: DocDb,
        ws_secret: String,
    ) -> Self {
        Site {
            name,
            host_provider,
            host_connections: Default::default(),
            hosts_status: Default::default(),
            known_hosts: Default::default(),
            dead_hosts: Default::default(),
            graceful_shutdown_hosts: Default::default(),
            ca_cert_pem,
            deployment_cache,
            host_cpu_cores,
            host_memory_in_gb,
            doc_db,
            ws_secret,
        }
    }
    #[tracing::instrument(skip_all)]
    pub async fn run(&mut self) {
        let (new_host_tx, new_host_rx) = mpsc::unbounded_channel();
        tokio::join!(
            self.run_list_host_loop(new_host_tx),
            self.run_send_ping_loop(),
            self.run_recv_pong_loop(new_host_rx),
            self.run_reaper(),
            self.run_metrics_reporter(),
            self.run_scaler(),
            self.run_worker_update_loop(),
        );
    }

    #[tracing::instrument(skip_all)]
    async fn run_metrics_reporter(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            telemetry::known_hosts(self.known_hosts.len());
            telemetry::dead_hosts(self.dead_hosts.len());
        }
    }
}

#[derive(Clone, Copy)]
struct HostStatus {
    received_at: Instant,
    host_timestamp: u64,
    instances: u64,
}
