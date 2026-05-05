mod inbound_proxy;
mod podman;
mod self_dns;
mod shutdown;
mod wasmtime_version_poller;
mod worker_container_pool;

use color_eyre::eyre::Result;
use shutdown::Shutdown;
use tokio::sync::{oneshot, watch};
use tokio::task::JoinSet;
use tracing::*;
use wasmtime_version_poller::TargetWasmtimeVersion;
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

    let (target_version_tx, target_version_rx) = watch::channel::<Option<TargetWasmtimeVersion>>(None);
    let (upstream_tx, upstream_rx) = watch::channel::<Vec<UpstreamRoute>>(Vec::new());
    let (worker_first_ready_tx, worker_first_ready_rx) = oneshot::channel::<()>();

    let mut tasks: JoinSet<()> = JoinSet::new();
    tasks.spawn(wasmtime_version_poller::run(
        shutdown.clone(),
        target_version_tx,
    ));
    tasks.spawn(worker_container_pool::run(
        shutdown.clone(),
        target_version_rx,
        upstream_tx,
        worker_first_ready_tx,
    ));
    tasks.spawn(inbound_proxy::run(shutdown.clone(), upstream_rx));

    tokio::select! {
        ready = worker_first_ready_rx => {
            match ready {
                Ok(()) => info!("first worker container ready; registering self DNS"),
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

    if let Err(err) = self_dns::register().await {
        warn!(?err, "self DNS register failed; continuing anyway");
    }

    shutdown::wait_for_signal().await;
    info!("shutdown signal received");

    if let Err(err) = self_dns::deregister().await {
        warn!(?err, "self DNS deregister failed");
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
