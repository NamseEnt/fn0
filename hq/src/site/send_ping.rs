use super::*;
use crate::{random_sleep::random_sleep, telemetry, *};
use host_hq_protocol::HqToHostDatagram;
use std::time::Duration;
use tokio::time::MissedTickBehavior;

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_send_ping_loop(&self) {
        let mut interval = tokio::time::interval(send_ping_interval_ms());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let deployment_id = self.deployment_cache.last_deployment_id();
            let datagram = HqToHostDatagram::AdvertiseLatestDeploymentId { deployment_id };

            for connection in self.host_connections.iter() {
                let connection = connection.value().clone();
                let datagram = datagram.clone();
                tokio::spawn(async move {
                    random_sleep(250).await;
                    match connection.send_datagram(datagram) {
                        Ok(_) => {
                            telemetry::ping_sent_status(true);
                        }
                        Err(err) => {
                            warn!(%err, "Fail to send ping");
                            telemetry::ping_sent_status(false);
                        }
                    }
                });
            }
        }
    }
}

fn send_ping_interval_ms() -> Duration {
    match std::env::var("SEND_PING_INTERVAL_MS") {
        Ok(s) => match s.parse() {
            Ok(v) => return Duration::from_millis(v),
            Err(err) => warn!(%err, "SEND_PING_INTERVAL_MS is not a valid number"),
        },
        Err(err) => warn!(%err, "Fail to get SEND_PING_INTERVAL_MS from env"),
    }
    Duration::from_secs(2)
}
