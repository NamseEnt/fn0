use crate::{dns::DnsProvide, dns::DnsProvider, host_connection::HostConnection, telemetry, Host};
use dashmap::DashMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::*;

pub async fn run(
    dns_provider: DnsProvider,
    all_host_connections: Vec<Arc<DashMap<Host, HostConnection>>>,
) {
    let mut interval = tokio::time::interval(dns_sync_interval());
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let addrs: BTreeSet<String> = all_host_connections
            .iter()
            .flat_map(|connections| {
                connections.iter().map(|conn| {
                    let host = conn.key();
                    host.dns_addr.clone().unwrap_or_else(|| host.addr.clone())
                })
            })
            .collect();

        telemetry::dns_healthy_ips(addrs.len());

        match dns_provider.sync_addrs(addrs).await {
            Ok(_) => {
                telemetry::dns_sync_status(true);
            }
            Err(err) => {
                warn!(%err, "Failed to sync DNS");
                telemetry::dns_sync_status(false);
            }
        }
    }
}

fn dns_sync_interval() -> Duration {
    match std::env::var("DNS_SYNC_INTERVAL_MS") {
        Ok(s) => s.parse().map(Duration::from_millis).unwrap_or(Duration::from_secs(1)),
        Err(_) => Duration::from_secs(1),
    }
}
