use crate::podman::{Podman, RunArgs};
use crate::shutdown::Shutdown;
use crate::wasmtime_version_poller::TargetWasmtimeVersion;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, watch};
use tracing::*;

const TICK_INTERVAL: Duration = Duration::from_millis(500);
const READY_PROBE_INTERVAL: Duration = Duration::from_millis(500);
const RAMP_DURATION_DEFAULT: Duration = Duration::from_secs(30);
const DRAIN_TIMEOUT_DEFAULT: Duration = Duration::from_secs(60);
const READY_TIMEOUT: Duration = Duration::from_secs(300);
const STOP_GRACE_SECS: u32 = 5;

#[derive(Clone, Debug)]
pub struct UpstreamRoute {
    pub container_name: String,
    pub local_addr: SocketAddr,
    pub weight: u32,
}

const OPS_PORT_OFFSET: u16 = 1;

pub async fn run(
    shutdown: Shutdown,
    mut target_rx: watch::Receiver<Option<TargetWasmtimeVersion>>,
    upstream_tx: watch::Sender<Vec<UpstreamRoute>>,
    first_ready_tx: oneshot::Sender<()>,
) {
    info!("worker container pool started");
    let podman = Podman::from_env();
    let ramp_duration = duration_from_env_secs("FN0_AGENT_RAMP_DURATION_SECS", RAMP_DURATION_DEFAULT);
    let drain_timeout = duration_from_env_secs("FN0_AGENT_DRAIN_TIMEOUT_SECS", DRAIN_TIMEOUT_DEFAULT);
    let mut next_port: u16 = std::env::var("FN0_AGENT_WORKER_BASE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(18443);
    // We allocate a (user, ops) port pair per container; bump by 2 each time.
    let env_file = std::env::var("FN0_AGENT_WORKER_ENV_FILE").ok();
    let mut state = PoolState::default();
    let mut first_ready_tx = Some(first_ready_tx);

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = target_rx.changed() => {}
            _ = tokio::time::sleep(TICK_INTERVAL) => {}
        }

        if shutdown.is_cancelled() {
            break;
        }

        let target = target_rx.borrow_and_update().clone();
        if let Some(target) = target {
            let already = state
                .active
                .as_ref()
                .map(|c| c.version == target)
                .unwrap_or(false);
            if !already {
                let (user_port, ops_port) = allocate_port_pair(&mut next_port);
                match start_new_active(&podman, env_file.as_deref(), &target, user_port, ops_port)
                    .await
                {
                    Ok(new_container) => {
                        if let Some(prev) = state.active.take() {
                            info!(
                                container_name = %prev.container_name,
                                version = %prev.version.fn0_wasmtime_version,
                                "demoting previous active to draining"
                            );
                            if !signal_worker_drain(&prev.ops_addr).await {
                                warn!(
                                    container_name = %prev.container_name,
                                    "POST /drain to previous active failed; relying on instances=0 polling"
                                );
                            }
                            for d in state.draining.iter_mut() {
                                d.ramp_overlap = false;
                            }
                            state.draining.push(DrainingContainer {
                                container: prev,
                                drain_started_at: Instant::now(),
                                ramp_overlap: true,
                            });
                        }
                        state.active = Some(new_container);
                        info!(
                            container_name = %state.active.as_ref().unwrap().container_name,
                            version = %target.fn0_wasmtime_version,
                            "new active worker container ready"
                        );
                        if let Some(tx) = first_ready_tx.take() {
                            let _ = tx.send(());
                        }
                    }
                    Err(err) => {
                        warn!(
                            ?err,
                            version = %target.fn0_wasmtime_version,
                            image = %target.worker_image_ref,
                            "failed to bring up new worker container; will retry next tick"
                        );
                    }
                }
            }
        }

        let routes = compute_routes(&state, ramp_duration);
        let _ = upstream_tx.send(routes);

        let mut still_draining = Vec::with_capacity(state.draining.len());
        for d in state.draining.drain(..) {
            if drain_done(&d, ramp_duration, drain_timeout).await {
                info!(
                    container_name = %d.container.container_name,
                    "drain complete; stopping container"
                );
                if let Err(err) = podman.stop(&d.container.container_name, STOP_GRACE_SECS).await {
                    warn!(?err, container_name = %d.container.container_name, "podman stop failed");
                }
                if let Err(err) = podman.remove(&d.container.container_name).await {
                    warn!(?err, container_name = %d.container.container_name, "podman rm failed");
                }
            } else {
                still_draining.push(d);
            }
        }
        state.draining = still_draining;
    }

    info!("worker container pool: shutdown received; tearing down all containers");
    let _ = upstream_tx.send(Vec::new());
    if let Some(active) = state.active.take() {
        teardown(&podman, &active.container_name).await;
    }
    for d in state.draining.drain(..) {
        teardown(&podman, &d.container.container_name).await;
    }
    info!("worker container pool stopped");
}

#[derive(Default)]
struct PoolState {
    active: Option<RunningContainer>,
    draining: Vec<DrainingContainer>,
}

struct RunningContainer {
    container_name: String,
    version: TargetWasmtimeVersion,
    local_addr: SocketAddr,
    ops_addr: SocketAddr,
}

struct DrainingContainer {
    container: RunningContainer,
    drain_started_at: Instant,
    ramp_overlap: bool,
}

fn allocate_port_pair(next_port: &mut u16) -> (u16, u16) {
    let user_port = *next_port;
    let ops_port = user_port + OPS_PORT_OFFSET;
    *next_port = next_port.checked_add(2).unwrap_or(18443);
    (user_port, ops_port)
}

