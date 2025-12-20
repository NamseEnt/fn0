mod dns_sync;
mod list_host;
mod reaper;
mod recv_pong;
mod send_ping;

use crate::{
    deployment_db::DeploymentDb, dns::DnsProvider, host_connection::HostConnection, telemetry, *,
};
use dashmap::{DashMap, DashSet};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

pub struct Site {
    host_provider: HostProvider,
    dns_provider: DnsProvider,
    host_connections: Arc<DashMap<Host, HostConnection>>,
    hosts_last_pong: Arc<DashMap<Host, Instant>>,
    cert: String,
    pub deployment_db: DeploymentDb,
    // Below fields won't be cleared so may occur out-of-memory.
    // But the size is expected to be too small to cause out-of-memory.
    known_hosts: Arc<DashSet<Host>>,
    dead_hosts: Arc<DashMap<Host, Instant>>,
}

impl Site {
    pub fn new(
        host_provider: HostProvider,
        dns_provider: DnsProvider,
        cert: String,
        deployment_db: DeploymentDb,
    ) -> Self {
        Site {
            host_provider,
            dns_provider,
            host_connections: Default::default(),
            hosts_last_pong: Default::default(),
            known_hosts: Default::default(),
            dead_hosts: Default::default(),
            cert,
            deployment_db,
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
        );
    }

    #[tracing::instrument(skip_all)]
    async fn run_metrics_reporter(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            telemetry::send_known_hosts(self.known_hosts.len());
            telemetry::send_dead_hosts(self.dead_hosts.len());
        }
    }
}
