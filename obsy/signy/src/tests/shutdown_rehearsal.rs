// M6 machine-replacement rehearsal. Reuses the e2e helpers `tmp_data_dir`,
// `build_push_req`, and `ingest_once` from the shared tests module.

use crate::shutdown::{
    FinalizeContext, ShutdownOutcome, ShutdownState, finalize_flush, finalize_flush_with_abort,
};
use std::collections::HashSet;

/// A fault-injecting object store used to exercise the shutdown force-flush
/// retry and operator-abort paths. It wraps an in-memory backend and fails
/// `put_opts` either while `always_fail` is set or for the first
/// `fail_remaining` calls, delegating every other operation.
mod faulty_store {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use async_trait::async_trait;
    use futures_util::stream::BoxStream;
    use object_store::path::Path;
    use object_store::{
        GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore,
        PutMultipartOptions, PutOptions, PutPayload, PutResult,
    };

    pub struct FaultyHandle {
        pub always_fail: Arc<AtomicBool>,
        pub fail_remaining: Arc<AtomicUsize>,
    }

    #[derive(Debug)]
    pub struct FaultyStore {
        inner: Arc<dyn ObjectStore>,
        always_fail: Arc<AtomicBool>,
        fail_remaining: Arc<AtomicUsize>,
    }

    /// Build a faulty store plus a handle that toggles its failure behavior.
    pub fn new() -> (Arc<dyn ObjectStore>, FaultyHandle) {
        let always_fail = Arc::new(AtomicBool::new(false));
        let fail_remaining = Arc::new(AtomicUsize::new(0));
        let store = FaultyStore {
            inner: Arc::new(object_store::memory::InMemory::new()),
            always_fail: always_fail.clone(),
            fail_remaining: fail_remaining.clone(),
        };
        (
            Arc::new(store),
            FaultyHandle {
                always_fail,
                fail_remaining,
            },
        )
    }

    impl FaultyStore {
        fn should_fail(&self) -> bool {
            if self.always_fail.load(Ordering::Acquire) {
                return true;
            }
            self.fail_remaining
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |remaining| {
                    if remaining > 0 {
                        Some(remaining - 1)
                    } else {
                        None
                    }
                })
                .is_ok()
        }
    }

    impl std::fmt::Display for FaultyStore {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "FaultyStore({})", self.inner)
        }
    }

    #[async_trait]
    impl ObjectStore for FaultyStore {
        async fn put_opts(
            &self,
            location: &Path,
            payload: PutPayload,
            opts: PutOptions,
        ) -> object_store::Result<PutResult> {
            if self.should_fail() {
                return Err(object_store::Error::Generic {
                    store: "faulty",
                    source: "injected object-store failure".into(),
                });
            }
            self.inner.put_opts(location, payload, opts).await
        }

        async fn put_multipart_opts(
            &self,
            location: &Path,
            opts: PutMultipartOptions,
        ) -> object_store::Result<Box<dyn MultipartUpload>> {
            self.inner.put_multipart_opts(location, opts).await
        }

        async fn get_opts(
            &self,
            location: &Path,
            options: GetOptions,
        ) -> object_store::Result<GetResult> {
            self.inner.get_opts(location, options).await
        }

        async fn delete(&self, location: &Path) -> object_store::Result<()> {
            self.inner.delete(location).await
        }

        fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, object_store::Result<ObjectMeta>> {
            self.inner.list(prefix)
        }

        async fn list_with_delimiter(
            &self,
            prefix: Option<&Path>,
        ) -> object_store::Result<ListResult> {
            self.inner.list_with_delimiter(prefix).await
        }

        async fn copy(&self, from: &Path, to: &Path) -> object_store::Result<()> {
            self.inner.copy(from, to).await
        }

        async fn copy_if_not_exists(&self, from: &Path, to: &Path) -> object_store::Result<()> {
            self.inner.copy_if_not_exists(from, to).await
        }
    }
}

fn file_object_store(dir: &std::path::Path) -> Arc<crate::object_storage::ObjectStorage> {
    let url = url::Url::from_directory_path(dir).unwrap();
    Arc::new(crate::object_storage::ObjectStorage::from_url(url.as_str()).unwrap())
}

