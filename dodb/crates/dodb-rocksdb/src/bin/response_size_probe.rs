use std::hint::black_box;
use std::time::Instant;

use dodb_core::{DocumentKey, Lsn, Revision, RevisionState};
use dodb_protocol::{
    ProtocolLimits, ResponseEnvelope, encode_response, encoded_success_response_size,
};
use dodb_service::{Document, Response, TransactionOutcome};
use serde_json::json;

fn check_equivalence(response: &Response) -> Result<usize, String> {
    let default_limits = ProtocolLimits::default();
    let mut budgets = vec![
        0,
        10,
        11,
        12,
        13,
        14,
        17,
        20,
        21,
        22,
        25,
        26,
        default_limits.max_response_frame_size,
    ];
    if let Ok(length) = encoded_success_response_size(response, default_limits) {
        budgets.extend([length.saturating_sub(1), length, length + 1]);
    }
    budgets.sort_unstable();
    budgets.dedup();
    let mut checks = 0;
    for budget in budgets {
        let base = ProtocolLimits {
            max_response_frame_size: budget,
            ..default_limits
        };
        let limits_to_check = [
            base,
            ProtocolLimits {
                max_value_size: 1,
                ..base
            },
            ProtocolLimits {
                max_key_component_size: 1,
                ..base
            },
            ProtocolLimits {
                max_encoded_key_size: 1,
                ..base
            },
            ProtocolLimits {
                max_scan_limit: 1,
                ..base
            },
        ];
        for limits in limits_to_check {
            let materialized =
                encode_response(&ResponseEnvelope::Success(response.clone()), limits)
                    .map(|bytes| bytes.len());
            let counted = encoded_success_response_size(response, limits);
            if materialized != counted {
                return Err(format!(
                    "size/error mismatch: {materialized:?} versus {counted:?}; limits={limits:?}"
                ));
            }
            checks += 1;
        }
    }
    Ok(checks)
}

fn main() -> Result<(), String> {
    let mut checks = 0;
    for revision in [Revision::ZERO, Revision::new(u64::MAX)] {
        for response in [
            Response::Get(RevisionState::missing(revision)),
            Response::Put(revision),
            Response::Delete(revision),
            Response::Transact(TransactionOutcome::conditions_satisfied()),
            Response::Transact(TransactionOutcome::committed(Lsn::new(revision.get()))),
            Response::Query(Vec::new()),
            Response::Scan(Vec::new()),
        ] {
            checks += check_equivalence(&response)?;
        }
    }
    for value_size in [
        0,
        1,
        512,
        2_000,
        65_536,
        1_048_576,
        64 * 1_048_576,
        64 * 1_048_576 + 1,
    ] {
        checks += check_equivalence(&Response::Get(RevisionState::present(
            vec![0x5a; value_size],
            Revision::new(42),
        )))?;
    }
    for row_count in [1, 2, 16] {
        let rows = (0..row_count)
            .map(|row_index| Document {
                key: DocumentKey::new(
                    vec![0, 0xff, row_index as u8],
                    vec![0xff, 0, row_index as u8],
                ),
                value: vec![row_index as u8; 512],
                revision: Revision::new(row_index as u64 + 1),
            })
            .collect::<Vec<_>>();
        checks += check_equivalence(&Response::Query(rows.clone()))?;
        checks += check_equivalence(&Response::Scan(rows))?;
    }
    checks += check_equivalence(&Response::Query(vec![Document {
        key: DocumentKey::new(vec![b'x'; 3_991], Vec::new()),
        value: Vec::new(),
        revision: Revision::new(1),
    }]))?;
    let mut timings = Vec::new();
    for (value_size, iterations) in [(512, 100_000), (65_536, 5_000), (1_048_576, 500)] {
        let response = Response::Get(RevisionState::present(
            vec![0x5a; value_size],
            Revision::new(42),
        ));
        for repetition in 1..=3 {
            let mut duration = [0_u128; 2];
            let order = if repetition == 2 { [1, 0] } else { [0, 1] };
            for method_index in order {
                let started = Instant::now();
                for _ in 0..iterations {
                    let borrowed = black_box(&response);
                    let limits = black_box(ProtocolLimits::default());
                    if method_index == 0 {
                        black_box(
                            encode_response(&ResponseEnvelope::Success(borrowed.clone()), limits)
                                .map_err(|error| error.to_string())?,
                        );
                    } else {
                        black_box(
                            encoded_success_response_size(borrowed, limits)
                                .map_err(|error| error.to_string())?,
                        );
                    }
                }
                duration[method_index] = started.elapsed().as_nanos();
            }
            timings.push(
                json!({"value_size": value_size, "iterations": iterations, "repetition": repetition,
                "clone_and_encode_ns_per_call": duration[0] as f64 / iterations as f64,
                "count_ns_per_call": duration[1] as f64 / iterations as f64}),
            );
        }
    }
    println!("{}", serde_json::to_string_pretty(&json!({
        "schema": "dodb-response-size-probe-v1", "passed": true,
        "equivalence_checks": checks, "timings": timings,
        "scope": "canonical serializer versus counted success response; identical sizes and errors at frame and field boundaries; isolated CPU probe, not service latency"
    })).map_err(|error| error.to_string())?);
    Ok(())
}
