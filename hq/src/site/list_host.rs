use super::*;
use crate::{host_connection::HostConnection, random_sleep::random_sleep, telemetry, *};
use std::time::{Duration, Instant};
use tokio::time::{MissedTickBehavior, timeout};

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_list_host_loop(&self, new_host_tx: mpsc::UnboundedSender<Host>) {
        let mut interval = tokio::time::interval(list_host_info_interval_ms());
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

            for host in hosts {
                if self.dead_hosts.contains_key(&host) {
                    self.on_dead_host_in_list(host);
                    continue;
                }
                if !self.host_connections.contains_key(&host) {
                    self.on_new_host_in_list(host, new_host_tx.clone());
                }
            }
        }
    }
    #[tracing::instrument(skip(self), fields(host_id = %host.id, host_ip = %host.ip))]
    fn on_dead_host_in_list(&self, host: Host) {
        let host_provider = self.host_provider.clone();
        tokio::spawn(async move {
            random_sleep(1000).await;
            if let Err(err) = host_provider.terminate(&host.id).await {
                warn!(%err, "Failed to terminate host");
            }
        });
    }
    #[tracing::instrument(skip(self, new_host_tx), fields(host_id = %host.id, host_ip = %host.ip))]
    fn on_new_host_in_list(&self, host: Host, new_host_tx: mpsc::UnboundedSender<Host>) {
        self.known_hosts.insert(host.clone());

        let cert = self.cert.clone();
        let addr = SocketAddr::new(host.ip, 10000);
        let dead_hosts = self.dead_hosts.clone();
        let host_connections = self.host_connections.clone();

        tokio::spawn(async move {
            let start_time = Instant::now();
            let deadline = start_time + Duration::from_secs(180);
            let connect_timeout = Duration::from_secs(2);

            loop {
                random_sleep(1000).await;

                if deadline < start_time {
                    dead_hosts.insert(host.clone(), Instant::now());
                    telemetry::host_connect_status(&host.id, false);
                    break;
                }

                telemetry::host_connect_attempt(&host.id);

                let connect_start = Instant::now();
                let connect_result =
                    timeout(connect_timeout, HostConnection::connect(addr, &cert)).await;

                match connect_result {
                    Ok(Ok(connection)) => {
                        let connect_duration = connect_start.elapsed();
                        telemetry::host_connect_status(&host.id, true);
                        telemetry::host_connect_duration(&host.id, connect_duration);
                        host_connections.insert(host.clone(), connection);
                        let _ = new_host_tx.send(host);
                        break;
                    }
                    Ok(Err(err)) => {
                        warn!(%err, "Failed to connect to host");
                        telemetry::host_connect_status(&host.id, false);
                    }
                    Err(_elapsed) => {
                        warn!("Timeout to connect to host");
                        telemetry::host_connect_status(&host.id, false);
                    }
                }
            }
        });
    }
}

fn list_host_info_interval_ms() -> Duration {
    match std::env::var("LIST_HOST_INFO_INTERVAL_MS") {
        Ok(s) => match s.parse() {
            Ok(v) => return Duration::from_millis(v),
            Err(err) => warn!(%err, "LIST_HOST_INFO_INTERVAL_MS is not a valid number"),
        },
        Err(err) => warn!(%err, "Fail to get LIST_HOST_INFO_INTERVAL_MS from env"),
    }
    Duration::from_secs(10)
}