async fn ingest_trace(journal: &Journal, marker: u8) -> String {
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use prost014::Message;

    let request = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: None,
            scope_spans: vec![ScopeSpans {
                scope: None,
                spans: vec![Span {
                    trace_id: vec![marker; 16],
                    span_id: vec![marker; 8],
                    start_time_unix_nano: 10,
                    end_time_unix_nano: 20,
                    ..Default::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };
    let spans = crate::trace::normalize_request(&test_tenant(), request.clone()).unwrap();
    let mut encoded = Vec::new();
    request.encode(&mut encoded).unwrap();
    journal.append_trace(test_tenant(), encoded, spans).await.unwrap();
    (0..16).map(|_| format!("{marker:02x}")).collect()
}

#[tokio::test]
async fn m6_draining_rejects_new_ingest_and_readiness() {
    let dir = tmp_data_dir("m6-gate");
    let config = Config {
        data_dir: dir.clone(),
        ..Config::default()
    };
    let memtable = Arc::new(MemTable::new());
    let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
    let parts = Arc::new(PartRegistry::new());
    let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));
    let state = crate::test_support::state(config, memtable, journal, parts, trace_parts, None);

    assert!(
        crate::query::ready(axum::extract::State(state.clone()))
            .await
            .is_ok(),
        "readiness is healthy before draining"
    );

    state.shutdown.begin_drain();

    let ready_error = crate::query::ready(axum::extract::State(state.clone()))
        .await
        .unwrap_err();
    assert_eq!(ready_error.0, axum::http::StatusCode::SERVICE_UNAVAILABLE);

    let collect_error = crate::collect::collect(
        axum::extract::State(state.clone()),
        axum::http::HeaderMap::new(),
        axum::body::Body::empty(),
    )
    .await
    .unwrap_err();
    assert_eq!(
        collect_error.status,
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
async fn m6_machine_replacement_force_flush_is_lossless() {
    // Original machine: ingest acknowledged log and trace data, then run the
    // shutdown force-flush against a file-backed object store.
    let remote = tmp_data_dir("m6-remote");
    let data_a = tmp_data_dir("m6-a");
    let config_a = Arc::new(Config {
        data_dir: data_a.clone(),
        ..Config::default()
    });
    std::fs::create_dir_all(data_a.join("parts")).unwrap();
    std::fs::create_dir_all(data_a.join("traces")).unwrap();

    let memtable = Arc::new(MemTable::new());
    let trace_memtable = Arc::new(crate::trace::TraceMemTable::new());
    let journal = Arc::new(
        Journal::spawn_with_traces(&config_a, memtable.clone(), trace_memtable.clone()).unwrap(),
    );

    ingest_once(&journal, &build_push_req()).await;
    let trace_id = ingest_trace(&journal, 7).await;
    assert!(!memtable.is_empty());
    assert!(!trace_memtable.is_empty());

    let parts = Arc::new(PartRegistry::new());
    let trace_registry = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));
    let remote_cache = Some(Arc::new(crate::object_storage::RemoteCache::new(
        file_object_store(&remote),
        data_a.join("parts"),
    )));

    let shutdown = Arc::new(ShutdownState::new());
    shutdown.begin_drain();

    finalize_flush(FinalizeContext {
        shutdown: shutdown.clone(),
        memtable: memtable.clone(),
        trace_memtable: trace_memtable.clone(),
        journal: journal.clone(),
        registry: parts.clone(),
        trace_registry: trace_registry.clone(),
        series_registry: Arc::new(crate::series_registry::SeriesRegistry::standalone()),
        remote_cache: remote_cache.clone(),
        config: config_a.clone(),
    })
    .await;

    assert!(shutdown.is_flush_complete());
    assert_eq!(shutdown.pending_flush_bytes(), 0);
    assert!(memtable.is_empty(), "force-flush drained the log memtable");
    assert!(
        trace_memtable.is_empty(),
        "force-flush drained the trace memtable"
    );

    // Discard the original machine, including its disk.
    drop(journal);
    drop(memtable);
    drop(trace_memtable);
    drop(parts);
    drop(trace_registry);
    drop(remote_cache);

    // Replacement machine: a fresh empty disk pointed at the same object store.
    let data_b = tmp_data_dir("m6-b");
    let parts_root_b = data_b.join("parts");
    let traces_root_b = data_b.join("traces");
    std::fs::create_dir_all(&parts_root_b).unwrap();
    std::fs::create_dir_all(&traces_root_b).unwrap();
    let storage_b = file_object_store(&remote);

    let log_manifest = storage_b.reconcile_local_cache(&parts_root_b).await.unwrap();
    assert!(
        log_manifest.generation >= 1,
        "log parts were published durably before shutdown completed"
    );
    let log_ids: HashSet<String> = log_manifest
        .parts
        .iter()
        .map(|part| part.id.clone())
        .collect();
    storage_b.restore_parts(&parts_root_b, &log_ids).await.unwrap();
    let registry_b = PartRegistry::load_from_manifest(&parts_root_b, &log_manifest).unwrap();
    let results = registry_b
        .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
        .unwrap();
    let total: usize = results.iter().map(|stream| stream.entries.len()).sum();
    assert_eq!(
        total, 3,
        "every acknowledged log line survived machine replacement"
    );

    let trace_manifest = storage_b
        .reconcile_trace_local_cache(&traces_root_b)
        .await
        .unwrap();
    let trace_ids: HashSet<String> = trace_manifest
        .parts
        .iter()
        .map(|part| part.id.clone())
        .collect();
    storage_b
        .restore_trace_parts(&traces_root_b, &trace_ids)
        .await
        .unwrap();
    let trace_registry_b = crate::trace_registry::TraceRegistry::load_from_manifest(
        &traces_root_b,
        &trace_manifest,
        registry_b.operation_lock(),
    )
    .unwrap();
    let spans = trace_registry_b.query_trace_id(&test_tenant(), &trace_id, None, None).unwrap();
    assert_eq!(
        spans.len(),
        1,
        "the acknowledged trace survived machine replacement"
    );
}

#[tokio::test]
async fn m6_force_flush_retries_until_object_store_recovers() {
    let dir = tmp_data_dir("m6-retry");
    std::fs::create_dir_all(dir.join("parts")).unwrap();
    std::fs::create_dir_all(dir.join("traces")).unwrap();
    let config = Arc::new(Config {
        data_dir: dir.clone(),
        ..Config::default()
    });

    let memtable = Arc::new(MemTable::new());
    let trace_memtable = Arc::new(crate::trace::TraceMemTable::new());
    let journal = Arc::new(
        Journal::spawn_with_traces(&config, memtable.clone(), trace_memtable.clone()).unwrap(),
    );
    ingest_once(&journal, &build_push_req()).await;
    assert!(!memtable.is_empty());

    let parts = Arc::new(PartRegistry::new());
    let trace_registry = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));

    let (store, handle) = faulty_store::new();
    // Fail the first two object-store writes, then let the backend recover.
    handle
        .fail_remaining
        .store(2, std::sync::atomic::Ordering::Release);
    let storage = Arc::new(crate::object_storage::ObjectStorage::from_store(
        store, "m6-retry",
    ));
    let remote_cache = Some(Arc::new(crate::object_storage::RemoteCache::new(
        storage.clone(),
        dir.join("parts"),
    )));

    let shutdown = Arc::new(ShutdownState::new());
    shutdown.begin_drain();

    let outcome = finalize_flush(FinalizeContext {
        shutdown: shutdown.clone(),
        memtable: memtable.clone(),
        trace_memtable: trace_memtable.clone(),
        journal: journal.clone(),
        registry: parts.clone(),
        trace_registry: trace_registry.clone(),
        series_registry: Arc::new(crate::series_registry::SeriesRegistry::standalone()),
        remote_cache: remote_cache.clone(),
        config: config.clone(),
    })
    .await;

    assert_eq!(outcome, ShutdownOutcome::Durable);
    assert!(shutdown.is_flush_complete());
    assert_eq!(shutdown.pending_flush_bytes(), 0);
    assert!(
        memtable.is_empty(),
        "force-flush drained the memtable once the store recovered"
    );
    assert_eq!(
        handle.fail_remaining.load(std::sync::atomic::Ordering::Acquire),
        0,
        "the injected failures were all consumed"
    );

    // The parts really landed in the object store, not just on local disk.
    let manifest = storage.reconcile_local_cache(&dir.join("parts")).await.unwrap();
    assert!(
        manifest.generation >= 1,
        "the retried force-flush published parts durably"
    );
}

