use super::*;
use crate::host_provider::HostProvide;
use crate::ssh::SshClient;
use base64::Engine;
use doc_db::WasmtimeRolloutPhase;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::*;

const SSH_USER: &str = "opc";

struct WorkerState {
    image_id: String,
    envs: BTreeMap<String, String>,
}

impl WorkerState {
    fn wasmtime_version(&self) -> Option<&str> {
        self.envs.get("FN0_WASMTIME_VERSION").map(String::as_str)
    }
}

struct HostSnapshot {
    host_id: String,
    ssh_addr: String,
    current: Option<WorkerState>,
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

            info!(host_count = hosts.len(), "Worker update tick");

            let desired_image = self.host_provider.worker_image_url().to_string();
            let desired_envs = self.host_provider.envs().clone();
            let ssh_key = self.host_provider.ssh_private_key_pem().to_string();
            let rollout_state = match self.reconcile_wasmtime_rollout().await {
                Ok(state) => state,
                Err(err) => {
                    warn!(%err, "Failed to reconcile wasmtime rollout");
                    continue;
                }
            };

            let host_snapshots = inspect_hosts(&hosts, &desired_envs, &ssh_key).await;

            match rollout_state.phase {
                WasmtimeRolloutPhase::Idle => {
                    for snapshot in &host_snapshots {
                        let result = update_worker_if_needed(
                            &snapshot.host_id,
                            &snapshot.ssh_addr,
                            &ssh_key,
                            &desired_image,
                            &desired_envs,
                        )
                        .await;

                        if let Err(err) = result {
                            warn!(host_id = snapshot.host_id, %err, "Failed to update worker");
                        }
                    }
                }
                WasmtimeRolloutPhase::Rebundling => {
                    info!(
                        site = %self.name,
                        target_version = %rollout_state.target_version,
                        "Wasmtime rollout waiting for bundle rebuilds"
                    );
                }
                WasmtimeRolloutPhase::ReadyToRoll | WasmtimeRolloutPhase::Rolling => {
                    let stale_hosts: Vec<_> = host_snapshots
                        .iter()
                        .filter(|snapshot| {
                            snapshot
                                .current
                                .as_ref()
                                .and_then(WorkerState::wasmtime_version)
                                != Some(rollout_state.target_version.as_str())
                        })
                        .collect();

                    if stale_hosts.is_empty() {
                        let mut cleaning_state = rollout_state.clone();
                        cleaning_state.current_version = cleaning_state.target_version.clone();
                        if let Some(previous_version) = &rollout_state.previous_version {
                            cleaning_state.previous_version = Some(previous_version.clone());
                        }
                        if let Err(err) = self
                            .set_wasmtime_rollout_phase(
                                &cleaning_state,
                                WasmtimeRolloutPhase::Cleaning,
                            )
                            .await
                        {
                            warn!(%err, "Failed to move rollout to cleaning");
                        }
                        continue;
                    }

                    if rollout_state.phase == WasmtimeRolloutPhase::ReadyToRoll {
                        if let Err(err) = self
                            .set_wasmtime_rollout_phase(
                                &rollout_state,
                                WasmtimeRolloutPhase::Rolling,
                            )
                            .await
                        {
                            warn!(%err, "Failed to move rollout to rolling");
                            continue;
                        }
                    }

                    let next_host = stale_hosts[0];
                    let result = update_worker_if_needed(
                        &next_host.host_id,
                        &next_host.ssh_addr,
                        &ssh_key,
                        &desired_image,
                        &desired_envs,
                    )
                    .await;

                    if let Err(err) = result {
                        warn!(host_id = next_host.host_id, %err, "Failed to roll worker");
                    } else {
                        info!(
                            host_id = next_host.host_id,
                            target_version = %rollout_state.target_version,
                            "Rolled one worker to the target wasmtime version"
                        );
                    }
                }
                WasmtimeRolloutPhase::Cleaning => {
                    let stale_hosts = host_snapshots.iter().any(|snapshot| {
                        snapshot
                            .current
                            .as_ref()
                            .and_then(WorkerState::wasmtime_version)
                            != Some(rollout_state.target_version.as_str())
                    });
                    if stale_hosts {
                        if let Err(err) = self
                            .set_wasmtime_rollout_phase(
                                &rollout_state,
                                WasmtimeRolloutPhase::Rolling,
                            )
                            .await
                        {
                            warn!(%err, "Failed to move rollout back to rolling");
                        }
                        continue;
                    }

                    if let Err(err) = self.finish_wasmtime_rollout_cleanup(&rollout_state).await {
                        warn!(%err, "Failed to finish wasmtime rollout cleanup");
                    } else {
                        info!(
                            site = %self.name,
                            current_version = %rollout_state.target_version,
                            "Wasmtime rollout completed"
                        );
                    }
                }
            }
        }
    }
}

