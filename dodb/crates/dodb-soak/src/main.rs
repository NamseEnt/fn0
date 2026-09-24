use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dodb_client::{ClientError, ClientTlsConfig, DodbClient, DodbConnection};
use dodb_core::{
    ConditionExpectation, DocumentKey, ObservedState, PrimaryKey, Revision, RevisionState,
    TenantId, TransactionCondition, TransactionMutation, TransactionRequest,
};
use dodb_protocol::{ApplicationErrorKind, ConflictDetails, ProtocolLimits};
use dodb_server::{
    DodbServer, DodbServerConfig, LocalTenantService, LocalTenantServiceConfig, ServerTlsConfig,
};
use dodb_service::{Document, TransactionOutcome};
use rcgen::generate_simple_self_signed;
use serde::Serialize;
use tokio::process::{Child, Command};
use tokio::sync::{Mutex as AsyncMutex, mpsc, watch};
use tokio::task::JoinSet;

use dodb_soak::{
    DeterministicRng, ExpectedDocument, ExpectedState, GeneratedOperation, LatencyStats,
    OperationGenerator, OperationRecord, PlannedMutation, PlannedMutationKind, RecentOperations,
    ReferenceModel, ResourceSample, TrendDiagnostic, UnknownMutationResolution, WorkloadConfig,
    parse_event_lines, reconcile_unknown_operations, trend_diagnostic,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

const SERVER_CHILD: &str = "--server-child";
const SERVER_NAME: &str = "localhost";

#[derive(Clone, Debug)]
struct HarnessConfig {
    profile: String,
    phase: String,
    seed: u64,
    duration_override: Option<Duration>,
    output_dir: PathBuf,
    data_dir: Option<PathBuf>,
    tenant_count: u64,
    concurrency: usize,
    hot_percent: Option<u32>,
    operation_weights: Option<[u32; 7]>,
    checkpoint_interval: Duration,
    connection_churn: bool,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            profile: "smoke".to_owned(),
            phase: "all".to_owned(),
            seed: 1,
            duration_override: None,
            output_dir: PathBuf::from("target/dodb-soak"),
            data_dir: None,
            tenant_count: 0,
            concurrency: 0,
            hot_percent: None,
            operation_weights: None,
            checkpoint_interval: Duration::from_secs(3),
            connection_churn: true,
        }
    }
}

impl HarnessConfig {
    fn parse() -> Result<Self, String> {
        let mut config = Self::default();
        let mut args = env::args().skip(1);
        while let Some(argument) = args.next() {
            let value = |name: &str, args: &mut std::iter::Skip<std::env::Args>| {
                args.next()
                    .ok_or_else(|| format!("{name} requires a value"))
            };
            match argument.as_str() {
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                "--profile" => config.profile = value("--profile", &mut args)?,
                "--phase" => config.phase = value("--phase", &mut args)?,
                "--seed" => {
                    config.seed = value("--seed", &mut args)?
                        .parse()
                        .map_err(|_| "--seed must be an unsigned integer".to_owned())?;
                }
                "--duration" => {
                    config.duration_override =
                        Some(parse_duration(&value("--duration", &mut args)?)?);
                }
                "--output" => config.output_dir = PathBuf::from(value("--output", &mut args)?),
                "--data-dir" => {
                    config.data_dir = Some(PathBuf::from(value("--data-dir", &mut args)?))
                }
                "--tenants" => {
                    config.tenant_count = value("--tenants", &mut args)?
                        .parse()
                        .map_err(|_| "--tenants must be an unsigned integer".to_owned())?;
                }
                "--concurrency" => {
                    config.concurrency = value("--concurrency", &mut args)?
                        .parse()
                        .map_err(|_| "--concurrency must be an unsigned integer".to_owned())?;
                }
                "--hot-percent" => {
                    config.hot_percent = Some(
                        value("--hot-percent", &mut args)?
                            .parse()
                            .map_err(|_| "--hot-percent must be an unsigned integer".to_owned())?,
                    );
                }
                "--weights" => {
                    config.operation_weights =
                        Some(parse_weights(&value("--weights", &mut args)?)?);
                }
                "--checkpoint-ms" => {
                    let milliseconds: u64 = value("--checkpoint-ms", &mut args)?
                        .parse()
                        .map_err(|_| "--checkpoint-ms must be an unsigned integer".to_owned())?;
                    config.checkpoint_interval = Duration::from_millis(milliseconds.max(1));
                }
                "--no-connection-churn" => config.connection_churn = false,
                argument if argument.starts_with('-') => {
                    return Err(format!("unknown option {argument}; use --help for usage"));
                }
                _ => return Err(format!("unexpected argument {argument}")),
            }
        }
        if !matches!(config.profile.as_str(), "smoke" | "accelerated") {
            return Err("--profile must be smoke or accelerated".to_owned());
        }
        if !matches!(
            config.phase.as_str(),
            "all" | "growth" | "steady" | "bounded" | "contention" | "crash"
        ) {
            return Err("--phase must be all, growth, bounded, contention, or crash".to_owned());
        }
        if config.hot_percent.is_some_and(|value| value > 100) {
            return Err("--hot-percent must be between 0 and 100".to_owned());
        }
        Ok(config)
    }

    fn normalize(&mut self) {
        if self.phase == "steady" {
            self.phase = "growth".to_owned();
        }
        if self.tenant_count == 0 {
            self.tenant_count = if self.profile == "accelerated" { 16 } else { 4 };
        }
        if self.concurrency == 0 {
            self.concurrency = if self.profile == "accelerated" { 16 } else { 4 };
        }
        self.tenant_count = self.tenant_count.clamp(1, 32);
        self.concurrency = self.concurrency.clamp(1, 128);
    }

    fn phase_durations(&self) -> Vec<(&'static str, Duration)> {
        if let Some(duration) = self.duration_override {
            return if self.phase == "all" {
                vec![
                    ("growth", duration),
                    ("bounded", duration),
                    ("contention", duration),
                    ("crash", duration),
                ]
            } else {
                vec![(phase_name(&self.phase), duration)]
            };
        }
        let durations = if self.profile == "accelerated" {
            [
                ("growth", 8 * 60),
                ("bounded", 6 * 60),
                ("contention", 8 * 60),
                ("crash", 10 * 60),
            ]
        } else {
            [
                ("growth", 30),
                ("bounded", 30),
                ("contention", 30),
                ("crash", 45),
            ]
        };
        durations
            .into_iter()
            .filter(|(name, _)| self.phase == "all" || self.phase == *name)
            .map(|(name, seconds)| (name, Duration::from_secs(seconds)))
            .collect()
    }
}

fn phase_name(phase: &str) -> &'static str {
    match phase {
        "growth" | "steady" => "growth",
        "bounded" => "bounded",
        "contention" => "contention",
        "crash" => "crash",
        _ => "growth",
    }
}

fn parse_duration(value: &str) -> Result<Duration, String> {
    let (number, multiplier) = if let Some(value) = value.strip_suffix("ms") {
        (value, 1u64)
    } else if let Some(value) = value.strip_suffix('s') {
        (value, 1_000)
    } else if let Some(value) = value.strip_suffix('m') {
        (value, 60_000)
    } else if let Some(value) = value.strip_suffix('h') {
        (value, 3_600_000)
    } else {
        return Err("duration must end in ms, s, m, or h".to_owned());
    };
    let milliseconds: u64 = number
        .parse()
        .map_err(|_| format!("invalid duration {value}"))?;
    Ok(Duration::from_millis(
        milliseconds.saturating_mul(multiplier).max(1),
    ))
}

fn parse_weights(value: &str) -> Result<[u32; 7], String> {
    let values = value
        .split(',')
        .map(|part| {
            part.parse::<u32>()
                .map_err(|_| "--weights must contain seven integers".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    values
        .try_into()
        .map_err(|_| "--weights must contain seven comma-separated integers".to_owned())
}

fn print_help() {
    println!(
        "dodb-soak --profile smoke|accelerated --seed N [options]\n\
         \nOptions:\n\
         --phase all|growth|bounded|contention|crash\n\
         --duration 10s|10m (override each selected phase)\n\
         --output DIR --data-dir DIR --tenants N --concurrency N\n\
         --hot-percent N --weights GET,PUT,DELETE,QUERY,SCAN,CONCURRENT_GET,TRANSACT\n\
         --checkpoint-ms N --no-connection-churn\n\
         \nExamples:\n\
         dodb-soak --profile smoke --seed 1\n\
         dodb-soak --profile accelerated --seed 1\n\
         dodb-soak --phase crash --duration 10m --seed 7"
    );
}

#[derive(Clone)]
struct TlsMaterial {
    certificate: Vec<u8>,
    private_key: Vec<u8>,
}

impl TlsMaterial {
    fn generate() -> Self {
        let certified =
            generate_simple_self_signed(vec![SERVER_NAME.to_owned()]).expect("TLS generation");
        Self {
            certificate: certified.cert.der().to_vec(),
            private_key: certified.signing_key.serialize_der(),
        }
    }

    fn write(&self, directory: &Path) -> io::Result<(PathBuf, PathBuf)> {
        let certificate = directory.join("soak-cert.der");
        let private_key = directory.join("soak-key.der");
        fs::write(&certificate, &self.certificate)?;
        fs::write(&private_key, &self.private_key)?;
        Ok((certificate, private_key))
    }
}

#[derive(Debug)]
struct ServerHandle {
    child: Child,
    pid: u32,
    address: SocketAddr,
    event_file: PathBuf,
}

fn argument(args: &[String], name: &str) -> Result<String, String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .ok_or_else(|| format!("server child is missing {name}"))
}

async fn run_server_child(args: &[String]) -> Result<(), BoxError> {
    let data_dir = PathBuf::from(argument(args, "--data-dir")?);
    let certificate = fs::read(argument(args, "--cert-file")?)?;
    let private_key = fs::read(argument(args, "--key-file")?)?;
    let ready_file = PathBuf::from(argument(args, "--ready-file")?);
    let event_file = PathBuf::from(argument(args, "--event-file")?);
    let shutdown_file = PathBuf::from(argument(args, "--shutdown-file")?);
    let checkpoint_ms: u64 = argument(args, "--checkpoint-ms")?.parse()?;
    let cycle: u64 = argument(args, "--cycle")?.parse()?;
    let event_writer = EventWriter::new(event_file.clone(), cycle);
    let service_data_dir = data_dir.clone();
    let service = Arc::new(LocalTenantService::new(LocalTenantServiceConfig {
        data_dir,
        max_open_shards: 1_024,
        coordinator_config: dodb_storage::CoordinatorConfig {
            queue_capacity: 512,
            max_group_requests: 128,
            max_group_bytes: 8 * 1024 * 1024,
            max_collection_delay: Duration::from_micros(100),
        },
        ..LocalTenantServiceConfig::default()
    })?);
    let server = Arc::new(DodbServer::bind(
        Arc::clone(&service),
        DodbServerConfig {
            listen_addr: "127.0.0.1:0".parse()?,
            tls: ServerTlsConfig::from_der(vec![certificate], private_key)?,
            protocol_limits: ProtocolLimits::default(),
            max_connections: 64,
            max_concurrent_streams: 256,
            max_concurrent_requests: 512,
        },
    )?);
    fs::write(&ready_file, server.local_addr()?.to_string())?;
    event_writer.append(&serde_json::json!({"kind":"ready", "pid":std::process::id()}))?;

    let checkpoint_service = Arc::clone(&service);
    let checkpoint_events = event_writer.clone();
    let checkpoint_shutdown = shutdown_file.clone();
    let checkpoint_data_dir = service_data_dir.clone();
    let event_error = Arc::new(Mutex::new(None::<String>));
    let checkpoint_event_error = Arc::clone(&event_error);
    let checkpoint_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(checkpoint_ms.max(1)));
        loop {
            interval.tick().await;
            if checkpoint_shutdown.exists() {
                break;
            }
            let wal_before = directory_bytes(&checkpoint_data_dir, "wal");
            match checkpoint_service.checkpoint_all().await {
                Ok(reports) => {
                    let mut checkpoint_lsn = 0;
                    let mut pages_flushed = 0;
                    let mut bytes_written = 0;
                    let mut wal_reclaimed = 0;
                    let mut duration_nanos = 0;
                    for (_, report) in reports {
                        checkpoint_lsn = checkpoint_lsn.max(report.checkpoint_lsn.get());
                        pages_flushed += report.pages_flushed;
                        bytes_written += report.bytes_written;
                        wal_reclaimed += report.wal_bytes_reclaimed;
                        duration_nanos += report.duration_nanos;
                    }
                    let invariant = checkpoint_service.check_invariants_all().await;
                    let invariant_count = invariant.as_ref().map_or(0, Vec::len);
                    let wal_after = directory_bytes(&checkpoint_data_dir, "wal");
                    if let Err(error) = checkpoint_events.append(&serde_json::json!({
                        "kind":"checkpoint",
                        "checkpoint_lsn":checkpoint_lsn,
                        "pages_flushed":pages_flushed,
                        "bytes_written":bytes_written,
                        "wal_bytes_reclaimed":wal_reclaimed,
                        "wal_bytes_before":wal_before,
                        "wal_bytes_after":wal_after,
                        "duration_nanos":duration_nanos,
                        "invariant_shards":invariant_count,
                        "invariant_error":invariant.err().map(|error| error.to_string())
                    })) {
                        if let Ok(mut event_error) = checkpoint_event_error.lock() {
                            *event_error = Some(error.to_string());
                        }
                        break;
                    }
                }
                Err(error) => {
                    if let Err(event_error) = checkpoint_events.append(
                        &serde_json::json!({"kind":"checkpoint_error", "error":error.to_string()}),
                    ) {
                        if let Ok(mut recorded) = checkpoint_event_error.lock() {
                            *recorded = Some(event_error.to_string());
                        }
                        break;
                    }
                }
            }
        }
    });

    let metrics_server = Arc::clone(&server);
    let metrics_events = event_writer;
    let metrics_shutdown = shutdown_file.clone();
    let metrics_event_error = Arc::clone(&event_error);
    let metrics_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            if metrics_shutdown.exists() {
                break;
            }
            let metrics = metrics_server.metrics().snapshot();
            let allocator = mimalloc::MiMalloc::stats_json()
                .ok()
                .map(|stats| stats.to_string_lossy().into_owned());
            if let Err(error) = metrics_events.append(&serde_json::json!({
                "kind":"metrics",
                "active_connections":metrics.active_connections,
                "active_streams":metrics.active_streams,
                "requests_total":metrics.requests_total,
                "connections_total":metrics.connections_total,
                "transport_errors":metrics.transport_errors,
                "application_errors":metrics.application_errors,
                "overloaded_responses":metrics.overloaded_responses,
                "allocator":allocator
            })) {
                if let Ok(mut event_error) = metrics_event_error.lock() {
                    *event_error = Some(error.to_string());
                }
                break;
            }
        }
    });

    let run_server = Arc::clone(&server);
    let mut server_task = Box::pin(tokio::spawn(async move { run_server.run().await }));
    loop {
        tokio::select! {
            result = &mut server_task => {
                result??;
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                if shutdown_file.exists() {
                    server.shutdown().await;
                    server_task.await??;
                    break;
                }
            }
        }
    }
    checkpoint_task.abort();
    metrics_task.abort();
    if let Ok(event_error) = event_error.lock()
        && let Some(error) = event_error.clone()
    {
        return Err(format!("event writer failed: {error}").into());
    }
    Ok(())
}

