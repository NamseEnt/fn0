use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dodb_core::{
    DocumentKey, RevisionState, TenantId, TransactionCondition, TransactionMutation,
    TransactionRequest,
};
use dodb_rocksdb::{RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use serde_json::json;
use tokio::sync::Barrier;

const CLIENTS: usize = 16;
const CLIENT_CALL_LIMIT: usize = 20_000;
const MEASUREMENT_BASE: u64 = 1 << 32;
const BUDGET: ExecutionBudget = ExecutionBudget::new(1 << 20);

struct ClientResult {
    client_index: usize,
    calls: usize,
    revision: u64,
    fingerprint: u64,
    latency_nanos: Vec<u64>,
}

fn key(client_index: usize) -> DocumentKey {
    DocumentKey::new(
        b"overwrite-probe".to_vec(),
        (client_index as u64).to_be_bytes().to_vec(),
    )
}

fn value(client_index: usize, sequence: u64) -> Vec<u8> {
    let mut bytes = vec![0x5a; 512];
    bytes[..8].copy_from_slice(&(client_index as u64).to_be_bytes());
    bytes[8..16].copy_from_slice(&sequence.to_be_bytes());
    bytes
}

async fn phase(
    service: &Arc<RocksDbService>,
    conditional: bool,
    measured: bool,
) -> Result<(Vec<ClientResult>, Duration), String> {
    let duration = Duration::from_secs(if measured { 5 } else { 1 });
    let started = Instant::now();
    let deadline = started + duration;
    let barrier = Arc::new(Barrier::new(CLIENTS + 1));
    let mut tasks = tokio::task::JoinSet::new();
    for client_index in 0..CLIENTS {
        let service = Arc::clone(service);
        let barrier = Arc::clone(&barrier);
        tasks.spawn(async move {
            let mut result = ClientResult {
                client_index,
                calls: 0,
                revision: 0,
                fingerprint: 0xcbf2_9ce4_8422_2325,
                latency_nanos: Vec::with_capacity(2_000),
            };
            barrier.wait().await;
            while Instant::now() < deadline && result.calls < CLIENT_CALL_LIMIT {
                let sequence = if measured { MEASUREMENT_BASE } else { 0 } + result.calls as u64;
                let target = key(client_index);
                let payload = value(client_index, sequence);
                if result.calls < 100 {
                    for byte in (client_index as u64)
                        .to_be_bytes()
                        .into_iter()
                        .chain(sequence.to_be_bytes())
                        .chain(payload.iter().copied())
                    {
                        result.fingerprint ^= byte as u64;
                        result.fingerprint = result.fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
                    }
                }
                let request = if conditional {
                    Request::Transact {
                        request: TransactionRequest::new(
                            vec![TransactionCondition::Exists {
                                key: target.clone(),
                            }],
                            vec![TransactionMutation::Put {
                                key: target,
                                value: payload,
                            }],
                        ),
                    }
                } else {
                    Request::Put {
                        key: target,
                        value: payload,
                    }
                };
                let call_started = Instant::now();
                let response = service
                    .execute(TenantId::new(1), request, BUDGET)
                    .await
                    .map_err(|error| error.to_string())?;
                result
                    .latency_nanos
                    .push(call_started.elapsed().as_nanos() as u64);
                let revision = match response {
                    Response::Put(revision) => revision.get(),
                    Response::Transact(outcome) => outcome
                        .commit_lsn
                        .ok_or("overwrite has no commit identity")?
                        .get(),
                    _ => return Err("unexpected overwrite response".to_owned()),
                };
                if revision <= result.revision {
                    return Err("per-key revision did not advance".to_owned());
                }
                result.revision = revision;
                result.calls += 1;
            }
            if result.calls == CLIENT_CALL_LIMIT {
                return Err("client call cap reached".to_owned());
            }
            Ok::<_, String>(result)
        });
    }
    barrier.wait().await;
    let mut results = Vec::with_capacity(CLIENTS);
    while let Some(result) = tasks.join_next().await {
        results.push(result.map_err(|error| error.to_string())??);
    }
    results.sort_by_key(|result| result.client_index);
    Ok((results, started.elapsed()))
}

async fn validate(service: &RocksDbService, clients: &[ClientResult]) -> Result<(), String> {
    let mut expected = BTreeMap::new();
    for client in clients {
        if client.calls < 100 {
            return Err("fewer than 100 measured workflows".to_owned());
        }
        let target = key(client.client_index);
        let payload = value(
            client.client_index,
            MEASUREMENT_BASE + client.calls as u64 - 1,
        );
        match service
            .execute(
                TenantId::new(1),
                Request::Get {
                    key: target.clone(),
                },
                BUDGET,
            )
            .await
            .map_err(|error| error.to_string())?
        {
            Response::Get(RevisionState::Present { value, revision })
                if value == payload && revision.get() == client.revision => {}
            _ => return Err("overwrite final value or revision mismatch".to_owned()),
        }
        expected.insert(target, payload);
    }
    match service
        .execute(
            TenantId::new(1),
            Request::Scan {
                exclusive_after_key: None,
                limit: CLIENTS + 1,
            },
            BUDGET,
        )
        .await
        .map_err(|error| error.to_string())?
    {
        Response::Scan(rows) => {
            if rows.len() != CLIENTS
                || rows
                    .into_iter()
                    .map(|row| (row.key, row.value))
                    .collect::<BTreeMap<_, _>>()
                    != expected
            {
                return Err("overwrite full-state mismatch".to_owned());
            }
            Ok(())
        }
        _ => Err("overwrite full-state mismatch".to_owned()),
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let mut options = BTreeMap::new();
    while let Some(name) = arguments.next() {
        options.insert(name, arguments.next().ok_or("missing option value")?);
    }
    let data_dir = PathBuf::from(options.get("--data-dir").ok_or("missing data directory")?);
    if data_dir.exists() {
        return Err("probe data directory must be new".to_owned());
    }
    let mode = options.get("--mode").ok_or("missing mode")?;
    let conditional = match mode.as_str() {
        "put" => false,
        "exists" => true,
        _ => return Err("invalid mode".to_owned()),
    };
    let enabled = options.get("--metrics").is_some_and(|value| value == "1");
    let mut config = RocksDbServiceConfig::new(&data_dir);
    config.revision_allocator = RevisionAllocatorMode::ReservedRanges;
    config.research_metrics = enabled;
    let service = Arc::new(RocksDbService::new(config.clone()).map_err(|error| error.to_string())?);
    service
        .execute(
            TenantId::new(1),
            Request::Transact {
                request: TransactionRequest::new(
                    Vec::new(),
                    (0..CLIENTS)
                        .map(|client_index| TransactionMutation::Put {
                            key: key(client_index),
                            value: value(client_index, u64::MAX),
                        })
                        .collect(),
                ),
            },
            BUDGET,
        )
        .await
        .map_err(|error| error.to_string())?;
    service.set_research_warmup_phase();
    phase(&service, conditional, false).await?;
    service.set_research_measurement_phase();
    let (clients, elapsed) = phase(&service, conditional, true).await?;
    let metrics = service
        .research_metrics_snapshot()
        .unwrap_or_else(|| json!({"enabled": false}));
    validate(&service, &clients).await?;
    service.shutdown().await;
    config.research_metrics = false;
    let reopened = RocksDbService::new(config).map_err(|error| error.to_string())?;
    validate(&reopened, &clients).await?;
    reopened.shutdown().await;
    let calls = clients.iter().map(|client| client.calls).sum::<usize>();
    let mut latencies = clients
        .iter()
        .flat_map(|client| client.latency_nanos.iter().copied())
        .collect::<Vec<_>>();
    latencies.sort_unstable();
    let percentile = |percent: usize| {
        latencies[((latencies.len() - 1) * percent).div_ceil(100)] as f64 / 1_000_000.0
    };
    let report = json!({
        "schema": "dodb-wrapper-overwrite-probe-v1", "mode": mode, "clients": CLIENTS,
        "value_bytes": 512, "initial_keys": CLIENTS, "warmup_seconds": 1, "measurement_seconds": 5,
        "revision_allocator": "reserved-ranges", "shared_slots": 8,
        "calls": calls, "elapsed_seconds_including_drain": elapsed.as_secs_f64(),
        "calls_per_second": calls as f64 / elapsed.as_secs_f64(),
        "latency": {"samples": latencies.len(), "p50_ms": percentile(50), "p95_ms": percentile(95), "p99_ms": percentile(99)},
        "client_prefixes": clients.iter().map(|client| json!({"client": client.client_index, "workflows": 100, "fingerprint": format!("{:016x}", client.fingerprint)})).collect::<Vec<_>>(),
        "research_metrics": metrics, "validation": true, "reopen_validation": true,
    });
    let text = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    std::fs::write(options.get("--report").ok_or("missing report path")?, &text)
        .map_err(|error| error.to_string())?;
    println!("{text}");
    Ok(())
}
