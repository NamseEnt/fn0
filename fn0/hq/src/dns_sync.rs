use crate::deployment_cache::DeploymentCache;
use crate::doc_db::DocDb;
use crate::{dns::DnsProvide, dns::DnsProvider, telemetry};
use std::collections::BTreeSet;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::*;

pub async fn run(
    dns_provider: DnsProvider,
    doc_db: DocDb,
    site_names: Vec<String>,
    deployment_cache: DeploymentCache,
) {
    let mut interval = tokio::time::interval(dns_sync_interval());
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let latest_generation = deployment_cache.last_deployment_id();

        let mut addrs: BTreeSet<String> = BTreeSet::new();
        for name in &site_names {
            match doc_db.list_host_statuses(name).await {
                Ok(statuses) => {
                    for s in statuses {
                        if !s.healthy || s.graceful {
                            continue;
                        }
                        if s.generation.unwrap_or(0) < latest_generation {
                            continue;
                        }
                        let addr = s.dns_addr.clone().unwrap_or_else(|| s.addr.clone());
                        addrs.insert(addr);
                    }
                }
                Err(err) => {
                    warn!(%err, site = %name, "dns_sync: failed to list host-status");
                }
            }
        }

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
        Ok(s) => s
            .parse()
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_secs(5)),
        Err(_) => Duration::from_secs(5),
    }
}
