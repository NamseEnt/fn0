//! Per-host supervisor for fn0-worker containers.
//!
//! Polls doc-db for the latest worker version, provisions the new
//! worker, and gracefully shuts down the previous one.
//!
//! Each agent acts independently. There is no host-to-host coordination,
//! so when the target advances every agent races to the new version
//! simultaneously and cluster capacity briefly doubles during the swap.
//! Rolling updates across hosts will be added later.

mod heartbeat;
mod host_status_reporter;
mod inbound_proxy;
mod podman;
mod dns_register;
mod shutdown;
mod target_config;
mod worker_container_pool;

use color_eyre::eyre::Result;
use shutdown::Shutdown;
use tokio::sync::{oneshot, watch};
use tokio::task::JoinSet;
use tracing::*;
use worker_container_pool::UpstreamRoute;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fn0_worker_agent=info,info".parse().unwrap()),
        )
        .init();

    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main())
}

async fn async_main() -> Result<()> {
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "fn0-worker-agent starting"
    );

    let shutdown = Shutdown::new();

    let host_id = std::env::var("FN0_WORKER_AGENT_HOST_ID")
        .expect("FN0_WORKER_AGENT_HOST_ID must be set");

    let public_ip = dns_register::detect_public_ipv4()
        .await
        .expect("detect public ipv4 failed");

    let (target_image_tx, target_image_rx) = watch::channel::<Option<String>>(None);
    let (active_image_tx, active_image_rx) = watch::channel::<Option<String>>(None);
    let (upstream_tx, upstream_rx) = watch::channel::<Vec<UpstreamRoute>>(Vec::new());
    let (worker_first_ready_tx, worker_first_ready_rx) = oneshot::channel::<()>();

    let mut tasks: JoinSet<()> = JoinSet::new();
    tasks.spawn(target_config::run(shutdown.clone(), target_image_tx));
    tasks.spawn(worker_container_pool::run(
        shutdown.clone(),
        target_image_rx,
        active_image_tx,
        upstream_tx,
        worker_first_ready_tx,
    ));
    tasks.spawn(host_status_reporter::run(
        shutdown.clone(),
        active_image_rx,
        host_id.clone(),
    ));
    tasks.spawn(heartbeat::run(
        shutdown.clone(),
        host_id.clone(),
        public_ip.clone(),
    ));
    tasks.spawn(inbound_proxy::run(shutdown.clone(), upstream_rx));

    tokio::select! {
        ready = worker_first_ready_rx => {
            match ready {
                Ok(()) => info!("first worker container ready; registering DNS"),
                Err(_) => {
                    warn!("worker container pool exited before first ready; aborting startup");
                    shutdown.trigger();
                    while tasks.join_next().await.is_some() {}
                    return Ok(());
                }
            }
        }
        _ = shutdown::wait_for_signal() => {
            info!("shutdown signal received before first worker ready; skipping DNS register");
            shutdown.trigger();
            while tasks.join_next().await.is_some() {}
            return Ok(());
        }
    }

    if let Err(err) = heartbeat::write_initial(&host_id, &public_ip).await {
        warn!(?err, "initial heartbeat write failed; continuing anyway");
    }

    if let Err(err) = dns_register::register().await {
        warn!(?err, "DNS register failed; continuing anyway");
    }

    shutdown::wait_for_signal().await;
    info!("shutdown signal received");

    if let Err(err) = dns_register::deregister().await {
        warn!(?err, "DNS deregister failed");
    }

    shutdown.trigger();

    while let Some(res) = tasks.join_next().await {
        if let Err(err) = res {
            warn!(?err, "task join failed");
        }
    }

    info!("fn0-worker-agent stopped");
    Ok(())
}