#[derive(Clone)]
struct EventWriter {
    path: PathBuf,
    cycle: u64,
    lock: Arc<Mutex<()>>,
}

impl EventWriter {
    fn new(path: PathBuf, cycle: u64) -> Self {
        Self {
            path,
            cycle,
            lock: Arc::new(Mutex::new(())),
        }
    }

    fn append(&self, value: &serde_json::Value) -> io::Result<()> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| io::Error::other("event writer lock poisoned"))?;
        let mut value = value.clone();
        if let Some(object) = value.as_object_mut() {
            object.insert(
                "timestamp_ms".to_owned(),
                serde_json::json!(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64
                ),
            );
            object.insert("cycle".to_owned(), serde_json::json!(self.cycle));
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        serde_json::to_writer(&mut file, &value).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.flush()
    }
}

async fn start_server(
    directory: &Path,
    data_dir: &Path,
    tls: &(PathBuf, PathBuf),
    cycle: u64,
    checkpoint_interval: Duration,
) -> Result<ServerHandle, BoxError> {
    let ready_file = directory.join(format!("ready-{cycle}.txt"));
    let event_file = directory.join(format!("server-{cycle}.jsonl"));
    let shutdown_file = directory.join(format!("shutdown-{cycle}"));
    for path in [&ready_file, &event_file, &shutdown_file] {
        let _ = fs::remove_file(path);
    }
    let stdout_file = directory.join(format!("server-{cycle}.stdout.log"));
    let stderr_file = directory.join(format!("server-{cycle}.stderr.log"));
    let stdout = File::create(stdout_file)?;
    let stderr = File::create(stderr_file)?;
    let mut child = Command::new(env::current_exe()?)
        .arg(SERVER_CHILD)
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--cert-file")
        .arg(&tls.0)
        .arg("--key-file")
        .arg(&tls.1)
        .arg("--ready-file")
        .arg(&ready_file)
        .arg("--event-file")
        .arg(&event_file)
        .arg("--shutdown-file")
        .arg(&shutdown_file)
        .arg("--cycle")
        .arg(cycle.to_string())
        .arg("--checkpoint-ms")
        .arg(checkpoint_interval.as_millis().max(1).to_string())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()?;
    let pid = child
        .id()
        .ok_or("server child did not expose a process id")?;
    let deadline = Instant::now() + Duration::from_secs(15);
    let address = loop {
        if let Ok(contents) = fs::read_to_string(&ready_file) {
            break contents.trim().parse()?;
        }
        if let Some(status) = child.try_wait()? {
            return Err(format!("server child exited before readiness: {status}").into());
        }
        if Instant::now() >= deadline {
            return Err("server child readiness timeout".into());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    Ok(ServerHandle {
        child,
        pid,
        address,
        event_file,
    })
}

async fn stop_server(
    mut server: ServerHandle,
    directory: &Path,
    graceful: bool,
) -> Result<(), BoxError> {
    let shutdown = directory.join(format!("shutdown-{}", server_cycle(&server.event_file)));
    if graceful {
        fs::write(shutdown, b"shutdown")?;
    } else {
        server.child.kill().await?;
    }
    let _ = server.child.wait().await?;
    Ok(())
}

fn server_cycle(event_file: &Path) -> String {
    event_file
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("server-"))
        .and_then(|name| name.strip_suffix(".jsonl"))
        .unwrap_or("0")
        .to_owned()
}

#[derive(Default)]
struct Counters {
    operation_counts: [AtomicU64; 7],
    total_operations: AtomicU64,
    commits: AtomicU64,
    conflicts: AtomicU64,
    overloads: AtomicU64,
    response_budget_errors: AtomicU64,
    unknown_resolved: AtomicU64,
    transport_errors: AtomicU64,
    application_errors: AtomicU64,
    checkpoints: AtomicU64,
    checkpoint_failures: AtomicU64,
    crashes: AtomicU64,
    recoveries: AtomicU64,
    verifications: AtomicU64,
    invariant_checks: AtomicU64,
    invariant_passes: AtomicU64,
    invariant_failures: AtomicU64,
    connection_churn: AtomicU64,
}

struct RunState {
    started: Instant,
    counters: Counters,
    model: Mutex<ReferenceModel>,
    pending: Mutex<Vec<PendingMutation>>,
    recent: Mutex<RecentOperations>,
    latency: Mutex<LatencyStats>,
    resources: Mutex<Vec<ResourceSample>>,
    phase_key_counts: Mutex<BTreeMap<String, PhaseKeyCounts>>,
    crash_history: Mutex<Vec<CrashRecord>>,
    failure: Mutex<Option<String>>,
    abort: AtomicBool,
    next_operation: AtomicU64,
}

impl RunState {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            started: Instant::now(),
            counters: Counters::default(),
            model: Mutex::new(ReferenceModel::default()),
            pending: Mutex::new(Vec::new()),
            recent: Mutex::new(RecentOperations::new(256)),
            latency: Mutex::new(LatencyStats::default()),
            resources: Mutex::new(Vec::new()),
            phase_key_counts: Mutex::new(BTreeMap::new()),
            crash_history: Mutex::new(Vec::new()),
            failure: Mutex::new(None),
            abort: AtomicBool::new(false),
            next_operation: AtomicU64::new(0),
        })
    }

    fn fail(&self, detail: impl Into<String>) {
        if let Ok(mut failure) = self.failure.lock()
            && failure.is_none()
        {
            *failure = Some(detail.into());
            self.abort.store(true, Ordering::Relaxed);
        }
    }

    fn elapsed_ms(&self) -> u128 {
        self.started.elapsed().as_millis()
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct PhaseKeyCounts {
    initial: Option<usize>,
    final_count: Option<usize>,
    initial_database_bytes: Option<u64>,
    final_database_bytes: Option<u64>,
    database_bytes_peak: Option<u64>,
    rss_initial: Option<u64>,
    rss_final: Option<u64>,
    rss_peak: Option<u64>,
    virtual_memory_initial: Option<u64>,
    virtual_memory_final: Option<u64>,
    virtual_memory_peak: Option<u64>,
    operations_initial: Option<u64>,
    operations_final: Option<u64>,
    checkpoints_initial: Option<u64>,
    checkpoints_final: Option<u64>,
    reference_retained_value_bytes_initial: Option<usize>,
    reference_retained_value_bytes_final: Option<usize>,
}

fn record_phase_key_count(state: &Arc<RunState>, phase: &str, initial: bool, data_dir: &Path) {
    let Ok(model) = state.model.lock() else {
        return;
    };
    let Ok(mut counts) = state.phase_key_counts.lock() else {
        return;
    };
    let entry = counts.entry(phase.to_owned()).or_default();
    if initial {
        entry.initial = Some(model.total_key_count());
        entry.initial_database_bytes = Some(directory_bytes(data_dir, "db"));
        entry.operations_initial = Some(state.counters.total_operations.load(Ordering::Relaxed));
        entry.checkpoints_initial = Some(state.counters.checkpoints.load(Ordering::Relaxed));
        entry.reference_retained_value_bytes_initial = Some(model.retained_value_bytes());
    } else {
        entry.final_count = Some(model.total_key_count());
        entry.final_database_bytes = Some(directory_bytes(data_dir, "db"));
        entry.operations_final = Some(state.counters.total_operations.load(Ordering::Relaxed));
        entry.checkpoints_final = Some(state.counters.checkpoints.load(Ordering::Relaxed));
        entry.reference_retained_value_bytes_final = Some(model.retained_value_bytes());
    }
}

#[derive(Clone)]
struct PendingMutation {
    index: u64,
    operation: GeneratedOperation,
}

#[derive(Clone, Copy)]
struct OperationIdentity<'a> {
    global_index: u64,
    phase: &'a str,
    phase_index: u64,
    seed: u64,
    workload_config: &'a WorkloadConfig,
}

