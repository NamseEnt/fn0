//! # Rules
//!
//! 1. If no response for `MaxHealthCheckRetries` times, terminate the instance.
//! 2. Wait for `HealthCheckTimeout` for each request.
//! 3. Interval is every minute, not guaranteed. Only at least next check is after 30 seconds is guaranteed.
//! 4. Watchdog tries to make single master at once, but second master would be started if first master takes more than 30 seconds.
//!
//! # Internal Implementation
//!
//! ## Lock
//!
//! Best effort to make single master at once.
//!
//! ### DynamoDB
//!
//! - PK: `master_lock`
//! - SK: `_`
//! - Attributes:
//!   - `last_start_time`: `timestamp`
//! - Description
//!   - Read `last_start_time` and if it is older than 30 seconds, try update with optimistic locking.
//!   - If success, you are successful to get master lock. If fail, exit.
//!
//! ## Health Recorder
//!
//! Save health information of each instance
//!
//! ### S3 or Single File
//!
//! One of big file, including all instances health information, like
//! ```json
//! {
//!     "<instance_id>": {
//!         "unhealty_count": 0,
//!     }
//! }
//! ```
//!

mod dns;
mod health_recorder;
mod lock;
mod worker_infra;

use crate::{
    dns::Dns,
    health_recorder::{
        HealthRecords, HealthState, get_healthy_ips, get_workers_to_terminate,
        update_health_records,
    },
    worker_infra::{WorkerHealthResponseMap, WorkerInstanceState},
};
use chrono::{DateTime, Duration, Utc};
use futures::{FutureExt, StreamExt};
use health_recorder::HealthRecorder;
use lock::Lock;
use std::{env, sync::Arc};
use worker_infra::{WorkerInfra, oci::OciWorkerInfra};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
#[serde(transparent)]
struct WorkerId(String);

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let context = Context::new();

        let lock_at = env::var("LOCK_AT").expect("env var LOCK_AT is not set");
        let lock: Arc<dyn Lock> = match lock_at.as_str() {
            "dynamodb" => Arc::new(lock::dynamodb::DynamoDbLock::new().await),
            _ => panic!("unknown lock type {lock_at}"),
        };

        let health_recorder_at =
            env::var("HEALTH_RECORDER_AT").expect("env var HEALTH_RECORDER_AT is not set");
        let health_recorder: Arc<dyn HealthRecorder> = match health_recorder_at.as_str() {
            "s3" => Arc::new(health_recorder::s3::S3HealthRecorder::new().await),
            _ => panic!("unknown health recorder type {health_recorder_at}"),
        };

        let worker_infra_at =
            env::var("WORKER_INFRA_AT").expect("env var WORKER_INFRA_AT is not set");
        let worker_infra: Arc<dyn WorkerInfra> = match worker_infra_at.as_str() {
            "oci" => Arc::new(OciWorkerInfra::new()),
            _ => panic!("unknown worker infra type {worker_infra_at}"),
        };

        let dns_at = env::var("DNS_AT").expect("env var DNS_AT is not set");
        let dns: Arc<dyn Dns> = match dns_at.as_str() {
            "cloudflare" => Arc::new(dns::cloudflare::CloudflareDns::new().await),
            _ => panic!("unknown dns type {dns_at}"),
        };

        let _result = run_watchdog(&context, lock, health_recorder, worker_infra, dns).await;
    });
}

async fn run_watchdog(
    context: &Context,
    lock: Arc<dyn Lock>,
    health_recorder: Arc<dyn HealthRecorder>,
    worker_infra: Arc<dyn WorkerInfra>,
    dns: Arc<dyn Dns>,
) -> anyhow::Result<()> {
    if !lock.try_lock(context).await? {
        println!("Failed to get lock");
        return Ok(());
    }

    let (mut health_records, worker_health_response_map) = futures::try_join!(
        health_recorder.read_all(),
        worker_infra.get_worker_health_responses(&context.domain)
    )?;

    update_health_records(
        context,
        &mut health_records,
        worker_health_response_map.clone(),
    )?;

    let workers_to_terminate = get_workers_to_terminate(&health_records);
    let helathy_ips = get_healthy_ips(&health_records);

    futures::join!(
        health_recorder
            .write_all(health_records.clone())
            .then(|result| async {
                if let Err(err) = result {
                    eprintln!("Failed to write health records: {err}");
                }
            }),
        try_scale_out(
            context,
            health_records,
            worker_health_response_map,
            worker_infra.as_ref(),
        )
        .then(|result| async {
            if let Err(err) = result {
                eprintln!("Failed to scale out: {err}");
            }
        }),
        worker_infra.send_terminate_workers(workers_to_terminate),
        dns.sync_ips(helathy_ips).then(|result| async {
            if let Err(err) = result {
                eprintln!("Failed to sync ips: {err}");
            }
        }),
    );

    Ok(())
}

async fn try_scale_out(
    context: &Context,
    health_records: HealthRecords,
    worker_health_response_map: WorkerHealthResponseMap,
    worker_infra: &dyn WorkerInfra,
) -> anyhow::Result<()> {
    let starting_workers = worker_health_response_map
        .values()
        .filter(|(info, _status)| matches!(info.instance_state, WorkerInstanceState::Starting))
        .map(|(info, _status)| info);

    let (old_starting_workers, fresh_starting_workers): (Vec<_>, Vec<_>) = starting_workers
        .partition(|info| context.start_time - info.instance_created > context.max_start_timeout);

    let terminate_olds =
        futures::stream::iter(old_starting_workers).for_each_concurrent(16, |info| async move {
            let _ = worker_infra.terminate(&info.id).await;
        });

    let start_new = async move {
        let Some(_left_starting_count) = context
            .max_starting_count
            .checked_sub(fresh_starting_workers.len())
        else {
            return anyhow::Ok(());
        };

        if !fresh_starting_workers.is_empty() {
            return Ok(());
        }

        let helathy_count = health_records
            .iter()
            .filter(|(_, record)| matches!(record.state, HealthState::Healthy { .. }))
            .count();

        if helathy_count > 0 {
            return Ok(());
        }

        worker_infra.launch_instances(1).await?;
        Ok(())
    };

    futures::try_join!(terminate_olds.map(|_| Ok(())), start_new)?;

    Ok(())
}

struct Context {
    start_time: DateTime<Utc>,
    domain: String,
    max_graceful_shutdown_wait_time: Duration,
    max_healthy_check_retrials: usize,
    max_start_timeout: Duration,
    max_starting_count: usize,
}
impl Context {
    fn new() -> Self {
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
