use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dodb_core::{
    DocumentKey, Revision, RevisionState, TenantId, TransactionMutation, TransactionRequest,
};
use dodb_rocksdb::{RocksDbService, RocksDbServiceConfig};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use serde_json::{Value, json};
use tokio::sync::Barrier;

const CLIENTS: usize = 16;
const CALL_CAP: usize = 100_000;
const BUDGET: ExecutionBudget = ExecutionBudget::new(68 * 1024 * 1024);

struct ClientResult {
    client: usize,
    latency: Vec<u64>,
}

fn key(client: usize) -> DocumentKey {
    DocumentKey::new(
        b"get-copy-probe".to_vec(),
        (client as u64).to_be_bytes().to_vec(),
    )
}

fn expected_value(client: usize, size: usize) -> Vec<u8> {
    let mut value = vec![0x5a; size];
    value[..8].copy_from_slice(&(client as u64).to_be_bytes());
    value
}

fn process_metrics() -> Value {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let rss = status.lines().find_map(|line| {
        line.strip_prefix("VmRSS:")
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
    });
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let fields = stat.split_whitespace().collect::<Vec<_>>();
    let ticks = if fields.len() > 15 {
        fields[13].parse::<u64>().unwrap_or(0) + fields[14].parse::<u64>().unwrap_or(0)
    } else {
        0
    };
    json!({"rss_kb":rss,"cpu_ticks":ticks})
}

async fn phase(
    service: &Arc<RocksDbService>,
    values: &Arc<Vec<Vec<u8>>>,
    revisions: &Arc<Vec<Revision>>,
    measured: bool,
) -> Result<(Vec<ClientResult>, Duration), String> {
    let duration = Duration::from_secs(if measured { 5 } else { 1 });
    let barrier = Arc::new(Barrier::new(CLIENTS + 1));
    let mut tasks = tokio::task::JoinSet::new();
    let start = Instant::now();
    let deadline = start + duration;
    for client in 0..CLIENTS {
        let service = Arc::clone(service);
        let values = Arc::clone(values);
        let revisions = Arc::clone(revisions);
        let barrier = Arc::clone(&barrier);
        tasks.spawn(async move {
            let mut result = ClientResult {
                client,
                latency: Vec::new(),
            };
            barrier.wait().await;
            while Instant::now() < deadline && result.latency.len() < CALL_CAP {
                let started = Instant::now();
                let response = service
                    .execute(TenantId::new(1), Request::Get { key: key(client) }, BUDGET)
                    .await
                    .map_err(|error| error.to_string())?;
                result.latency.push(started.elapsed().as_nanos() as u64);
                match response {
                    Response::Get(RevisionState::Present { value, revision })
                        if value == values[client] && revision == revisions[client] => {}
                    _ => return Err("read value or revision mismatch".to_owned()),
                }
            }
            if result.latency.len() == CALL_CAP {
                return Err("client read cap reached".to_owned());
            }
            Ok::<_, String>(result)
        });
    }
    barrier.wait().await;
    let mut clients = Vec::new();
    while let Some(result) = tasks.join_next().await {
        clients.push(result.map_err(|error| error.to_string())??);
    }
    clients.sort_by_key(|result| result.client);
    Ok((clients, start.elapsed()))
}

