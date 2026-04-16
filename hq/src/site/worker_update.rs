use super::*;
use crate::doc_db::WorkerTarget;
use crate::host_provider::HostProvide;
use crate::ssh_pool::SshPool;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::*;

const WORKER_UPDATE_INTERVAL: Duration = Duration::from_secs(10);

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_worker_update_loop(&self) {
        let mut interval = tokio::time::interval(WORKER_UPDATE_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let target = match self.doc_db.get_worker_target(&self.name).await {
                Ok(Some(t)) => t,
                Ok(None) => {
                    continue;
                }
                Err(err) => {
                    warn!(%err, "worker_update: failed to read worker-target");
                    continue;
                }
            };

            let ready = match self.doc_db.get_cwasm_ready(&self.name).await {
                Ok(r) => r,
                Err(err) => {
                    warn!(%err, "worker_update: failed to read cwasm-ready");
                    continue;
                }
            };
            let cwasm_ready = ready
                .as_ref()
                .map(|r| r.fn0_wasmtime_version == target.fn0_wasmtime_version)
                .unwrap_or(false);
            if !cwasm_ready {
                debug!(
                    target_version = %target.fn0_wasmtime_version,
                    ready_version = ?ready.as_ref().map(|r| r.fn0_wasmtime_version.as_str()),
                    "worker_update: waiting for cwasm fan-out"
                );
                continue;
            }

            let hosts = match self.host_provider.list_hosts().await {
                Ok(hosts) => hosts,
                Err(err) => {
                    warn!(%err, "worker_update: failed to list hosts");
                    continue;
                }
            };

            let desired_envs = self.host_provider.envs().clone();
            let desired_ref = target.image_ref();

            let mut tasks = Vec::with_capacity(hosts.len());
            for host in hosts {
                let pool = self.ssh_pool.clone();
                let envs = desired_envs.clone();
                let image = desired_ref.clone();
                let target_clone = target.clone();
                tasks.push(tokio::spawn(async move {
                    let addr = host.dns_addr.clone().unwrap_or_else(|| host.addr.clone());
                    if let Err(err) =
                        reconcile_one(&pool, &addr, &image, &envs, &target_clone).await
                    {
                        warn!(%err, host_id = %host.id, "worker_update: reconcile failed");
                    }
                }));
            }
            for t in tasks {
                let _ = t.await;
            }
        }
    }
}

async fn reconcile_one(
    ssh_pool: &SshPool,
    addr: &str,
    desired_image_ref: &str,
    desired_envs: &BTreeMap<String, String>,
    target: &WorkerTarget,
) -> color_eyre::Result<()> {
    let current = fetch_current_state(ssh_pool, addr).await?;

    let current_ref_matches = current
        .image_ref
        .as_ref()
        .map(|r| normalize_ref(r) == normalize_ref(desired_image_ref))
        .unwrap_or(false);
    let envs_match = current
        .envs
        .as_ref()
        .map(|e| subset_matches(e, desired_envs))
        .unwrap_or(false);

    if current_ref_matches && envs_match && current.running {
        return Ok(());
    }

    info!(
        %addr,
        current_image = ?current.image_ref,
        desired_image = %desired_image_ref,
        running = current.running,
        "Deploying worker container"
    );

    let script = build_deploy_script(desired_image_ref, desired_envs, target);
    let (code, output) = ssh_pool
        .exec_with_timeout(addr, &script, crate::ssh_pool::DEPLOY_TIMEOUT)
        .await?;
    if code != 0 {
        return Err(color_eyre::eyre::eyre!(
            "deploy_worker failed on {addr} (exit {code}): {}",
            output.trim()
        ));
    }
    Ok(())
}

struct ContainerState {
    running: bool,
    image_ref: Option<String>,
    envs: Option<BTreeMap<String, String>>,
}

async fn fetch_current_state(ssh_pool: &SshPool, addr: &str) -> color_eyre::Result<ContainerState> {
    let (code, output) = ssh_pool
        .exec(
            addr,
            "podman inspect fn0-worker --format '{{.State.Running}}|{{.ImageName}}|{{json .Config.Env}}' 2>/dev/null || echo 'MISSING'",
        )
        .await?;
    if code != 0 || output.trim() == "MISSING" {
        return Ok(ContainerState {
            running: false,
            image_ref: None,
            envs: None,
        });
    }
    let line = output.trim();
    let parts: Vec<&str> = line.splitn(3, '|').collect();
    if parts.len() != 3 {
        return Ok(ContainerState {
            running: false,
            image_ref: None,
            envs: None,
        });
    }
    let running = parts[0] == "true";
    let image_ref = Some(parts[1].to_string()).filter(|s| !s.is_empty());
    let envs = parse_env_json(parts[2]);
    Ok(ContainerState {
        running,
        image_ref,
        envs: Some(envs),
    })
}

fn parse_env_json(json_str: &str) -> BTreeMap<String, String> {
    let list: Vec<String> = serde_json::from_str(json_str).unwrap_or_default();
    let mut m = BTreeMap::new();
    for entry in list {
        if let Some((k, v)) = entry.split_once('=') {
            m.insert(k.to_string(), v.to_string());
        }
    }
    m
}

fn subset_matches(
    container_envs: &BTreeMap<String, String>,
    desired: &BTreeMap<String, String>,
) -> bool {
    use base64::Engine;
    for (k, desired_v) in desired {
        let got = container_envs
            .get(k)
            .cloned()
            .or_else(|| {
                container_envs
                    .get(&format!("{k}_BASE64"))
                    .and_then(|b64| {
                        base64::engine::general_purpose::STANDARD
                            .decode(b64.as_bytes())
                            .ok()
                    })
                    .and_then(|bytes| String::from_utf8(bytes).ok())
            });
        if got.as_deref() != Some(desired_v.as_str()) {
            return false;
        }
    }
    true
}

fn normalize_ref(s: &str) -> String {
    s.trim().to_string()
}

fn build_env_flags(envs: &BTreeMap<String, String>) -> String {
    use base64::Engine;
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

fn build_deploy_script(image: &str, envs: &BTreeMap<String, String>, _target: &WorkerTarget) -> String {
    let env_flags = build_env_flags(envs);
    format!(
        "bash -c 'set -e; \
podman stop fn0-worker 2>/dev/null || true; \
podman rm -f fn0-worker 2>/dev/null || true; \
podman pull {image}; \
podman create --replace --name fn0-worker \
  --network=host \
  --restart=always \
  -v /etc/fn0-worker:/etc/fn0-worker:ro {env_flags} {image}; \
podman start fn0-worker'"
    )
}
