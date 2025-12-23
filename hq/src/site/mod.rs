mod dns_sync;
mod list_host;
mod reaper;
mod recv_pong;
mod scaler;
mod send_ping;

use crate::{
    deployment_cache::DeploymentCache, dns::DnsProvider, host_connection::HostConnection,
    telemetry, *,
};
use dashmap::{DashMap, DashSet};
use doc_db::DocDb;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

pub struct Site {
    host_provider: HostProvider,
    dns_provider: DnsProvider,
    host_connections: Arc<DashMap<Host, HostConnection>>,
    hosts_status: Arc<DashMap<Host, HostStatus>>,
    cert: String,
    pub deployment_cache: DeploymentCache,
    // Below fields won't be cleared so may occur out-of-memory.
    // But the size is expected to be too small to cause out-of-memory.
    known_hosts: Arc<DashSet<Host>>,
    dead_hosts: Arc<DashMap<Host, Instant>>,
    graceful_shutdown_hosts: Arc<DashMap<Host, Instant>>,
    host_cpu_cores: NonZeroUsize,
    host_memory_in_gb: NonZeroUsize,
    doc_db: DocDb,
}

impl Site {
    pub fn new(
        host_provider: HostProvider,
        dns_provider: DnsProvider,
        cert: String,
        deployment_cache: DeploymentCache,
        host_cpu_cores: NonZeroUsize,
        host_memory_in_gb: NonZeroUsize,
        doc_db: DocDb,
    ) -> Self {
        Site {
            host_provider,
            dns_provider,
            host_connections: Default::default(),
            hosts_status: Default::default(),
            known_hosts: Default::default(),
            dead_hosts: Default::default(),
            graceful_shutdown_hosts: Default::default(),
            cert,
            deployment_cache,
            host_cpu_cores,
            host_memory_in_gb,
            doc_db,
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
            self.run_dns_sync(),
            self.run_metrics_reporter(),
            self.run_scaler()
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
