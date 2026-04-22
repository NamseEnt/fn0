use super::*;
use crate::doc_db::GracefulPurpose;
use crate::host_provider::HostProvide;
use crate::telemetry;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use tokio::time::MissedTickBehavior;
use tracing::warn;

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_scaler(&self) {
        let host_cpu_cores = self.host_cpu_cores;
        let host_memory_in_gb = self.host_memory_in_gb;

        let mut interval = tokio::time::interval(scale_interval_ms());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut scale_in_tick_count = 0;
        let mut last_scale_in_at: Option<Instant> = None;
        let mut last_scale_to_at: Option<Instant> = None;

        loop {
            interval.tick().await;

            let scale_config = match self.doc_db.get_scale_config(&self.name).await {
                Ok(Some(scale_config)) => {
                    telemetry::scaler_config_fetch_status(true);
                    scale_config
                }
                Ok(None) => {
                    telemetry::scaler_config_fetch_status(true);
                    warn!("No scale config found in DocDB, skipping scaling");
                    continue;
                }
                Err(err) => {
                    telemetry::scaler_config_fetch_status(false);
                    warn!(%err, "Fail to get scale config");
                    continue;
                }
            };

            let statuses = match self.doc_db.list_host_statuses(&self.name).await {
                Ok(list) => list,
                Err(err) => {
                    warn!(%err, "Scaler: failed to list host-status");
                    continue;
                }
            };

            let mut running: Vec<_> = statuses
                .into_iter()
                .filter(|s| s.healthy && !s.graceful)
                .collect();

            let instances: u64 = running.iter().map(|s| s.instances.unwrap_or(0)).sum();
            let hosts = running.len();

            telemetry::scaler_running_hosts(hosts);
            telemetry::scaler_total_instances(instances);

            let max_instances_per_host = (scale_config
                .instances_per_core
                .saturating_mul(host_cpu_cores))
            .min(
                scale_config
                    .instances_per_gb
                    .saturating_mul(host_memory_in_gb),
            );

            telemetry::scaler_max_instances_per_host(max_instances_per_host.get() as u64);

            let calculate_target = |threshold_percent: NonZeroUsize| -> usize {
                ((instances as f32 / max_instances_per_host.get() as f32 * 100.0
                    / threshold_percent.get() as f32)
                    .ceil() as usize)
                    .min(scale_config.max_hosts.get())
                    .max(scale_config.min_hosts.get())
            };

            let scale_out_target = calculate_target(scale_config.scale_out_threshold_percent);
            let scale_in_target = calculate_target(scale_config.scale_in_threshold_percent);

            telemetry::scaler_targets(scale_out_target, scale_in_target);

            if scale_in_target < hosts {
                if let Some(last_scale_in_at) = last_scale_in_at
                    && last_scale_in_at.elapsed().as_secs()
                        < scale_config.scale_in_cooldown_secs.get() as _
                {
                    continue;
                }

                scale_in_tick_count += 1;

                if scale_in_tick_count < scale_config.scale_in_threshold_ticks.get() {
                    continue;
                }

                last_scale_in_at = Some(Instant::now());

                let count = hosts - scale_in_target;
                telemetry::scaler_action_triggered("scale_in", count);

                running.sort_by_key(|s| s.instances.unwrap_or(0));

                let now = chrono::Utc::now().to_rfc3339();
                for s in running.into_iter().take(count) {
                    match self
                        .doc_db
                        .set_host_graceful(&s.host_id, GracefulPurpose::Terminate, &now)
                        .await
                    {
                        Ok(true) => {
                            telemetry::scaler_shutdown_command_status(true);
                        }
                        Ok(false) => {}
                        Err(err) => {
                            telemetry::scaler_shutdown_command_status(false);
                            warn!(%err, host_id = %s.host_id, "Failed to set graceful");
                        }
                    }
                }

                continue;
            }

            scale_in_tick_count = 0;

            if scale_out_target <= hosts {
                continue;
            }

            if let Some(last) = last_scale_to_at
                && last.elapsed().as_secs() < scale_config.scale_out_cooldown_secs.get() as _
            {
                continue;
            }

            last_scale_to_at = Some(Instant::now());

            let count = scale_out_target - hosts;
            telemetry::scaler_action_triggered("scale_out", count);

            if let Err(err) = self.host_provider.scale_to(scale_out_target).await {
                warn!(%err, "Fail to scale_to");
            }
        }
    }
}

fn scale_interval_ms() -> Duration {
    match std::env::var("SCALE_INTERVAL_MS") {
        Ok(s) => s
            .parse()
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_secs(5)),
        Err(_) => Duration::from_secs(5),
    }
}
