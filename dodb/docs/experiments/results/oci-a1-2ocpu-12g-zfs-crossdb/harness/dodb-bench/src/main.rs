#![recursion_limit = "256"]

#[path = "../../common/mod.rs"]
mod common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use common::driver::{self, Args, Attempt, Engine, EngineFactory, ReadObservation};
use common::workload::{Mutation, QueryRequest};
use dodb_core::{
    DocumentKey, Error, PrimaryKey, RevisionState, SortKey, TransactionMutation, TransactionRequest,
};
use dodb_storage::{
    AsyncShard, BatchRequest, BatchResponse, BlinkReadHandle, BlinkStore, CoordinatorConfig,
    DatabaseConfig, ProductionFile,
};
use serde_json::{Value, json};
use tokio::runtime::{Handle, Runtime};

const DODB_CACHE_BYTES: usize = 64 * 1024 * 1024;

type MainShard = AsyncShard<ProductionFile, ProductionFile>;

struct BlinkMessage {
    request: TransactionRequest,
    response: tokio::sync::oneshot::Sender<dodb_core::Result<dodb_core::TransactionResult>>,
}

struct PlannedAdapter {
    sender: tokio::sync::mpsc::Sender<BlinkMessage>,
    store: Arc<Mutex<BlinkStore<ProductionFile, ProductionFile>>>,
    read_handle: BlinkReadHandle,
}

impl PlannedAdapter {
    fn start(mut store: BlinkStore<ProductionFile, ProductionFile>) -> Self {
        store.enable_planned_execution();
        let read_handle = store.versioned_read_handle();
        let store = Arc::new(Mutex::new(store));
        let worker_store = Arc::clone(&store);
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<BlinkMessage>(256);
        tokio::spawn(async move {
            while let Some(first) = receiver.recv().await {
                let mut group = vec![first];
                while group.len() < 64 {
                    match receiver.try_recv() {
                        Ok(message) => group.push(message),
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
                        | Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                    }
                }
                let requests: Vec<_> = group
                    .iter()
                    .map(|message| message.request.clone())
                    .collect();
                let results = worker_store
                    .lock()
                    .map_err(|_| Error::invariant("planned store lock poisoned"))
                    .and_then(|mut store| store.apply_transaction_group(&requests));
                match results {
                    Ok(results) if results.len() == group.len() => {
                        for (message, result) in group.into_iter().zip(results) {
                            let _ = message.response.send(result);
                        }
                    }
                    Ok(_) => {
                        for message in group {
                            let _ = message.response.send(Err(Error::invariant(
                                "planned coordinator returned an incomplete group",
                            )));
                        }
                    }
                    Err(error) => {
                        for message in group {
                            let _ = message.response.send(Err(Error::invariant(format!(
                                "planned coordinator failed: {error:?}"
                            ))));
                        }
                    }
                }
            }
        });
        Self {
            sender,
            store,
            read_handle,
        }
    }

    async fn transact(
        &self,
        request: TransactionRequest,
    ) -> dodb_core::Result<dodb_core::TransactionResult> {
        let (response, receiver) = tokio::sync::oneshot::channel();
        self.sender
            .send(BlinkMessage { request, response })
            .await
            .map_err(|_| Error::invariant("planned coordinator stopped"))?;
        receiver
            .await
            .map_err(|_| Error::invariant("planned transaction response dropped"))?
    }

    fn metrics(&self) -> Value {
        let wal = self
            .store
            .lock()
            .ok()
            .and_then(|store| store.wal_metrics().ok().flatten());
        json!({
            "versioned_read": format!("{:?}", self.read_handle.metrics()),
            "wal": wal.map(|metrics| json!({
                "wal_bytes": metrics.wal_bytes,
                "wal_syncs": metrics.wal_syncs,
                "sync_nanos": metrics.sync_nanos,
                "append_nanos": metrics.append_nanos,
            })),
            "store": self.store.lock().ok().map(|store| format!("{:?}", store.storage_metrics())),
        })
    }