/// A fenced instance must stop, not retry. Every other force-flush failure is
/// transient and worth waiting out; this one is not, and looping would keep the
/// process alive until the orchestrator killed it — after which the pod can be
/// rescheduled onto a different node and its disk, holding the only copy of the
/// unflushed data, thrown away.
#[tokio::test]
async fn m6_a_fenced_writer_stops_instead_of_retrying_forever() {
    let dir = tmp_data_dir("m6-fenced");
    std::fs::create_dir_all(dir.join("parts")).unwrap();
    std::fs::create_dir_all(dir.join("traces")).unwrap();
    let config = Arc::new(Config {
        data_dir: dir.clone(),
        ..Config::default()
    });

    let memtable = Arc::new(MemTable::new());
    let trace_memtable = Arc::new(crate::trace::TraceMemTable::new());
    let journal = Arc::new(
        Journal::spawn_with_traces(&config, memtable.clone(), trace_memtable.clone()).unwrap(),
    );
    ingest_once(&journal, &build_push_req()).await;
    assert!(!memtable.is_empty());

    let parts = Arc::new(PartRegistry::new());
    let trace_registry = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));

    // Two instances over one backing store, in the order an orchestrator that
    // ignores the drain would produce them.
    let store: Arc<dyn object_store::ObjectStore> =
        Arc::new(object_store::memory::InMemory::new());
    let old = crate::object_storage::ObjectStorage::sharing_store_for_test(store.clone());
    let new = crate::object_storage::ObjectStorage::sharing_store_for_test(store);
    old.claim_writer_epoch().await.unwrap();
    new.claim_writer_epoch().await.unwrap();

    let shutdown = Arc::new(ShutdownState::new());
    old.set_fence_sink(shutdown.clone());
    let remote_cache = Some(Arc::new(crate::object_storage::RemoteCache::new(
        old,
        dir.join("parts"),
    )));
    shutdown.begin_drain();

    let outcome = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        finalize_flush_with_abort(
            FinalizeContext {
                shutdown: shutdown.clone(),
                memtable: memtable.clone(),
                trace_memtable: trace_memtable.clone(),
                journal: journal.clone(),
                registry: parts.clone(),
                trace_registry: trace_registry.clone(),
                series_registry: Arc::new(crate::series_registry::SeriesRegistry::standalone()),
                remote_cache: remote_cache.clone(),
                config: config.clone(),
            },
            std::time::Duration::from_secs(3600),
            || tokio::sync::mpsc::channel::<()>(1).1,
        ),
    )
    .await
    .expect("a fenced force-flush must end on its own, without an operator");

    assert_eq!(outcome, ShutdownOutcome::Fenced);
    assert!(shutdown.is_fenced());
    assert!(
        !shutdown.is_flush_complete(),
        "a fenced force-flush must not report durability"
    );

    // The data is still on this disk, which is why the exit code has to be
    // non-zero and the disk has to be kept.
    let replayed = MemTable::new();
    crate::journal::replay(
        journal.wal_path(),
        journal.ckpt_path(),
        &replayed)
    .unwrap();
    assert!(!replayed.is_empty(), "the WAL must still hold the records");
}