#[derive(Clone, Debug, Serialize)]
struct CrashRecord {
    cycle: u64,
    interval_ms: u128,
    recovery_ms: Option<u128>,
    recovered: bool,
}

#[derive(Clone)]
struct WorkloadHandle {
    stop: watch::Sender<bool>,
    join: Arc<AsyncMutex<Option<tokio::task::JoinHandle<()>>>>,
}

async fn stop_workload(handle: WorkloadHandle) {
    let _ = handle.stop.send(true);
    if let Some(mut join) = handle.join.lock().await.take()
        && tokio::time::timeout(Duration::from_secs(10), &mut join)
            .await
            .is_err()
    {
        join.abort();
        let _ = join.await;
    }
}

fn workload_config(config: &HarnessConfig, phase: &str) -> WorkloadConfig {
    let mut workload = WorkloadConfig {
        tenant_count: config.tenant_count,
        ..WorkloadConfig::default()
    };
    if phase == "bounded" {
        workload.bounded_keyspace = true;
        workload.wide_key_count = 256;
        workload.hot_percent = 90;
        workload.operation_weights = [15, 25, 20, 10, 5, 10, 15];
        workload.max_transaction_mutations = 4;
    } else if phase == "contention" {
        workload.bounded_keyspace = true;
        workload.wide_key_count = 64;
        workload.hot_percent = 95;
        workload.operation_weights = [10, 15, 10, 5, 5, 15, 40];
        workload.max_transaction_mutations = 4;
    } else if phase == "crash" {
        workload.bounded_keyspace = true;
        workload.wide_key_count = 128;
        workload.hot_percent = 85;
        workload.operation_weights = [15, 20, 15, 10, 5, 10, 25];
        workload.max_transaction_mutations = 4;
    }
    if let Some(hot_percent) = config.hot_percent {
        workload.hot_percent = hot_percent;
    }
    if let Some(operation_weights) = config.operation_weights {
        workload.operation_weights = operation_weights;
    }
    workload
}

fn make_clients(connection: &DodbConnection, tenant_count: u64) -> Vec<DodbClient> {
    (1..=tenant_count)
        .map(|tenant| connection.for_tenant(TenantId::new(tenant)))
        .collect()
}

fn operation_client(clients: &[DodbClient], tenant: TenantId) -> &DodbClient {
    &clients[(tenant.get().saturating_sub(1) as usize) % clients.len()]
}

fn before_states(
    model: &ReferenceModel,
    tenant: TenantId,
    keys: &[DocumentKey],
) -> Vec<(DocumentKey, ExpectedState)> {
    keys.iter()
        .map(|key| (key.clone(), model.state(tenant, key)))
        .collect()
}

fn transaction_request(
    model: &ReferenceModel,
    tenant: TenantId,
    mutations: &[PlannedMutation],
) -> (TransactionRequest, Vec<(DocumentKey, ExpectedState)>) {
    let keys = mutations
        .iter()
        .map(|mutation| mutation.key.clone())
        .collect::<Vec<_>>();
    let before = before_states(model, tenant, &keys);
    let conditions = before
        .iter()
        .map(|(key, state)| TransactionCondition::RevisionEquals {
            key: key.clone(),
            expected_revision: state.revision(),
        })
        .collect();
    let wire_mutations = mutations
        .iter()
        .map(|mutation| match mutation.kind {
            PlannedMutationKind::Put => TransactionMutation::Put {
                key: mutation.key.clone(),
                value: mutation.value.clone(),
            },
            PlannedMutationKind::Delete => TransactionMutation::Delete {
                key: mutation.key.clone(),
            },
        })
        .collect();
    (TransactionRequest::new(conditions, wire_mutations), before)
}

fn apply_planned_mutations(
    model: &mut ReferenceModel,
    tenant: TenantId,
    mutations: &[PlannedMutation],
    revision: Revision,
) {
    for mutation in mutations {
        match mutation.kind {
            PlannedMutationKind::Put => model.apply_put(
                tenant,
                mutation.key.clone(),
                mutation.value.clone(),
                revision,
            ),
            PlannedMutationKind::Delete => {
                model.apply_delete(tenant, mutation.key.clone(), revision)
            }
        }
    }
}

fn record_operation(
    state: &Arc<RunState>,
    identity: OperationIdentity<'_>,
    operation: &GeneratedOperation,
    status: &str,
    started: Instant,
) {
    let elapsed = started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    state
        .counters
        .total_operations
        .fetch_add(1, Ordering::Relaxed);
    state.counters.operation_counts[operation.kind().counter_index()]
        .fetch_add(1, Ordering::Relaxed);
    if let Ok(mut latency) = state.latency.lock() {
        latency.record(elapsed);
    }
    if let Ok(mut recent) = state.recent.lock() {
        recent.push(OperationRecord {
            index: identity.global_index,
            global_index: identity.global_index,
            phase: identity.phase.to_owned(),
            phase_index: identity.phase_index,
            seed: identity.seed,
            workload_config: identity.workload_config.clone(),
            elapsed_ms: elapsed as u128 / 1_000_000,
            tenant: operation.tenant().get(),
            operation: operation.kind().name().to_owned(),
            generated_operation: operation_summary(operation),
            status: status.to_owned(),
        });
    }
}

