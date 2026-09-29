#[path = "../../common/workload.rs"]
mod workload;

mod original {
    #![allow(dead_code)]
    use std::collections::HashSet;

    use dodb_core::{
        DocumentKey, PrimaryKey, TransactionCondition, TransactionMutation, TransactionRequest,
    };
    use dodb_storage::BatchRequest;

    include!(concat!(env!("OUT_DIR"), "/original_generator.rs"));

    pub struct OriginalGenerator(WorkloadGenerator);

    impl OriginalGenerator {
        pub fn new(distribution: &str, working_set: usize, width: usize, seed: u64, worker_id: usize) -> Self {
            let config = WorkloadConfig {
                distribution: Distribution::parse(distribution),
                working_set,
                key_size: 16,
                value_size: 64,
                width,
                transaction_mode: TransactionMode::Unconditional,
                read_limit: 16,
            };
            Self(WorkloadGenerator::new(config, seed, worker_id))
        }

        pub fn next_transaction(&mut self) -> Vec<(Vec<u8>, Vec<u8>)> {
            let request: TransactionRequest = self.0.next_transaction();
            assert!(request.conditions.is_empty());
            request
                .mutations
                .into_iter()
                .map(|mutation| match mutation {
                    TransactionMutation::Put { key, value } => {
                        let mut bytes = key.pk.as_bytes().to_vec();
                        bytes.extend_from_slice(key.sk.as_bytes());
                        (bytes, value)
                    }
                    TransactionMutation::Delete { .. } => panic!("unexpected delete"),
                })
                .collect()
        }

        pub fn seed_rows(distribution: &str, working_set: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
            let generator = Self::new(distribution, working_set, 25.min(working_set), 0, 0);
            generator
                .0
                .seed_keys()
                .enumerate()
                .map(|(index, key)| {
                    let mut bytes = key.pk.as_bytes().to_vec();
                    bytes.extend_from_slice(key.sk.as_bytes());
                    (bytes, value_bytes(64, index as u64, 0))
                })
                .collect()
        }
    }
}

use original::OriginalGenerator;
use serde_json::{Value, json};
use workload::{
    Distribution, TraceHash, WorkloadConfig, WorkloadGenerator, invocation_seed, seed_rows,
    writer_phase_seed,
};

fn original_trace(distribution: &str, working_set: usize, width: usize, seed: u64, writer: usize, count: u64) -> TraceHash {
    let mut generator = OriginalGenerator::new(distribution, working_set, width, seed, writer);
    let mut trace = TraceHash::new();
    for _ in 0..count {
        let transaction = generator.next_transaction();
        trace.push_transaction(transaction.iter().map(|(key, value)| (key.as_slice(), value.as_slice())));
    }
    trace
}

fn shared_trace(distribution: &str, working_set: usize, width: usize, seed: u64, writer: usize, count: u64) -> TraceHash {
    let config = WorkloadConfig {
        distribution: Distribution::parse(distribution),
        working_set,
        key_size: 16,
        value_size: 64,
        width,
    };
    let mut generator = WorkloadGenerator::new(config, seed, writer);
    let mut trace = TraceHash::new();
    for _ in 0..count {
        let transaction = generator.next_transaction();
        trace.push_transaction(transaction.iter().map(|mutation| (mutation.key.as_slice(), mutation.value.as_slice())));
    }
    trace
}

fn scenarios() -> Vec<(u64, usize, usize, &'static str, usize)> {
    let mut list = Vec::new();
    let mut index = 0u64;
    for writers in [16usize, 64] {
        for width in [1usize, 16] {
            for distribution in ["uniform", "same-leaf-heavy", "different-leaf-heavy"] {
                list.push((index, writers, width, distribution, 100_000));
                index += 1;
            }
        }
    }
    for width in [1usize, 16] {
        list.push((index, 1, width, "uniform", 100_000));
        index += 1;
    }
    list.push((14, 16, 1, "uniform", 1_000_000));
    list.push((15, 64, 1, "uniform", 1_000_000));
    list
}

