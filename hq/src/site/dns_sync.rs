use super::*;
use crate::{dns::DnsProvide, telemetry, *};
use std::collections::BTreeSet;
use std::time::Duration;
use tokio::time::MissedTickBehavior;

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_dns_sync(&self) {
        let mut interval = tokio::time::interval(dns_sync_interval_ms());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let addrs = self
                .host_connections
                .iter()
                .map(|conn| {
                    let host = conn.key();
                    host.dns_addr.clone().unwrap_or_else(|| host.addr.clone())
                })
                .collect::<BTreeSet<_>>();

            telemetry::dns_healthy_ips(addrs.len());

            match self.dns_provider.sync_addrs(addrs).await {
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
}

fn dns_sync_interval_ms() -> Duration {
    match std::env::var("DNS_SYNC_INTERVAL_MS") {
        Ok(s) => match s.parse() {
            Ok(v) => return Duration::from_millis(v),
            Err(err) => warn!(%err, "DNS_SYNC_INTERVAL_MS is not a valid number"),
        },
        Err(err) => warn!(%err, "Fail to get DNS_SYNC_INTERVAL_MS from env"),
    }
    Duration::from_secs(1)
}
