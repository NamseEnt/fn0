use crate::podman::{Podman, RunArgs};
use crate::shutdown::Shutdown;
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
const ADOPTION_TARGET_WAIT: Duration = Duration::from_secs(30);
const CONTAINER_NAME_PREFIX: &str = "fn0-worker-";

#[derive(Clone, Debug)]
pub struct UpstreamRoute {
    pub container_name: String,
    pub local_addr: SocketAddr,
    pub weight: u32,
}

const OPS_PORT_OFFSET: u16 = 1;
const WORKER_BASE_PORT: u16 = 18443;

pub async fn run(
    shutdown: Shutdown,
    mut target_rx: watch::Receiver<Option<String>>,
    active_image_tx: watch::Sender<Option<String>>,
    upstream_tx: watch::Sender<Vec<UpstreamRoute>>,
    first_ready_tx: oneshot::Sender<()>,
) {
    info!("worker container pool started");
    let podman = Podman::from_env();
    let ramp_duration = duration_from_env_secs("FN0_WORKER_AGENT_RAMP_DURATION_SECS", RAMP_DURATION_DEFAULT);
    let drain_timeout = duration_from_env_secs("FN0_WORKER_AGENT_DRAIN_TIMEOUT_SECS", DRAIN_TIMEOUT_DEFAULT);
    let mut next_port: u16 = WORKER_BASE_PORT;
    let env_file = std::env::var("FN0_WORKER_ENV_FILE")
        .expect("FN0_WORKER_ENV_FILE must be set");
    let mut first_ready_tx = Some(first_ready_tx);

    // === Adoption ===
    // Recover state from pre-existing podman containers (left by a previous
    // agent process: crash, soft-shutdown restart, or image replace). Without
    // this the new agent would start a duplicate worker and ignore the live
    // ones until manual cleanup.
    let initial_target = wait_initial_target(&mut target_rx, ADOPTION_TARGET_WAIT).await;
    let mut state =
        adopt_existing_containers(&podman, &mut next_port, initial_target.as_deref()).await;
    if let Some(active) = state.active.as_ref() {
        let _ = active_image_tx.send(Some(active.image_ref.clone()));
        if let Some(tx) = first_ready_tx.take() {
            let _ = tx.send(());
        }
    }
    let routes = compute_routes(&state, ramp_duration);
    let _ = upstream_tx.send(routes);

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
        if let Some(image_ref) = target {
            let already = state
                .active
                .as_ref()
                .map(|c| c.image_ref == image_ref)
                .unwrap_or(false);
            if !already {
                let (user_port, ops_port) = allocate_port_pair(&mut next_port);
                match start_new_active(
                    &podman,
                    &env_file,
                    &image_ref,
                    user_port,
                    ops_port,
                )
                .await
                {
                    Ok(new_container) => {
                        if let Some(prev) = state.active.take() {
                            info!(
                                container_name = %prev.container_name,
                                image_ref = %prev.image_ref,
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
                            image_ref = %image_ref,
                            "new active worker container ready"
                        );
                        let _ = active_image_tx.send(Some(image_ref.clone()));
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

    if shutdown.is_soft() {
        info!(
            "worker container pool: soft shutdown; leaving worker containers running for the next agent to adopt"
        );
        let _ = upstream_tx.send(Vec::new());
        info!("worker container pool stopped");
        return;
    }

    info!("worker container pool: shutdown received; tearing down all containers");
    let _ = upstream_tx.send(Vec::new());
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
    ramp_overlap: bool,
}

fn allocate_port_pair(next_port: &mut u16) -> (u16, u16) {
    let user_port = *next_port;
    let ops_port = user_port + OPS_PORT_OFFSET;
    *next_port = next_port
        .checked_add(2)
        .expect("worker port range exhausted");
    (user_port, ops_port)
}

async fn start_new_active(
    podman: &Podman,
    env_file: &str,
    image_ref: &str,
    user_port: u16,
    ops_port: u16,
) -> Result<RunningContainer, PoolError> {
    let container_name = format!(
        "fn0-worker-{tag}-{port}",
        tag = sanitize(image_ref),
        port = user_port,
    );
    podman
        .pull_image(image_ref)
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
            image_ref,
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
        image_ref: image_ref.to_string(),
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
    if new_weight < 100
        && let Some(r) = ramp_partner
    {
        routes.push(UpstreamRoute {
            container_name: r.container.container_name.clone(),
            local_addr: r.container.local_addr,
            weight: 100 - new_weight,
        });
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

    // Advance next_port past any adopted container so subsequent allocations don't collide.
    if max_port >= WORKER_BASE_PORT {
        *next_port = max_port.saturating_add(2);
    }

    if candidates.is_empty() {
        return PoolState::default();
    }

    // Decide which adopted container is "current active".
    //
    // - Target known: containers matching target are active candidates; the rest get
    //   /drain signaled (image differs from target = obsolete from a prior rollout).
    // - Target unknown: there's no truth to compare against, so adopt all without
    //   /drain signal and let the main loop reconcile when target arrives.
    // - No matching candidate: still promote the highest-port non-matching one as
    //   pseudo-active so traffic keeps flowing; the main tick will then run the
    //   normal blue-green replacement.
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
            ramp_overlap: false,
        });
    }
    for c in drain_pool {
        if !signal_worker_drain(&c.ops_addr).await {
            warn!(container_name = %c.container_name, "drain signal failed during adoption");
        }
        info!(
            container_name = %c.container_name,
            image_ref = %c.image_ref,
            "adopted as draining (image differs from target)"
        );
        state.draining.push(DrainingContainer {
            container: c,
            drain_started_at: Instant::now(),
            ramp_overlap: false,
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
    name.rsplit_once('-').and_then(|(_, p)| p.parse::<u16>().ok())
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
