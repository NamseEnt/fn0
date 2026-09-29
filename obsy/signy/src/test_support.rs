use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::AppState;
use crate::app_state::AppStateDependencies;
use crate::config::Config;
use crate::journal::Journal;
use crate::memtable::MemTable;
use crate::metrics::RuntimeMetrics;
use crate::object_storage::RemoteCache;
use crate::part_registry::PartRegistry;
use crate::shutdown::ShutdownState;
use crate::tenant_policy::TenantPolicy;
use crate::trace_registry::TraceRegistry;

/// Canonical state fixture used by Loki, Tempo, and ingest tests.
pub fn state(
    config: Config,
    memtable: Arc<MemTable>,
    journal: Arc<Journal>,
    parts: Arc<PartRegistry>,
    trace_parts: Arc<TraceRegistry>,
    remote_cache: Option<Arc<RemoteCache>>,
) -> Arc<AppState> {
    state_with_tenant_policy(
        config,
        memtable,
        journal,
        parts,
        trace_parts,
        remote_cache,
        Arc::new(TenantPolicy::disabled()),
    )
}

/// The same fixture with an explicit tenant policy, for retention tests.
/// The same state, with a clock the caller controls.
#[allow(clippy::too_many_arguments)]
pub fn state_with_clock(
    config: Config,
    memtable: Arc<MemTable>,
    journal: Arc<Journal>,
    parts: Arc<PartRegistry>,
    trace_parts: Arc<TraceRegistry>,
    remote_cache: Option<Arc<RemoteCache>>,
    clock: Arc<crate::clock::Clock>,
) -> Arc<AppState> {
    state_inner(
        config,
        memtable,
        journal,
        parts,
        trace_parts,
        remote_cache,
        Arc::new(crate::tenant_policy::TenantPolicy::disabled()),
        clock,
    )
}

pub fn state_with_tenant_policy(
    config: Config,
    memtable: Arc<MemTable>,
    journal: Arc<Journal>,
    parts: Arc<PartRegistry>,
    trace_parts: Arc<TraceRegistry>,
    remote_cache: Option<Arc<RemoteCache>>,
    tenant_policy: Arc<TenantPolicy>,
) -> Arc<AppState> {
    state_inner(
        config,
        memtable,
        journal,
        parts,
        trace_parts,
        remote_cache,
        tenant_policy,
        crate::clock::Clock::system(),
    )
}

#[allow(clippy::too_many_arguments)]
fn state_inner(
    config: Config,
    memtable: Arc<MemTable>,
    journal: Arc<Journal>,
    parts: Arc<PartRegistry>,
    trace_parts: Arc<TraceRegistry>,
    remote_cache: Option<Arc<RemoteCache>>,
    tenant_policy: Arc<TenantPolicy>,
    clock: Arc<crate::clock::Clock>,
) -> Arc<AppState> {
    Arc::new(AppState::from_config(
        Arc::new(config),
        AppStateDependencies {
            memtable,
            journal,
            parts,
            trace_parts,
            series_parts: Arc::new(crate::series_registry::SeriesRegistry::standalone()),
            flush_healthy: Arc::new(AtomicBool::new(true)),
            merge_healthy: Arc::new(AtomicBool::new(true)),
            retention_healthy: Arc::new(AtomicBool::new(true)),
            remote_cache,
            tenant_policy,
            metrics: Arc::new(RuntimeMetrics::new()),
            shutdown: Arc::new(ShutdownState::new()),
            clock,
            delete_requests: None,
        },
    ))
}

pub fn temp_dir(label: &str) -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "signy-{label}-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(test)]
mod tests {
    #[test]
    fn concurrent_callers_with_one_label_never_agree_on_a_path() {
        let thread_count = 16;
        let per_thread = 64;
        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                std::thread::spawn(move || {
                    (0..per_thread)
                        .map(|_| super::temp_dir("collision-probe"))
                        .collect::<Vec<_>>()
                })
            })
            .collect();

        let mut seen = std::collections::HashSet::new();
        let mut made = Vec::new();
        for handle in handles {
            for dir in handle.join().unwrap() {
                assert!(seen.insert(dir.clone()), "two callers agreed on {dir:?}");
                made.push(dir);
            }
        }
        assert_eq!(seen.len(), thread_count * per_thread);

        for dir in made {
            std::fs::remove_dir_all(dir).ok();
        }
    }
}