async fn validate(
    service: &RocksDbService,
    values: &[Vec<u8>],
    revisions: &[Revision],
) -> Result<(), String> {
    let response = service
        .execute(
            TenantId::new(1),
            Request::Scan {
                exclusive_after_key: None,
                limit: CLIENTS + 1,
            },
            BUDGET,
        )
        .await
        .map_err(|error| error.to_string())?;
    let Response::Scan(rows) = response else {
        return Err("unexpected validation response".to_owned());
    };
    if rows.len() != CLIENTS {
        return Err("final read dataset has a different key count".to_owned());
    }
    for (client, row) in rows.into_iter().enumerate() {
        if row.key != key(client)
            || row.value != values[client]
            || row.revision != revisions[client]
        {
            return Err("final read dataset mismatch".to_owned());
        }
    }
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut options = std::collections::BTreeMap::new();
    while let Some(name) = args.next() {
        options.insert(name, args.next().ok_or("missing option value")?);
    }
    let size = options
        .get("--value-size")
        .ok_or("missing value size")?
        .parse::<usize>()
        .map_err(|error| error.to_string())?;
    if ![512, 65_536, 1_048_576].contains(&size) {
        return Err("unsupported read payload size".to_owned());
    }
    let data_dir = PathBuf::from(options.get("--data-dir").ok_or("missing data directory")?);
    if data_dir.exists() {
        return Err("read probe directory must be new".to_owned());
    }
    let enabled = options.get("--metrics").is_some_and(|value| value == "1");
    let mut config = RocksDbServiceConfig::new(&data_dir);
    config.research_metrics = enabled;
    let service = Arc::new(RocksDbService::new(config.clone()).map_err(|error| error.to_string())?);
    let values = Arc::new(
        (0..CLIENTS)
            .map(|client| expected_value(client, size))
            .collect::<Vec<_>>(),
    );
    service
        .execute(
            TenantId::new(1),
            Request::Transact {
                request: TransactionRequest::new(
                    Vec::new(),
                    (0..CLIENTS)
                        .map(|client| TransactionMutation::Put {
                            key: key(client),
                            value: values[client].clone(),
                        })
                        .collect(),
                ),
            },
            BUDGET,
        )
        .await
        .map_err(|error| error.to_string())?;
    let mut revisions = Vec::new();
    let mut fingerprints = Vec::new();
    for client in 0..CLIENTS {
        let response = service
            .execute(TenantId::new(1), Request::Get { key: key(client) }, BUDGET)
            .await
            .map_err(|error| error.to_string())?;
        match response {
            Response::Get(RevisionState::Present { value, revision })
                if value == values[client] =>
            {
                revisions.push(revision)
            }
            _ => return Err("seeded read value mismatch".to_owned()),
        }
        let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
        for byte in key(client).encode().iter().chain(values[client].iter()) {
            fingerprint ^= *byte as u64;
            fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        }
        fingerprints.push(format!("{fingerprint:016x}"));
    }
    let revisions = Arc::new(revisions);
    service.set_research_warmup_phase();
    phase(&service, &values, &revisions, false).await?;
    service.set_research_measurement_phase();
    let before = process_metrics();
    let samples = Arc::new(Mutex::new(Vec::new()));
    let stopped = Arc::new(AtomicBool::new(false));
    let sample_values = Arc::clone(&samples);
    let sample_stop = Arc::clone(&stopped);
    let sampler = tokio::spawn(async move {
        while !sample_stop.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(200)).await;
            if !sample_stop.load(Ordering::Relaxed) {
                sample_values.lock().unwrap().push(process_metrics());
            }
        }
    });
    let (clients, elapsed) = phase(&service, &values, &revisions, true).await?;
    let after = process_metrics();
    let metrics = service
        .research_metrics_snapshot()
        .unwrap_or_else(|| json!({"enabled":false}));
    stopped.store(true, Ordering::Relaxed);
    sampler.await.map_err(|error| error.to_string())?;
    let samples = samples.lock().unwrap();
    let peak_rss = samples
        .iter()
        .filter_map(|sample| sample["rss_kb"].as_u64())
        .max();
    let ticks = after["cpu_ticks"]
        .as_u64()
        .unwrap()
        .saturating_sub(before["cpu_ticks"].as_u64().unwrap());
    let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as f64;
    let cpu_seconds = ticks as f64 / ticks_per_second;
    validate(&service, &values, &revisions).await?;
    service.shutdown().await;
    config.research_metrics = false;
    let reopened = RocksDbService::new(config).map_err(|error| error.to_string())?;
    validate(&reopened, &values, &revisions).await?;
    reopened.shutdown().await;
    let mut latencies = clients
        .iter()
        .flat_map(|client| client.latency.iter().copied())
        .collect::<Vec<_>>();
    latencies.sort_unstable();
    let calls = latencies.len();
    if clients.iter().any(|client| client.latency.is_empty()) {
        return Err("empty measured client".to_owned());
    }
    let percentile =
        |percent: usize| latencies[((calls - 1) * percent).div_ceil(100)] as f64 / 1_000_000.0;
    let report = json!({"schema":"dodb-get-copy-probe-v1","value_size":size,"clients":CLIENTS,"initial_keys":CLIENTS,
        "warmup_seconds":1,"measurement_seconds":5,"shared_slots":8,"revision_allocator":"serialized",
        "calls":calls,"calls_per_second":calls as f64/elapsed.as_secs_f64(),"elapsed_seconds_including_drain":elapsed.as_secs_f64(),
        "latency":{"samples":calls,"p50_ms":percentile(50),"p95_ms":percentile(95),"p99_ms":percentile(99)},
        "cpu_seconds":cpu_seconds,"process_cpu_us_per_get":cpu_seconds*1_000_000.0/calls as f64,
        "peak_sampled_rss_kb":peak_rss,"process_samples":samples.len(),"sampling_complete":samples.len()>=2&&peak_rss.is_some(),
        "input_fingerprints":fingerprints,"validation":true,"reopen_validation":true,
        "validation_scope":"every measured value and revision checked; complete final and reopened ordered state",
        "cpu_scope":"measurement plus drain; includes driver full-value comparisons; excludes setup and final validation",
        "research_metrics":metrics});
    let text = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    std::fs::write(options.get("--report").ok_or("missing report path")?, &text)
        .map_err(|error| error.to_string())?;
    println!("{text}");
    Ok(())
}