fn synthetic(transactions_per_writer: u64) {
    println!("seed formula in phase0-bench run(): {}", include_str!(concat!(env!("OUT_DIR"), "/seed_formula.txt")).trim());
    println!("seed row value in phase0-bench seed_requests(): {}", include_str!(concat!(env!("OUT_DIR"), "/seed_value.txt")).trim());
    println!("writer seed masks present in phase0-bench:\n{}", include_str!(concat!(env!("OUT_DIR"), "/writer_masks.txt")));
    let mut compared = 0u64;
    let mut mismatched = 0u64;
    for (scenario_index, writers, width, distribution, working_set) in scenarios() {
        for repetition in 0..3u64 {
            let seed = invocation_seed(scenario_index, repetition);
            for warmup in [true, false] {
                let phase_seed = writer_phase_seed(seed, warmup);
                let mut combined_original = TraceHash::new();
                let mut combined_shared = TraceHash::new();
                for writer in 0..writers {
                    let left = original_trace(distribution, working_set, width, phase_seed, writer, transactions_per_writer);
                    let right = shared_trace(distribution, working_set, width, phase_seed, writer, transactions_per_writer);
                    compared += 1;
                    if left.state != right.state || left.transactions != right.transactions {
                        mismatched += 1;
                    }
                    combined_original.push_transaction([(&left.state.to_be_bytes()[..], &[][..])]);
                    combined_shared.push_transaction([(&right.state.to_be_bytes()[..], &[][..])]);
                }
                println!(
                    "scenario={scenario_index:02} writers={writers} width={width} distribution={distribution} working_set={working_set} repetition={} phase={} invocation_seed={seed} writer_seed={phase_seed} transactions_per_writer={transactions_per_writer} original={:016x} shared={:016x} equal={}",
                    repetition + 1,
                    if warmup { "warmup" } else { "measured" },
                    combined_original.state,
                    combined_shared.state,
                    combined_original.state == combined_shared.state,
                );
            }
        }
    }
    for distribution in ["uniform", "same-leaf-heavy", "different-leaf-heavy"] {
        for working_set in [100_000usize, 1_000_000] {
            let original_rows = OriginalGenerator::seed_rows(distribution, working_set);
            let config = WorkloadConfig {
                distribution: Distribution::parse(distribution),
                working_set,
                key_size: 16,
                value_size: 64,
                width: 1,
            };
            let shared_rows: Vec<(Vec<u8>, Vec<u8>)> = seed_rows(&config).map(|mutation| (mutation.key, mutation.value)).collect();
            let mut original_hash = TraceHash::new();
            let mut shared_hash = TraceHash::new();
            for (key, value) in &original_rows {
                original_hash.push_transaction([(key.as_slice(), value.as_slice())]);
            }
            for (key, value) in &shared_rows {
                shared_hash.push_transaction([(key.as_slice(), value.as_slice())]);
            }
            let equal = original_rows == shared_rows;
            if !equal {
                mismatched += 1;
            }
            println!(
                "seed-rows distribution={distribution} working_set={working_set} rows={} original={:016x} shared={:016x} equal={equal} first_key={}",
                original_rows.len(),
                original_hash.state,
                shared_hash.state,
                original_rows[0].0.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            );
        }
    }
    println!("writer streams compared={compared} mismatched_streams_or_seed_sets={mismatched}");
    if mismatched > 0 {
        std::process::exit(1);
    }
}

fn verify_raw(paths: &[String]) {
    let mut records = 0u64;
    let mut streams = 0u64;
    let mut transactions = 0u64;
    let mut failures = 0u64;
    for path in paths {
        let text = std::fs::read_to_string(path).expect("raw file should be readable");
        for line in text.lines() {
            let record: Value = serde_json::from_str(line).expect("json line");
            if record["record_type"] != json!("crossdb-run") {
                continue;
            }
            records += 1;
            let seed = record["seed"].as_u64().unwrap();
            let scenario_index = record["scenario_index"].as_u64().unwrap();
            let repetition = record["repetition"].as_u64().unwrap();
            if seed != invocation_seed(scenario_index, repetition - 1) {
                println!("{path}: seed {seed} does not match scenario {scenario_index} repetition {repetition}");
                failures += 1;
            }
            let distribution = record["distribution"].as_str().unwrap();
            let working_set = record["working_set"].as_u64().unwrap() as usize;
            let width = record["transaction_width"].as_u64().unwrap() as usize;
            for trace in record["traces"].as_array().unwrap() {
                let writer = trace["writer"].as_u64().unwrap() as usize;
                let warmup = trace["phase"] == json!("warmup");
                let count = trace["transactions"].as_u64().unwrap();
                let expected = original_trace(distribution, working_set, width, writer_phase_seed(seed, warmup), writer, count);
                streams += 1;
                transactions += count;
                if format!("{:016x}", expected.state) != trace["hash"].as_str().unwrap() {
                    println!("{path}: writer {writer} phase {} hash mismatch", trace["phase"]);
                    failures += 1;
                }
            }
        }
    }
    println!("raw records={records} writer_streams={streams} generated_transactions={transactions} failures={failures}");
    if failures > 0 {
        std::process::exit(1);
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("synthetic") => synthetic(arguments.get(1).map_or(2_000, |value| value.parse().unwrap())),
        Some("verify-raw") => verify_raw(&arguments[1..]),
        _ => panic!("usage: workload-equivalence synthetic [transactions] | verify-raw <files>"),
    }
}
