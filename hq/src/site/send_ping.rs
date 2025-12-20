use super::*;
use crate::{random_sleep::random_sleep, telemetry, *};
use std::time::Duration;
use tokio::time::MissedTickBehavior;

impl Site {
    #[tracing::instrument(skip_all)]
    pub async fn run_send_ping_loop(&self) {
        let mut interval = tokio::time::interval(send_ping_interval_ms());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let deployment_id = self.deployment_db.last_deployment_id();
            let bytes = Bytes::copy_from_slice(&deployment_id.to_le_bytes());

            for connection in self.host_connections.iter() {
                let connection = connection.value().clone();
                let bytes = bytes.clone();
                tokio::spawn(async move {
                    random_sleep(250).await;
                    match connection.send_unreliable_small_message(bytes) {
                        Ok(_) => {
                            telemetry::send_ping_sent_status(true);
                        }
                        Err(err) => {
                            warn!(%err, "Fail to send ping");
                            telemetry::send_ping_sent_status(false);
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
