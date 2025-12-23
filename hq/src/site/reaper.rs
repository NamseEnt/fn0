use super::*;
use crate::{telemetry, *};
use std::time::{Duration, Instant};
use tokio::time::MissedTickBehavior;

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_reaper(&self) {
        let mut interval = tokio::time::interval(reaper_interval_ms());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let timeout_threshold = host_connection_timeout_ms();
            let terminate_candidates = self
                .hosts_status
                .iter()
                .filter(|entry| entry.value().received_at.elapsed() < timeout_threshold)
                .count();

            telemetry::reaper_terminate_candidates(terminate_candidates);
            telemetry::active_connections(self.host_connections.len());

            let mut removed_count = 0;

            self.hosts_status.retain(|host, status| {
                if status.received_at.elapsed() < timeout_threshold {
                    return true;
                }
                if let Some((_host, connection)) = self.host_connections.remove(host) {
                    connection.close();
                };
                self.dead_hosts.insert(host.clone(), Instant::now());
                removed_count += 1;
                telemetry::reaper_terminate_attempts();

                false
            });

            if removed_count > 0 {
                telemetry::reaper_removed_count(removed_count);
            }
        }
    }
}

fn reaper_interval_ms() -> Duration {
    match std::env::var("REAPER_INTERVAL_MS") {
        Ok(s) => match s.parse() {
            Ok(v) => return Duration::from_millis(v),
            Err(err) => warn!(%err, "REAPER_INTERVAL_MS is not a valid number"),
        },
        Err(err) => warn!(%err, "Fail to get REAPER_INTERVAL_MS from env"),
    }
    Duration::from_secs(1)
}

fn host_connection_timeout_ms() -> Duration {
    match std::env::var("HOST_CONNECTION_TIMEOUT_MS") {
        Ok(s) => match s.parse() {
            Ok(v) => return Duration::from_millis(v),
            Err(err) => warn!(%err, "HOST_CONNECTION_TIMEOUT_MS is not a valid number"),
        },
        Err(err) => warn!(%err, "Fail to get HOST_CONNECTION_INTERVAL_MS from env"),
    }
    Duration::from_secs(6)
}
