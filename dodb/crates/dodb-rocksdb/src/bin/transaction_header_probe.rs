use std::hint::black_box;
use std::time::Instant;

use dodb_core::Error;
use dodb_rocksdb::research::{
    header_transaction_observation, materialized_transaction_observation,
};
use serde_json::json;

fn check_projection(encoded: &[u8]) -> Result<bool, String> {
    match (
        materialized_transaction_observation(encoded),
        header_transaction_observation(encoded),
    ) {
        (Ok(materialized), Ok(header)) if materialized == header => Ok(true),
        (Err(Error::Corruption(materialized)), Err(Error::Corruption(header)))
            if materialized == header =>
        {
            Ok(false)
        }
        (materialized, header) => Err(format!(
            "projection mismatch: {materialized:?} versus {header:?}; length={}",
            encoded.len()
        )),
    }
}

fn main() -> Result<(), String> {
    let mut valid = 0_u64;
    let mut malformed = 0_u64;
    let mut observe = |encoded: &[u8]| -> Result<(), String> {
        if check_projection(encoded)? {
            valid += 1;
        } else {
            malformed += 1;
        }
        Ok(())
    };
    for length in 0..=32 {
        for format in 0..=3 {
            for tag in 0..=3 {
                for revision in [0_u64, 1, u64::MAX] {
                    let mut encoded = vec![0x5a; length];
                    if length > 0 {
                        encoded[0] = format;
                    }
                    if length > 1 {
                        encoded[1] = tag;
                    }
                    if length >= 10 {
                        encoded[2..10].copy_from_slice(&revision.to_be_bytes());
                    }
                    observe(&encoded)?;
                }
            }
        }
    }
    let mut random_state = 0x91e1_0da5_1234_5678_u64;
    for record_index in 0..100_000 {
        random_state ^= random_state << 13;
        random_state ^= random_state >> 7;
        random_state ^= random_state << 17;
        let length = (random_state as usize) % 65;
        let mut encoded = vec![0; length];
        for byte in &mut encoded {
            random_state = random_state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            *byte = (random_state >> 32) as u8;
        }
        if length >= 2 && record_index % 3 == 0 {
            encoded[0] = 1;
            encoded[1] = if record_index % 2 == 0 { 1 } else { 2 };
        }
        observe(&encoded)?;
    }
    for value_size in [0, 512, 65_536, 1_048_576] {
        let mut encoded = vec![0x5a; value_size + 10];
        encoded[0] = 1;
        encoded[1] = 1;
        observe(&encoded)?;
        encoded[1] = 2;
        observe(&encoded)?;
    }
    let mut timings = Vec::new();
    for (value_size, iterations) in [(512, 100_000), (65_536, 5_000), (1_048_576, 500)] {
        let mut encoded = vec![0x5a; value_size + 10];
        encoded[0] = 1;
        encoded[1] = 1;
        for repetition in 1..=3 {
            let mut elapsed = [0_u128; 2];
            for method_index in if repetition == 2 { [1, 0] } else { [0, 1] } {
                let started = Instant::now();
                for _ in 0..iterations {
                    let input = black_box(encoded.as_slice());
                    let observed = if method_index == 0 {
                        materialized_transaction_observation(input)
                    } else {
                        header_transaction_observation(input)
                    };
                    black_box(observed.map_err(|error| error.to_string())?);
                }
                elapsed[method_index] = started.elapsed().as_nanos();
            }
            timings.push(
                json!({"value_size": value_size, "iterations": iterations, "repetition": repetition,
                "materialized_decode_ns_per_call": elapsed[0] as f64 / iterations as f64,
                "header_decode_ns_per_call": elapsed[1] as f64 / iterations as f64}),
            );
        }
    }
    println!("{}", serde_json::to_string_pretty(&json!({
        "schema": "dodb-transaction-header-projection-v1", "passed": true,
        "equivalence_checks": valid + malformed, "valid_records": valid, "malformed_records": malformed,
        "timings": timings,
        "scope": "presence/revision and corruption errors versus original full-state decoder; isolated Rust decoder projection, excludes native reads, locks, FFI copying and service I/O"
    })).map_err(|error| error.to_string())?);
    Ok(())
}
