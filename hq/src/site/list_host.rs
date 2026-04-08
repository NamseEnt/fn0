use super::*;
use crate::{host_connection::HostConnection, host_provider::HostTransport, random_sleep::random_sleep, telemetry};
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::time::{MissedTickBehavior, timeout};

enum ConnectTarget {
    Quic(SocketAddr, String),
    WebSocket(String),
}

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

            let current_hosts: HashSet<_> = hosts.iter().cloned().collect();

            for host in &hosts {
                if self.dead_hosts.contains_key(host) {
                    self.on_dead_host_in_list(host.clone());
                    continue;
                }
                if !self.host_connections.contains_key(host) {
                    self.on_new_host_in_list(host.clone(), new_host_tx.clone());
                }
            }

            // Clean up hosts that are no longer in the list (e.g. DWS removed from doc-db)
            let stale: Vec<_> = self
                .host_connections
                .iter()
                .filter(|e| !current_hosts.contains(e.key()))
                .map(|e| e.key().clone())
                .collect();
            for host in stale {
                if let Some((_, conn)) = self.host_connections.remove(&host) {
                    conn.close();
                }
                self.hosts_status.remove(&host);
                self.dead_hosts.remove(&host);
                self.graceful_shutdown_hosts.remove(&host);
            }
        }
    }
    #[tracing::instrument(skip(self), fields(host_id = %host.id, host_addr = %host.addr))]
    fn on_dead_host_in_list(&self, host: Host) {
        let host_provider = self.host_provider.clone();
        tokio::spawn(async move {
            random_sleep(1000).await;
            if let Err(err) = host_provider.terminate(&host.id).await {
                warn!(%err, "Failed to terminate host");
            }
        });
    }
    #[tracing::instrument(skip(self, new_host_tx), fields(host_id = %host.id, host_addr = %host.addr))]
    fn on_new_host_in_list(&self, host: Host, new_host_tx: mpsc::UnboundedSender<Host>) {
        self.known_hosts.insert(host.clone());

        let ca_cert_pem = self.ca_cert_pem.clone();
        let host_addr = host.addr.clone();
        let host_port = host.port;
        let host_transport = host.transport.clone();
        let ws_secret = self.ws_secret.clone();
        let dead_hosts = self.dead_hosts.clone();
        let host_connections = self.host_connections.clone();

        tokio::spawn(async move {
            let deadline = Instant::now() + Duration::from_secs(180);
            let connect_timeout = Duration::from_secs(2);

            let connect_target = match &host_transport {
                HostTransport::Quic => {
                    let resolved_addr = match resolve_addr(&host_addr, host_port).await {
                        Ok(addr) => addr,
                        Err(err) => {
                            warn!(%err, "Failed to resolve host addr");
                            dead_hosts.insert(host.clone(), Instant::now());
                            telemetry::host_connect_status(&host.id, false);
                            return;
                        }
                    };
                    ConnectTarget::Quic(resolved_addr, ca_cert_pem)
                }
                HostTransport::WebSocket => {
                    ConnectTarget::WebSocket(format!("wss://{}",  host_addr))
                }
            };

            loop {
                random_sleep(1000).await;

                if Instant::now() > deadline {
                    dead_hosts.insert(host.clone(), Instant::now());
                    telemetry::host_connect_status(&host.id, false);
                    break;
                }

                telemetry::host_connect_attempt(&host.id);

                let connect_start = Instant::now();
                let connect_result = match &connect_target {
                    ConnectTarget::Quic(addr, ca_pem) => {
                        timeout(connect_timeout, HostConnection::connect_quic(*addr, ca_pem)).await
                    }
                    ConnectTarget::WebSocket(url) => {
                        let secret = if ws_secret.is_empty() { None } else { Some(ws_secret.as_str()) };
                        timeout(connect_timeout, HostConnection::connect_websocket(url, secret)).await
                    }
                };

                match connect_result {
                    Ok(Ok(connection)) => {
                        let connect_duration = connect_start.elapsed();
                        info!(host_id = %host.id, host_addr = %host_addr, ?connect_duration, "Connected to host");
                        telemetry::host_connect_status(&host.id, true);
                        telemetry::host_connect_duration(&host.id, connect_duration);
                        host_connections.insert(host.clone(), connection);
                        let _ = new_host_tx.send(host);
                        break;
                    }
                    Ok(Err(err)) => {
                        warn!(host_id = %host.id, host_addr = %host_addr, %err, "Failed to connect to host");
                        telemetry::host_connect_status(&host.id, false);
                    }
                    Err(_elapsed) => {
                        warn!(host_id = %host.id, host_addr = %host_addr, "Timeout to connect to host");
                        telemetry::host_connect_status(&host.id, false);
                    }
                }
            }
        });
    }
}

async fn resolve_addr(addr: &str, port: u16) -> color_eyre::Result<SocketAddr> {
    if let Ok(ip) = addr.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }
    let resolved = tokio::net::lookup_host(format!("{addr}:{port}"))
        .await?
        .next()
        .ok_or_else(|| color_eyre::eyre::eyre!("DNS resolution returned no results for {addr}"))?;
    Ok(resolved)
}

fn list_host_info_interval_ms() -> Duration {
    match std::env::var("LIST_HOST_INFO_INTERVAL_MS") {
        Ok(s) => s.parse().map(Duration::from_millis).unwrap_or(Duration::from_secs(10)),
        Err(_) => Duration::from_secs(10),
    }
}