    fn scan_count(&self) -> dodb_core::Result<u64> {
        let mut cursor: Option<DocumentKey> = None;
        let mut count = 0u64;
        loop {
            let rows = self.read_handle.scan(cursor.as_ref(), 1024)?;
            if rows.is_empty() {
                return Ok(count);
            }
            count += rows.len() as u64;
            cursor = rows.last().map(|row| row.key.clone());
        }
    }
}

enum StoreAdapter {
    Main(Arc<MainShard>),
    Planned(PlannedAdapter),
}

struct DodbEngine {
    runtime: Runtime,
    runtime_workers: usize,
    adapter: StoreAdapter,
    data_dir: PathBuf,
    file_path: PathBuf,
    wal_path: PathBuf,
    engine_name: String,
}

impl DodbEngine {
    fn handle(&self) -> Handle {
        self.runtime.handle().clone()
    }

    fn open(args: &Args, engine_name: String) -> Self {
        let data_dir = args.data_dir.join(&engine_name);
        std::fs::create_dir_all(&data_dir).expect("dodb data directory should be created");
        let file_path = data_dir.join("database.dodb");
        let wal_path = data_dir.join("database.wal");
        let runtime_workers = args
            .option("dodb-workers")
            .unwrap_or("2")
            .parse::<usize>()
            .expect("dodb worker count should be an integer");
        assert!(runtime_workers > 0, "dodb worker count should be positive");
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(runtime_workers)
            .thread_name("dodb-bench-runtime")
            .enable_all()
            .build()
            .expect("dodb benchmark runtime should start");
        let page_count = DODB_CACHE_BYTES / dodb_storage::page::PAGE_SIZE;
        let config = DatabaseConfig::default().with_cache_capacity(page_count);
        let adapter = {
            let _runtime_guard = runtime.enter();
            match engine_name.as_str() {
                "main-btree" => {
                    let store = dodb_storage::BTreeStore::open_with_wal(
                        ProductionFile::open(&file_path).expect("dodb data file should open"),
                        ProductionFile::open(&wal_path).expect("dodb WAL file should open"),
                        config,
                    )
                    .expect("main B-tree should open");
                    let shard = Arc::new(AsyncShard::start_with_config(
                        store,
                        CoordinatorConfig {
                            queue_capacity: 256,
                            max_group_requests: 64,
                            max_group_bytes: 4 * 1024 * 1024,
                            max_collection_delay: std::time::Duration::ZERO,
                        },
                    ));
                    StoreAdapter::Main(shard)
                }
                "planned-blink" => {
                    let store = BlinkStore::open_with_wal(
                        ProductionFile::open(&file_path).expect("dodb data file should open"),
                        ProductionFile::open(&wal_path).expect("dodb WAL file should open"),
                        config,
                    )
                    .expect("planned B-link should open");
                    StoreAdapter::Planned(PlannedAdapter::start(store))
                }
                other => panic!("unsupported dodb engine {other}"),
            }
        };
        Self {
            runtime,
            runtime_workers,
            adapter,
            data_dir,
            file_path,
            wal_path,
            engine_name,
        }
    }

    fn transact(
        &self,
        handle: &Handle,
        mutations: &[Mutation],
    ) -> dodb_core::Result<dodb_core::TransactionResult> {
        let request = TransactionRequest::new(
            Vec::new(),
            mutations
                .iter()
                .map(|mutation| TransactionMutation::Put {
                    key: document_key(&mutation.key),
                    value: mutation.value.clone(),
                })
                .collect(),
        );
        match &self.adapter {
            StoreAdapter::Main(shard) => handle.block_on(shard.execute_transaction(request)),
            StoreAdapter::Planned(adapter) => handle.block_on(adapter.transact(request)),
        }
    }

