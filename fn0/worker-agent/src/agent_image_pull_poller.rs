//! Agent self-update via OCIR pull.
//!
//! Periodically `podman pull`s the agent's own image ref and compares the
//! local digest against the one observed at boot. On change it triggers a
//! soft shutdown; systemd `Restart=always` then re-pulls and starts the new
//! digest, worker containers surviving for the next agent to adopt.

use crate::podman::Podman;
use crate::shutdown::Shutdown;
use std::time::Duration;
use tracing::*;

const POLL_INTERVAL_DEFAULT: Duration = Duration::from_secs(60);

pub async fn run(shutdown: Shutdown) {
    let image_ref = match std::env::var("FN0_WORKER_AGENT_IMAGE_REF") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            warn!(
                "FN0_WORKER_AGENT_IMAGE_REF not set; agent image pull poller disabled"
            );
            return;
        }
    };
    let interval = duration_from_env_secs(
        "FN0_WORKER_AGENT_PULL_INTERVAL_SECS",
        POLL_INTERVAL_DEFAULT,
    );
    let podman = Podman::from_env();

    let boot_digest = match podman.image_digest(&image_ref).await {
        Ok(d) => d,
        Err(err) => {
            error!(
                ?err,
                %image_ref,
                event = "boot_digest_unavailable",
                "failed to read boot image digest; pull poller disabled for this run"
            );
            return;
        }
    };
    info!(
        %image_ref,
        %boot_digest,
        interval_secs = interval.as_secs(),
        "agent image pull poller started"
    );

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(interval) => {}
        }
        if shutdown.is_cancelled() {
            break;
        }

        if let Err(err) = podman.pull_image(&image_ref).await {
            error!(
                ?err,
                %image_ref,
                event = "ocir_pull_failed",
                "OCIR pull failed; will retry next tick"
            );
            continue;
        }

        let new_digest = match podman.image_digest(&image_ref).await {
            Ok(d) => d,
            Err(err) => {
                error!(
                    ?err,
                    %image_ref,
                    event = "image_inspect_failed",
                    "podman inspect after pull failed; will retry next tick"
                );
                continue;
            }
        };

        if new_digest != boot_digest {
            info!(
                %image_ref,
                %boot_digest,
                %new_digest,
                "agent image digest changed; triggering soft restart"
            );
            shutdown.trigger_soft();
            return;
        }
    }
    info!("agent image pull poller stopped");
}

fn duration_from_env_secs(name: &str, default: Duration) -> Duration {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(default)
}
