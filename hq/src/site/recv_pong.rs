use bytes::{BufMut, BytesMut};

use super::*;
use crate::{
    deployment_db::{Deployment, DeploymentId},
    telemetry, *,
};

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_recv_pong_loop(&self, mut new_host_rx: mpsc::UnboundedReceiver<Host>) {
        while let Some(new_host) = new_host_rx.recv().await {
            let Some(connection) = self.host_connections.get(&new_host) else {
                warn!("Host {:?} not found", new_host);
                continue;
            };

            let connection = connection.clone();
            let hosts_last_pong = self.hosts_last_pong.clone();
            let deployment_db = self.deployment_db.clone();

            tokio::spawn(async move {
                while let Ok(bytes) = connection.read_unreliable_small_message().await {
                    let len = size_of::<u64>();
                    let Ok(array) = bytes[0..len].try_into() else {
                        warn!("Invalid pong message {}", bytes.len());
                        return;
                    };

                    telemetry::send_pong_received(&new_host.id);
                    hosts_last_pong.insert(new_host.clone(), Instant::now());

                    let host_deployment_id = u64::from_le_bytes(array);
                    let updates = deployment_db.slice_updates(DeploymentId(host_deployment_id));
                    send_updates(&connection, &new_host.id, updates, host_deployment_id);
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

    telemetry::send_deployment_updates_sent(host_id, updates.len());

    let mut bytes =
        BytesMut::with_capacity(updates.len() * size_of::<u64>() * 2 + size_of::<u64>());

    bytes.put_u64_le(host_deployment_id);
    for update in updates {
        bytes.put_u64_le(update.code_id);
        bytes.put_u64_le(update.code_version);
    }
    let connection = connection.clone();
    tokio::spawn(async move {
        match connection.send_reliable_big_message(bytes.freeze()).await {
            Ok(_) => {
                telemetry::send_deployment_updates_status(true);
            }
            Err(err) => {
                warn!(%err, "Failed to send updates");
                telemetry::send_deployment_updates_status(false);
            }
        }
    });
}
