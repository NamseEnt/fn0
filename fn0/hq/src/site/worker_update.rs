use super::*;
use crate::doc_db::{GracefulPurpose, HostStatus, WorkerLastStable, WorkerTarget};
use crate::host_id::HostId;
use crate::ssh_pool::SshPool;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::*;

const ROLLING_INTERVAL: Duration = Duration::from_secs(5);

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_worker_update_loop(&self) {
        let mut interval = tokio::time::interval(ROLLING_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if let Err(err) = self.rolling_tick().await {
                warn!(%err, "rolling_coordinator tick failed");
            }
        }
    }

    async fn rolling_tick(&self) -> color_eyre::Result<()> {
        let Some(target) = self.doc_db.get_worker_target(&self.name).await? else {
            return Ok(());
        };
        let last_stable = self.doc_db.get_worker_last_stable(&self.name).await?;

        let ready = self.doc_db.get_cwasm_ready(&self.name).await?;
        let cwasm_ok = ready
            .as_ref()
            .map(|r| r.fn0_wasmtime_version == target.fn0_wasmtime_version)
            .unwrap_or(false);

        let statuses = self.doc_db.list_host_statuses(&self.name).await?;

        let mut at_target = 0usize;
        let mut at_last_stable: Vec<HostStatus> = Vec::new();
        let mut stale: Vec<HostStatus> = Vec::new();
        let mut graceful_in_progress = false;

        for s in &statuses {
            if s.graceful {
                graceful_in_progress = true;
                continue;
            }
            let image = match self.fetch_container_image(&s.addr).await {
                Ok(Some(i)) => i,
                Ok(None) => {
                    if cwasm_ok {
                        self.deploy_fresh(s, &target).await;
                    }
                    continue;
                }
                Err(err) => {
                    warn!(%err, host_id = %s.host_id, "rolling_coordinator: inspect failed");
                    continue;
                }
            };
            let class = classify(&image, &target, last_stable.as_ref());
            match class {
                HostClass::AtTarget => at_target += 1,
                HostClass::AtLastStable => at_last_stable.push(s.clone()),
                HostClass::Stale => stale.push(s.clone()),
            }
        }

        for s in stale {
            info!(
                host_id = %s.host_id,
                "Terminating stale host (neither target nor last_stable)"
            );
            let provider = self.host_provider.clone();
            let db = self.doc_db.clone();
            let pool = self.ssh_pool.clone();
            let id = s.host_id.clone();
            let addr = s.addr.clone();
            tokio::spawn(async move {
                super::terminate::terminate_host(&provider, &db, &pool, &id, &addr).await;
            });
        }

        if cwasm_ok
            && !graceful_in_progress
            && let Some(victim) = at_last_stable.into_iter().next()
        {
            info!(
                host_id = %victim.host_id,
                "Starting graceful image swap toward target"
            );
            let now = chrono::Utc::now().to_rfc3339();
            if let Err(err) = self
                .doc_db
                .set_host_graceful(&victim.host_id, GracefulPurpose::ImageSwap, &now)
                .await
            {
                warn!(%err, host_id = %victim.host_id, "set_host_graceful(ImageSwap) failed");
            }
        }

        if at_target > 0
            && at_target == statuses.iter().filter(|s| !s.graceful).count()
            && !graceful_in_progress
            && last_stable_differs(&last_stable, &target)
        {
            let ls = WorkerLastStable {
                image_registry: target.image_registry.clone(),
                image_repository: target.image_repository.clone(),
                image_tag: target.image_tag.clone(),
                fn0_wasmtime_version: target.fn0_wasmtime_version.clone(),
            };
            if let Err(err) = self.doc_db.set_worker_last_stable(&self.name, &ls).await {
                warn!(%err, "Failed to write worker-last-stable");
            } else {
                info!(
                    site = %self.name,
                    version = %target.image_tag,
                    wasmtime = %target.fn0_wasmtime_version,
                    "Rollout converged; last_stable updated"
                );
            }
        }

        Ok(())
    }

    async fn fetch_container_image(&self, addr: &str) -> color_eyre::Result<Option<String>> {
        let (code, output) = self
            .ssh_pool
            .exec(
                addr,
                "podman inspect fn0-worker --format '{{.ImageName}}' 2>/dev/null || echo 'MISSING'",
            )
            .await?;
        if code != 0 {
            return Ok(None);
        }
        let trimmed = output.trim();
        if trimmed == "MISSING" || trimmed.is_empty() {
            return Ok(None);
        }
        Ok(Some(trimmed.to_string()))
    }

    async fn deploy_fresh(&self, s: &HostStatus, target: &WorkerTarget) {
        let envs = self.host_provider.envs().clone();
        let image = target.image_ref();
        let script = build_deploy_script(&image, &envs, target);
        match self
            .ssh_pool
            .exec_with_timeout(&s.addr, &script, crate::ssh_pool::DEPLOY_TIMEOUT)
            .await
        {
            Ok((0, _)) => {
                info!(host_id = %s.host_id, %image, "Fresh container deployed");
            }
            Ok((code, output)) => {
                warn!(host_id = %s.host_id, exit = code, out = %output.trim(), "deploy_fresh non-zero");
            }
            Err(err) => {
                warn!(%err, host_id = %s.host_id, "deploy_fresh ssh failed");
            }
        }
    }
}

#[derive(Debug)]
enum HostClass {
    AtTarget,
    AtLastStable,
    Stale,
}

fn normalize(s: &str) -> String {
    s.trim().to_string()
}

fn classify(
    image: &str,
    target: &WorkerTarget,
    last_stable: Option<&WorkerLastStable>,
) -> HostClass {
    let image = normalize(image);
    if image == normalize(&target.image_ref()) {
        return HostClass::AtTarget;
    }
    if let Some(ls) = last_stable
        && image == normalize(&ls.image_ref())
    {
        return HostClass::AtLastStable;
    }
    HostClass::Stale
}

fn last_stable_differs(ls: &Option<WorkerLastStable>, target: &WorkerTarget) -> bool {
    match ls {
        None => true,
        Some(l) => {
            l.image_tag != target.image_tag
                || l.image_registry != target.image_registry
                || l.image_repository != target.image_repository
                || l.fn0_wasmtime_version != target.fn0_wasmtime_version
        }
    }
}

pub(super) fn build_deploy_script(
    image: &str,
    envs: &BTreeMap<String, String>,
    _target: &WorkerTarget,
) -> String {
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

#[allow(dead_code)]
pub(crate) async fn legacy_unused(_pool: &SshPool, _id: &HostId) {}