async fn inspect_hosts(
    hosts: &[Host],
    desired_envs: &BTreeMap<String, String>,
    ssh_key: &str,
) -> Vec<HostSnapshot> {
    let mut snapshots = Vec::with_capacity(hosts.len());

    for host in hosts {
        let ssh_addr = match &host.dns_addr {
            Some(addr) => addr.clone(),
            None => host.addr.clone(),
        };
        let host_id = host.id.to_string();

        let current = match SshClient::connect(&ssh_addr, SSH_USER, ssh_key).await {
            Ok(ssh) => {
                let state = get_worker_state(&ssh, desired_envs).await.ok();
                let _ = ssh.close().await;
                state
            }
            Err(err) => {
                warn!(host_id, %err, "Failed to inspect worker");
                None
            }
        };

        snapshots.push(HostSnapshot {
            host_id,
            ssh_addr,
            current,
        });
    }

    snapshots
}

async fn update_worker_if_needed(
    host_id: &str,
    ssh_addr: &str,
    ssh_key: &str,
    desired_image: &str,
    desired_envs: &BTreeMap<String, String>,
) -> color_eyre::Result<()> {
    let ssh = SshClient::connect(ssh_addr, SSH_USER, ssh_key).await?;

    let current = match get_worker_state(&ssh, desired_envs).await {
        Ok(state) => state,
        Err(err) => {
            info!(host_id, %err, "Worker container not found or not inspectable, will deploy");
            deploy_worker(&ssh, desired_image, desired_envs).await?;
            let _ = ssh.close().await;
            return Ok(());
        }
    };

    let desired_id = get_remote_image_id(&ssh, desired_image).await?;
    let image_changed = current.image_id != desired_id;
    let envs_changed = current.envs != *desired_envs;

    if !image_changed && !envs_changed {
        let _ = ssh.close().await;
        return Ok(());
    }

    if image_changed {
        info!(
            host_id,
            current_id = current.image_id,
            desired_id,
            "Worker image mismatch, updating"
        );
    }
    if envs_changed {
        info!(host_id, "Worker envs mismatch, updating");
    }

    deploy_worker(&ssh, desired_image, desired_envs).await?;
    let _ = ssh.close().await;
    Ok(())
}

async fn get_remote_image_id(ssh: &SshClient, image: &str) -> color_eyre::Result<String> {
    let (pull_status, pull_output) = ssh.exec(&format!("sudo podman pull {image}")).await?;
    if pull_status != 0 {
        return Err(color_eyre::eyre::eyre!(
            "podman pull failed (exit {}): {}",
            pull_status,
            pull_output
        ));
    }

    let (status, output) = ssh
        .exec(&format!(
            "sudo podman image inspect {image} --format '{{{{.Id}}}}'"
        ))
        .await?;
    if status != 0 {
        return Err(color_eyre::eyre::eyre!(
            "podman image inspect failed (exit {}): {}",
            status,
            output
        ));
    }
    Ok(normalize_image_id(output.trim()))
}

async fn get_worker_state(
    ssh: &SshClient,
    desired_envs: &BTreeMap<String, String>,
) -> color_eyre::Result<WorkerState> {
    let (status, output) = ssh
        .exec("sudo podman inspect fn0-worker --format '{{.Image}}'")
        .await?;
    if status != 0 {
        return Err(color_eyre::eyre::eyre!(
            "podman inspect failed (exit {}): {}",
            status,
            output
        ));
    }
    let image_id = normalize_image_id(output.trim());

    let (status, output) = ssh
        .exec("sudo podman inspect fn0-worker --format '{{json .Config.Env}}'")
        .await?;
    if status != 0 {
        return Err(color_eyre::eyre::eyre!(
            "podman inspect envs failed (exit {}): {}",
            status,
            output
        ));
    }
    let container_envs = parse_container_envs(output.trim());

    let envs = filter_envs_by_desired_keys(&container_envs, desired_envs);

    Ok(WorkerState { image_id, envs })
}

fn filter_envs_by_desired_keys(
    container_envs: &BTreeMap<String, String>,
    desired_envs: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for key in desired_envs.keys() {
        if let Some(value) = container_envs.get(key) {
            result.insert(key.clone(), value.clone());
        } else if let Some(value) = container_envs.get(&format!("{key}_BASE64")) {
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(value.as_bytes())
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
                .unwrap_or_default();
            result.insert(key.clone(), decoded);
        }
    }
    result
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

fn normalize_image_id(id: &str) -> String {
    id.strip_prefix("sha256:").unwrap_or(id).to_string()
}

fn build_env_flags(envs: &BTreeMap<String, String>) -> String {
    envs.iter()
        .map(|(k, v)| {
            if v.contains('\n') {
                let encoded = base64::engine::general_purpose::STANDARD.encode(v.as_bytes());
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
systemctl disable fn0-worker 2>/dev/null || true
podman stop fn0-worker 2>/dev/null || true
podman rm -f fn0-worker 2>/dev/null || true
podman pull {image}
podman create --replace --name fn0-worker --network=host {env_flags} {image}
podman generate systemd --name fn0-worker > /etc/systemd/system/fn0-worker.service
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

    let (check_status, check_output) = ssh
        .exec("sudo podman ps --filter name=fn0-worker --format '{{.Status}}'")
        .await?;
    info!(
        container_status = check_output.trim(),
        check_exit = check_status,
        "Worker deployed"
    );
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
