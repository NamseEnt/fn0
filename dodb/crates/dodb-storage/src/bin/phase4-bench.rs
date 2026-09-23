use std::sync::Arc;
use std::time::{Duration, Instant};

use dodb_core::{DocumentKey, Error, Result, TransactionMutation, TransactionRequest};
use dodb_storage::{
    AsyncShard, BTreeStore, BatchRequest, BatchResponse, CoordinatorConfig, DurableFile,
    ProductionFile,
};

const OPERATIONS_PER_CLIENT: usize = 16;
const READS_PER_CLIENT: usize = 64;

struct DelayedFile {
    inner: ProductionFile,
    sync_delay: Duration,
}

impl DelayedFile {
    fn open(path: &std::path::Path, sync_delay: Duration) -> Result<Self> {
        Ok(Self {
            inner: ProductionFile::open(path)?,
            sync_delay,
        })
    }

    fn delay(&self) {
        if !self.sync_delay.is_zero() {
            std::thread::sleep(self.sync_delay);
        }
    }
}

impl DurableFile for DelayedFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
        self.inner.write_at(offset, bytes)
    }

    fn len(&self) -> Result<u64> {
        self.inner.len()
    }

    fn set_len(&mut self, length: u64) -> Result<()> {
        self.inner.set_len(length)
    }

    fn sync_data(&mut self) -> Result<()> {
        self.delay();
        self.inner.sync_data()
    }

    fn sync_all(&mut self) -> Result<()> {
        self.delay();
        self.inner.sync_all()
    }
}

fn key(index: usize) -> DocumentKey {
    DocumentKey::new(
        (index % 128).to_le_bytes().to_vec(),
        (index as u64).to_le_bytes().to_vec(),
    )
}

fn transaction(index: usize, width: usize) -> TransactionRequest {
    let mutations = (0..width)
        .map(|offset| TransactionMutation::Put {
            key: key(index.saturating_mul(width).saturating_add(offset)),
            value: index.to_le_bytes().to_vec(),
        })
        .collect();
    TransactionRequest::new(Vec::new(), mutations)
}

