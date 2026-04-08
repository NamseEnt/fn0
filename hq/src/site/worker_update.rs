use super::*;
use crate::host_provider::HostProvide;
use crate::ssh::SshClient;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::*;

const SSH_USER: &str = "opc";

struct WorkerState {
    image: String,
    envs: BTreeMap<String, String>,
}

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_worker_update_loop(&self) {
        let mut interval = tokio::time::interval(worker_update_interval());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let hosts = match self.host_provider.list_hosts().await {
                Ok(hosts) => hosts,
                Err(err) => {
                    warn!(%err, "Failed to list hosts for worker update");
                    continue;
                }
            };

            let desired_image = self.host_provider.worker_image_url().to_string();
            let desired_envs = self.host_provider.envs().clone();
            let ssh_key = self.host_provider.ssh_private_key_pem().to_string();

            for host in hosts {
                let ssh_addr = match &host.dns_addr {
                    Some(addr) => addr.clone(),
                    None => host.addr.clone(),
                };

                let desired_image = desired_image.clone();
                let desired_envs = desired_envs.clone();
                let ssh_key = ssh_key.clone();
                let host_id = host.id.to_string();

                tokio::spawn(async move {
                    if let Err(err) =
                        update_worker_if_needed(&ssh_addr, &ssh_key, &desired_image, &desired_envs)
                            .await
                    {
                        warn!(host_id, %err, "Failed to update worker");
                    }
                });
            }
        }
    }
}

async fn update_worker_if_needed(
    ssh_addr: &str,
    ssh_key: &str,
    desired_image: &str,
    desired_envs: &BTreeMap<String, String>,
) -> color_eyre::Result<()> {
    let ssh = SshClient::connect(ssh_addr, SSH_USER, ssh_key).await?;

    let current = match get_worker_state(&ssh).await {
        Ok(state) => state,
        Err(err) => {
            info!(%err, "Worker container not found or not inspectable, will deploy");
            deploy_worker(&ssh, desired_image, desired_envs).await?;
            let _ = ssh.close().await;
            return Ok(());
        }
    };

    let image_changed = current.image != *desired_image;
    let envs_changed = current.envs != *desired_envs;

    if !image_changed && !envs_changed {
        let _ = ssh.close().await;
        return Ok(());
    }

    if image_changed {
        info!(
            current = current.image,
            desired = desired_image,
            "Worker image mismatch, updating"
        );
    }
    if envs_changed {
        info!("Worker envs mismatch, updating");
    }

    deploy_worker(&ssh, desired_image, desired_envs).await?;
    let _ = ssh.close().await;
    Ok(())
}

async fn get_worker_state(ssh: &SshClient) -> color_eyre::Result<WorkerState> {
    let (status, output) = ssh
        .exec("podman inspect fn0-worker --format '{{.ImageName}}'")
        .await?;
    if status != 0 {
        return Err(color_eyre::eyre::eyre!(
            "podman inspect failed (exit {}): {}",
            status,
            output
        ));
    }
    let image = output.trim().to_string();

    let (status, output) = ssh
        .exec("podman inspect fn0-worker --format '{{json .Config.Env}}'")
        .await?;
    if status != 0 {
        return Err(color_eyre::eyre::eyre!(
            "podman inspect envs failed (exit {}): {}",
            status,
            output
        ));
    }
    let envs = parse_container_envs(output.trim());

    Ok(WorkerState { image, envs })
}

fn parse_container_envs(json_str: &str) -> BTreeMap<String, String> {
    let env_list: Vec<String> = serde_json::from_str(json_str).unwrap_or_default();
    let mut envs = BTreeMap::new();
    for entry in env_list {
        if let Some((k, v)) = entry.split_once('=') {
            envs.insert(k.to_string(), v.to_string());
        }
    }
    envs
}

fn build_env_flags(envs: &BTreeMap<String, String>) -> String {
    envs.iter()
        .map(|(k, v)| {
            if v.contains('\n') {
                let encoded =
                    base64::engine::general_purpose::STANDARD.encode(v.as_bytes());
                format!("-e {k}_BASE64={encoded}")
            } else {
                format!("-e {k}={v}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

async fn deploy_worker(
    ssh: &SshClient,
    image: &str,
    envs: &BTreeMap<String, String>,
) -> color_eyre::Result<()> {
    let env_flags = build_env_flags(envs);

    let script = format!(
        r#"set -e
systemctl stop fn0-worker 2>/dev/null || true
podman rm -f fn0-worker 2>/dev/null || true
podman pull {image}
podman create --name fn0-worker --network=host {env_flags} {image}
podman generate systemd --new --name fn0-worker > /etc/systemd/system/fn0-worker.service
systemctl daemon-reload
systemctl enable --now fn0-worker"#,
    );

    let (status, output) = ssh.exec(&format!("sudo bash -c '{script}'")).await?;
    if status != 0 {
        return Err(color_eyre::eyre::eyre!(
            "Worker deploy failed (exit {}): {}",
            status,
            output
        ));
    }

    info!("Worker deployed successfully");
    Ok(())
}

fn worker_update_interval() -> Duration {
    match std::env::var("WORKER_UPDATE_INTERVAL_MS") {
        Ok(s) => s
            .parse()
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_secs(60)),
        Err(_) => Duration::from_secs(60),
    }
}