async fn start_new_active(
    podman: &Podman,
    env_file: Option<&str>,
    target: &TargetWasmtimeVersion,
    user_port: u16,
    ops_port: u16,
) -> Result<RunningContainer, PoolError> {
    let container_name = format!(
        "fn0-worker-{ver}-{port}",
        ver = sanitize(&target.fn0_wasmtime_version),
        port = user_port,
    );
    podman
        .pull_image(&target.worker_image_ref)
        .await
        .map_err(PoolError::Podman)?;
    let user_port_str = user_port.to_string();
    let ops_port_str = ops_port.to_string();
    let env: &[(&str, &str)] = &[
        ("HTTP_PORT", &user_port_str),
        ("FN0_WORKER_OPS_PORT", &ops_port_str),
    ];
    podman
        .run_detached(RunArgs {
            container_name: &container_name,
            image_ref: &target.worker_image_ref,
            env,
            env_file,
        })
        .await
        .map_err(PoolError::Podman)?;
    let local_addr: SocketAddr = format!("127.0.0.1:{user_port}")
        .parse()
        .expect("loopback addr");
    let ops_addr: SocketAddr = format!("127.0.0.1:{ops_port}")
        .parse()
        .expect("loopback addr");
    wait_until_ready(podman, &container_name, &ops_addr).await?;
    Ok(RunningContainer {
        container_name,
        version: target.clone(),
        local_addr,
        ops_addr,
    })
}

async fn wait_until_ready(
    podman: &Podman,
    container_name: &str,
    ops_addr: &SocketAddr,
) -> Result<(), PoolError> {
    let started = Instant::now();
    loop {
        if started.elapsed() > READY_TIMEOUT {
            return Err(PoolError::ReadyTimeout {
                container_name: container_name.to_string(),
            });
        }
        match podman.is_running(container_name).await {
            Ok(false) => {
                return Err(PoolError::ContainerExited {
                    container_name: container_name.to_string(),
                });
            }
            Ok(true) => {}
            Err(err) => {
                debug!(?err, %container_name, "is_running probe failed");
            }
        }
        if tcp_probe(ops_addr).await && worker_ready_ok(ops_addr).await {
            return Ok(());
        }
        tokio::time::sleep(READY_PROBE_INTERVAL).await;
    }
}

async fn tcp_probe(addr: &SocketAddr) -> bool {
    matches!(
        tokio::time::timeout(Duration::from_millis(500), tokio::net::TcpStream::connect(addr)).await,
        Ok(Ok(_))
    )
}

async fn worker_ready_ok(ops_addr: &SocketAddr) -> bool {
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
    else {
        return false;
    };
    let url = format!("http://{ops_addr}/ready");
    matches!(client.get(&url).send().await, Ok(r) if r.status().is_success())
}

async fn signal_worker_drain(ops_addr: &SocketAddr) -> bool {
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    else {
        return false;
    };
    let url = format!("http://{ops_addr}/drain");
    matches!(client.post(&url).send().await, Ok(r) if r.status().is_success())
}

fn compute_routes(state: &PoolState, ramp_duration: Duration) -> Vec<UpstreamRoute> {
    let mut routes = Vec::new();
    let now = Instant::now();
    let Some(active) = &state.active else {
        return routes;
    };
    let ramp_partner = state.draining.iter().find(|d| d.ramp_overlap);
    let new_weight = match ramp_partner {
        Some(r) => {
            let elapsed = now.duration_since(r.drain_started_at);
            ((elapsed.as_secs_f32() / ramp_duration.as_secs_f32()) * 100.0).min(100.0) as u32
        }
        None => 100,
    };
    routes.push(UpstreamRoute {
        container_name: active.container_name.clone(),
        local_addr: active.local_addr,
        weight: new_weight,
    });
    if new_weight < 100 {
        if let Some(r) = ramp_partner {
            routes.push(UpstreamRoute {
                container_name: r.container.container_name.clone(),
                local_addr: r.container.local_addr,
                weight: 100 - new_weight,
            });
        }
    }
    routes
}

async fn drain_done(
    draining: &DrainingContainer,
    ramp_duration: Duration,
    drain_timeout: Duration,
) -> bool {
    let elapsed = Instant::now().duration_since(draining.drain_started_at);
    if elapsed < ramp_duration {
        return false;
    }
    if elapsed > ramp_duration + drain_timeout {
        warn!(
            container_name = %draining.container.container_name,
            elapsed_secs = elapsed.as_secs(),
            "drain timeout exceeded; force killing"
        );
        return true;
    }
    match worker_inflight_count(&draining.container.ops_addr).await {
        Some(0) => true,
        Some(n) => {
            debug!(
                container_name = %draining.container.container_name,
                inflight = n,
                "still draining"
            );
            false
        }
        None => false,
    }
}

async fn worker_inflight_count(ops_addr: &SocketAddr) -> Option<u64> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .ok()?;
    let url = format!("http://{ops_addr}/status");
    let resp = client.get(&url).send().await.ok()?;
    let body: serde_json::Value = resp.json().await.ok()?;
    body.get("instances").and_then(|v| v.as_u64())
}

async fn teardown(podman: &Podman, container_name: &str) {
    if let Err(err) = podman.stop(container_name, STOP_GRACE_SECS).await {
        warn!(?err, %container_name, "podman stop on shutdown failed");
    }
    if let Err(err) = podman.remove(container_name).await {
        warn!(?err, %container_name, "podman rm on shutdown failed");
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn duration_from_env_secs(name: &str, default: Duration) -> Duration {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(default)
}

#[derive(Debug, thiserror::Error)]
enum PoolError {
    #[error("podman: {0}")]
    Podman(#[from] crate::podman::PodmanError),
    #[error("container {container_name} did not become ready before timeout")]
    ReadyTimeout { container_name: String },
    #[error("container {container_name} exited before becoming ready")]
    ContainerExited { container_name: String },
}