fn operation_summary(operation: &GeneratedOperation) -> String {
    match operation {
        GeneratedOperation::Get { tenant, key } => {
            format!("Get {{ tenant: {}, key: {key:?} }}", tenant.get())
        }
        GeneratedOperation::Put { tenant, key, value } => format!(
            "Put {{ tenant: {}, key: {key:?}, value_len: {} }}",
            tenant.get(),
            value.len()
        ),
        GeneratedOperation::Delete { tenant, key } => {
            format!("Delete {{ tenant: {}, key: {key:?} }}", tenant.get())
        }
        GeneratedOperation::Query {
            tenant,
            pk,
            exclusive_after_sk,
            limit,
        } => format!(
            "Query {{ tenant: {}, pk: {pk:?}, exclusive_after_sk: {exclusive_after_sk:?}, limit: {limit} }}",
            tenant.get()
        ),
        GeneratedOperation::Scan {
            tenant,
            exclusive_after_key,
            limit,
        } => format!(
            "Scan {{ tenant: {}, exclusive_after_key: {exclusive_after_key:?}, limit: {limit} }}",
            tenant.get()
        ),
        GeneratedOperation::ConcurrentGet { tenant, keys } => {
            format!(
                "ConcurrentGet {{ tenant: {}, keys: {keys:?} }}",
                tenant.get()
            )
        }
        GeneratedOperation::Transact { tenant, mutations } => {
            let mutations = mutations
                .iter()
                .map(|mutation| {
                    format!(
                        "{{ key: {:?}, kind: {:?}, value_len: {} }}",
                        mutation.key,
                        mutation.kind,
                        mutation.value.len()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "Transact {{ tenant: {}, mutations: [{mutations}] }}",
                tenant.get()
            )
        }
    }
}

fn validate_documents(
    model: &ReferenceModel,
    tenant: TenantId,
    documents: &[Document],
    pk: Option<&PrimaryKey>,
    cursor: Option<&DocumentKey>,
    cursor_sk: Option<&dodb_core::SortKey>,
) -> Result<(), String> {
    for pair in documents.windows(2) {
        if pair[0].key >= pair[1].key {
            return Err("response ordering is not strictly increasing".to_owned());
        }
    }
    for document in documents {
        if pk.is_some_and(|pk| &document.key.pk != pk)
            || cursor.is_some_and(|cursor| &document.key <= cursor)
            || cursor_sk.is_some_and(|cursor| &document.key.sk <= cursor)
        {
            return Err("response violated cursor or partition-key semantics".to_owned());
        }
        let actual = RevisionState::present(document.value.clone(), document.revision);
        let known_at_last_model_view = model.matches_state(tenant, &document.key, &actual);
        if !known_at_last_model_view {
            return Err(format!(
                "read returned an unrecognized value or revision for key {:?}",
                document.key
            ));
        }
    }
    Ok(())
}

fn is_expected_conflict(error: &ClientError) -> bool {
    matches!(
        error,
        ClientError::Application(application)
            if application.kind == ApplicationErrorKind::Conflict
    )
}

fn validate_conflict_details(
    request: &TransactionRequest,
    conflict: &ConflictDetails,
) -> Result<(), String> {
    let condition = request
        .conditions
        .iter()
        .find(|condition| condition.key() == &conflict.key)
        .ok_or_else(|| "transaction conflict key was not one of the conditions".to_owned())?;
    if condition.expectation() != conflict.expected {
        return Err("transaction conflict returned the wrong expected condition".to_owned());
    }
    let condition_failed = match conflict.expected {
        ConditionExpectation::RevisionEquals(expected) => conflict.actual.revision() != expected,
        ConditionExpectation::Exists => matches!(conflict.actual, ObservedState::Missing { .. }),
        ConditionExpectation::NotExists => {
            matches!(conflict.actual, ObservedState::Present { .. })
        }
    };
    if !condition_failed {
        return Err("transaction conflict actual state satisfied the failed condition".to_owned());
    }
    Ok(())
}

fn is_overload(error: &ClientError) -> bool {
    matches!(
        &error,
        ClientError::Application(application)
            if application.kind == ApplicationErrorKind::Overloaded
    )
}

fn is_allowed_transport_error(error: &ClientError, allow_transport_errors: bool) -> bool {
    allow_transport_errors
        && (matches!(error, ClientError::Transport(_) | ClientError::Protocol(_))
            || matches!(error, ClientError::Endpoint(detail) if detail.contains("explicitly closed")))
}

async fn execute_operation(
    state: &Arc<RunState>,
    client: &DodbClient,
    identity: OperationIdentity<'_>,
    operation: GeneratedOperation,
    allow_transport_errors: bool,
) -> Result<(), String> {
    let started = Instant::now();
    let tenant = operation.tenant();
    if let Ok(mut model) = state.model.lock() {
        match &operation {
            GeneratedOperation::Put { key, value, .. } => {
                model.record_expected_put(tenant, key.clone(), value);
            }
            GeneratedOperation::Delete { key, .. } => {
                model.record_expected_delete(tenant, key.clone());
            }
            GeneratedOperation::Transact { mutations, .. } => {
                for mutation in mutations {
                    if matches!(mutation.kind, PlannedMutationKind::Put) {
                        model.record_expected_put(tenant, mutation.key.clone(), &mutation.value);
                    } else {
                        model.record_expected_delete(tenant, mutation.key.clone());
                    }
                }
            }
            _ => {}
        }
    }
    let result = match &operation {
        GeneratedOperation::Get { key, .. } => match client.get(key.clone()).await {
            Ok(actual) => state
                .model
                .lock()
                .map_err(|_| "reference model lock poisoned".to_owned())
                .and_then(|model| {
                    model
                        .matches_state(tenant, key, &actual)
                        .then_some("ok".to_owned())
                        .ok_or_else(|| {
                            format!("Get returned an unrecognized value or revision for {key:?}")
                        })
                }),
            Err(error) => handle_non_mutation_error(state, error, allow_transport_errors),
        },
        GeneratedOperation::Put { key, value, .. } => {
            match client.put(key.clone(), value.clone()).await {
                Ok(revision) => {
                    if let Ok(mut model) = state.model.lock() {
                        model.apply_put(tenant, key.clone(), value.clone(), revision);
                    }
                    state.counters.commits.fetch_add(1, Ordering::Relaxed);
                    Ok("commit".to_owned())
                }
                Err(error) => handle_mutation_error(
                    state,
                    identity.global_index,
                    &operation,
                    error,
                    allow_transport_errors,
                ),
            }
        }
        GeneratedOperation::Delete { key, .. } => match client.delete(key.clone()).await {
            Ok(revision) => {
                if let Ok(mut model) = state.model.lock() {
                    model.apply_delete(tenant, key.clone(), revision);
                }
                state.counters.commits.fetch_add(1, Ordering::Relaxed);
                Ok("commit".to_owned())
            }
            Err(error) => handle_mutation_error(
                state,
                identity.global_index,
                &operation,
                error,
                allow_transport_errors,
            ),
        },
        GeneratedOperation::Query {
            pk,
            exclusive_after_sk,
            limit,
            ..
        } => match client
            .query(pk.clone(), exclusive_after_sk.clone(), *limit)
            .await
        {
            Ok(documents) => state
                .model
                .lock()
                .map_err(|_| "reference model lock poisoned".to_owned())
                .and_then(|model| {
                    validate_documents(
                        &model,
                        tenant,
                        &documents,
                        Some(pk),
                        None,
                        exclusive_after_sk.as_ref(),
                    )
                })
                .map(|_| "ok".to_owned()),
            Err(error) => handle_non_mutation_error(state, error, allow_transport_errors),
        },
        GeneratedOperation::Scan {
            exclusive_after_key,
            limit,
            ..
        } => match client.scan(exclusive_after_key.clone(), *limit).await {
            Ok(documents) => state
                .model
                .lock()
                .map_err(|_| "reference model lock poisoned".to_owned())
                .and_then(|model| {
                    validate_documents(
                        &model,
                        tenant,
                        &documents,
                        None,
                        exclusive_after_key.as_ref(),
                        None,
                    )
                })
                .map(|_| "ok".to_owned()),
            Err(error) => handle_non_mutation_error(state, error, allow_transport_errors),
        },
        GeneratedOperation::ConcurrentGet { keys, .. } => {
            match concurrent_gets(client, keys).await {
                Ok(actual) => state
                    .model
                    .lock()
                    .map_err(|_| "reference model lock poisoned".to_owned())
                    .and_then(|model| {
                        keys.iter()
                            .zip(actual.iter())
                            .all(|(key, state)| model.matches_state(tenant, key, state))
                            .then_some("ok".to_owned())
                            .ok_or_else(|| {
                                "ConcurrentGet returned an unrecognized value or revision"
                                    .to_owned()
                            })
                    }),
                Err(error) => handle_non_mutation_error(state, error, allow_transport_errors),
            }
        }
        GeneratedOperation::Transact { mutations, .. } => {
            let (request, _before) = state
                .model
                .lock()
                .map(|model| transaction_request(&model, tenant, mutations))
                .map_err(|_| "reference model lock poisoned".to_owned())?;
            let validation_request = request.clone();
            match client.transact(request).await {
                Ok(TransactionOutcome {
                    commit_lsn: Some(lsn),
                }) => {
                    if let Ok(mut model) = state.model.lock() {
                        apply_planned_mutations(&mut model, tenant, mutations, Revision::from(lsn));
                    }
                    state.counters.commits.fetch_add(1, Ordering::Relaxed);
                    Ok("commit".to_owned())
                }
                Ok(TransactionOutcome { commit_lsn: None }) => {
                    Err("mutation transaction returned no commit identity".to_owned())
                }
                Err(error) if is_expected_conflict(&error) => {
                    state.counters.conflicts.fetch_add(1, Ordering::Relaxed);
                    if let ClientError::Application(application) = &error
                        && let Some(conflict) = &application.conflict
                    {
                        validate_conflict_details(&validation_request, conflict)?;
                    } else {
                        return Err("transaction conflict omitted structured details".to_owned());
                    }
                    Ok("conflict".to_owned())
                }
                Err(error) => handle_mutation_error(
                    state,
                    identity.global_index,
                    &operation,
                    error,
                    allow_transport_errors,
                ),
            }
        }
    };
    match result {
        Ok(status) => {
            record_operation(state, identity, &operation, &status, started);
            Ok(())
        }
        Err(error) => {
            record_operation(state, identity, &operation, "error", started);
            Err(error)
        }
    }
}

fn handle_non_mutation_error(
    state: &RunState,
    error: ClientError,
    allow_transport_errors: bool,
) -> Result<String, String> {
    if is_overload(&error) {
        state.counters.overloads.fetch_add(1, Ordering::Relaxed);
        Ok("overload".to_owned())
    } else if matches!(
        &error,
        ClientError::Application(application)
            if application.kind == ApplicationErrorKind::ResponseTooLarge
    ) {
        state
            .counters
            .response_budget_errors
            .fetch_add(1, Ordering::Relaxed);
        Ok("response_budget".to_owned())
    } else if is_allowed_transport_error(&error, allow_transport_errors) {
        state
            .counters
            .transport_errors
            .fetch_add(1, Ordering::Relaxed);
        Ok("transport".to_owned())
    } else {
        state
            .counters
            .application_errors
            .fetch_add(1, Ordering::Relaxed);
        Err(error.to_string())
    }
}

fn handle_mutation_error(
    state: &RunState,
    index: u64,
    operation: &GeneratedOperation,
    error: ClientError,
    allow_transport_errors: bool,
) -> Result<String, String> {
    if is_overload(&error) {
        state.counters.overloads.fetch_add(1, Ordering::Relaxed);
        return Ok("overload".to_owned());
    }
    if matches!(error, ClientError::UnknownMutationOutcome { .. }) {
        if let Ok(mut pending) = state.pending.lock() {
            pending.push(PendingMutation {
                index,
                operation: operation.clone(),
            });
        }
        return Ok("unknown".to_owned());
    }
    if is_allowed_transport_error(&error, allow_transport_errors) {
        state
            .counters
            .transport_errors
            .fetch_add(1, Ordering::Relaxed);
        return Ok("transport".to_owned());
    }
    state
        .counters
        .application_errors
        .fetch_add(1, Ordering::Relaxed);
    Err(error.to_string())
}

async fn spawn_workload(
    state: Arc<RunState>,
    connection: DodbConnection,
    config: HarnessConfig,
    phase: String,
    duration: Duration,
    allow_transport_errors: bool,
) -> WorkloadHandle {
    let (stop, stop_receiver) = watch::channel(false);
    let stop_for_task = stop.clone();
    let join = tokio::spawn(async move {
        let clients = Arc::new(make_clients(&connection, config.tenant_count));
        let (sender, receiver) = mpsc::channel::<(u64, u64, GeneratedOperation)>(
            config.concurrency.saturating_mul(4).max(8),
        );
        let receiver = Arc::new(AsyncMutex::new(receiver));
        let generated_workload = workload_config(&config, &phase);
        let mut workers = JoinSet::new();
        for _ in 0..config.concurrency {
            let worker_receiver = Arc::clone(&receiver);
            let worker_clients = Arc::clone(&clients);
            let worker_state = Arc::clone(&state);
            let mut worker_stop = stop_receiver.clone();
            let worker_phase = phase.clone();
            let worker_seed = config.seed;
            let worker_workload = generated_workload.clone();
            workers.spawn(async move {
                loop {
                    let item = tokio::select! {
                        changed = worker_stop.changed() => {
                            if changed.is_ok() && *worker_stop.borrow() {
                                break;
                            }
                            continue;
                        }
                        item = async {
                            let mut receiver = worker_receiver.lock().await;
                            receiver.recv().await
                        } => item,
                    };
                    let Some((global_index, phase_index, operation)) = item else {
                        break;
                    };
                    if worker_state.abort.load(Ordering::Relaxed) {
                        break;
                    }
                    let client = operation_client(&worker_clients, operation.tenant());
                    let kind = operation.kind();
                    let description = operation_summary(&operation);
                    if let Err(error) = execute_operation(
                        &worker_state,
                        client,
                        OperationIdentity {
                            global_index,
                            phase: &worker_phase,
                            phase_index,
                            seed: worker_seed,
                            workload_config: &worker_workload,
                        },
                        operation,
                        allow_transport_errors,
                    )
                    .await
                    {
                        worker_state.fail(format!(
                            "operation {global_index} {kind:?} {description} failed: {error}"
                        ));
                        break;
                    }
                }
            });
        }
        let generator_seed = config.seed ^ phase_seed(&phase);
        let mut generator = OperationGenerator::new(generator_seed, generated_workload)
            .expect("validated workload configuration");
        let deadline = Instant::now() + duration;
        let mut producer_stop = stop_receiver.clone();
        let mut phase_index = 0;
        loop {
            if *producer_stop.borrow()
                || state.abort.load(Ordering::Relaxed)
                || Instant::now() >= deadline
            {
                break;
            }
            let global_index = state.next_operation.fetch_add(1, Ordering::Relaxed);
            let operation = generator.next(phase_index);
            let item = (global_index, phase_index, operation);
            phase_index += 1;
            let sent = tokio::select! {
                changed = producer_stop.changed() => changed.is_ok() && !*producer_stop.borrow(),
                result = sender.send(item) => result.is_ok(),
            };
            if !sent {
                break;
            }
        }
        drop(sender);
        let _ = stop_for_task.send(true);
        while workers.join_next().await.is_some() {}
    });
    WorkloadHandle {
        stop,
        join: Arc::new(AsyncMutex::new(Some(join))),
    }
}

fn phase_seed(phase: &str) -> u64 {
    match phase {
        "growth" | "steady" => 0x1111,
        "bounded" => 0x4444,
        "contention" => 0x2222,
        "crash" => 0x3333,
        _ => 0,
    }
}

async fn connection_churn(
    state: Arc<RunState>,
    address: SocketAddr,
    tls: TlsMaterial,
    tenant_count: u64,
    duration: Duration,
    enabled: bool,
    mut stop: watch::Receiver<bool>,
) {
    if !enabled {
        return;
    }
    let mut rng = DeterministicRng::new(state.next_operation.load(Ordering::Relaxed) ^ 0xfeed);
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline && !state.abort.load(Ordering::Relaxed) && !*stop.borrow() {
        let tenant = TenantId::new(1 + rng.next_u64() % tenant_count.max(1));
        match DodbConnection::connect(
            "0.0.0.0:0".parse().expect("bind address"),
            address,
            SERVER_NAME,
            ClientTlsConfig::from_der(vec![tls.certificate.clone()]).expect("TLS roots"),
            ProtocolLimits::default(),
        )
        .await
        {
            Ok(connection) => {
                let client = connection.for_tenant(tenant);
                let key = DocumentKey::new(format!("tenant-{}-hot-pk-0", tenant.get()), "hot-sk-0");
                let _ = client.get(key).await;
                connection.close();
                state
                    .counters
                    .connection_churn
                    .fetch_add(1, Ordering::Relaxed);
            }
            Err(error) => {
                if !matches!(error, ClientError::Transport(_) | ClientError::Endpoint(_)) {
                    state.fail(format!("connection churn failed: {error}"));
                    break;
                }
                state
                    .counters
                    .transport_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(250)) => {}
            changed = stop.changed() => {
                if changed.is_err() || *stop.borrow() {
                    break;
                }
            }
        }
    }
}

async fn sample_resources(
    state: Arc<RunState>,
    pid: u32,
    data_dir: PathBuf,
    event_file: PathBuf,
    phase: String,
    stop: watch::Receiver<bool>,
) {
    let mut stop = stop;
    while !*stop.borrow() && !state.abort.load(Ordering::Relaxed) {
        push_resource_sample(&state, pid, &data_dir, &event_file, &phase, false);
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
            changed = stop.changed() => if changed.is_err() { break; },
        }
    }
}

fn push_resource_sample(
    state: &Arc<RunState>,
    pid: u32,
    data_dir: &Path,
    event_file: &Path,
    phase: &str,
    post_quiescence: bool,
) {
    let (rss, virtual_memory, threads) = proc_status(pid);
    let active = latest_metrics(event_file);
    let sample = ResourceSample {
        phase: phase.to_owned(),
        elapsed_ms: state.elapsed_ms(),
        post_quiescence,
        rss_bytes: rss,
        virtual_bytes: virtual_memory,
        fd_count: fd_count(pid),
        thread_count: threads,
        wal_bytes: directory_bytes(data_dir, "wal"),
        database_bytes: directory_bytes(data_dir, "db"),
        active_connections: active
            .as_ref()
            .and_then(|value| value.get("active_connections"))
            .and_then(serde_json::Value::as_u64),
        active_streams: active
            .as_ref()
            .and_then(|value| value.get("active_streams"))
            .and_then(serde_json::Value::as_u64),
    };
    if let Ok(mut samples) = state.resources.lock() {
        samples.push(sample);
    }
}

async fn collect_post_quiescence_samples(
    state: &Arc<RunState>,
    pid: u32,
    data_dir: &Path,
    event_file: &Path,
    phase: &str,
) -> Result<(), String> {
    // The child publishes metrics once per second. The first wait gives that
    // publisher time to observe the closed QUIC connection; later samples
    // make the cleanup observation explicit rather than relying on one read.
    for sample in 0..4 {
        tokio::time::sleep(if sample == 0 {
            Duration::from_millis(1_200)
        } else {
            Duration::from_millis(250)
        })
        .await;
        push_resource_sample(state, pid, data_dir, event_file, phase, true);
    }
    let samples = state
        .resources
        .lock()
        .map_err(|_| "resource sample lock poisoned".to_owned())?
        .iter()
        .filter(|sample| sample.post_quiescence)
        .cloned()
        .collect::<Vec<_>>();
    validate_post_quiescence_samples(&samples)
}

fn validate_post_quiescence_samples(samples: &[ResourceSample]) -> Result<(), String> {
    let final_sample = samples
        .last()
        .ok_or_else(|| "post-quiescence resources were not sampled".to_owned())?;
    if final_sample.active_streams != Some(0) {
        return Err(format!(
            "post-quiescence active streams were not zero: {:?}",
            final_sample.active_streams
        ));
    }
    if final_sample.active_connections != Some(0) {
        return Err(format!(
            "post-quiescence active connections did not reach idle baseline: {:?}",
            final_sample.active_connections
        ));
    }
    Ok(())
}

fn proc_status(pid: u32) -> (Option<u64>, Option<u64>, Option<u64>) {
    let path = format!("/proc/{pid}/status");
    let Ok(contents) = fs::read_to_string(path) else {
        return (None, None, None);
    };
    let mut rss = None;
    let mut virtual_memory = None;
    let mut threads = None;
    for line in contents.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("VmRSS:") => {
                rss = parts
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|value| value * 1024)
            }
            Some("VmSize:") => {
                virtual_memory = parts
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|value| value * 1024)
            }
            Some("Threads:") => threads = parts.next().and_then(|value| value.parse().ok()),
            _ => {}
        }
    }
    (rss, virtual_memory, threads)
}

