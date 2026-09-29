#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;

use dodb_core::{DocumentKey, TransactionCondition, TransactionMutation, TransactionRequest};
use dodb_storage::{BlinkStore, DatabaseConfig, ProductionFile};

fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn key(index: u64) -> DocumentKey {
    DocumentKey::new(
        vec![0x51, (index % 97) as u8, 0, 7],
        index.to_be_bytes().to_vec(),
    )
}

fn file_hash(path: &Path) -> (String, usize) {
    let bytes = std::fs::read(path).unwrap();
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    (output.split_whitespace().next().unwrap().to_owned(), bytes.len())
}

fn run(directory: &Path, seed: u64, workers: usize, max_value: u64) -> String {
    let _ = std::fs::remove_dir_all(directory);
    std::fs::create_dir_all(directory).unwrap();
    let path = directory.join("store.db");
    let mut store = BlinkStore::<ProductionFile, ProductionFile>::open_path(&path, DatabaseConfig::default()).unwrap();
    store.enable_planned_execution();
    if workers > 0 {
        store.enable_parallel_execution(workers).unwrap();
    }
    let mut state = seed;
    let mut results = DefaultHasher::new();
    let mut accepted = 0u64;
    let mut group_errors = 0u64;
    for group_index in 0..120u64 {
        let request_count = 1 + next(&mut state) % 48;
        let mut requests = Vec::new();
        for _ in 0..request_count {
            let width = 1 + next(&mut state) % 16;
            let mut mutations = Vec::new();
            let mut used = std::collections::BTreeSet::new();
            for _ in 0..width {
                let index = next(&mut state) % 4_000;
                if !used.insert(index) {
                    continue;
                }
                if next(&mut state) % 10 == 0 {
                    mutations.push(TransactionMutation::Delete { key: key(index) });
                } else {
                    let length = if next(&mut state) % 40 == 0 { 900 } else { 1 + next(&mut state) % max_value } as usize;
                    let fill = next(&mut state) as u8;
                    mutations.push(TransactionMutation::Put { key: key(index), value: vec![fill; length] });
                }
            }
            let mut conditions = Vec::new();
            if next(&mut state) % 6 == 0 {
                let index = next(&mut state) % 4_000;
                if !used.contains(&index) {
                    conditions.push(if next(&mut state) % 2 == 0 {
                        TransactionCondition::Exists { key: key(index) }
                    } else {
                        TransactionCondition::NotExists { key: key(index) }
                    });
                }
            }
            requests.push(TransactionRequest::new(conditions, mutations));
        }
        let outcome = match store.apply_transaction_group(&requests) {
            Ok(outcome) => outcome,
            Err(error) => {
                format!("group error {error}").hash(&mut results);
                group_errors += 1;
                Vec::new()
            }
        };
        for result in &outcome {
            match result {
                Ok(result) => {
                    accepted += 1;
                    result.commit_lsn.get().hash(&mut results);
                }
                Err(error) => error.to_string().hash(&mut results),
            }
        }
        if group_index == 70 {
            store.checkpoint().unwrap();
        }
        if group_index == 95 {
            store.flush().unwrap();
        }
    }
    let mut reads = DefaultHasher::new();
    for index in (0..4_000u64).step_by(7) {
        format!("{:?}", store.get(&key(index)).unwrap()).hash(&mut reads);
    }
    for partition in 0..97u8 {
        let primary = dodb_core::PrimaryKey::new(vec![0x51, partition, 0, 7]);
        format!("{:?}", store.query(&primary, None, 50).unwrap()).hash(&mut reads);
    }
    let documents = store.scan(None, usize::MAX).unwrap();
    let mut scan = DefaultHasher::new();
    format!("{documents:?}").hash(&mut scan);
    drop(store);
    let (data_hash, data_length) = file_hash(&path);
    let (wal_hash, wal_length) = file_hash(&path.with_extension("wal"));
    let mut reopened = BlinkStore::<ProductionFile, ProductionFile>::open_path(&path, DatabaseConfig::default()).unwrap();
    let reopened_documents = reopened.scan(None, usize::MAX).unwrap();
    let mut reopened_scan = DefaultHasher::new();
    format!("{reopened_documents:?}").hash(&mut reopened_scan);
    reopened.check_invariants().unwrap();
    format!(
        "seed={seed} max_value={max_value} workers={workers} accepted={accepted} group_errors={group_errors} results={:016x} reads={:016x} documents={} scan={:016x} reopen_scan={:016x} data={data_hash}/{data_length} wal={wal_hash}/{wal_length}",
        results.finish(),
        reads.finish(),
        documents.len(),
        scan.finish(),
        reopened_scan.finish(),
    )
}

fn main() {
    let root = std::env::temp_dir().join(format!("dodb-crossver-{}", std::process::id()));
    for max_value in [120u64, 40] {
        for seed in [11u64, 12, 13] {
            for workers in [0usize, 2] {
                println!("{}", run(&root.join(format!("{seed}-{workers}")), seed, workers, max_value));
            }
        }
    }
    let _ = std::fs::remove_dir_all(root);
}
