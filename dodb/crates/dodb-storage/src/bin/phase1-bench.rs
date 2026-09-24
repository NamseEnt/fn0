use std::sync::Arc;
use std::time::{Duration, Instant};

use dodb_core::{DocumentKey, TransactionCondition, TransactionMutation, TransactionRequest};
use dodb_storage::{BTreeStore, DatabaseConfig, ProductionFile};

const ROWS: usize = 2_000;
const ASYNC_OPERATIONS_PER_CLIENT: usize = 64;

fn key(index: usize) -> DocumentKey {
    DocumentKey::new(
        (index % 64).to_le_bytes().to_vec(),
        (index as u64).to_le_bytes().to_vec(),
    )
}

fn random_index(state: &mut u64) -> usize {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state as usize) % ROWS
}

fn transaction_request(start: usize, width: usize) -> TransactionRequest {
    let mutations = (0..width)
        .map(|offset| TransactionMutation::Put {
            key: key(start + offset),
            value: (start + offset).to_le_bytes().to_vec(),
        })
        .collect();
    TransactionRequest::new(Vec::new(), mutations)
}

fn fresh_store(cache_capacity: usize, name: &str) -> BTreeStore<ProductionFile, ProductionFile> {
    let path = std::env::temp_dir().join(format!(
        "dodb-phase1-bench-{}-{name}.db",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("wal"));
    BTreeStore::<ProductionFile>::open_path(
        &path,
        DatabaseConfig::default().with_cache_capacity(cache_capacity),
    )
    .expect("benchmark database should open")
}

fn report(name: &str, elapsed: Duration, operations: usize) {
    let seconds = elapsed.as_secs_f64();
    println!(
        "{name:24} {:>8.0} ops/s {:>8.3} ms/op",
        operations as f64 / seconds,
        seconds * 1_000.0 / operations as f64
    );
}

fn report_latency(name: &str, elapsed: Duration, operations: usize, samples: &mut [Duration]) {
    report(name, elapsed, operations);
    samples.sort_unstable();
    let percentile = |fraction: f64| {
        let index = ((samples.len().saturating_sub(1)) as f64 * fraction).round() as usize;
        samples[index].as_secs_f64() * 1_000_000.0
    };
    println!(
        "  latency: p50 {:.1} us, p95 {:.1} us, p99 {:.1} us",
        percentile(0.50),
        percentile(0.95),
        percentile(0.99)
    );
}

fn report_wal(store: &BTreeStore<ProductionFile, ProductionFile>, operations: usize) {
    if let Some(metrics) = store.wal_metrics().expect("WAL metrics should be readable") {
        println!(
            "  WAL: {} bytes, {} syncs, {:.2} operations/sync, {:.1} us append, {:.1} us sync",
            metrics.wal_bytes,
            metrics.wal_syncs,
            operations as f64 / metrics.wal_syncs.max(1) as f64,
            metrics.append_nanos as f64 / 1_000.0 / metrics.committed_batches.max(1) as f64,
            metrics.sync_nanos as f64 / 1_000.0 / metrics.committed_batches.max(1) as f64,
        );
    }
}

fn main() {
    println!("dodb transaction and WAL benchmark; rows={ROWS}; WAL-first publisher");
    for cache_capacity in [0, 16, 256] {
        println!("\ncache_capacity={cache_capacity}");

        let mut store = fresh_store(cache_capacity, "sequential-put");
        let start = Instant::now();
        for index in 0..ROWS {
            store
                .put(key(index), (index as u64).to_le_bytes())
                .expect("sequential put should succeed");
        }
        report("small-value PUT", start.elapsed(), ROWS);
        report_wal(&store, ROWS);

        let mut store = fresh_store(cache_capacity, "random-put");
        let mut state = 0x5eed_cafe_u64;
        let start = Instant::now();
        for _ in 0..ROWS {
            let index = random_index(&mut state);
            store
                .put(key(index), (index as u64).to_le_bytes())
                .expect("random put should succeed");
        }
        report("random PUT", start.elapsed(), ROWS);
        report_wal(&store, ROWS);

        if cache_capacity == 256 {
            let mut store = fresh_store(cache_capacity, "overflow-put");
            let overflow_value = vec![0x5a; 9_000];
            let start = Instant::now();
            for index in 0..ROWS {
                store
                    .put(key(index), overflow_value.clone())
                    .expect("overflow put should succeed");
            }
            report("overflow-value PUT", start.elapsed(), ROWS);
            report_wal(&store, ROWS);
        }

        let mut store = fresh_store(cache_capacity, "reads");
        for index in 0..ROWS {
            store
                .put(key(index), (index as u64).to_le_bytes())
                .expect("seed put should succeed");
        }
        let mut state = 0xd0db_2026_u64;
        let start = Instant::now();
        for _ in 0..ROWS {
            let _ = store
                .get(&key(random_index(&mut state)))
                .expect("get should succeed");
        }
        report("random GET", start.elapsed(), ROWS);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = store.scan(None, 100).expect("scan should succeed");
        }
        report("sequential scan/range", start.elapsed(), 100);

        let start = Instant::now();
        for pk in 0..100 {
            let _ = store
                .query(
                    &dodb_core::PrimaryKey::new((pk as u64 % 64).to_le_bytes().to_vec()),
                    None,
                    32,
                )
                .expect("query should succeed");
        }
        report("single-pk query", start.elapsed(), 100);

        let mut state = 0x1234_5678_u64;
        let start = Instant::now();
        for operation in 0..ROWS {
            let index = random_index(&mut state);
            if operation % 2 == 0 {
                let _ = store.get(&key(index)).expect("mixed get should succeed");
            } else {
                store
                    .put(key(index), (operation as u64).to_le_bytes())
                    .expect("mixed put should succeed");
            }
        }
        report("mixed read/write", start.elapsed(), ROWS);
    }

    println!("\noptimistic transaction sizes");
    for width in [1, 4, 16] {
        let transaction_count = ROWS / width;
        let mut store = fresh_store(256, &format!("transaction-{width}"));
        let mut latencies = Vec::with_capacity(transaction_count);
        let start = Instant::now();
        for transaction_index in 0..transaction_count {
            let transaction_start = Instant::now();
            store
                .transact(transaction_request(transaction_index * width, width))
                .expect("transaction should succeed");
            latencies.push(transaction_start.elapsed());
        }
        report_latency(
            &format!("unconditional txn {width} keys"),
            start.elapsed(),
            transaction_count,
            &mut latencies,
        );
        report_wal(&store, transaction_count);
    }

    let mut store = fresh_store(256, "conditional-transactions");
    for index in 0..ROWS {
        store
            .put(key(index), (index as u64).to_le_bytes())
            .expect("conditional benchmark seed should succeed");
    }
    let start = Instant::now();
    let mut latencies = Vec::with_capacity(ROWS);
    for index in 0..ROWS {
        let transaction_start = Instant::now();
        let document_key = key(index);
        let revision = store
            .get(&document_key)
            .expect("conditional benchmark read should succeed")
            .revision();
        store
            .transact(TransactionRequest::new(
                vec![TransactionCondition::RevisionEquals {
                    key: document_key.clone(),
                    expected_revision: revision,
                }],
                vec![TransactionMutation::Put {
                    key: document_key,
                    value: (index as u64 + 1).to_le_bytes().to_vec(),
                }],
            ))
            .expect("conditional transaction should succeed");
        latencies.push(transaction_start.elapsed());
    }
    report_latency("conditional txn", start.elapsed(), ROWS, &mut latencies);
    report_wal(&store, ROWS);

    println!(
        "\nasync WAL coordinator; default group limit=64; zero-delay evidence-based collection"
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("benchmark runtime should build");
    runtime.block_on(async {
        for clients in [1, 4, 16, 64] {
            let store = fresh_store(256, &format!("async-{clients}"));
            let shard = Arc::new(dodb_storage::AsyncShard::start(
                store,
                if clients == 1 { 1 } else { 256 },
            ));
            let start = Instant::now();
            let mut tasks = Vec::with_capacity(clients);
            for client in 0..clients {
                let shard = Arc::clone(&shard);
                tasks.push(tokio::spawn(async move {
                    for operation in 0..ASYNC_OPERATIONS_PER_CLIENT {
                        let index = (client * ASYNC_OPERATIONS_PER_CLIENT + operation) % ROWS;
                        shard
                            .execute(dodb_storage::BatchRequest::Put {
                                key: key(index),
                                value: (index as u64).to_le_bytes().to_vec(),
                            })
                            .await
                            .expect("async put should succeed");
                    }
                }));
            }
            for task in tasks {
                task.await.expect("async benchmark task should succeed");
            }
            report(
                &format!("async PUT {clients} clients"),
                start.elapsed(),
                clients * ASYNC_OPERATIONS_PER_CLIENT,
            );
            if let Some(metrics) = shard.wal_metrics() {
                println!(
                    "  WAL: {} bytes, {} syncs, {:.2} operations/sync, {:.1} us append, {:.1} us sync",
                    metrics.wal_bytes,
                    metrics.wal_syncs,
                    (clients * ASYNC_OPERATIONS_PER_CLIENT) as f64
                        / metrics.wal_syncs.max(1) as f64,
                    metrics.append_nanos as f64 / 1_000.0
                        / metrics.committed_batches.max(1) as f64,
                    metrics.sync_nanos as f64 / 1_000.0
                        / metrics.committed_batches.max(1) as f64,
                );
            }
        }

        println!("\nasync optimistic transactions; conflict rate is reported");
        for clients in [1, 4, 16, 64] {
            for pattern in [
                "disjoint",
                "10%-hot",
                "hot",
                "insert-if-absent",
                "read-dependent",
            ] {
                let store = fresh_store(256, &format!("async-txn-{pattern}-{clients}"));
                let shard = Arc::new(dodb_storage::AsyncShard::start(
                    store,
                    if clients == 1 { 1 } else { 256 },
                ));
                let start = Instant::now();
                let mut tasks = Vec::with_capacity(clients);
                for client in 0..clients {
                    let shard = Arc::clone(&shard);
                    tasks.push(tokio::spawn(async move {
                        let mut conflicts = 0usize;
                        let mut latencies = Vec::with_capacity(ASYNC_OPERATIONS_PER_CLIENT);
                        for operation in 0..ASYNC_OPERATIONS_PER_CLIENT {
                            let transaction_start = Instant::now();
                            let index = match pattern {
                                "disjoint" => client * ASYNC_OPERATIONS_PER_CLIENT + operation,
                                "10%-hot" if operation % 10 == 0 => 0,
                                "hot" | "insert-if-absent" => 0,
                                _ => (client * ASYNC_OPERATIONS_PER_CLIENT + operation) % ROWS,
                            };
                            let request = if pattern == "insert-if-absent" {
                                TransactionRequest::new(
                                    vec![TransactionCondition::NotExists { key: key(index) }],
                                    vec![TransactionMutation::Put {
                                        key: key(index),
                                        value: (operation as u64).to_le_bytes().to_vec(),
                                    }],
                                )
                            } else if pattern == "read-dependent" {
                                let document_key = key(index);
                                let observed = match shard
                                    .execute(dodb_storage::BatchRequest::Get {
                                        key: document_key.clone(),
                                    })
                                    .await
                                    .expect("read-dependent benchmark get should succeed")
                                {
                                    dodb_storage::BatchResponse::Get(state) => state.revision(),
                                    _ => unreachable!("get returned the wrong benchmark response"),
                                };
                                TransactionRequest::new(
                                    vec![TransactionCondition::RevisionEquals {
                                        key: document_key.clone(),
                                        expected_revision: observed,
                                    }],
                                    vec![TransactionMutation::Put {
                                        key: document_key,
                                        value: (operation as u64).to_le_bytes().to_vec(),
                                    }],
                                )
                            } else {
                                transaction_request(index, 1)
                            };
                            if shard.execute_transaction(request).await.is_err() {
                                conflicts += 1;
                            }
                            latencies.push(transaction_start.elapsed());
                        }
                        (conflicts, latencies)
                    }));
                }
                let mut conflicts = 0usize;
                let mut latencies = Vec::with_capacity(clients * ASYNC_OPERATIONS_PER_CLIENT);
                for task in tasks {
                    let (task_conflicts, task_latencies) = task
                        .await
                        .expect("transaction benchmark task should succeed");
                    conflicts += task_conflicts;
                    latencies.extend(task_latencies);
                }
                let operations = clients * ASYNC_OPERATIONS_PER_CLIENT;
                report_latency(
                    &format!("async txn {pattern} {clients} clients"),
                    start.elapsed(),
                    operations,
                    &mut latencies,
                );
                println!(
                    "  conflicts: {} / {} ({:.1}%)",
                    conflicts,
                    operations,
                    conflicts as f64 * 100.0 / operations as f64
                );
                if let Some(metrics) = shard.wal_metrics() {
                    println!(
                        "  WAL: {} syncs, {:.2} transactions/sync, {:.1} us append, {:.1} us sync",
                        metrics.wal_syncs,
                        operations as f64 / metrics.wal_syncs.max(1) as f64,
                        metrics.append_nanos as f64 / 1_000.0
                            / metrics.committed_batches.max(1) as f64,
                        metrics.sync_nanos as f64 / 1_000.0
                            / metrics.committed_batches.max(1) as f64,
                    );
                }
                drop(shard);
            }
        }
    });
}
