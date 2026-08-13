use crate::podman::{Podman, RunArgs};
use crate::shutdown::Shutdown;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, watch};
use tracing::*;

const TICK_INTERVAL: Duration = Duration::from_millis(500);
const READY_PROBE_INTERVAL: Duration = Duration::from_millis(500);
const DRAIN_TIMEOUT_DEFAULT: Duration = Duration::from_secs(60);
const READY_TIMEOUT: Duration = Duration::from_secs(300);
const STOP_GRACE_SECS: u32 = 5;
const ADOPTION_TARGET_WAIT: Duration = Duration::from_secs(30);
const CONTAINER_NAME_PREFIX: &str = "fn0-worker-";

const OPS_PORT_OFFSET: u16 = 1;
const QUIC_PORT_OFFSET: u16 = 2;
const WORKER_BASE_PORT: u16 = 18443;

pub async fn run(
    shutdown: Shutdown,
    mut target_rx: watch::Receiver<Option<String>>,
    active_image_tx: watch::Sender<Option<String>>,
    active_addr_tx: watch::Sender<Option<SocketAddr>>,
    first_ready_tx: oneshot::Sender<()>,
    private_ip: String,
) {
    info!("worker container pool started");
    let podman = Podman::from_env();
    let drain_timeout =
        duration_from_env_secs("FN0_WORKER_AGENT_DRAIN_TIMEOUT_SECS", DRAIN_TIMEOUT_DEFAULT);
    let mut next_port: u16 = WORKER_BASE_PORT;
    let env_file = std::env::var("FN0_WORKER_ENV_FILE").expect("FN0_WORKER_ENV_FILE must be set");
    let mut first_ready_tx = Some(first_ready_tx);

    let initial_target = wait_initial_target(&mut target_rx, ADOPTION_TARGET_WAIT).await;
    let mut state =
        adopt_existing_containers(&podman, &mut next_port, initial_target.as_deref()).await;
    if let Some(active) = state.active.as_ref() {
        let _ = active_image_tx.send(Some(active.image_ref.clone()));
        let _ = active_addr_tx.send(Some(active.local_addr));
        if let Some(tx) = first_ready_tx.take() {
            let _ = tx.send(());
        }
    }

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = target_rx.changed() => {}
            _ = tokio::time::sleep(TICK_INTERVAL) => {}
        }

        if shutdown.is_cancelled() {
            break;
        }

        if let Some(active) = state.active.as_ref()
            && matches!(podman.is_running(&active.container_name).await, Ok(false))
        {
            let dead = state.active.take().expect("checked Some above");
            warn!(
                container_name = %dead.container_name,
                image_ref = %dead.image_ref,
                "active worker container is no longer running; clearing and will restart on the same tick"
            );
            let _ = active_addr_tx.send(None);
            let _ = active_image_tx.send(None);
            if let Err(err) = podman.remove(&dead.container_name).await {
                debug!(?err, container_name = %dead.container_name, "podman rm on dead container failed");
            }
        }

        let target = target_rx.borrow_and_update().clone();
        if let Some(image_ref) = target {
            let already = state
                .active
                .as_ref()
                .map(|c| c.image_ref == image_ref)
                .unwrap_or(false);
            if !already {
                let worker_ports = allocate_ports(&mut next_port);
                match start_new_active(&podman, &env_file, &image_ref, worker_ports, &private_ip)
                    .await
                {
                    Ok(new_container) => {
                        if let Some(prev) = state.active.take() {
                            info!(
                                container_name = %prev.container_name,
                                image_ref = %prev.image_ref,
                                "demoting previous active to draining"
                            );
                            state.draining.push(DrainingContainer {
                                container: prev,
                                drain_started_at: Instant::now(),
                                drain_confirmed: false,
                            });
                        }
                        let new_addr = new_container.local_addr;
                        state.active = Some(new_container);
                        info!(
                            container_name = %state.active.as_ref().unwrap().container_name,
                            image_ref = %image_ref,
                            "new active worker container ready"
                        );
                        let _ = active_image_tx.send(Some(image_ref.clone()));
                        let _ = active_addr_tx.send(Some(new_addr));
                        if let Some(tx) = first_ready_tx.take() {
                            let _ = tx.send(());
                        }
                    }
                    Err(err) => {
                        warn!(
                            ?err,
                            image_ref = %image_ref,
                            "failed to bring up new worker container; will retry next tick"
                        );
                    }
                }
            }
        }

        let mut still_draining = Vec::with_capacity(state.draining.len());
        for mut d in state.draining.drain(..) {
            if !d.drain_confirmed && signal_worker_drain(&d.container.ops_addr).await {
                d.drain_confirmed = true;
                info!(container_name = %d.container.container_name, "/drain confirmed");
            }
            if drain_done(&d, drain_timeout).await {
                info!(
                    container_name = %d.container.container_name,
                    "drain complete; stopping container"
                );
                if let Err(err) = podman
                    .stop(&d.container.container_name, STOP_GRACE_SECS)
                    .await
                {
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

    if shutdown.is_soft() {
        info!(
            "worker container pool: soft shutdown; leaving worker containers running for the next agent to adopt (proxy keeps its target)"
        );
        info!("worker container pool stopped");
        return;
    }

    info!("worker container pool: shutdown received; tearing down all containers");
    let _ = active_addr_tx.send(None);
    let _ = active_image_tx.send(None);
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
    image_ref: String,
    local_addr: SocketAddr,
    ops_addr: SocketAddr,
}

struct DrainingContainer {
    container: RunningContainer,
    drain_started_at: Instant,
    drain_confirmed: bool,
}

#[derive(Clone, Copy)]
struct WorkerPorts {
    user: u16,
    ops: u16,
    quic: u16,
}

fn allocate_ports(next_port: &mut u16) -> WorkerPorts {
    let user_port = *next_port;
    let ops_port = user_port + OPS_PORT_OFFSET;
    let quic_port = user_port + QUIC_PORT_OFFSET;
    *next_port = next_port
        .checked_add(3)
        .expect("worker port range exhausted");
    WorkerPorts {
        user: user_port,
        ops: ops_port,
        quic: quic_port,
    }
}

async fn start_new_active(
    podman: &Podman,
    env_file: &str,
    image_ref: &str,
    worker_ports: WorkerPorts,
    private_ip: &str,
) -> Result<RunningContainer, PoolError> {
    let container_name = format!(
        "fn0-worker-{tag}-{port}",
        tag = sanitize(image_ref),
        port = worker_ports.user,
    );
    podman
        .pull_image(image_ref)
        .await
        .map_err(PoolError::Podman)?;
    let user_port_str = worker_ports.user.to_string();
    let ops_port_str = worker_ports.ops.to_string();
    let (quic_bind, quic_endpoint) = websocket_quic_env(worker_ports, private_ip);
    let env = [
        ("HTTP_PORT", user_port_str.as_str()),
        ("FN0_WORKER_OPS_PORT", ops_port_str.as_str()),
        ("FN0_WEBSOCKET_QUIC_BIND", quic_bind.as_str()),
        ("FN0_WEBSOCKET_QUIC_ENDPOINT", quic_endpoint.as_str()),
    ];
    podman
        .run_detached(RunArgs {
            container_name: &container_name,
            image_ref,
            env: &env,
            env_file,
        })
        .await
        .map_err(PoolError::Podman)?;
    let local_addr: SocketAddr = format!("127.0.0.1:{}", worker_ports.user)
        .parse()
        .expect("loopback addr");
    let ops_addr: SocketAddr = format!("127.0.0.1:{}", worker_ports.ops)
        .parse()
        .expect("loopback addr");
    wait_until_ready(podman, &container_name, &ops_addr).await?;
    Ok(RunningContainer {
        container_name,
        image_ref: image_ref.to_string(),
        local_addr,
        ops_addr,
    })
}

fn websocket_quic_env(worker_ports: WorkerPorts, private_ip: &str) -> (String, String) {
    (
        format!("0.0.0.0:{}", worker_ports.quic),
        format!("{private_ip}:{}", worker_ports.quic),
    )
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
        tokio::time::timeout(
            Duration::from_millis(500),
            tokio::net::TcpStream::connect(addr)
        )
        .await,
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

async fn drain_done(draining: &DrainingContainer, drain_timeout: Duration) -> bool {
    let elapsed = Instant::now().duration_since(draining.drain_started_at);
    if elapsed > drain_timeout {
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

async fn wait_initial_target(
    target_rx: &mut watch::Receiver<Option<String>>,
    timeout: Duration,
) -> Option<String> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(t) = target_rx.borrow().clone() {
            return Some(t);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        if tokio::time::timeout(remaining, target_rx.changed())
            .await
            .is_err()
        {
            return None;
        }
    }
}

async fn adopt_existing_containers(
    podman: &Podman,
    next_port: &mut u16,
    target: Option<&str>,
) -> PoolState {
    let snapshots = match podman.list_with_name_prefix(CONTAINER_NAME_PREFIX).await {
        Ok(s) => s,
        Err(err) => {
            warn!(?err, "podman list for adoption failed; starting fresh");
            return PoolState::default();
        }
    };
    if snapshots.is_empty() {
        return PoolState::default();
    }

    let mut candidates: Vec<RunningContainer> = Vec::new();
    let mut max_port: u16 = 0;
    for snap in &snapshots {
        let Some(name) = snap.primary_name() else {
            continue;
        };
        if !snap.is_running() {
            warn!(
                container_name = %name,
                state = %snap.state,
                "found non-running container with worker prefix; leaving alone (no adoption, no teardown)"
            );
            continue;
        }
        let Some(user_port) = parse_port_from_container_name(name) else {
            warn!(container_name = %name, "could not parse port from container name; skipping adoption");
            continue;
        };
        let ops_port = user_port + OPS_PORT_OFFSET;
        let local_addr: SocketAddr = format!("127.0.0.1:{user_port}")
            .parse()
            .expect("loopback addr");
        let ops_addr: SocketAddr = format!("127.0.0.1:{ops_port}")
            .parse()
            .expect("loopback addr");
        if !worker_ready_ok(&ops_addr).await {
            warn!(
                container_name = %name,
                "adopted candidate did not respond on /ready; skipping (will leave running)"
            );
            continue;
        }
        if user_port > max_port {
            max_port = user_port;
        }
        candidates.push(RunningContainer {
            container_name: name.to_string(),
            image_ref: snap.image_ref.clone(),
            local_addr,
            ops_addr,
        });
    }

    if max_port >= WORKER_BASE_PORT {
        *next_port = max_port.saturating_add(3);
    }

    if candidates.is_empty() {
        return PoolState::default();
    }

    let (active_pool, drain_pool): (Vec<_>, Vec<_>) = match target {
        Some(t) => {
            let (matching, non_matching): (Vec<_>, Vec<_>) =
                candidates.into_iter().partition(|c| c.image_ref == t);
            if matching.is_empty() {
                (non_matching, Vec::new())
            } else {
                (matching, non_matching)
            }
        }
        None => {
            warn!(
                "adopted containers exist but target unknown; promoting highest-port as active without drain signal"
            );
            (candidates, Vec::new())
        }
    };

    let mut state = PoolState::default();
    let (active, extra) = pick_highest_port(active_pool);
    if let Some(active) = active {
        info!(
            container_name = %active.container_name,
            image_ref = %active.image_ref,
            "adopted as active"
        );
        state.active = Some(active);
    }
    for c in extra {
        info!(
            container_name = %c.container_name,
            "duplicate active-pool container demoted to draining during adoption"
        );
        state.draining.push(DrainingContainer {
            container: c,
            drain_started_at: Instant::now(),
            drain_confirmed: false,
        });
    }
    for c in drain_pool {
        info!(
            container_name = %c.container_name,
            image_ref = %c.image_ref,
            "adopted as draining (image differs from target)"
        );
        state.draining.push(DrainingContainer {
            container: c,
            drain_started_at: Instant::now(),
            drain_confirmed: false,
        });
    }
    state
}

fn pick_highest_port(
    mut candidates: Vec<RunningContainer>,
) -> (Option<RunningContainer>, Vec<RunningContainer>) {
    candidates.sort_by_key(|c| parse_port_from_container_name(&c.container_name).unwrap_or(0));
    let active = candidates.pop();
    (active, candidates)
}

fn parse_port_from_container_name(name: &str) -> Option<u16> {
    name.rsplit_once('-')
        .and_then(|(_, p)| p.parse::<u16>().ok())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_container_quic_env_uses_worker_port_and_private_ip() {
        let worker_ports = WorkerPorts {
            user: 18443,
            ops: 18444,
            quic: 18445,
        };
        let (quic_bind, quic_endpoint) = websocket_quic_env(worker_ports, "127.0.0.2");
        assert_eq!(quic_bind, "0.0.0.0:18445");
        assert_eq!(quic_endpoint, "127.0.0.2:18445");
    }
}
