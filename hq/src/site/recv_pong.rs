use super::*;
use crate::{deployment_cache::DeploymentId, telemetry, *};
use doc_db::Deployment;
use host_hq_protocol::{HostToHq, HqToHostReliable};

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_recv_pong_loop(&self, mut new_host_rx: mpsc::UnboundedReceiver<Host>) {
        while let Some(new_host) = new_host_rx.recv().await {
            let Some(connection) = self.host_connections.get(&new_host) else {
                warn!("Host {:?} not found", new_host);
                continue;
            };

            let connection = connection.clone();
            let hosts_status = self.hosts_status.clone();
            let deployment_cache = self.deployment_cache.clone();

            tokio::spawn(async move {
                while let Ok(bytes) = connection.read_unreliable_small_message().await {
                    let host_to_hq = match HostToHq::from_bytes(bytes) {
                        Ok(host_to_hq) => host_to_hq,
                        Err(err) => {
                            warn!(%err, "Failed to parse host message");
                            return;
                        }
                    };

                    telemetry::pong_received(&new_host.id);

                    match host_to_hq {
                        HostToHq::NotifyHostStatus {
                            timestamp,
                            deployment_id,
                            instances,
                        } => {
                            let new_status = HostStatus {
                                received_at: Instant::now(),
                                host_timestamp: timestamp,
                                instances,
                            };
                            let entry = hosts_status.entry(new_host.clone());
                            entry
                                .and_modify(|host_status| {
                                    if host_status.host_timestamp < timestamp {
                                        *host_status = new_status;
                                    }
                                })
                                .or_insert(new_status);

                            let updates =
                                deployment_cache.slice_updates(DeploymentId(deployment_id));
                            send_updates(&connection, &new_host.id, updates, deployment_id);
                        }
                    }
                }
            });
        }
    }
}

#[tracing::instrument(skip(connection, updates), fields(update_count = updates.len()))]
fn send_updates(
    connection: &HostConnection,
    host_id: &str,
    updates: Vec<Deployment>,
    host_deployment_id: u64,
) {
    if updates.is_empty() {
        return;
    }

    telemetry::deployment_updates_sent(host_id, updates.len());

    let message = HqToHostReliable::DeploymentUpdates {
        deployment_id: host_deployment_id,
        code_id_and_versions: updates
            .into_iter()
            .map(|update| (update.code_id, update.code_version))
            .collect(),
    };
    let connection = connection.clone();
    tokio::spawn(async move {
        match connection.send_reliable(message).await {
            Ok(_) => {
                telemetry::deployment_updates_status(true);
            }
            Err(err) => {
                warn!(%err, "Failed to send updates");
                telemetry::deployment_updates_status(false);
            }
        }
    });
}
