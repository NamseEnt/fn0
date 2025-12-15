pub mod context;
pub mod dns;
pub mod health_recorder;
pub mod host_id;
pub mod host_infra;
pub mod scaling;

pub use context::Context;
pub use host_id::HostId;

use chrono::Duration;
use dns::Dns;
use futures::StreamExt;
use health_recorder::{HealthAction, InMemoryHealthRecorder};
use host_infra::{HostHealthKind, HostHealthResponse, HostInfra, HostInfo};
use std::{collections::HashSet, net::IpAddr};

const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::seconds(2);

pub async fn run_watchdog_once(
    health_recorder: &InMemoryHealthRecorder,
    host_infra: &dyn HostInfra,
    dns: &dyn Dns,
) -> color_eyre::Result<()> {
    let context = Context::new();

    let host_infos = host_infra.get_host_infos().await?;
    let host_ids_from_infra: HashSet<HostId> = host_infos.iter().map(|h| h.id.clone()).collect();

    let all_records = health_recorder.get_all_records().await;
    for (host_id, _) in all_records.iter() {
        if !host_ids_from_infra.contains(host_id) {
            health_recorder.mark_as_invisible(&context, host_id).await;
        }
    }

    futures::stream::iter(host_infos)
        .map(|host_info| async {
            if let Err(e) = process_single_host(&context, health_recorder, host_infra, host_info).await {
                eprintln!("Error processing host: {e:?}");
            }
        })
        .buffer_unordered(16)
        .collect::<Vec<_>>()
        .await;

    tokio::try_join!(
        scaling::try_scale_out(&context, health_recorder, host_infra),
        async {
            let healthy_ips = health_recorder.get_healthy_ips().await;
            dns.sync_ips(healthy_ips).await
        },
    )?;

    health_recorder.cleanup_old_records(context.start_time).await;

    Ok(())
}

async fn process_single_host(
    context: &Context,
    health_recorder: &InMemoryHealthRecorder,
    host_infra: &dyn HostInfra,
    host_info: HostInfo,
) -> color_eyre::Result<()> {
    let health_response = check_health_single(&host_info, &context.domain).await;

    let action = health_recorder
        .update_single(context, &host_info.id, &host_info, health_response)
        .await?;

    if matches!(action, HealthAction::Terminate) {
        host_infra.terminate(&host_info.id).await?;
    }

    Ok(())
}

async fn check_health_single(
    host_info: &HostInfo,
    domain: &str,
) -> Option<HostHealthResponse> {
    let Some(ip) = host_info.ip else {
        return None;
    };

    let client = reqwest::Client::new();
    let url = format!("https://{}/health", ip);

    let response = tokio::time::timeout(
        DEFAULT_HEALTH_CHECK_TIMEOUT.to_std().unwrap(),
        client.get(&url).header("Host", domain).send(),
    )
    .await;

    match response {
        Ok(Ok(resp)) if resp.status().is_success() => {
            let body = resp.text().await.ok()?;
            Some(HostHealthResponse {
                kind: match body.trim() {
                    "good" => HostHealthKind::Good,
                    "graceful_shutting_down" => HostHealthKind::GracefulShuttingDown,
                    _ => return None,
                },
                ip,
            })
        }
        _ => None,
    }
}
