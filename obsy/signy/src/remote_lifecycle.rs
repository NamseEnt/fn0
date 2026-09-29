use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::metrics::RuntimeMetrics;
use crate::object_storage::RemoteCache;
use crate::runtime_error::RuntimeError;

#[derive(Clone, Copy)]
pub(crate) enum RemoteDomain {
    Logs,
    Traces,
    // Constructed by the metric read path (M14 Phase 7); the restore arm it
    // selects is live from Phase 6 so the two land decoupled.
    #[allow(dead_code)]
    Metrics,
}

/// Pin local immutable bodies for one query while preserving the shared
/// registry lifecycle protocol: plan under a read guard, leave a deliberate
/// re-plan gap for registry changes, then restore while holding a shared guard
/// so network latency never blocks lifecycle writers exclusively.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn pin_remote_parts<Required, Missing, Hook>(
    operation_lock: Arc<tokio::sync::RwLock<()>>,
    remote: Option<Arc<RemoteCache>>,
    required_parts: Required,
    missing_parts: Missing,
    domain: RemoteDomain,
    timeout: Duration,
    after_read_guard: Hook,
    metrics: Option<Arc<RuntimeMetrics>>,
) -> Result<tokio::sync::OwnedRwLockReadGuard<()>, RuntimeError>
where
    Required: Fn() -> HashSet<String>,
    Missing: Fn(&HashSet<String>) -> HashSet<String>,
    Hook: FnOnce() -> Result<(), String>,
{
    let read_guard = operation_lock.clone().read_owned().await;
    let Some(remote) = remote else {
        return Ok(read_guard);
    };
    let required = required_parts();
    if missing_parts(&required).is_empty() {
        return Ok(read_guard);
    }

    drop(read_guard);
    after_read_guard().map_err(RuntimeError::Internal)?;
    let read_guard = operation_lock.read_owned().await;
    let required = required_parts();
    let missing = missing_parts(&required);
    if !missing.is_empty() {
        let started = std::time::Instant::now();
        let result = match domain {
            RemoteDomain::Logs => {
                tokio::time::timeout(
                    timeout,
                    remote.storage.restore_parts(&remote.parts_root, &missing),
                )
                .await
            }
            RemoteDomain::Traces => {
                tokio::time::timeout(
                    timeout,
                    remote
                        .storage
                        .restore_trace_parts(&remote.trace_parts_root(), &missing),
                )
                .await
            }
            RemoteDomain::Metrics => {
                tokio::time::timeout(
                    timeout,
                    remote
                        .storage
                        .restore_metric_parts(&remote.metric_parts_root(), &missing),
                )
                .await
            }
        };
        match result {
            Ok(Ok(())) => {
                remote.record_remote_success();
                if let Some(metrics) = &metrics {
                    metrics
                        .remote_restore_success
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    RuntimeMetrics::observe(
                        &metrics.remote_restore_latency,
                        &metrics.remote_restore_latency_ns,
                        started.elapsed(),
                    );
                }
            }
            Ok(Err(error)) => {
                // A restore that fails leaves nothing behind but a counter, and
                // a counter cannot say whether the store refused or the part was
                // retired while the query was planning. Those need opposite
                // responses from an operator, so the reason is logged.
                tracing::warn!(error = %error, "object-store restore failed");
                remote.record_remote_failure();
                if let Some(metrics) = &metrics {
                    metrics
                        .remote_restore_errors
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    RuntimeMetrics::observe(
                        &metrics.remote_restore_latency,
                        &metrics.remote_restore_latency_ns,
                        started.elapsed(),
                    );
                }
                return Err(RuntimeError::Remote(error));
            }
            Err(_) => {
                remote.record_remote_failure();
                if let Some(metrics) = &metrics {
                    metrics
                        .remote_restore_errors
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    RuntimeMetrics::observe(
                        &metrics.remote_restore_latency,
                        &metrics.remote_restore_latency_ns,
                        started.elapsed(),
                    );
                }
                return Err(RuntimeError::Timeout(match domain {
                    RemoteDomain::Logs => "object store restore timed out".to_string(),
                    RemoteDomain::Traces => "trace object store restore timed out".to_string(),
                    RemoteDomain::Metrics => "metric object store restore timed out".to_string(),
                }));
            }
        }
    }
    Ok(read_guard)
}
