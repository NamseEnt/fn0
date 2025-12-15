use chrono::{DateTime, Duration, Utc};
use std::env;

pub struct Context {
    pub start_time: DateTime<Utc>,
    pub domain: String,
    pub max_graceful_shutdown_wait_time: Duration,
    pub max_healthy_check_retrials: usize,
    pub max_start_timeout: Duration,
    pub max_starting_count: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            start_time: Utc::now(),
            domain: env::var("DOMAIN").expect("env var DOMAIN is not set"),
            max_graceful_shutdown_wait_time: Duration::seconds(
                env::var("MAX_GRACEFUL_SHUTDOWN_WAIT_SECS")
                    .expect("MAX_GRACEFUL_SHUTDOWN_WAIT_SECS must be set")
                    .parse::<u64>()
                    .expect("Failed to parse MAX_GRACEFUL_SHUTDOWN_WAIT_SECS")
                    as i64,
            ),
            max_healthy_check_retrials: env::var("MAX_HEALTHY_CHECK_RETRIES")
                .expect("MAX_HEALTHY_CHECK_RETRIES must be set")
                .parse::<usize>()
                .unwrap(),
            max_start_timeout: Duration::seconds(
                env::var("MAX_START_TIMEOUT_SECS")
                    .expect("MAX_START_TIMEOUT_SECS must be set")
                    .parse::<u64>()
                    .unwrap() as i64,
            ),
            max_starting_count: env::var("MAX_STARTING_COUNT")
                .expect("MAX_STARTING_COUNT must be set")
                .parse::<usize>()
                .unwrap(),
        }
    }
}