    fn read_value(&self, handle: &Handle, key: &[u8]) -> Option<ReadObservation> {
        let key = document_key(key);
        match &self.adapter {
            StoreAdapter::Main(shard) => handle
                .block_on(shard.execute(BatchRequest::Get { key }))
                .ok()
                .and_then(|response| match response {
                    BatchResponse::Get(RevisionState::Present { value, revision }) => {
                        Some(ReadObservation {
                            value,
                            revision: Some(revision.get()),
                        })
                    }
                    BatchResponse::Get(RevisionState::Missing { .. }) => None,
                    _ => None,
                }),
            StoreAdapter::Planned(adapter) => {
                adapter.read_handle.get(&key).ok().and_then(|state| {
                    state.value().map(|value| ReadObservation {
                        value: value.to_vec(),
                        revision: Some(state.revision().get()),
                    })
                })
            }
        }
    }

    fn query_rows(&self, request: &QueryRequest, limit: usize, handle: &Handle) -> Vec<Mutation> {
        let primary_key = PrimaryKey::new(request.primary_key.clone());
        let after = SortKey::new(request.exclusive_after_sort_key.clone());
        let rows = match &self.adapter {
            StoreAdapter::Main(shard) => handle
                .block_on(shard.execute(BatchRequest::Query {
                    pk: primary_key,
                    exclusive_after_sk: Some(after),
                    limit,
                }))
                .ok()
                .and_then(|response| match response {
                    BatchResponse::Query(rows) => Some(rows),
                    _ => None,
                })
                .unwrap_or_default(),
            StoreAdapter::Planned(adapter) => adapter
                .read_handle
                .query(&primary_key, Some(&after), limit)
                .unwrap_or_default(),
        };
        rows.into_iter()
            .map(|row| {
                let mut key = row.key.pk.as_bytes().to_vec();
                key.extend_from_slice(row.key.sk.as_bytes());
                Mutation {
                    key,
                    value: row.value,
                }
            })
            .collect()
    }

    fn count_rows(&self, handle: &Handle) -> u64 {
        match &self.adapter {
            StoreAdapter::Main(shard) => {
                let mut cursor = None;
                let mut count = 0u64;
                loop {
                    let rows = handle
                        .block_on(shard.execute(BatchRequest::Scan {
                            exclusive_after_key: cursor.clone(),
                            limit: 1024,
                        }))
                        .ok()
                        .and_then(|response| match response {
                            BatchResponse::Scan(rows) => Some(rows),
                            _ => None,
                        })
                        .unwrap_or_default();
                    if rows.is_empty() {
                        return count;
                    }
                    count += rows.len() as u64;
                    cursor = rows.last().map(|row| row.key.clone());
                }
            }
            StoreAdapter::Planned(adapter) => adapter.scan_count().unwrap_or(0),
        }
    }

    fn metrics(&self) -> Value {
        match &self.adapter {
            StoreAdapter::Main(shard) => json!({
                "coordinator": format!("{:?}", shard.coordinator_metrics()),
                "storage": shard.storage_metrics().map(|metrics| format!("{metrics:?}")),
                "wal": shard.wal_metrics().map(|metrics| json!({
                    "wal_bytes": metrics.wal_bytes,
                    "wal_syncs": metrics.wal_syncs,
                    "sync_nanos": metrics.sync_nanos,
                    "append_nanos": metrics.append_nanos,
                })),
            }),
            StoreAdapter::Planned(adapter) => adapter.metrics(),
        }
    }
}

fn document_key(key: &[u8]) -> DocumentKey {
    assert_eq!(key.len(), 16);
    DocumentKey::new(key[..8].to_vec(), key[8..].to_vec())
}

impl Engine for DodbEngine {
    type Writer = Handle;

    fn open_writer(&self, _writer_id: usize) -> Self::Writer {
        self.handle()
    }

    fn attempt(&self, writer: &mut Self::Writer, mutations: &[Mutation]) -> Attempt {
        match self.transact(writer, mutations) {
            Ok(result) => Attempt::Committed(result.revision.map(|revision| revision.get())),
            Err(Error::Conflict(error)) => {
                Attempt::Retryable(common::driver::RetryKind::Conflict, error.to_string())
            }
            Err(Error::Overloaded(message)) => {
                Attempt::Retryable(common::driver::RetryKind::Busy, message)
            }
            Err(error) => Attempt::Failed(format!("{error:?}")),
        }
    }