fn fresh_store(
    cache_capacity: usize,
    sync_delay: Duration,
    label: &str,
) -> BTreeStore<DelayedFile, DelayedFile> {
    let path = std::env::temp_dir().join(format!("dodb-phase4-{}-{label}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("wal"));
    BTreeStore::open_with_wal(
        DelayedFile::open(&path, sync_delay).expect("benchmark data file should open"),
        DelayedFile::open(&path.with_extension("wal"), sync_delay)
            .expect("benchmark WAL file should open"),
        dodb_storage::DatabaseConfig::default().with_cache_capacity(cache_capacity),
    )
    .expect("benchmark database should open")
}

fn cleanup_benchmark_files() {
    let prefix = format!("dodb-phase4-{}-", std::process::id());
    if let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix))
            {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

fn percentile(samples: &mut [Duration], fraction: f64) -> f64 {
    samples.sort_unstable();
    let index = ((samples.len().saturating_sub(1)) as f64 * fraction).round() as usize;
    samples[index].as_secs_f64() * 1_000_000.0
}

struct TransactionRun {
    attempts: usize,
    successes: usize,
    conflicts: usize,
    overloaded: usize,
    latencies: Vec<Duration>,
}

fn report_metrics(
    name: &str,
    elapsed: Duration,
    mut run: TransactionRun,
    shard: &AsyncShard<DelayedFile, DelayedFile>,
) {
    let metrics = shard.coordinator_metrics();
    let wal = shard.wal_metrics().expect("WAL metrics should be readable");
    let durable_syncs = wal.wal_syncs.saturating_sub(1);
    let tx_per_sync = if durable_syncs == 0 {
        0.0
    } else {
        wal.committed_batches as f64 / durable_syncs as f64
    };
    println!(
        "{name:30} attempts={attempts:5} success={successes:5} conflict={conflicts:5} overload={overloaded:3} tx/s={throughput:9.0} p50={p50:7.1}us p95={p95:7.1}us p99={p99:7.1}us",
        attempts = run.attempts,
        successes = run.successes,
        conflicts = run.conflicts,
        overloaded = run.overloaded,
        throughput = run.successes as f64 / elapsed.as_secs_f64(),
        p50 = percentile(&mut run.latencies, 0.50),
        p95 = percentile(&mut run.latencies, 0.95),
        p99 = percentile(&mut run.latencies, 0.99),
    );
    println!(
        "  coordinator: groups={} tx/group={:.2} max_group={} queue_wait={:.1}us collection={:.1}us processing={:.1}us",
        metrics.groups,
        metrics.logical_transactions as f64 / metrics.groups.max(1) as f64,
        metrics.max_group_requests,
        metrics.queue_wait_nanos as f64 / metrics.queued_requests.max(1) as f64 / 1_000.0,
        metrics.batch_collection_nanos as f64 / metrics.groups.max(1) as f64 / 1_000.0,
        metrics.processing_nanos as f64 / metrics.groups.max(1) as f64 / 1_000.0,
    );
    if let Some(storage) = shard.storage_metrics() {
        println!(
            "  storage: validation={:.1}us btree_prepare={:.1}us publication={:.1}us",
            storage.validation_nanos as f64 / run.attempts.max(1) as f64 / 1_000.0,
            storage.btree_preparation_nanos as f64 / run.attempts.max(1) as f64 / 1_000.0,
            storage.publication_nanos as f64 / run.attempts.max(1) as f64 / 1_000.0,
        );
    }
    println!(
        "  WAL: bytes={} syncs={} tx/sync={tx_per_sync:.2} page_images={} serialize_append={:.1}us sync={:.1}us",
        wal.wal_bytes,
        durable_syncs,
        wal.page_images,
        wal.append_nanos as f64 / wal.committed_batches.max(1) as f64 / 1_000.0,
        wal.sync_nanos as f64 / durable_syncs.max(1) as f64 / 1_000.0,
    );
}

fn report_read(
    cache_capacity: usize,
    clients: usize,
    elapsed: Duration,
    latencies: &mut [Duration],
    shard: &AsyncShard<DelayedFile, DelayedFile>,
) {
    let operations = latencies.len();
    let metrics = shard.coordinator_metrics();
    println!(
        "read cache={cache_capacity:3} clients={clients:2} ops/s={:9.0} p50={:7.1}us p95={:7.1}us p99={:7.1}us queued={}",
        operations as f64 / elapsed.as_secs_f64(),
        percentile(latencies, 0.50),
        percentile(latencies, 0.95),
        percentile(latencies, 0.99),
        metrics.queued_requests,
    );
}

async fn run_transactions(sync_delay: Duration, clients: usize, width: usize) {
    let store = fresh_store(
        256,
        sync_delay,
        &format!("txn-{sync_delay:?}-{clients}-{width}"),
    );
    let shard = Arc::new(AsyncShard::start_with_config(
        store,
        CoordinatorConfig {
            queue_capacity: clients.saturating_mul(OPERATIONS_PER_CLIENT).max(1),
            ..CoordinatorConfig::default()
        },
    ));
    let start = Instant::now();
    let mut tasks = Vec::with_capacity(clients);
    for client in 0..clients {
        let shard = Arc::clone(&shard);
        tasks.push(tokio::spawn(async move {
            let mut conflicts = 0;
            let mut overloaded = 0;
            let mut latencies = Vec::with_capacity(OPERATIONS_PER_CLIENT);
            for operation in 0..OPERATIONS_PER_CLIENT {
                let transaction_start = Instant::now();
                let result = shard
                    .execute_transaction(transaction(
                        client * OPERATIONS_PER_CLIENT + operation,
                        width,
                    ))
                    .await;
                match result {
                    Ok(_) => {}
                    Err(Error::Conflict(_)) => conflicts += 1,
                    Err(Error::Overloaded(_)) => overloaded += 1,
                    Err(error) => panic!("benchmark transaction failed: {error}"),
                }
                latencies.push(transaction_start.elapsed());
            }
            (conflicts, overloaded, latencies)
        }));
    }
    let mut conflicts = 0;
    let mut overloaded = 0;
    let mut latencies = Vec::with_capacity(clients * OPERATIONS_PER_CLIENT);
    for task in tasks {
        let (task_conflicts, task_overloaded, task_latencies) =
            task.await.expect("benchmark client should finish");
        conflicts += task_conflicts;
        overloaded += task_overloaded;
        latencies.extend(task_latencies);
    }
    let elapsed = start.elapsed();
    let shard = match Arc::try_unwrap(shard) {
        Ok(shard) => shard,
        Err(_) => panic!("benchmark client references should be gone"),
    };
    report_metrics(
        &format!("txn {width} keys {clients} clients {sync_delay:?}"),
        elapsed,
        TransactionRun {
            attempts: clients * OPERATIONS_PER_CLIENT,
            successes: clients * OPERATIONS_PER_CLIENT - conflicts - overloaded,
            conflicts,
            overloaded,
            latencies,
        },
        &shard,
    );
    shard.close().await.expect("benchmark shard should close");
}

async fn run_reads(cache_capacity: usize, clients: usize) {
    let mut store = fresh_store(
        cache_capacity,
        Duration::ZERO,
        &format!("reads-{cache_capacity}-{clients}"),
    );
    for index in 0..256usize {
        store
            .put(key(index), index.to_le_bytes().to_vec())
            .expect("read benchmark seed write should succeed");
    }
    let shard = Arc::new(AsyncShard::start(store, clients.saturating_mul(2).max(1)));
    let start = Instant::now();
    let mut tasks = Vec::with_capacity(clients);
    for client in 0..clients {
        let shard = Arc::clone(&shard);
        tasks.push(tokio::spawn(async move {
            let mut latencies = Vec::with_capacity(READS_PER_CLIENT);
            for operation in 0..READS_PER_CLIENT {
                let operation_start = Instant::now();
                let index = (client * READS_PER_CLIENT + operation) % 256;
                let response = shard
                    .execute(BatchRequest::Get { key: key(index) })
                    .await
                    .expect("read benchmark get should succeed");
                assert!(matches!(response, BatchResponse::Get(_)));
                latencies.push(operation_start.elapsed());
            }
            latencies
        }));
    }
    let mut latencies = Vec::with_capacity(clients * READS_PER_CLIENT);
    for task in tasks {
        latencies.extend(task.await.expect("read benchmark client should finish"));
    }
    let elapsed = start.elapsed();
    let shard = match Arc::try_unwrap(shard) {
        Ok(shard) => shard,
        Err(_) => panic!("read benchmark client references should be gone"),
    };
    report_read(cache_capacity, clients, elapsed, &mut latencies, &shard);
    shard
        .close()
        .await
        .expect("read benchmark shard should close");
}

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("benchmark runtime should build");
    cleanup_benchmark_files();
    runtime.block_on(async {
        println!("dodb Phase 4 scheduling benchmark");
        println!("ordinary committed reads: cache 0/16/256, clients 1/4/16/64");
        for cache_capacity in [0, 16, 256] {
            for clients in [1, 4, 16, 64] {
                run_reads(cache_capacity, clients).await;
            }
        }
        println!("transactions: widths 1/4/16, clients 1/4/16/64");
        for width in [1, 4, 16] {
            for clients in [1, 4, 16, 64] {
                run_transactions(Duration::ZERO, clients, width).await;
            }
        }
        println!("\nrepresentative fsync-latency matrix: width=1, clients=1/16/64");
        for sync_delay in [
            Duration::ZERO,
            Duration::from_micros(100),
            Duration::from_millis(1),
            Duration::from_millis(5),
            Duration::from_millis(10),
        ] {
            for clients in [1, 16, 64] {
                run_transactions(sync_delay, clients, 1).await;
            }
        }
        println!("\nordinary read cache configurations are exercised by the storage tests");
    });
    cleanup_benchmark_files();
}
