use crate::shutdown::Shutdown;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::watch;
use tracing::*;

const POLL_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetWasmtimeVersion {
    pub fn0_wasmtime_version: String,
    pub worker_image_ref: String,
}

pub async fn run(
    shutdown: Shutdown,
    target_tx: watch::Sender<Option<TargetWasmtimeVersion>>,
) {
    if let Some(initial) = TargetWasmtimeVersion::from_env_initial() {
        info!(
            fn0_wasmtime_version = %initial.fn0_wasmtime_version,
            worker_image_ref = %initial.worker_image_ref,
            "publishing initial target wasmtime version (env-derived)"
        );
        let _ = target_tx.send(Some(initial));
    } else {
        warn!(
            "FN0_AGENT_INITIAL_WASMTIME_VERSION / FN0_AGENT_INITIAL_WORKER_IMAGE_REF not set; \
             worker_container_pool will idle until target is published"
        );
    }

    info!("wasmtime version poller started (TODO: read Fn0WasmtimeVersionDoc + WorkerTargetDoc from doc-db)");
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
        }
    }
    info!("wasmtime version poller stopped");
}

impl TargetWasmtimeVersion {
    fn from_env_initial() -> Option<Self> {
        let fn0_wasmtime_version = std::env::var("FN0_AGENT_INITIAL_WASMTIME_VERSION").ok()?;
        let worker_image_ref = std::env::var("FN0_AGENT_INITIAL_WORKER_IMAGE_REF").ok()?;
        Some(Self {
            fn0_wasmtime_version,
            worker_image_ref,
        })
    }
}