#[tokio::test]
async fn m6_operator_abort_preserves_wal_for_restart_recovery() {
    let dir = tmp_data_dir("m6-abort");
    std::fs::create_dir_all(dir.join("parts")).unwrap();
    std::fs::create_dir_all(dir.join("traces")).unwrap();
    let config = Arc::new(Config {
        data_dir: dir.clone(),
        ..Config::default()
    });

    let memtable = Arc::new(MemTable::new());
    let trace_memtable = Arc::new(crate::trace::TraceMemTable::new());
    let journal = Arc::new(
        Journal::spawn_with_traces(&config, memtable.clone(), trace_memtable.clone()).unwrap(),
    );
    ingest_once(&journal, &build_push_req()).await;
    assert!(!memtable.is_empty());

    let parts = Arc::new(PartRegistry::new());
    let trace_registry = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));

    // An object store that never accepts writes, so force-flush can never reach
    // durability and only an operator abort can end it.
    let (store, handle) = faulty_store::new();
    handle
        .always_fail
        .store(true, std::sync::atomic::Ordering::Release);
    let storage = Arc::new(crate::object_storage::ObjectStorage::from_store(
        store, "m6-abort",
    ));
    let remote_cache = Some(Arc::new(crate::object_storage::RemoteCache::new(
        storage,
        dir.join("parts"),
    )));

    let shutdown = Arc::new(ShutdownState::new());
    shutdown.begin_drain();

    // Pre-buffer the operator abort so the first failed pass observes it.
    let (abort_tx, abort_rx) = tokio::sync::mpsc::channel::<()>(1);
    abort_tx.send(()).await.unwrap();

    let outcome = finalize_flush_with_abort(
        FinalizeContext {
            shutdown: shutdown.clone(),
            memtable: memtable.clone(),
            trace_memtable: trace_memtable.clone(),
            journal: journal.clone(),
            registry: parts.clone(),
            trace_registry: trace_registry.clone(),
            series_registry: Arc::new(crate::series_registry::SeriesRegistry::standalone()),
            remote_cache: remote_cache.clone(),
            config: config.clone(),
        },
        std::time::Duration::ZERO,
        move || abort_rx,
    )
    .await;

    assert_eq!(outcome, ShutdownOutcome::AbortedByOperator);
    assert!(
        !shutdown.is_flush_complete(),
        "an aborted force-flush must not report durability"
    );
    assert!(
        shutdown.pending_flush_bytes() > 0,
        "pending bytes stay non-zero so /metrics reports the data as not durable"
    );
    assert!(
        !memtable.is_empty(),
        "the unflushed data is still held in memory / on the WAL"
    );

    // Simulate a restart on the same disk: replay the WAL into a fresh memtable.
    drop(journal);
    let recovered = MemTable::new();
    let recovered_traces = crate::trace::TraceMemTable::new();
    crate::startup::recover_with_traces(&config, &recovered, &recovered_traces).unwrap();
    assert!(
        !recovered.is_empty(),
        "the WAL replay recovers the unflushed data after a forced exit"
    );
}
