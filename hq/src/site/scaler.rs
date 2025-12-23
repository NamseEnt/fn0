use super::*;
use crate::{random_sleep::random_sleep, telemetry, *};
use host_hq_protocol::HqToHostReliable;
use std::time::Duration;
use tokio::time::MissedTickBehavior;

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_scaler(&self) {
        let mut interval = tokio::time::interval(scale_interval_ms());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut scale_in_tick_count = 0;
        let mut last_scale_out_at: Option<Instant> = None;
        let mut last_scale_in_at: Option<Instant> = None;

        loop {
            interval.tick().await;

            let scale_config = match self.doc_db.get_scale_config().await {
                Ok(Some(scale_config)) => {
                    telemetry::scaler_config_fetch_status(true);
                    scale_config
                }
                Ok(None) => {
                    telemetry::scaler_config_fetch_status(false);
                    error!("Scale config not found");
                    continue;
                }
                Err(err) => {
                    telemetry::scaler_config_fetch_status(false);
                    warn!(%err, "Fail to get scale config");
                    continue;
                }
            };

            let mut running_hosts = self
                .hosts_status
                .iter()
                .filter(|status| {
                    !self.graceful_shutdown_hosts.contains_key(status.key())
                        && !self.dead_hosts.contains_key(status.key())
                })
                .collect::<Vec<_>>();

            let instances: u64 = running_hosts.iter().map(|status| status.instances).sum();
            let hosts = running_hosts.len();

            telemetry::scaler_running_hosts(hosts);
            telemetry::scaler_total_instances(instances);

            let max_instances_per_host = (scale_config
                .instances_per_core
                .saturating_mul(self.host_cpu_cores))
            .min(
                scale_config
                    .instances_per_gb
                    .saturating_mul(self.host_memory_in_gb),
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

                running_hosts.sort_by_key(|h| h.instances);

                for host in running_hosts.into_iter().take(count) {
                    self.graceful_shutdown_hosts
                        .insert(host.key().clone(), Instant::now());

                    if let Some((_host, connection)) = self.host_connections.remove(host.key()) {
                        tokio::spawn(async move {
                            let result = connection
                                .send_reliable(HqToHostReliable::GracefulShutdown)
                                .await;

                            telemetry::scaler_shutdown_command_status(result.is_ok());

                            if let Err(err) = result {
                                warn!(%err, "Fail to send graceful shutdown");
                            };
                        });
                    }
                }

                continue;
            }

            scale_in_tick_count = 0;

            if hosts < scale_out_target {
                if let Some(last_scale_out_at) = last_scale_out_at
                    && last_scale_out_at.elapsed().as_secs()
                        < scale_config.scale_out_cooldown_secs.get() as _
                {
                    continue;
                }

                last_scale_out_at = Some(Instant::now());

                let count = scale_out_target - hosts;

                telemetry::scaler_action_triggered("scale_out", count);

                for _ in 0..count {
                    let host_provider = self.host_provider.clone();
                    tokio::spawn(async move {
                        random_sleep(1000).await;
                        let result = host_provider.launch_instance().await;
                        telemetry::scaler_launch_attempt_status(result.is_ok());

                        if let Err(err) = result {
                            warn!(%err, "Fail to scale out");
                        };
                    });
                }
            }
        }
    }
}

fn scale_interval_ms() -> Duration {
    match std::env::var("SCALE_INTERVAL_MS") {
        Ok(s) => match s.parse() {
            Ok(v) => return Duration::from_millis(v),
            Err(err) => warn!(%err, "SCALE_INTERVAL_MS is not a valid number"),
        },
        Err(err) => warn!(%err, "Fail to get SCALE_INTERVAL_MS from env"),
    }
    Duration::from_secs(5)
}
