use anyhow::{Result, anyhow};
use doc_db::{Database, DocGet, DocKey, Document, TrxResult};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub const PARTITION_KEY: &str = "fn0-ops-canary/write-health";
pub const SORT_KEY: &str = "current";
pub const FORMAT_VERSION: u32 = 1;
pub const CADENCE_MILLIS: u64 = 5 * 60 * 1000;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct State {
    pub format_version: u32,
    pub last_write_unix_ms: u64,
    pub nonce: String,
}

impl Document for State {
    fn key(&self) -> DocKey {
        DocKey::new(PARTITION_KEY, SORT_KEY)
    }
}

pub struct StateGet;

impl DocGet for StateGet {
    type Doc = State;

    fn key(&self) -> DocKey {
        DocKey::new(PARTITION_KEY, SORT_KEY)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Success {
    pub write_performed: bool,
    pub write_age_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Failure {
    WriteFailed,
    ReadFailed,
    Mismatch,
}

#[derive(Debug)]
struct ObservedReadFailure;

impl Display for ObservedReadFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("observed DODB state read failed")
    }
}

impl std::error::Error for ObservedReadFailure {}

enum Decision {
    Cached { last_write_unix_ms: u64 },
    Written,
}

type AfterObservedHook =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

pub async fn probe(
    database: &Database,
    now_unix_ms: u64,
    nonce: String,
) -> Result<Success, Failure> {
    probe_with_hook(database, now_unix_ms, nonce, None).await
}

async fn probe_with_hook(
    database: &Database,
    now_unix_ms: u64,
    nonce: String,
    after_observed: Option<AfterObservedHook>,
) -> Result<Success, Failure> {
    let written_state = State {
        format_version: FORMAT_VERSION,
        last_write_unix_ms: now_unix_ms,
        nonce,
    };
    let transaction_result = database
        .trx(|transaction| {
            let written_state = written_state.clone();
            let after_observed = after_observed.clone();
            async move {
                let mut observed = transaction
                    .get(StateGet)
                    .await
                    .map_err(|_| anyhow::Error::new(ObservedReadFailure))?;
                if let Some(hook) = after_observed {
                    hook().await?;
                }
                match observed.as_mut() {
                    Some(current) if state_is_fresh(current, now_unix_ms) => {
                        transaction.cancel(Decision::Cached {
                            last_write_unix_ms: current.last_write_unix_ms,
                        })
                    }
                    Some(current) => {
                        **current = written_state;
                        transaction.commit(Decision::Written)
                    }
                    None => {
                        drop(transaction.create(written_state)?);
                        transaction.commit(Decision::Written)
                    }
                }
            }
        })
        .await;

    match transaction_result {
        TrxResult::Cancelled(Decision::Cached { last_write_unix_ms }) => Ok(Success {
            write_performed: false,
            write_age_seconds: now_unix_ms.saturating_sub(last_write_unix_ms) / 1000,
        }),
        TrxResult::Committed(Decision::Written) => {
            let stored = database
                .get(PARTITION_KEY, SORT_KEY)
                .await
                .map_err(|_| Failure::ReadFailed)?;
            let Some(stored) = stored else {
                return Err(Failure::Mismatch);
            };
            let stored_state: State =
                serde_json::from_slice(&stored).map_err(|_| Failure::Mismatch)?;
            if stored_state != written_state {
                return Err(Failure::Mismatch);
            }
            Ok(Success {
                write_performed: true,
                write_age_seconds: 0,
            })
        }
        TrxResult::Committed(Decision::Cached { .. })
        | TrxResult::Cancelled(Decision::Written)
        | TrxResult::Conflict(_) => Err(Failure::WriteFailed),
        TrxResult::Err(failure) => {
            if failure.downcast_ref::<ObservedReadFailure>().is_some() {
                Err(Failure::ReadFailed)
            } else {
                Err(Failure::WriteFailed)
            }
        }
    }
}

fn state_is_fresh(state: &State, now_unix_ms: u64) -> bool {
    state.format_version == FORMAT_VERSION
        && now_unix_ms >= state.last_write_unix_ms
        && now_unix_ms - state.last_write_unix_ms < CADENCE_MILLIS
}

pub fn unix_time_millis() -> Result<u64> {
    let duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    u64::try_from(duration.as_millis()).map_err(|_| anyhow!("system time exceeds u64 milliseconds"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Barrier;

    fn new_state(now_unix_ms: u64, nonce: &str) -> State {
        State {
            format_version: FORMAT_VERSION,
            last_write_unix_ms: now_unix_ms,
            nonce: nonce.to_string(),
        }
    }

    async fn row_count(database: &Database) -> usize {
        database
            .query(PARTITION_KEY, None::<&str>, 10)
            .await
            .expect("query write-health rows")
            .len()
    }

    #[tokio::test]
    async fn first_probe_commits_once_and_reads_back_exact_state() {
        let database = doc_db::memory();
        let result = probe(&database, 1_800_000_000_000, "first".to_string())
            .await
            .expect("initial write probe");

        assert_eq!(
            result,
            Success {
                write_performed: true,
                write_age_seconds: 0
            }
        );
        assert_eq!(row_count(&database).await, 1);
        assert_eq!(
            database
                .get(PARTITION_KEY, SORT_KEY)
                .await
                .unwrap()
                .unwrap()
                .as_ref(),
            serde_json::to_vec(&new_state(1_800_000_000_000, "first")).unwrap()
        );
    }

    #[tokio::test]
    async fn recent_probe_is_read_only_and_exact_five_minute_boundary_writes() {
        let database = doc_db::memory();
        let first = 1_800_000_000_000;
        probe(&database, first, "first".to_string()).await.unwrap();

        let cached = probe(&database, first + CADENCE_MILLIS - 1, "cached".to_string())
            .await
            .unwrap();
        assert_eq!(
            cached,
            Success {
                write_performed: false,
                write_age_seconds: 299
            }
        );
        assert_eq!(
            serde_json::from_slice::<State>(
                &database
                    .get(PARTITION_KEY, SORT_KEY)
                    .await
                    .unwrap()
                    .unwrap()
            )
            .unwrap(),
            new_state(first, "first")
        );

        let boundary = probe(&database, first + CADENCE_MILLIS, "boundary".to_string())
            .await
            .unwrap();
        assert_eq!(boundary.write_performed, true);
        assert_eq!(
            database
                .get(PARTITION_KEY, SORT_KEY)
                .await
                .unwrap()
                .unwrap()
                .as_ref(),
            serde_json::to_vec(&new_state(first + CADENCE_MILLIS, "boundary")).unwrap()
        );

        let overdue = probe(
            &database,
            first + CADENCE_MILLIS * 2 + 1,
            "overdue".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(overdue.write_performed, true);
        assert_eq!(row_count(&database).await, 1);
    }

    #[tokio::test]
    async fn concurrent_16_probes_commit_one_mutation_and_retry_losers() {
        assert_concurrent_probes(16).await;
    }

    #[tokio::test]
    async fn concurrent_64_probes_commit_one_mutation_and_retry_losers() {
        assert_concurrent_probes(64).await;
    }

    async fn assert_concurrent_probes(probe_count: usize) {
        let database = doc_db::memory();
        let barrier = Arc::new(Barrier::new(probe_count));
        let hook_calls = Arc::new(AtomicUsize::new(0));
        let after_observed: AfterObservedHook = {
            let barrier = Arc::clone(&barrier);
            let hook_calls = Arc::clone(&hook_calls);
            Arc::new(move || {
                let barrier = Arc::clone(&barrier);
                let attempt = hook_calls.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if attempt < probe_count {
                        barrier.wait().await;
                    }
                    Ok(())
                })
            })
        };
        let mut tasks = Vec::with_capacity(probe_count);
        for probe_index in 0..probe_count {
            let database = database.clone();
            let after_observed = Arc::clone(&after_observed);
            tasks.push(tokio::spawn(async move {
                probe_with_hook(
                    &database,
                    1_800_000_000_000,
                    format!("nonce-{probe_index}"),
                    Some(after_observed),
                )
                .await
            }));
        }
        let mut mutation_count = 0;
        for task in tasks {
            let result = task.await.expect("probe task").expect("concurrent probe");
            mutation_count += usize::from(result.write_performed);
        }

        assert_eq!(mutation_count, 1);
        assert!(hook_calls.load(Ordering::SeqCst) > probe_count);
        assert_eq!(row_count(&database).await, 1);
    }

    #[tokio::test]
    async fn fresh_row_read_error_is_not_cached_success() {
        let database = doc_db::memory();
        database
            .put(PARTITION_KEY, SORT_KEY, b"malformed-state")
            .await
            .unwrap();

        assert_eq!(
            probe(&database, 1_800_000_000_000, "replacement".to_string()).await,
            Err(Failure::ReadFailed)
        );
        assert_eq!(row_count(&database).await, 1);
    }

    #[tokio::test]
    async fn transaction_failure_is_write_failed_without_a_row() {
        let database = doc_db::memory();
        let after_observed: AfterObservedHook =
            Arc::new(|| Box::pin(async { Err(anyhow!("injected transaction failure")) }));

        assert_eq!(
            probe_with_hook(
                &database,
                1_800_000_000_000,
                "failed".to_string(),
                Some(after_observed)
            )
            .await,
            Err(Failure::WriteFailed)
        );
        assert_eq!(row_count(&database).await, 0);
    }

    #[tokio::test]
    async fn post_commit_read_error_is_read_failed() {
        let database = doc_db::memory();
        database
            .mock_get(PARTITION_KEY, SORT_KEY)
            .returns_err("injected read error");

        assert_eq!(
            probe(&database, 1_800_000_000_000, "written".to_string()).await,
            Err(Failure::ReadFailed)
        );
        assert_eq!(row_count(&database).await, 1);
    }

    #[tokio::test]
    async fn post_commit_mismatch_is_reported() {
        let database = doc_db::memory();
        database
            .mock_get(PARTITION_KEY, SORT_KEY)
            .returns(serde_json::to_vec(&new_state(1_800_000_000_000, "different")).unwrap());

        assert_eq!(
            probe(&database, 1_800_000_000_000, "written".to_string()).await,
            Err(Failure::Mismatch)
        );
        assert_eq!(row_count(&database).await, 1);
    }

    #[tokio::test]
    async fn future_and_unsupported_states_do_not_suppress_fresh_write() {
        let database = doc_db::memory();
        database
            .put(
                PARTITION_KEY,
                SORT_KEY,
                &serde_json::to_vec(&new_state(1_800_000_001_000, "future")).unwrap(),
            )
            .await
            .unwrap();
        let future = probe(&database, 1_800_000_000_000, "replace-future".to_string())
            .await
            .unwrap();
        assert!(future.write_performed);

        let mut unsupported = new_state(1_800_000_000_000, "unsupported");
        unsupported.format_version = FORMAT_VERSION + 1;
        database
            .put(
                PARTITION_KEY,
                SORT_KEY,
                &serde_json::to_vec(&unsupported).unwrap(),
            )
            .await
            .unwrap();
        let replaced = probe(&database, 1_800_000_000_001, "replace-version".to_string())
            .await
            .unwrap();
        assert!(replaced.write_performed);
        assert_eq!(row_count(&database).await, 1);
    }
}