fn fd_count(pid: u32) -> Option<u64> {
    fs::read_dir(format!("/proc/{pid}/fd"))
        .ok()
        .map(|entries| entries.count() as u64)
}

fn directory_bytes(directory: &Path, suffix: &str) -> u64 {
    fs::read_dir(directory)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| {
            entry
                .path()
                .extension()
                .and_then(|extension| (extension == suffix).then_some(entry))
        })
        .filter_map(|entry| entry.metadata().ok().map(|metadata| metadata.len()))
        .sum()
}

fn latest_metrics(path: &Path) -> Option<serde_json::Value> {
    let contents = fs::read_to_string(path).ok()?;
    contents.lines().rev().find_map(|line| {
        let value: serde_json::Value = serde_json::from_str(line).ok()?;
        (value.get("kind")?.as_str()? == "metrics").then_some(value)
    })
}

fn process_child_events(
    state: &Arc<RunState>,
    path: &Path,
    tolerate_trailing_partial: bool,
) -> Result<(), String> {
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let summary = parse_event_lines(&contents, tolerate_trailing_partial)?;
    state
        .counters
        .checkpoints
        .fetch_add(summary.checkpoint_successes, Ordering::Relaxed);
    state
        .counters
        .checkpoint_failures
        .fetch_add(summary.checkpoint_failures.len() as u64, Ordering::Relaxed);
    state
        .counters
        .invariant_checks
        .fetch_add(summary.invariant_checks, Ordering::Relaxed);
    state
        .counters
        .invariant_passes
        .fetch_add(summary.invariant_passes, Ordering::Relaxed);
    state
        .counters
        .invariant_failures
        .fetch_add(summary.invariant_failures.len() as u64, Ordering::Relaxed);
    if !summary.checkpoint_failures.is_empty() {
        return Err(format!(
            "checkpoint failed: {}",
            summary.checkpoint_failures.join("; ")
        ));
    }
    if !summary.invariant_failures.is_empty() {
        return Err(format!(
            "invariant check failed: {}",
            summary.invariant_failures.join("; ")
        ));
    }
    Ok(())
}

