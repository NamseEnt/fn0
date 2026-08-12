//! Per-host supervisor for fn0-worker containers.
//!
//! Polls doc-db for the latest worker version, provisions the new
//! worker, and gracefully shuts down the previous one.
//!
//! Each agent acts independently. There is no host-to-host coordination,
//! so when the target advances every agent races to the new version
//! simultaneously and cluster capacity briefly doubles during the swap.
//! Rolling updates across hosts will be added later.

mod agent_image_pull_poller;
mod db;
mod host_status_reporter;
mod podman;
mod proxy_target;
mod shutdown;
mod target_config;
mod worker_container_pool;

use color_eyre::eyre::{Result, eyre};
use shutdown::Shutdown;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::{oneshot, watch};
use tokio::task::JoinSet;
use tracing::*;

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

    let host_id =
        std::env::var("FN0_WORKER_AGENT_HOST_ID").expect("FN0_WORKER_AGENT_HOST_ID must be set");

    let public_ip = detect_public_ipv4()
        .await
        .expect("detect public ipv4 failed");
    let private_ip = detect_private_ipv4().expect("detect private ipv4 failed");

    let (target_image_tx, target_image_rx) = watch::channel::<Option<String>>(None);
    let (active_image_tx, active_image_rx) = watch::channel::<Option<String>>(None);
    let (active_addr_tx, active_addr_rx) = watch::channel::<Option<SocketAddr>>(None);
    let (worker_first_ready_tx, worker_first_ready_rx) = oneshot::channel::<()>();

    let mut tasks: JoinSet<()> = JoinSet::new();
    tasks.spawn(target_config::run(shutdown.clone(), target_image_tx));
    tasks.spawn(worker_container_pool::run(
        shutdown.clone(),
        target_image_rx,
        active_image_tx,
        active_addr_tx,
        worker_first_ready_tx,
        private_ip,
    ));
    tasks.spawn(host_status_reporter::run(
        shutdown.clone(),
        active_image_rx,
        host_id.clone(),
        public_ip.clone(),
    ));
    tasks.spawn(proxy_target::run(shutdown.clone(), active_addr_rx));
    tasks.spawn(agent_image_pull_poller::run(shutdown.clone()));

    tokio::select! {
        ready = worker_first_ready_rx => {
            match ready {
                Ok(()) => info!("first worker container ready"),
                Err(_) => {
                    warn!("worker container pool exited before first ready; aborting startup");
                    shutdown.trigger();
                    while tasks.join_next().await.is_some() {}
                    return Ok(());
                }
            }
        }
        _ = shutdown::wait_for_signal() => {
            info!("shutdown signal received before first worker ready");
            shutdown.trigger();
            while tasks.join_next().await.is_some() {}
            return Ok(());
        }
    }

    if let Err(err) = host_status_reporter::write_initial(&host_id, &public_ip).await {
        warn!(?err, "initial host status write failed; continuing anyway");
    }

    tokio::select! {
        _ = shutdown::wait_for_signal() => {
            let soft = std::env::var("FN0_WORKER_AGENT_SOFT_SHUTDOWN_ON_SIGTERM")
                .ok()
                .as_deref()
                == Some("1");
            info!(soft, "shutdown signal received");
            if soft {
                info!(
                    "soft shutdown: keeping WorkerHostStatusDoc + worker containers for next agent to adopt"
                );
                shutdown.trigger_soft();
            } else {
                shutdown.trigger();
            }
        }
        _ = shutdown.cancelled() => {
            info!(soft = shutdown.is_soft(), "shutdown triggered by internal task");
        }
    }

    while let Some(res) = tasks.join_next().await {
        if let Err(err) = res {
            warn!(?err, "task join failed");
        }
    }

    info!("fn0-worker-agent stopped");
    Ok(())
}

async fn detect_public_ipv4() -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let body = client
        .get("https://checkip.amazonaws.com")
        .send()
        .await?
        .text()
        .await?;
    let ip = body.trim().to_string();
    if ip.is_empty() {
        return Err(eyre!("public IP probe returned empty body"));
    }
    Ok(ip)
}

fn detect_private_ipv4() -> Result<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("1.1.1.1:80")?;
    Ok(socket.local_addr()?.ip().to_string())
}