    fn seed(&self, writer: &mut Self::Writer, rows: &[Mutation]) {
        match self.transact(writer, rows) {
            Ok(_) => {}
            Err(error) => panic!("dodb seed transaction failed: {error:?}"),
        }
    }

    fn read(&self, writer: &mut Self::Writer, key: &[u8]) -> Option<Vec<u8>> {
        self.read_value(writer, key)
            .map(|observation| observation.value)
    }

    fn read_observation(&self, writer: &mut Self::Writer, key: &[u8]) -> Option<ReadObservation> {
        self.read_value(writer, key)
    }

    fn checks_revisions(&self) -> bool {
        true
    }

    fn query(
        &self,
        writer: &mut Self::Writer,
        request: &QueryRequest,
        limit: usize,
    ) -> Vec<Mutation> {
        self.query_rows(request, limit, writer)
    }

    fn count_rows(&self, writer: &mut Self::Writer) -> u64 {
        DodbEngine::count_rows(self, writer)
    }

    fn settings(&self, _writer: &mut Self::Writer) -> Value {
        json!({
            "engine": self.engine_name,
            "sync_mode": "ProductionFile real sync",
            "page_cache_bytes": DODB_CACHE_BYTES,
            "collection_policy": "drain queued requests up to the group limit with nonblocking collection",
            "collection_max_requests": 64,
            "collection_max_bytes": if self.engine_name == "main-btree" {
                json!(4 * 1024 * 1024)
            } else {
                Value::Null
            },
            "collection_delay_us": 0,
            "queue_capacity": 256,
            "coordinator_workers": 1,
            "parallel_worker_count": 0,
            "tokio_runtime_workers": self.runtime_workers,
            "coordinator_workers": 1,
            "parallel_execution": false,
            "borrowed_page_views": false,
            "read_observation_metrics": true,
        })
    }

    fn metrics(&self) -> Value {
        DodbEngine::metrics(self)
    }

    fn monitor_sample(&self) -> Value {
        self.metrics()
    }
}

struct DodbFactory;

impl EngineFactory for DodbFactory {
    type Engine = DodbEngine;

    fn engine_name(args: &Args) -> String {
        args.option("dodb-engine")
            .unwrap_or("main-btree")
            .to_owned()
    }

    fn build_info() -> Value {
        json!({
            "database": "dodb",
            "source_sha": option_env!("DODB_SOURCE_SHA").unwrap_or("unknown"),
            "source_clean": option_env!("DODB_SOURCE_CLEAN").unwrap_or("unknown"),
            "client_threads": "one named OS thread per external client, shared crossdb workload driver",
            "internal_runtime": "Tokio worker thread count configured per run",
        })
    }

    fn create(args: &Args) -> Self::Engine {
        DodbEngine::open(args, Self::engine_name(args))
    }

    fn reopen(args: &Args) -> Self::Engine {
        DodbEngine::open(args, Self::engine_name(args))
    }

    fn close(engine: Self::Engine) -> Value {
        let metrics = engine.metrics();
        match &engine.adapter {
            StoreAdapter::Main(shard) => {
                engine
                    .runtime
                    .block_on(shard.shutdown())
                    .expect("main coordinator should close");
            }
            StoreAdapter::Planned(_) => {}
        }
        let file_sizes = [engine.file_path.clone(), engine.wal_path.clone()]
            .into_iter()
            .map(|path| {
                (
                    path.display().to_string(),
                    std::fs::metadata(path).map(|metadata| metadata.len()).ok(),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        json!({
            "metrics": metrics,
            "files": file_sizes,
            "data_dir": engine.data_dir,
        })
    }
}

fn main() {
    driver::bench_main::<DodbFactory>();
}