async fn scan_page_resilient(
    client: &DodbClient,
    cursor: Option<DocumentKey>,
    requested_limit: usize,
) -> Result<Vec<Document>, String> {
    let mut limit = requested_limit.max(1);
    loop {
        match client.scan(cursor.clone(), limit).await {
            Ok(page) => return Ok(page),
            Err(ClientError::Application(application))
                if application.kind == ApplicationErrorKind::ResponseTooLarge && limit > 1 =>
            {
                limit = limit.div_ceil(2);
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

async fn query_page_resilient(
    client: &DodbClient,
    pk: PrimaryKey,
    cursor: Option<dodb_core::SortKey>,
    requested_limit: usize,
) -> Result<Vec<Document>, String> {
    let mut limit = requested_limit.max(1);
    loop {
        match client.query(pk.clone(), cursor.clone(), limit).await {
            Ok(page) => return Ok(page),
            Err(ClientError::Application(application))
                if application.kind == ApplicationErrorKind::ResponseTooLarge && limit > 1 =>
            {
                limit = limit.div_ceil(2);
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

async fn concurrent_gets(
    client: &DodbClient,
    keys: &[DocumentKey],
) -> Result<Vec<RevisionState>, ClientError> {
    let mut requests = JoinSet::new();
    for (index, key) in keys.iter().cloned().enumerate() {
        let client = client.clone();
        requests.spawn(async move { (index, client.get(key).await) });
    }
    let mut states = vec![None; keys.len()];
    while let Some(result) = requests.join_next().await {
        let (index, state) = result.map_err(|error| ClientError::Transport(error.to_string()))?;
        states[index] = Some(state?);
    }
    states
        .into_iter()
        .map(|state| {
            state.ok_or_else(|| {
                ClientError::Transport("concurrent Get task returned no result".to_owned())
            })
        })
        .collect()
}

async fn verify_full(
    state: &Arc<RunState>,
    connection: &DodbConnection,
    tenant_count: u64,
) -> Result<(), String> {
    let model = state
        .model
        .lock()
        .map_err(|_| "reference model lock poisoned".to_owned())?
        .clone();
    let clients = make_clients(connection, tenant_count);
    for tenant_number in 1..=tenant_count {
        let tenant = TenantId::new(tenant_number);
        let client = operation_client(&clients, tenant);
        let expected = model.documents(tenant);
        let mut cursor = None;
        let mut offset = 0;
        loop {
            let page = scan_page_resilient(client, cursor.clone(), 128).await?;
            if page.is_empty() {
                break;
            }
            let end = (offset + page.len()).min(expected.len());
            if end - offset != page.len() {
                return Err(format!(
                    "full scan mismatch for tenant {tenant_number} at offset {offset}: expected {} rows, got {}",
                    end - offset,
                    page.len()
                ));
            }
            if let Some((actual, expected)) = page
                .iter()
                .zip(&expected[offset..end])
                .find(|(actual, expected)| !expected_document_matches(actual, expected))
            {
                return Err(format!(
                    "full scan mismatch for tenant {tenant_number} at offset {offset}: key {:?} expected revision {} value length {} but got key {:?} revision {} value length {}",
                    expected.key,
                    expected.revision,
                    expected.value.length(),
                    actual.key,
                    actual.revision,
                    actual.value.len()
                ));
            }
            cursor = page.last().map(|document| document.key.clone());
            offset = end;
        }
        if offset != expected.len() {
            return Err(format!("full scan ended early for tenant {tenant_number}"));
        }
        let mut partitions = BTreeSet::new();
        for document in &expected {
            partitions.insert(document.key.pk.clone());
        }
        for pk in partitions {
            let partition = expected
                .iter()
                .filter(|document| document.key.pk == pk)
                .cloned()
                .collect::<Vec<_>>();
            let mut query_cursor = None;
            let mut query_offset = 0;
            loop {
                let page =
                    query_page_resilient(client, pk.clone(), query_cursor.clone(), 128).await?;
                if page.is_empty() {
                    break;
                }
                let end = (query_offset + page.len()).min(partition.len());
                if end - query_offset != page.len()
                    || page
                        .iter()
                        .zip(&partition[query_offset..end])
                        .any(|(actual, expected)| !expected_document_matches(actual, expected))
                {
                    return Err(format!("query mismatch for tenant {tenant_number}"));
                }
                query_cursor = page.last().map(|document| document.key.sk.clone());
                query_offset = end;
            }
            if query_offset != partition.len() {
                return Err(format!("query ended early for tenant {tenant_number}"));
            }
        }
        let keys = model.keys(tenant);
        for chunk in keys.chunks(128) {
            let actual = concurrent_gets(client, chunk)
                .await
                .map_err(|error| error.to_string())?;
            if actual.len() != chunk.len() {
                return Err("concurrent Get verification length mismatch".to_owned());
            }
            for (key, state_value) in chunk.iter().zip(actual.iter()) {
                if !model.matches_state(tenant, key, state_value) {
                    return Err(format!(
                        "concurrent Get mismatch for tenant {tenant_number}"
                    ));
                }
            }
        }
    }
    state.counters.verifications.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

fn expected_document_matches(actual: &Document, expected: &ExpectedDocument) -> bool {
    actual.key == expected.key
        && actual.revision == expected.revision
        && expected.value.matches(&actual.value)
}

async fn reconcile_unknown(
    state: &Arc<RunState>,
    connection: &DodbConnection,
    tenant_count: u64,
) -> Result<(), String> {
    let pending = state
        .pending
        .lock()
        .map_err(|_| "pending lock poisoned".to_owned())?
        .drain(..)
        .collect::<Vec<_>>();
    if pending.is_empty() {
        return Ok(());
    }
    let clients = make_clients(connection, tenant_count);
    let mut components = Vec::<Vec<usize>>::new();
    let mut assigned = vec![false; pending.len()];
    for start in 0..pending.len() {
        if assigned[start] {
            continue;
        }
        let mut component = vec![start];
        assigned[start] = true;
        let mut cursor = 0;
        while cursor < component.len() {
            let current = component[cursor];
            let current_keys = pending[current].operation.keys();
            for candidate in 0..pending.len() {
                if assigned[candidate] {
                    continue;
                }
                let overlaps = pending[candidate]
                    .operation
                    .keys()
                    .iter()
                    .any(|key| current_keys.iter().any(|candidate| candidate == key));
                if overlaps {
                    assigned[candidate] = true;
                    component.push(candidate);
                }
            }
            cursor += 1;
        }
        components.push(component);
    }

    for component in components {
        let tenant = pending[component[0]].operation.tenant();
        let mut keys = Vec::new();
        for index in &component {
            for key in pending[*index].operation.keys() {
                if !keys.iter().any(|known: &DocumentKey| known == key) {
                    keys.push(key.clone());
                }
            }
        }
        let actual = concurrent_gets(operation_client(&clients, tenant), &keys)
            .await
            .map_err(|error| error.to_string())?;
        let model = state
            .model
            .lock()
            .map_err(|_| "reference model lock poisoned".to_owned())?
            .clone();
        let current = keys
            .iter()
            .map(|key| model.state(tenant, key))
            .collect::<Vec<_>>();
        let operations = component
            .iter()
            .map(|index| pending[*index].operation.clone())
            .collect::<Vec<_>>();
        match reconcile_unknown_operations(&operations, &keys, &current, &actual)? {
            UnknownMutationResolution::Applied => {
                if let Ok(mut target) = state.model.lock() {
                    for (key, state_value) in keys.iter().zip(actual.iter()) {
                        target.apply(tenant, key.clone(), state_value.clone());
                    }
                }
            }
            UnknownMutationResolution::NotApplied => {}
            UnknownMutationResolution::Partial => {
                let indexes = component
                    .iter()
                    .map(|index| pending[*index].index.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                return Err(format!(
                    "unknown mutation set [{indexes}] resolved to an impossible partial state"
                ));
            }
        }
        state
            .counters
            .unknown_resolved
            .fetch_add(component.len() as u64, Ordering::Relaxed);
    }
    if state
        .pending
        .lock()
        .map_err(|_| "pending lock poisoned".to_owned())?
        .is_empty()
    {
        Ok(())
    } else {
        Err("unknown mutations remained after reconciliation".to_owned())
    }
}

async fn run_aba_probe(
    state: &Arc<RunState>,
    client: &DodbClient,
    tenant: TenantId,
    marker: &str,
) -> Result<(), String> {
    let key = DocumentKey::new(format!("aba-{marker}-{}", tenant.get()), "key");
    let first = client
        .delete(key.clone())
        .await
        .map_err(|error| error.to_string())?;
    if let Ok(mut model) = state.model.lock() {
        model.apply_delete(tenant, key.clone(), first);
    }
    let missing = client
        .get(key.clone())
        .await
        .map_err(|error| error.to_string())?;
    let missing_revision = missing.revision();
    let value = format!("soak_operation_id=aba-{marker};payload=present").into_bytes();
    let present = client
        .put(key.clone(), value.clone())
        .await
        .map_err(|error| error.to_string())?;
    if let Ok(mut model) = state.model.lock() {
        model.apply_put(tenant, key.clone(), value, present);
    }
    let deleted = client
        .delete(key.clone())
        .await
        .map_err(|error| error.to_string())?;
    if let Ok(mut model) = state.model.lock() {
        model.apply_delete(tenant, key.clone(), deleted);
    }
    let stale = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: key.clone(),
                expected_revision: missing_revision,
            }],
            vec![TransactionMutation::Put {
                key: key.clone(),
                value: b"stale-aba-write".to_vec(),
            }],
        ))
        .await;
    if !is_expected_conflict_result(&stale) {
        return Err("stale missing RevisionEquals unexpectedly succeeded".to_owned());
    }
    state.counters.conflicts.fetch_add(1, Ordering::Relaxed);
    let stale_present = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: key.clone(),
                expected_revision: present,
            }],
            vec![TransactionMutation::Put {
                key,
                value: b"stale-present-write".to_vec(),
            }],
        ))
        .await;
    if !is_expected_conflict_result(&stale_present) {
        return Err("stale present RevisionEquals unexpectedly succeeded".to_owned());
    }
    state.counters.conflicts.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

fn is_expected_conflict_result(result: &Result<TransactionOutcome, ClientError>) -> bool {
    matches!(result, Err(error) if is_expected_conflict(error))
}

#[allow(clippy::too_many_arguments)]
async fn run_normal_phase(
    state: &Arc<RunState>,
    config: &HarnessConfig,
    tls: &TlsMaterial,
    tls_files: &(PathBuf, PathBuf),
    data_dir: &Path,
    output: &Path,
    phase: &str,
    duration: Duration,
    cycle: u64,
) -> Result<(), BoxError> {
    let server = start_server(
        output,
        data_dir,
        tls_files,
        cycle,
        config.checkpoint_interval,
    )
    .await?;
    let connection = DodbConnection::connect(
        "0.0.0.0:0".parse()?,
        server.address,
        SERVER_NAME,
        ClientTlsConfig::from_der(vec![tls.certificate.clone()])?,
        ProtocolLimits::default(),
    )
    .await?;
    for tenant in 1..=config.tenant_count {
        let marker = format!("{phase}-aba");
        run_aba_probe(
            state,
            &connection.for_tenant(TenantId::new(tenant)),
            TenantId::new(tenant),
            &marker,
        )
        .await?;
    }
    let (sample_stop, sample_receiver) = watch::channel(false);
    let sampler = tokio::spawn(sample_resources(
        Arc::clone(state),
        server.pid,
        data_dir.to_owned(),
        server.event_file.clone(),
        phase.to_owned(),
        sample_receiver,
    ));
    let workload = spawn_workload(
        Arc::clone(state),
        connection.clone(),
        config.clone(),
        phase.to_owned(),
        duration,
        false,
    )
    .await;
    let (churn_stop, churn_receiver) = watch::channel(false);
    let churn = tokio::spawn(connection_churn(
        Arc::clone(state),
        server.address,
        tls.clone(),
        config.tenant_count,
        duration,
        config.connection_churn,
        churn_receiver,
    ));
    tokio::time::sleep(duration).await;
    stop_workload(workload).await;
    let _ = churn_stop.send(true);
    let _ = churn.await;
    let reconciliation = reconcile_unknown(state, &connection, config.tenant_count).await;
    let verification = if reconciliation.is_ok() {
        verify_full(state, &connection, config.tenant_count).await
    } else {
        Ok(())
    };
    connection.close();
    let post_quiescence =
        collect_post_quiescence_samples(state, server.pid, data_dir, &server.event_file, phase)
            .await;
    let _ = sample_stop.send(true);
    let _ = sampler.await;
    let server_result = stop_server(server, output, true).await;
    let events = server_result.and_then(|_| {
        process_child_events(state, &output.join(format!("server-{cycle}.jsonl")), false)
            .map_err(Into::into)
    });
    events?;
    reconciliation.map_err(|error| -> BoxError { error.into() })?;
    verification.map_err(|error| -> BoxError { error.into() })?;
    post_quiescence.map_err(|error| -> BoxError { error.into() })?;
    Ok(())
}

async fn run_crash_phase(
    state: &Arc<RunState>,
    config: &HarnessConfig,
    tls: &TlsMaterial,
    tls_files: &(PathBuf, PathBuf),
    data_dir: &Path,
    output: &Path,
    duration: Duration,
) -> Result<(), BoxError> {
    let cycle_duration = if duration <= Duration::from_secs(120) {
        duration / 2
    } else {
        Duration::from_secs(35)
    };
    let cycles = ((duration.as_secs_f64() / cycle_duration.as_secs_f64()).ceil() as u64).max(1);
    for cycle in 0..cycles {
        if state.abort.load(Ordering::Relaxed) {
            break;
        }
        let started = Instant::now();
        let server = start_server(
            output,
            data_dir,
            tls_files,
            100 + cycle,
            config.checkpoint_interval,
        )
        .await?;
        let connection = DodbConnection::connect(
            "0.0.0.0:0".parse()?,
            server.address,
            SERVER_NAME,
            ClientTlsConfig::from_der(vec![tls.certificate.clone()])?,
            ProtocolLimits::default(),
        )
        .await?;
        let tenant = TenantId::new((cycle % config.tenant_count) + 1);
        run_aba_probe(state, &connection.for_tenant(tenant), tenant, "crash-aba").await?;
        let (sample_stop, sample_receiver) = watch::channel(false);
        let sampler = tokio::spawn(sample_resources(
            Arc::clone(state),
            server.pid,
            data_dir.to_owned(),
            server.event_file.clone(),
            "crash".to_owned(),
            sample_receiver,
        ));
        let workload = spawn_workload(
            Arc::clone(state),
            connection.clone(),
            config.clone(),
            "crash".to_owned(),
            cycle_duration,
            true,
        )
        .await;
        let (churn_stop, churn_receiver) = watch::channel(false);
        let churn = tokio::spawn(connection_churn(
            Arc::clone(state),
            server.address,
            tls.clone(),
            config.tenant_count,
            cycle_duration,
            config.connection_churn,
            churn_receiver,
        ));
        tokio::time::sleep(cycle_duration.saturating_mul(7) / 10).await;
        let mut crashed_server = server;
        crashed_server.child.kill().await?;
        let _ = crashed_server.child.wait().await?;
        state.counters.crashes.fetch_add(1, Ordering::Relaxed);
        let _ = sample_stop.send(true);
        connection.close();
        stop_workload(workload).await;
        let _ = sampler.await;
        let _ = churn_stop.send(true);
        let _ = churn.await;
        process_child_events(state, &crashed_server.event_file, true)
            .map_err(|error| -> BoxError { error.into() })?;

        let recovery_started = Instant::now();
        let recovered_server = start_server(
            output,
            data_dir,
            tls_files,
            200 + cycle,
            config.checkpoint_interval,
        )
        .await?;
        let recovered_connection = DodbConnection::connect(
            "0.0.0.0:0".parse()?,
            recovered_server.address,
            SERVER_NAME,
            ClientTlsConfig::from_der(vec![tls.certificate.clone()])?,
            ProtocolLimits::default(),
        )
        .await?;
        reconcile_unknown(state, &recovered_connection, config.tenant_count)
            .await
            .map_err(|error| -> BoxError { error.into() })?;
        verify_full(state, &recovered_connection, config.tenant_count)
            .await
            .map_err(|error| -> BoxError { error.into() })?;
        let recovery_ms = recovery_started.elapsed().as_millis();
        state.counters.recoveries.fetch_add(1, Ordering::Relaxed);
        state
            .crash_history
            .lock()
            .map_err(|_| "crash history lock poisoned")?
            .push(CrashRecord {
                cycle,
                interval_ms: started.elapsed().as_millis(),
                recovery_ms: Some(recovery_ms),
                recovered: true,
            });
        let post_quiescence = {
            recovered_connection.close();
            collect_post_quiescence_samples(
                state,
                recovered_server.pid,
                data_dir,
                &recovered_server.event_file,
                "crash",
            )
            .await
        };
        process_child_events(state, &recovered_server.event_file, false)
            .map_err(|error| -> BoxError { error.into() })?;
        post_quiescence.map_err(|error| -> BoxError { error.into() })?;
        stop_server(recovered_server, output, true).await?;
    }
    Ok(())
}

#[derive(Serialize)]
struct MetricSummary {
    initial: Option<u64>,
    final_value: Option<u64>,
    peak: Option<u64>,
}

#[derive(Serialize)]
struct Report {
    duration_ms: u128,
    seed: u64,
    profile: String,
    phase: String,
    operations_total: u64,
    operations_per_second: f64,
    operation_counts: [u64; 7],
    server_requests: u64,
    server_connections: u64,
    server_transport_errors: u64,
    server_application_errors: u64,
    server_overloads: u64,
    commits: u64,
    conflicts: u64,
    overloads: u64,
    response_budget_errors: u64,
    unknown_outcomes_resolved: u64,
    unresolved_unknown_mutations: u64,
    transport_errors: u64,
    application_errors: u64,
    checkpoints: u64,
    checkpoint_failures: u64,
    crashes: u64,
    successful_recoveries: u64,
    full_verification_passes: u64,
    invariant_checks: u64,
    invariant_passes: u64,
    invariant_failures: u64,
    connection_churn: u64,
    latency_p50_ms: f64,
    latency_p95_ms: f64,
    latency_p99_ms: f64,
    latency_max_ms: f64,
    rss: MetricSummary,
    virtual_memory: MetricSummary,
    threads: MetricSummary,
    file_descriptors: MetricSummary,
    active_connections: MetricSummary,
    active_streams: MetricSummary,
    post_quiescence_active_streams: Option<u64>,
    post_quiescence_active_connections: Option<u64>,
    post_quiescence_fd_count: Option<u64>,
    wal_bytes: MetricSummary,
    database_bytes: MetricSummary,
    allocator_stats: Option<serde_json::Value>,
    phase_key_counts: BTreeMap<String, PhaseKeyCounts>,
    resource_trend_warnings_by_phase: BTreeMap<String, Vec<TrendDiagnostic>>,
    trend_warnings: Vec<TrendDiagnostic>,
    crash_history: Vec<CrashRecord>,
    failure: Option<String>,
}

fn metric_summary(
    samples: &[ResourceSample],
    value: impl Fn(&ResourceSample) -> Option<u64>,
) -> MetricSummary {
    let values = samples.iter().filter_map(value).collect::<Vec<_>>();
    MetricSummary {
        initial: values.first().copied(),
        final_value: values.last().copied(),
        peak: values.iter().copied().max(),
    }
}

fn trend(
    samples: &[ResourceSample],
    metric: &str,
    value: impl Fn(&ResourceSample) -> Option<u64>,
) -> TrendDiagnostic {
    trend_diagnostic(
        metric,
        &samples
            .iter()
            .filter_map(|sample| {
                value(sample).map(|value| (sample.elapsed_ms as f64 / 1_000.0, value))
            })
            .collect::<Vec<_>>(),
        Duration::from_secs(10),
    )
}

fn build_report(
    state: &Arc<RunState>,
    config: &HarnessConfig,
    output: &Path,
    allocator_stats: Option<serde_json::Value>,
) -> Report {
    let duration_ms = state.started.elapsed().as_millis();
    let operations_total = state.counters.total_operations.load(Ordering::Relaxed);
    let operation_counts =
        std::array::from_fn(|index| state.counters.operation_counts[index].load(Ordering::Relaxed));
    let latency = state
        .latency
        .lock()
        .map(|latency| {
            (
                latency.percentile(50.0),
                latency.percentile(95.0),
                latency.percentile(99.0),
                latency.maximum(),
            )
        })
        .unwrap_or_default();
    let samples = state
        .resources
        .lock()
        .map(|samples| samples.clone())
        .unwrap_or_default();
    let post_quiescence = samples.iter().rev().find(|sample| sample.post_quiescence);
    let unresolved_unknown_mutations = state
        .pending
        .lock()
        .map(|pending| pending.len() as u64)
        .unwrap_or(u64::MAX);
    let mut samples_by_phase = BTreeMap::<String, Vec<ResourceSample>>::new();
    for sample in &samples {
        if !sample.post_quiescence {
            samples_by_phase
                .entry(sample.phase.clone())
                .or_default()
                .push(sample.clone());
        }
    }
    let mut resource_trend_warnings_by_phase = BTreeMap::new();
    for (phase, phase_samples) in &samples_by_phase {
        let warnings = [
            trend(phase_samples, "rss", |sample| sample.rss_bytes),
            trend(phase_samples, "virtual_memory", |sample| {
                sample.virtual_bytes
            }),
            trend(phase_samples, "file_descriptors", |sample| sample.fd_count),
            trend(phase_samples, "active_connections", |sample| {
                sample.active_connections
            }),
            trend(phase_samples, "active_streams", |sample| {
                sample.active_streams
            }),
            trend(phase_samples, "wal_bytes", |sample| Some(sample.wal_bytes)),
            trend(phase_samples, "database_bytes", |sample| {
                Some(sample.database_bytes)
            }),
        ]
        .into_iter()
        .filter(|diagnostic| diagnostic.warning)
        .collect::<Vec<_>>();
        if !warnings.is_empty() {
            resource_trend_warnings_by_phase.insert(phase.clone(), warnings);
        }
    }
    let trend_warnings = resource_trend_warnings_by_phase
        .values()
        .flat_map(|warnings| warnings.iter().cloned())
        .collect::<Vec<_>>();
    let phase_key_counts = state
        .phase_key_counts
        .lock()
        .map(|counts| counts.clone())
        .unwrap_or_default();
    let mut phase_key_counts = phase_key_counts;
    for (phase, phase_samples) in &samples_by_phase {
        let entry = phase_key_counts.entry(phase.clone()).or_default();
        let rss = metric_summary(phase_samples, |sample| sample.rss_bytes);
        let virtual_memory = metric_summary(phase_samples, |sample| sample.virtual_bytes);
        let database = metric_summary(phase_samples, |sample| Some(sample.database_bytes));
        entry.rss_initial = rss.initial;
        entry.rss_final = rss.final_value;
        entry.rss_peak = rss.peak;
        entry.virtual_memory_initial = virtual_memory.initial;
        entry.virtual_memory_final = virtual_memory.final_value;
        entry.virtual_memory_peak = virtual_memory.peak;
        entry.database_bytes_peak = database.peak;
    }
    let (
        server_requests,
        server_connections,
        server_transport_errors,
        server_application_errors,
        server_overloads,
    ) = server_metric_totals(output);
    Report {
        duration_ms,
        seed: config.seed,
        profile: config.profile.clone(),
        phase: config.phase.clone(),
        operations_total,
        operations_per_second: operations_total as f64 / (duration_ms.max(1) as f64 / 1_000.0),
        operation_counts,
        server_requests,
        server_connections,
        server_transport_errors,
        server_application_errors,
        server_overloads,
        commits: state.counters.commits.load(Ordering::Relaxed),
        conflicts: state.counters.conflicts.load(Ordering::Relaxed),
        overloads: state.counters.overloads.load(Ordering::Relaxed),
        response_budget_errors: state
            .counters
            .response_budget_errors
            .load(Ordering::Relaxed),
        unknown_outcomes_resolved: state.counters.unknown_resolved.load(Ordering::Relaxed),
        unresolved_unknown_mutations,
        transport_errors: state.counters.transport_errors.load(Ordering::Relaxed),
        application_errors: state.counters.application_errors.load(Ordering::Relaxed),
        checkpoints: state.counters.checkpoints.load(Ordering::Relaxed),
        checkpoint_failures: state.counters.checkpoint_failures.load(Ordering::Relaxed),
        crashes: state.counters.crashes.load(Ordering::Relaxed),
        successful_recoveries: state.counters.recoveries.load(Ordering::Relaxed),
        full_verification_passes: state.counters.verifications.load(Ordering::Relaxed),
        invariant_checks: state.counters.invariant_checks.load(Ordering::Relaxed),
        invariant_passes: state.counters.invariant_passes.load(Ordering::Relaxed),
        invariant_failures: state.counters.invariant_failures.load(Ordering::Relaxed),
        connection_churn: state.counters.connection_churn.load(Ordering::Relaxed),
        latency_p50_ms: latency.0 as f64 / 1_000_000.0,
        latency_p95_ms: latency.1 as f64 / 1_000_000.0,
        latency_p99_ms: latency.2 as f64 / 1_000_000.0,
        latency_max_ms: latency.3 as f64 / 1_000_000.0,
        rss: metric_summary(&samples, |sample| sample.rss_bytes),
        virtual_memory: metric_summary(&samples, |sample| sample.virtual_bytes),
        threads: metric_summary(&samples, |sample| sample.thread_count),
        file_descriptors: metric_summary(&samples, |sample| sample.fd_count),
        active_connections: metric_summary(&samples, |sample| sample.active_connections),
        active_streams: metric_summary(&samples, |sample| sample.active_streams),
        post_quiescence_active_streams: post_quiescence.and_then(|sample| sample.active_streams),
        post_quiescence_active_connections: post_quiescence
            .and_then(|sample| sample.active_connections),
        post_quiescence_fd_count: post_quiescence.and_then(|sample| sample.fd_count),
        wal_bytes: metric_summary(&samples, |sample| Some(sample.wal_bytes)),
        database_bytes: metric_summary(&samples, |sample| Some(sample.database_bytes)),
        allocator_stats,
        phase_key_counts,
        resource_trend_warnings_by_phase,
        trend_warnings,
        crash_history: state
            .crash_history
            .lock()
            .map(|history| history.clone())
            .unwrap_or_default(),
        failure: state
            .failure
            .lock()
            .ok()
            .and_then(|failure| failure.clone()),
    }
}

fn server_metric_totals(directory: &Path) -> (u64, u64, u64, u64, u64) {
    let mut totals = (0, 0, 0, 0, 0);
    let Ok(entries) = fs::read_dir(directory) else {
        return totals;
    };
    for entry in entries.flatten() {
        if !entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            continue;
        }
        let Ok(contents) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let mut latest = None;
        for line in contents.lines() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line)
                && value.get("kind").and_then(serde_json::Value::as_str) == Some("metrics")
            {
                latest = Some(value);
            }
        }
        if let Some(value) = latest {
            totals.0 += value
                .get("requests_total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            totals.1 += value
                .get("connections_total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            totals.2 += value
                .get("transport_errors")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            totals.3 += value
                .get("application_errors")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            totals.4 += value
                .get("overloaded_responses")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
        }
    }
    totals
}

fn write_failure_artifacts(
    state: &Arc<RunState>,
    report: &Report,
    output: &Path,
) -> Result<(), BoxError> {
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(report)?,
    )?;
    let recent = state
        .recent
        .lock()
        .map(|recent| recent.as_vec())
        .unwrap_or_default();
    fs::write(
        output.join("recent-operations.json"),
        serde_json::to_vec_pretty(&recent)?,
    )?;
    let resources = state
        .resources
        .lock()
        .map(|resources| resources.clone())
        .unwrap_or_default();
    fs::write(
        output.join("resource-samples.json"),
        serde_json::to_vec_pretty(&resources)?,
    )?;
    if let Some(failure) = &report.failure {
        let mut summary = File::create(output.join("failure.txt"))?;
        writeln!(
            summary,
            "seed={} profile={} operation_index={}",
            report.seed,
            report.profile,
            state.next_operation.load(Ordering::Relaxed)
        )?;
        writeln!(summary, "{failure}")?;
        writeln!(
            summary,
            "server stdout/stderr and JSONL event logs are retained in this directory"
        )?;
    }
    Ok(())
}

fn print_report(report: &Report) {
    println!(
        "duration={}s seed={} profile={} operations={} ops/s={:.1}",
        report.duration_ms as f64 / 1_000.0,
        report.seed,
        report.profile,
        report.operations_total,
        report.operations_per_second
    );
    println!(
        "gets={} puts={} deletes={} queries={} scans={} concurrent_gets={} transactions={}",
        report.operation_counts[0],
        report.operation_counts[1],
        report.operation_counts[2],
        report.operation_counts[3],
        report.operation_counts[4],
        report.operation_counts[5],
        report.operation_counts[6]
    );
    println!(
        "commits={} conflicts={} overloads={} response_budget={} unknown_resolved={} unresolved_unknowns={} checkpoints={} checkpoint_failures={} crashes={} recoveries={} verifications={} invariant_checks={} invariant_failures={} invariant_passes={}",
        report.commits,
        report.conflicts,
        report.overloads,
        report.response_budget_errors,
        report.unknown_outcomes_resolved,
        report.unresolved_unknown_mutations,
        report.checkpoints,
        report.checkpoint_failures,
        report.crashes,
        report.successful_recoveries,
        report.full_verification_passes,
        report.invariant_checks,
        report.invariant_failures,
        report.invariant_passes
    );
    println!(
        "server_requests={} server_connections={} server_transport_errors={} server_application_errors={} server_overloads={}",
        report.server_requests,
        report.server_connections,
        report.server_transport_errors,
        report.server_application_errors,
        report.server_overloads
    );
    println!(
        "latency_ms p50={:.3} p95={:.3} p99={:.3} max={:.3}",
        report.latency_p50_ms, report.latency_p95_ms, report.latency_p99_ms, report.latency_max_ms
    );
    println!(
        "rss={:?}->{:?} peak={:?} threads_peak={:?} fd={:?}->{:?} peak={:?} streams_peak={:?} connections_peak={:?} post_quiescence_streams={:?} post_quiescence_connections={:?} post_quiescence_fd={:?}",
        report.rss.initial,
        report.rss.final_value,
        report.rss.peak,
        report.threads.peak,
        report.file_descriptors.initial,
        report.file_descriptors.final_value,
        report.file_descriptors.peak,
        report.active_streams.peak,
        report.active_connections.peak,
        report.post_quiescence_active_streams,
        report.post_quiescence_active_connections,
        report.post_quiescence_fd_count
    );
    if report.trend_warnings.is_empty() {
        println!("resource trends: no sustained late-window warnings");
    } else {
        println!(
            "resource trend warnings: {}",
            report
                .resource_trend_warnings_by_phase
                .iter()
                .map(|(phase, warnings)| {
                    format!(
                        "{phase}:{}",
                        warnings
                            .iter()
                            .map(|warning| warning.metric.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    if let Some(bounded) = report.phase_key_counts.get("bounded") {
        println!(
            "bounded keyspace: {:?}->{:?}",
            bounded.initial, bounded.final_count
        );
    }
    for (phase, summary) in &report.phase_key_counts {
        println!(
            "phase={phase} keys={:?}->{:?} db_bytes={:?}->{:?} db_peak={:?} rss={:?}->{:?} rss_peak={:?} vm={:?}->{:?} vm_peak={:?} operations={:?}->{:?} checkpoints={:?}->{:?} reference_retained_value_bytes={:?}->{:?}",
            summary.initial,
            summary.final_count,
            summary.initial_database_bytes,
            summary.final_database_bytes,
            summary.database_bytes_peak,
            summary.rss_initial,
            summary.rss_final,
            summary.rss_peak,
            summary.virtual_memory_initial,
            summary.virtual_memory_final,
            summary.virtual_memory_peak,
            summary.operations_initial,
            summary.operations_final,
            summary.checkpoints_initial,
            summary.checkpoints_final,
            summary.reference_retained_value_bytes_initial,
            summary.reference_retained_value_bytes_final
        );
    }
    if let Some(failure) = &report.failure {
        println!("FAILURE: {failure}");
    } else {
        println!("correctness: PASS (not a production-safety claim)");
    }
}

async fn run(config: HarnessConfig) -> Result<(), BoxError> {
    fs::create_dir_all(&config.output_dir)?;
    let run_directory = config.output_dir.join(format!(
        "{}-{}-{}",
        config.profile,
        config.seed,
        timestamp()
    ));
    fs::create_dir_all(&run_directory)?;
    let data_dir = config
        .data_dir
        .clone()
        .unwrap_or_else(|| run_directory.join("data"));
    fs::create_dir_all(&data_dir)?;
    let tls = TlsMaterial::generate();
    let tls_files = tls.write(&run_directory)?;
    let state = RunState::new();
    for (phase, duration) in config.phase_durations() {
        if state.abort.load(Ordering::Relaxed) {
            break;
        }
        record_phase_key_count(&state, phase, true, &data_dir);
        let phase_result = match phase {
            "crash" => {
                run_crash_phase(
                    &state,
                    &config,
                    &tls,
                    &tls_files,
                    &data_dir,
                    &run_directory,
                    duration,
                )
                .await
            }
            _ => {
                run_normal_phase(
                    &state,
                    &config,
                    &tls,
                    &tls_files,
                    &data_dir,
                    &run_directory,
                    phase,
                    duration,
                    state.next_operation.load(Ordering::Relaxed),
                )
                .await
            }
        };
        if let Err(error) = phase_result {
            state.fail(format!("phase {phase} failed: {error}"));
            break;
        }
        record_phase_key_count(&state, phase, false, &data_dir);
    }
    let pending_count = state
        .pending
        .lock()
        .map(|pending| pending.len())
        .unwrap_or(usize::MAX);
    if pending_count != 0 {
        state.fail(format!(
            "successful run ended with {pending_count} unresolved unknown mutation(s)"
        ));
    }
    let (allocator_stats, _) = latest_allocator(&run_directory);
    let report = build_report(&state, &config, &run_directory, allocator_stats);
    write_failure_artifacts(&state, &report, &run_directory)?;
    println!("artifacts={}", run_directory.display());
    print_report(&report);
    if report.failure.is_some() {
        return Err("soak run failed; see failure.txt and report.json".into());
    }
    Ok(())
}

type AllocatorOrderKey = (u64, u64, u128, String, usize);
type AllocatorCandidate = (AllocatorOrderKey, serde_json::Value, PathBuf);

fn latest_allocator(directory: &Path) -> (Option<serde_json::Value>, Option<PathBuf>) {
    let mut latest: Option<AllocatorCandidate> = None;
    if let Ok(entries) = fs::read_dir(directory) {
        let mut entries = entries.flatten().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "jsonl")
                && let Ok(contents) = fs::read_to_string(entry.path())
            {
                let Ok(summary) = parse_event_lines(&contents, false) else {
                    continue;
                };
                let modified = entry
                    .metadata()
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |duration| duration.as_nanos());
                for sample in summary.metrics {
                    if let Some(allocator) = sample.allocator {
                        let file_name = entry.file_name().to_string_lossy().into_owned();
                        let key = (
                            sample.timestamp_ms,
                            sample.cycle,
                            modified,
                            file_name,
                            sample.sequence,
                        );
                        let candidate = (key, allocator, entry.path());
                        if latest
                            .as_ref()
                            .is_none_or(|current| candidate.0 > current.0)
                        {
                            latest = Some(candidate);
                        }
                    }
                }
            }
        }
    }
    latest
        .map(|(_, allocator, path)| (Some(allocator), Some(path)))
        .unwrap_or((None, None))
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), BoxError> {
    let args = env::args().collect::<Vec<_>>();
    if args.iter().any(|argument| argument == SERVER_CHILD) {
        return run_server_child(&args).await;
    }
    let mut config = HarnessConfig::parse().map_err(|error| -> BoxError { error.into() })?;
    config.normalize();
    run(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource_sample(
        streams: Option<u64>,
        connections: Option<u64>,
        fds: Option<u64>,
    ) -> ResourceSample {
        ResourceSample {
            phase: "test".to_owned(),
            elapsed_ms: 1,
            post_quiescence: true,
            rss_bytes: None,
            virtual_bytes: None,
            fd_count: fds,
            thread_count: None,
            wal_bytes: 0,
            database_bytes: 0,
            active_connections: connections,
            active_streams: streams,
        }
    }

    #[test]
    fn conflict_validation_does_not_require_model_history() {
        let key = DocumentKey::new("p", "s");
        let request = TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: key.clone(),
                expected_revision: Revision::new(4),
            }],
            vec![TransactionMutation::Delete { key: key.clone() }],
        );
        let conflict = ConflictDetails {
            key: key.clone(),
            expected: ConditionExpectation::RevisionEquals(Revision::new(4)),
            actual: ObservedState::present(Revision::new(5)),
        };
        assert!(validate_conflict_details(&request, &conflict).is_ok());
        let satisfied = ConflictDetails {
            actual: ObservedState::present(Revision::new(4)),
            ..conflict
        };
        assert!(validate_conflict_details(&request, &satisfied).is_err());
    }

    #[test]
    fn post_quiescence_cleanup_requires_zero_streams_and_connections() {
        assert!(
            validate_post_quiescence_samples(&[resource_sample(Some(0), Some(0), Some(12),)])
                .is_ok()
        );
        assert!(
            validate_post_quiescence_samples(&[resource_sample(Some(1), Some(0), Some(12),)])
                .is_err()
        );
        assert!(
            validate_post_quiescence_samples(&[resource_sample(Some(0), Some(1), Some(12),)])
                .is_err()
        );
        assert!(validate_post_quiescence_samples(&[]).is_err());
    }

    #[test]
    fn explicit_close_is_expected_only_during_crash_quiescence() {
        let closed = ClientError::Endpoint("client connection is explicitly closed".to_owned());
        assert!(is_allowed_transport_error(&closed, true));
        assert!(!is_allowed_transport_error(&closed, false));
        assert!(!is_allowed_transport_error(
            &ClientError::Endpoint("other endpoint error".to_owned()),
            true
        ));
    }

    #[test]
    fn latest_allocator_uses_event_order_not_directory_iteration_order() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("server-2.jsonl"),
            "{\"kind\":\"metrics\",\"timestamp_ms\":20,\"cycle\":2,\"allocator\":\"{\\\"sample\\\":2}\"}\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("server-10.jsonl"),
            "{\"kind\":\"metrics\",\"timestamp_ms\":10,\"cycle\":10,\"allocator\":\"{\\\"sample\\\":10}\"}\n",
        )
        .unwrap();
        let (allocator, path) = latest_allocator(directory.path());
        assert_eq!(allocator.unwrap()["sample"], serde_json::json!(2));
        assert_eq!(path.unwrap().file_name().unwrap(), "server-2.jsonl");
    }

    #[test]
    fn synchronized_event_writer_keeps_records_parseable() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("events.jsonl");
        let writer = EventWriter::new(path.clone(), 7);
        let mut threads = Vec::new();
        for index in 0..8 {
            let writer = writer.clone();
            threads.push(std::thread::spawn(move || {
                writer
                    .append(&serde_json::json!({"kind":"metrics", "index":index}))
                    .unwrap();
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
        let parsed = parse_event_lines(&fs::read_to_string(path).unwrap(), false).unwrap();
        assert_eq!(parsed.metrics.len(), 8);
    }

    #[test]
    fn phase_workload_configs_keep_explicit_keyspace_bounds() {
        let config = HarnessConfig {
            tenant_count: 4,
            ..HarnessConfig::default()
        };
        let growth = workload_config(&config, "growth");
        assert!(!growth.bounded_keyspace);
        assert!(growth.growth_fill_operations >= 4 * u64::from(growth.growth_target_key_count));
        assert!(growth.maximum_generated_key_count() < 5_000);

        for (phase, wide_key_count) in [("bounded", 256), ("contention", 64), ("crash", 128)] {
            let workload = workload_config(&config, phase);
            assert!(workload.bounded_keyspace, "{phase} must be bounded");
            assert_eq!(workload.wide_key_count, wide_key_count);
            assert!(workload.maximum_generated_key_count() < 2_000);
        }
    }
}
