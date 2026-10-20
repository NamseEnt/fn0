use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dodb_client::{ClientError, ClientTlsConfig, DodbConnection};
use dodb_core::{Error, TenantId, TransactionConflict};
use dodb_protocol::{ApplicationErrorKind, ProtocolLimits};
use dodb_service::{DodbService, ExecutionBudget, Request, Response, ServiceFuture};
use rcgen::generate_simple_self_signed;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const PROCESS_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const MAX_MEMORY_DIAGNOSTIC_SAMPLES: usize = 4_096;
const WORKER_START_TIMEOUT: Duration = Duration::from_secs(20);
const WORKER_STOP_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Debug)]
struct ProcessIdentity {
    role: String,
    pid: u32,
    process_group_id: i32,
    start_time_ticks: u64,
    start_time_unix_ms: u64,
}

pub struct MemoryDiagnosticSession {
    source_revision: String,
    parent_binary: PathBuf,
    parent_binary_sha256: String,
    certificate_der: Vec<u8>,
    tls_directory: TempDir,
    process_identities: Arc<Mutex<Vec<ProcessIdentity>>>,
    phase: Arc<Mutex<String>>,
    process_samples: Arc<Mutex<Vec<Value>>>,
    host_samples: Arc<Mutex<Vec<Value>>>,
    process_samples_truncated: Arc<AtomicBool>,
    host_samples_truncated: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    sampler: Mutex<Option<thread::JoinHandle<()>>>,
    worker_events: Mutex<Vec<Value>>,
    health_reports: Mutex<Vec<Value>>,
    residency_reports: Mutex<Vec<Value>>,
}

impl MemoryDiagnosticSession {
    pub fn new(source_revision: impl Into<String>) -> Result<Arc<Self>, String> {
        let executable_path = std::env::current_exe().map_err(|error| error.to_string())?;
        let parent_binary_sha256 = sha256_file(&executable_path)?;
        let certified_key =
            generate_simple_self_signed(vec!["localhost".to_owned()]).map_err(|error| {
                format!("temporary diagnostic certificate generation failed: {error}")
            })?;
        let certificate_der = certified_key.cert.der().to_vec();
        let private_key_der = certified_key.signing_key.serialize_der();
        let tls_directory = tempfile::Builder::new()
            .prefix("dodb-memory-diagnostic-")
            .tempdir()
            .map_err(|error| error.to_string())?;
        let certificate_path = tls_directory.path().join("server-cert.der");
        let private_key_path = tls_directory.path().join("server-key.der");
        fs::write(&certificate_path, &certificate_der).map_err(|error| error.to_string())?;
        fs::write(&private_key_path, &private_key_der).map_err(|error| error.to_string())?;
        fs::set_permissions(&private_key_path, fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;

        let parent_identity = process_identity("harness-process", std::process::id())
            .ok_or_else(|| "could not read harness process start time".to_owned())?;
        let process_identities = Arc::new(Mutex::new(vec![parent_identity]));
        let phase = Arc::new(Mutex::new("setup".to_owned()));
        let process_samples = Arc::new(Mutex::new(Vec::with_capacity(1_024)));
        let host_samples = Arc::new(Mutex::new(Vec::with_capacity(1_024)));
        let process_samples_truncated = Arc::new(AtomicBool::new(false));
        let host_samples_truncated = Arc::new(AtomicBool::new(false));
        let stopped = Arc::new(AtomicBool::new(false));
        let sampler = start_sampling(
            Arc::clone(&process_identities),
            Arc::clone(&phase),
            Arc::clone(&process_samples),
            Arc::clone(&host_samples),
            Arc::clone(&process_samples_truncated),
            Arc::clone(&host_samples_truncated),
            Arc::clone(&stopped),
        )?;

        Ok(Arc::new(Self {
            source_revision: source_revision.into(),
            parent_binary: executable_path,
            parent_binary_sha256,
            certificate_der,
            tls_directory,
            process_identities,
            phase,
            process_samples,
            host_samples,
            process_samples_truncated,
            host_samples_truncated,
            stopped,
            sampler: Mutex::new(Some(sampler)),
            worker_events: Mutex::new(Vec::new()),
            health_reports: Mutex::new(Vec::new()),
            residency_reports: Mutex::new(Vec::new()),
        }))
    }

    pub fn set_phase(&self, phase_name: impl Into<String>) {
        if let Ok(mut phase) = self.phase.lock() {
            *phase = phase_name.into();
        }
    }

    pub fn boundary_snapshot(&self) -> Value {
        host_snapshot()
    }

    pub fn process_boundary_snapshot(&self) -> Value {
        let identities = self
            .process_identities
            .lock()
            .map(|identities| identities.clone())
            .unwrap_or_default();
        json!({
            "timestamp_unix_ms": unix_time_ms(),
            "processes": identities.iter().map(process_usage).collect::<Vec<_>>(),
        })
    }

    pub fn record_residency_report(&self, report: Value) {
        if let Ok(mut reports) = self.residency_reports.lock() {
            reports.push(report);
        }
    }

    pub fn finish_sampling(&self) -> Value {
        self.stopped.store(true, Ordering::Relaxed);
        if let Ok(mut sampler) = self.sampler.lock()
            && let Some(sampler) = sampler.take()
        {
            let _ = sampler.join();
        }
        self.report()
    }

    pub async fn spawn_worker(
        self: &Arc<Self>,
        worker_executable: &Path,
        worker_arguments: &[String],
        response_budget_bytes: usize,
        maximum_mutations: usize,
        data_dir: &Path,
    ) -> Result<MemoryDiagnosticWorker, String> {
        let worker_binary_sha256 = sha256_file(worker_executable)?;
        let health_path = self
            .tls_directory
            .path()
            .join(format!("engine-health-{}.json", unix_time_ms()));
        let certificate_path = self.tls_directory.path().join("server-cert.der");
        let private_key_path = self.tls_directory.path().join("server-key.der");
        let mut command = Command::new(worker_executable);
        command
            .args(worker_arguments)
            .arg("--memory-diagnostic-worker")
            .arg("--memory-source-revision")
            .arg(&self.source_revision)
            .arg("--memory-parent-binary-sha256")
            .arg(&self.parent_binary_sha256)
            .arg("--memory-worker-binary-sha256")
            .arg(&worker_binary_sha256)
            .arg("--memory-certificate")
            .arg(&certificate_path)
            .arg("--memory-private-key")
            .arg(&private_key_path)
            .arg("--memory-health-report")
            .arg(&health_path)
            .arg("--memory-data-dir")
            .arg(data_dir)
            .arg("--memory-response-budget")
            .arg(response_budget_bytes.to_string())
            .arg("--memory-max-mutations")
            .arg(maximum_mutations.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        let child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| "diagnostic worker stdin was not captured".to_owned())?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| "diagnostic worker stdout was not captured".to_owned())?;
        let (ready_sender, ready_receiver) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let mut ready_line = String::new();
            let read_result = BufReader::new(child_stdout)
                .read_line(&mut ready_line)
                .map_err(|error| error.to_string())
                .and_then(|read_length| {
                    if read_length == 0 {
                        Err("diagnostic worker closed stdout before readiness".to_owned())
                    } else {
                        Ok(ready_line)
                    }
                });
            let _ = ready_sender.send(read_result);
        });
        let readiness_line = match tokio::task::spawn_blocking(move || {
            ready_receiver.recv_timeout(WORKER_START_TIMEOUT)
        })
        .await
        .map_err(|error| format!("worker readiness wait task failed: {error}"))?
        {
            Ok(Ok(readiness_line)) => readiness_line,
            Ok(Err(error)) => {
                terminate_child(&mut child);
                return Err(error);
            }
            Err(error) => {
                terminate_child(&mut child);
                return Err(format!("diagnostic worker readiness timed out: {error}"));
            }
        };
        let readiness: WorkerReadiness = match serde_json::from_str(&readiness_line) {
            Ok(readiness) => readiness,
            Err(error) => {
                terminate_child(&mut child);
                return Err(format!("invalid diagnostic worker readiness: {error}"));
            }
        };
        if !readiness.ready
            || readiness.pid != child.id()
            || readiness.source_revision != self.source_revision
            || readiness.worker_binary_sha256 != worker_binary_sha256
        {
            terminate_child(&mut child);
            return Err("diagnostic worker provenance did not match the parent".to_owned());
        }
        let server_address = match readiness.listen_address.parse::<SocketAddr>() {
            Ok(server_address) => server_address,
            Err(error) => {
                terminate_child(&mut child);
                return Err(format!("invalid diagnostic worker address: {error}"));
            }
        };
        let protocol_limits = ProtocolLimits {
            max_response_frame_size: response_budget_bytes,
            max_mutations: maximum_mutations,
            ..ProtocolLimits::default()
        };
        let tls = match ClientTlsConfig::from_der(vec![self.certificate_der.clone()]) {
            Ok(tls) => tls,
            Err(error) => {
                terminate_child(&mut child);
                return Err(error.to_string());
            }
        };
        let connection = match tokio::time::timeout(
            Duration::from_secs(15),
            DodbConnection::connect(
                "127.0.0.1:0".parse().unwrap(),
                server_address,
                "localhost",
                tls,
                protocol_limits,
            ),
        )
        .await
        {
            Ok(Ok(connection)) => connection,
            Ok(Err(error)) => {
                terminate_child(&mut child);
                return Err(format!("diagnostic worker connection failed: {error}"));
            }
            Err(error) => {
                terminate_child(&mut child);
                return Err(format!("diagnostic worker connection timed out: {error}"));
            }
        };
        let identity = ProcessIdentity {
            role: "backend-process".to_owned(),
            pid: readiness.pid,
            process_group_id: readiness.process_group_id,
            start_time_ticks: readiness.process_start_time_ticks,
            start_time_unix_ms: readiness.started_unix_ms,
        };
        let parent_process_group_id = match self.process_identities.lock() {
            Ok(identities) => identities.first().map(|identity| identity.process_group_id),
            Err(_) => None,
        };
        let Some(parent_process_group_id) = parent_process_group_id else {
            terminate_child(&mut child);
            return Err("harness process identity is missing".to_owned());
        };
        if identity.process_group_id != parent_process_group_id {
            terminate_child(&mut child);
            return Err("diagnostic worker did not inherit the workload process group".to_owned());
        }
        match self.process_identities.lock() {
            Ok(mut identities) => identities.push(identity),
            Err(_) => {
                terminate_child(&mut child);
                return Err("process identity lock poisoned".to_owned());
            }
        }
        let worker = MemoryDiagnosticWorker {
            child: Mutex::new(child),
            stdin: Mutex::new(child_stdin),
            connection,
            health_path,
            session: Arc::clone(self),
            readiness,
            worker_binary_sha256,
            stopped: AtomicBool::new(false),
        };
        worker.record_event("started");
        Ok(worker)
    }

    pub fn report(&self) -> Value {
        let process_samples = self
            .process_samples
            .lock()
            .map(|samples| samples.clone())
            .unwrap_or_default();
        let host_samples = self
            .host_samples
            .lock()
            .map(|samples| samples.clone())
            .unwrap_or_default();
        let process_identities = self
            .process_identities
            .lock()
            .map(|identities| {
                identities
                    .iter()
                    .map(|identity| {
                        json!({
                            "role": identity.role,
                            "pid": identity.pid,
                            "process_group_id": identity.process_group_id,
                            "start_time_ticks": identity.start_time_ticks,
                            "start_time_unix_ms": identity.start_time_unix_ms,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let worker_events = self
            .worker_events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default();
        let health_reports = self
            .health_reports
            .lock()
            .map(|reports| reports.clone())
            .unwrap_or_default();
        let residency_reports = self
            .residency_reports
            .lock()
            .map(|reports| reports.clone())
            .unwrap_or_default();
        json!({
            "enabled": true,
            "label": "memory-only diagnostic; loopback QUIC/TLS transport adds costs and does not replace direct-path throughput or latency evidence",
            "source_revision": self.source_revision,
            "harness_binary": self.parent_binary,
            "harness_binary_sha256": self.parent_binary_sha256,
            "backend_process_definition": "backend, service adapter, runtime, and loopback transport in the worker process; not a pure native allocator RSS measurement",
            "process_memory_sampling_interval_ms": PROCESS_SAMPLE_INTERVAL.as_millis(),
            "process_memory_sample_count": process_samples.len(),
            "process_memory_sample_limit": MAX_MEMORY_DIAGNOSTIC_SAMPLES,
            "process_memory_samples_truncated": self.process_samples_truncated.load(Ordering::Relaxed),
            "processes": process_identities,
            "worker_lifecycle": worker_events,
            "process_memory_samples": process_samples,
            "process_group_scope": "sum of sampled harness-process and backend-process RSS/PSS for the recorded PID/starttime identities",
            "workload_process_group_id": process_identities.first().and_then(|identity| identity["process_group_id"].as_i64()),
            "host_sampling_interval_ms": PROCESS_SAMPLE_INTERVAL.as_millis(),
            "host_sample_count": host_samples.len(),
            "host_samples_truncated": self.host_samples_truncated.load(Ordering::Relaxed),
            "host_samples": host_samples,
            "vdev_mapping": {
                "block_device": std::env::var("DODB_MEMORY_VDEV_DEVICE").unwrap_or_else(|_| "sda4".to_owned()),
                "source": "/sys/class/block/<device>/stat field 3",
                "read_sector_size_bytes": 512,
                "scope": "whole host device counter; unrelated concurrent I/O is not attributed to this workload"
            },
            "engine_health_reports": health_reports,
            "residency_scans": residency_reports,
        })
    }

    fn record_event(&self, event: Value) {
        if let Ok(mut events) = self.worker_events.lock() {
            events.push(event);
        }
    }
}

impl Drop for MemoryDiagnosticSession {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Ok(worker) = self.sampler.get_mut()
            && let Some(worker) = worker.take()
        {
            let _ = worker.join();
        }
    }
}

pub struct MemoryDiagnosticWorker {
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    connection: DodbConnection,
    health_path: PathBuf,
    session: Arc<MemoryDiagnosticSession>,
    readiness: WorkerReadiness,
    worker_binary_sha256: String,
    stopped: AtomicBool,
}

impl MemoryDiagnosticWorker {
    pub fn connection(&self) -> &DodbConnection {
        &self.connection
    }

    pub fn start_measurement(&self) -> Result<(), String> {
        self.send_command("start-measurement")
    }

    pub fn stop_measurement(&self) -> Result<(), String> {
        self.send_command("stop-measurement")
    }

    pub fn set_phase(&self, phase_name: impl Into<String>) {
        self.session.set_phase(phase_name);
    }

    pub fn shutdown(&self) -> Result<(), String> {
        if self.stopped.swap(true, Ordering::Relaxed) {
            return Ok(());
        }
        self.connection.close();
        if let Err(error) = self.send_command("shutdown") {
            if let Ok(mut child) = self.child.lock() {
                terminate_child(&mut child);
            }
            return Err(error);
        }
        let stop_started = Instant::now();
        let exit_status = loop {
            let exit_status = self
                .child
                .lock()
                .map_err(|_| "diagnostic worker process lock poisoned".to_owned())?
                .try_wait()
                .map_err(|error| error.to_string())?;
            if exit_status.is_some() {
                break exit_status;
            }
            if stop_started.elapsed() >= WORKER_STOP_TIMEOUT {
                if let Ok(mut child) = self.child.lock() {
                    terminate_child(&mut child);
                }
                return Err("diagnostic worker shutdown timed out and was killed".to_owned());
            }
            thread::sleep(Duration::from_millis(50));
        };
        if !exit_status.is_some_and(|status| status.success()) {
            return Err(format!(
                "diagnostic worker exited unsuccessfully: {exit_status:?}"
            ));
        }
        let report = fs::read(&self.health_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        let report =
            report.ok_or_else(|| "diagnostic worker omitted its health report".to_owned())?;
        if let Ok(mut reports) = self.session.health_reports.lock() {
            reports.push(report);
        }
        self.record_event("stopped");
        Ok(())
    }

    fn send_command(&self, command: &str) -> Result<(), String> {
        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| "diagnostic worker command pipe lock poisoned".to_owned())?;
        writeln!(stdin, "{command}").map_err(|error| error.to_string())?;
        stdin.flush().map_err(|error| error.to_string())
    }

    fn record_event(&self, event_name: &str) {
        self.session.record_event(json!({
            "event": event_name,
            "role": "backend-process",
            "pid": self.readiness.pid,
            "started_unix_ms": self.readiness.started_unix_ms,
            "source_revision": self.readiness.source_revision,
            "harness_binary_sha256": self.session.parent_binary_sha256,
            "worker_binary_sha256": self.worker_binary_sha256,
        }));
    }
}

impl Drop for MemoryDiagnosticWorker {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            if child.try_wait().ok().flatten().is_none() {
                self.connection.close();
                terminate_child(&mut child);
            }
        }
    }
}

#[derive(Deserialize)]
pub struct WorkerReadiness {
    pub ready: bool,
    pub listen_address: String,
    pub pid: u32,
    pub process_group_id: i32,
    pub started_unix_ms: u64,
    pub process_start_time_ticks: u64,
    pub source_revision: String,
    pub worker_binary_sha256: String,
}

pub struct RemoteDodbService {
    connection: DodbConnection,
}

pub async fn bounded_scan_digest(
    connection: &DodbConnection,
    tenant: TenantId,
    response_budget_bytes: usize,
) -> Result<Value, String> {
    const PAGE_LIMIT: usize = 128;
    let client = connection.for_tenant(tenant);
    let mut cursor = None;
    let mut row_count = 0_u64;
    let mut key_bytes = 0_u64;
    let mut value_bytes = 0_u64;
    let mut digest = Sha256::new();
    loop {
        let page = client
            .scan(cursor.clone(), PAGE_LIMIT)
            .await
            .map_err(|error| error.to_string())?;
        if page.is_empty() {
            break;
        }
        for document in &page {
            let encoded_key = document.key.encode();
            key_bytes = key_bytes.saturating_add(encoded_key.len() as u64);
            value_bytes = value_bytes.saturating_add(document.value.len() as u64);
            row_count = row_count.saturating_add(1);
            digest.update((encoded_key.len() as u64).to_be_bytes());
            digest.update(&encoded_key);
            digest.update((document.value.len() as u64).to_be_bytes());
            digest.update(&document.value);
            digest.update(document.revision.get().to_be_bytes());
        }
        cursor = page.last().map(|document| document.key.clone());
        if page.len() < PAGE_LIMIT {
            break;
        }
    }
    let digest = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(json!({
        "row_count": row_count,
        "key_bytes": key_bytes,
        "value_bytes": value_bytes,
        "row_value_bytes": key_bytes.saturating_add(value_bytes),
        "canonical_page_size": PAGE_LIMIT,
        "response_budget_bytes": response_budget_bytes,
        "digest_sha256": digest,
        "bounded_worker_materialization": "one protocol page at a time; no full-result worker materialization",
    }))
}

pub fn residency_scan_report(
    first_scan: &Value,
    second_scan: &Value,
    host_before: &Value,
    host_after: &Value,
    process_before: &Value,
    process_after: &Value,
    reopen_oracle_passed: bool,
) -> Value {
    let read_before = host_before["vdev_read_bytes"].as_u64();
    let read_after = host_after["vdev_read_bytes"].as_u64();
    let physical_read_delta_bytes = read_before
        .zip(read_after)
        .and_then(|(before, after)| after.checked_sub(before));
    let same_vdev_mapping = host_before["vdev_block_device"]
        .as_str()
        .zip(host_after["vdev_block_device"].as_str())
        .is_some_and(|(before, after)| before == after)
        && host_before["vdev_device_stat_path"] == host_after["vdev_device_stat_path"];
    let backend_swap = |snapshot: &Value| {
        snapshot["processes"]
            .as_array()
            .and_then(|processes| {
                processes
                    .iter()
                    .rev()
                    .find(|process| process["role"] == "backend-process")
            })
            .and_then(|process| process["swap_kb"].as_u64())
    };
    let backend_swap_before_kb = backend_swap(process_before);
    let backend_swap_after_kb = backend_swap(process_after);
    let backend_identity = |snapshot: &Value| {
        snapshot["processes"]
            .as_array()
            .and_then(|processes| {
                processes.iter().rev().find(|process| {
                    process["role"] == "backend-process" && process["status"] == "sampled"
                })
            })
            .map(|process| {
                (
                    process["pid"].as_u64(),
                    process["start_time_ticks"].as_u64(),
                )
            })
    };
    let backend_identity_before = backend_identity(process_before);
    let backend_identity_after = backend_identity(process_after);
    let same_backend_process =
        backend_identity_before.is_some() && backend_identity_before == backend_identity_after;
    let same_data = first_scan["row_count"] == second_scan["row_count"]
        && first_scan["key_bytes"] == second_scan["key_bytes"]
        && first_scan["value_bytes"] == second_scan["value_bytes"]
        && first_scan["digest_sha256"] == second_scan["digest_sha256"];
    let served_from_ram = reopen_oracle_passed
        && same_data
        && same_vdev_mapping
        && same_backend_process
        && physical_read_delta_bytes == Some(0)
        && backend_swap_before_kb == Some(0)
        && backend_swap_after_kb == Some(0);
    json!({
        "first_scan_cache_warmup": first_scan,
        "second_scan": second_scan,
        "second_scan_physical_read_delta_bytes": physical_read_delta_bytes,
        "same_vdev_mapping_before_and_after": same_vdev_mapping,
        "same_backend_pid_and_starttime_before_and_after": same_backend_process,
        "backend_identity_before": backend_identity_before,
        "backend_identity_after": backend_identity_after,
        "first_scan_boundary_host": host_before,
        "second_scan_boundary_host": host_after,
        "second_scan_boundary_backend_process_before": process_before,
        "second_scan_boundary_backend_process_after": process_after,
        "backend_swap_before_kb": backend_swap_before_kb,
        "backend_swap_after_kb": backend_swap_after_kb,
        "reopen_oracle_passed": reopen_oracle_passed,
        "scan_fingerprints_match": same_data,
        "served_from_ram_for_second_bounded_scan": served_from_ram,
        "conclusion": if served_from_ram {
            "the second bounded full Scan matched the reopened data oracle with zero vdev read delta and zero backend-process Swap"
        } else {
            "RAM residency was not verified for the second bounded full Scan"
        },
        "scope": "this second bounded Scan only; not a retrospective proof of earlier benchmark residency",
    })
}

impl RemoteDodbService {
    pub fn new(connection: DodbConnection) -> Self {
        Self { connection }
    }

    pub fn connection(&self) -> &DodbConnection {
        &self.connection
    }
}

impl DodbService for RemoteDodbService {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        _budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move {
            let client = self.connection.for_tenant(tenant);
            match request {
                Request::Get { key } => client
                    .get(key)
                    .await
                    .map(Response::Get)
                    .map_err(client_error),
                Request::Put { key, value } => client
                    .put(key, value)
                    .await
                    .map(Response::Put)
                    .map_err(client_error),
                Request::Delete { key } => client
                    .delete(key)
                    .await
                    .map(Response::Delete)
                    .map_err(client_error),
                Request::Query {
                    pk,
                    exclusive_after_sk,
                    limit,
                } => client
                    .query(pk, exclusive_after_sk, limit)
                    .await
                    .map(Response::Query)
                    .map_err(client_error),
                Request::Scan {
                    exclusive_after_key,
                    limit,
                } => client
                    .scan(exclusive_after_key, limit)
                    .await
                    .map(Response::Scan)
                    .map_err(client_error),
                Request::Transact { request } => client
                    .transact(request)
                    .await
                    .map(Response::Transact)
                    .map_err(client_error),
            }
        })
    }
}

fn client_error(error: ClientError) -> Error {
    match error {
        ClientError::Application(application)
            if application.kind == ApplicationErrorKind::Conflict =>
        {
            match application.conflict {
                Some(conflict) => Error::Conflict(TransactionConflict {
                    key: conflict.key,
                    expected: conflict.expected,
                    actual: conflict.actual,
                }),
                None => Error::invariant("transport conflict omitted structured details"),
            }
        }
        ClientError::Application(application)
            if application.kind == ApplicationErrorKind::Overloaded =>
        {
            Error::overloaded(application.detail)
        }
        ClientError::Application(application) => Error::invariant(format!(
            "remote service returned {:?}: {}",
            application.kind, application.detail
        )),
        ClientError::UnknownMutationOutcome { detail, .. } => Error::durability(detail),
        other => Error::invariant(other.to_string()),
    }
}

pub fn worker_readiness(
    listen_address: SocketAddr,
    source_revision: String,
    expected_worker_binary_sha256: String,
) -> Result<WorkerReadiness, String> {
    let pid = std::process::id();
    let worker_binary_sha256 = sha256_file(Path::new("/proc/self/exe"))?;
    if worker_binary_sha256 != expected_worker_binary_sha256 {
        return Err(
            "diagnostic worker executable hash did not match the parent expectation".to_owned(),
        );
    }
    let process_group_id = process_group_id(pid)
        .ok_or_else(|| "could not read diagnostic worker process group".to_owned())?;
    let start_ticks = process_start_time_ticks(pid)
        .ok_or_else(|| "could not read diagnostic worker process start time".to_owned())?;
    Ok(WorkerReadiness {
        ready: true,
        listen_address: listen_address.to_string(),
        pid,
        process_group_id,
        started_unix_ms: unix_time_ms(),
        process_start_time_ticks: start_ticks,
        source_revision,
        worker_binary_sha256,
    })
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let contents = fs::read(path).map_err(|error| error.to_string())?;
    let digest = Sha256::digest(contents);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub fn worker_argument_value(arguments: &[String], name: &str) -> Result<String, String> {
    let position = arguments
        .iter()
        .position(|argument| argument == name)
        .ok_or_else(|| format!("missing worker argument {name}"))?;
    arguments
        .get(position + 1)
        .cloned()
        .ok_or_else(|| format!("missing value for worker argument {name}"))
}

fn start_sampling(
    process_identities: Arc<Mutex<Vec<ProcessIdentity>>>,
    phase: Arc<Mutex<String>>,
    process_samples: Arc<Mutex<Vec<Value>>>,
    host_samples: Arc<Mutex<Vec<Value>>>,
    process_samples_truncated: Arc<AtomicBool>,
    host_samples_truncated: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
) -> Result<thread::JoinHandle<()>, String> {
    thread::Builder::new()
        .name("memory-diagnostic-sampler".to_owned())
        .spawn(move || {
            let started = Instant::now();
            while !stopped.load(Ordering::Relaxed) {
                let timestamp_ms = unix_time_ms();
                let phase_name = phase
                    .lock()
                    .map(|phase| phase.clone())
                    .unwrap_or_else(|_| "phase-lock-poisoned".to_owned());
                let identities = process_identities
                    .lock()
                    .map(|identities| identities.clone())
                    .unwrap_or_default();
                let process_usage = identities.iter().map(process_usage).collect::<Vec<_>>();
                let group_total = sum_process_usage(&process_usage);
                let sample = json!({
                    "timestamp_unix_ms": timestamp_ms,
                    "elapsed_ms": started.elapsed().as_millis(),
                    "phase": phase_name,
                    "processes": process_usage,
                    "process_group_total": group_total,
                });
                if let Ok(mut samples) = process_samples.lock() {
                    if samples.len() < MAX_MEMORY_DIAGNOSTIC_SAMPLES {
                        samples.push(sample);
                    } else {
                        process_samples_truncated.store(true, Ordering::Relaxed);
                    }
                }
                if let Ok(mut samples) = host_samples.lock() {
                    if samples.len() < MAX_MEMORY_DIAGNOSTIC_SAMPLES {
                        samples.push(host_snapshot());
                    } else {
                        host_samples_truncated.store(true, Ordering::Relaxed);
                    }
                }
                let sleep_started = Instant::now();
                while !stopped.load(Ordering::Relaxed)
                    && sleep_started.elapsed() < PROCESS_SAMPLE_INTERVAL
                {
                    thread::sleep(
                        PROCESS_SAMPLE_INTERVAL
                            .saturating_sub(sleep_started.elapsed())
                            .min(Duration::from_millis(100)),
                    );
                }
            }
        })
        .map_err(|error| format!("memory diagnostic sampler creation failed: {error}"))
}

fn process_usage(identity: &ProcessIdentity) -> Value {
    let process_path = PathBuf::from(format!("/proc/{}", identity.pid));
    let current_start_ticks = process_start_time_ticks(identity.pid);
    if current_start_ticks != Some(identity.start_time_ticks) {
        return json!({
            "role": identity.role,
            "pid": identity.pid,
            "process_group_id": identity.process_group_id,
            "start_time_ticks": identity.start_time_ticks,
            "status": "exited-or-pid-reused",
        });
    }
    let status = fs::read_to_string(process_path.join("status")).ok();
    let rollup = fs::read_to_string(process_path.join("smaps_rollup")).ok();
    let status_values = status
        .as_deref()
        .map(parse_memory_fields)
        .unwrap_or_default();
    let rollup_values = rollup
        .as_deref()
        .map(parse_memory_fields)
        .unwrap_or_default();
    json!({
        "role": identity.role,
        "pid": identity.pid,
        "process_group_id": identity.process_group_id,
        "start_time_ticks": identity.start_time_ticks,
        "status": if rollup.is_some() { "sampled" } else { "smaps_rollup_unavailable" },
        "rss_kb": status_values.get("VmRSS").copied().or_else(|| rollup_values.get("Rss").copied()),
        "pss_kb": rollup_values.get("Pss").copied(),
        "anonymous_kb": rollup_values.get("Anonymous").copied(),
        "file_kb": rollup_values.get("Pss_File").copied(),
        "swap_kb": rollup_values.get("Swap").copied().or_else(|| status_values.get("VmSwap").copied()),
    })
}

fn sum_process_usage(processes: &[Value]) -> Value {
    let mut totals = BTreeMap::<&str, u64>::new();
    for process in processes {
        for field_name in ["rss_kb", "pss_kb", "anonymous_kb", "file_kb", "swap_kb"] {
            if let Some(value) = process[field_name].as_u64() {
                let total = totals.entry(field_name).or_default();
                *total = total.saturating_add(value);
            }
        }
    }
    json!(totals)
}

fn host_snapshot() -> Value {
    let memory = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|contents| parse_memory_fields(&contents).get("MemAvailable").copied());
    let block_device =
        std::env::var("DODB_MEMORY_VDEV_DEVICE").unwrap_or_else(|_| "sda4".to_owned());
    let device_stats_path = PathBuf::from(format!("/sys/class/block/{block_device}/stat"));
    let device_stats = fs::read_to_string(&device_stats_path)
        .ok()
        .and_then(|contents| {
            let fields = contents
                .split_whitespace()
                .map(str::parse::<u64>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            fields.get(2).copied()
        });
    let arcstats = fs::read_to_string("/proc/spl/kstat/zfs/arcstats")
        .ok()
        .map(|contents| parse_arcstats(&contents));
    json!({
        "timestamp_unix_ms": unix_time_ms(),
        "mem_available_kb": memory,
        "zfs_arcstats": arcstats,
        "vdev_block_device": block_device,
        "vdev_read_sectors_512_bytes": device_stats,
        "vdev_read_bytes": device_stats.map(|sectors| sectors.saturating_mul(512)),
        "vdev_device_stat_path": device_stats_path,
    })
}

fn parse_memory_fields(contents: &str) -> BTreeMap<String, u64> {
    contents
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            let amount = value.split_whitespace().next()?.parse::<u64>().ok()?;
            Some((name.to_owned(), amount))
        })
        .collect()
}

fn parse_arcstats(contents: &str) -> Value {
    let supported_names = [
        "c",
        "c_min",
        "c_max",
        "size",
        "compressed_size",
        "uncompressed_size",
        "hits",
        "misses",
        "demand_data_hits",
        "demand_data_misses",
    ];
    let values = contents
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            if !supported_names.contains(&name) {
                return None;
            }
            let _data_type = fields.next()?;
            let value = fields.next()?.parse::<u64>().ok()?;
            Some((name.to_owned(), json!(value)))
        })
        .collect::<serde_json::Map<_, _>>();
    Value::Object(values)
}

fn process_identity(role: &str, pid: u32) -> Option<ProcessIdentity> {
    Some(ProcessIdentity {
        role: role.to_owned(),
        pid,
        process_group_id: process_group_id(pid)?,
        start_time_ticks: process_start_time_ticks(pid)?,
        start_time_unix_ms: unix_time_ms(),
    })
}

fn process_group_id(pid: u32) -> Option<i32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let fields = stat
        .rsplit_once(')')?
        .1
        .split_whitespace()
        .collect::<Vec<_>>();
    fields.get(2)?.parse().ok()
}

fn process_start_time_ticks(pid: u32) -> Option<u64> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let fields = stat
        .rsplit_once(')')?
        .1
        .split_whitespace()
        .collect::<Vec<_>>();
    fields.get(19)?.parse().ok()
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::residency_scan_report;
    use serde_json::json;

    fn scan() -> serde_json::Value {
        json!({
            "row_count": 2,
            "key_bytes": 20,
            "value_bytes": 1024,
            "digest_sha256": "same",
        })
    }

    fn host(read_bytes: u64, device: &str) -> serde_json::Value {
        json!({
            "vdev_read_bytes": read_bytes,
            "vdev_block_device": device,
            "vdev_device_stat_path": format!("/sys/class/block/{device}/stat"),
        })
    }

    fn host_without_counter(device: &str) -> serde_json::Value {
        json!({
            "vdev_block_device": device,
            "vdev_device_stat_path": format!("/sys/class/block/{device}/stat"),
        })
    }

    fn process(pid: u64, start_time_ticks: u64) -> serde_json::Value {
        json!({
            "processes": [{
                "role": "backend-process",
                "pid": pid,
                "start_time_ticks": start_time_ticks,
                "status": "sampled",
                "swap_kb": 0,
            }]
        })
    }

    fn report(
        read_before: u64,
        read_after: u64,
        device_after: &str,
        pid_after: u64,
    ) -> serde_json::Value {
        residency_scan_report(
            &scan(),
            &scan(),
            &host(read_before, "sda4"),
            &host(read_after, device_after),
            &process(100, 500),
            &process(pid_after, 500),
            true,
        )
    }

    #[test]
    fn residency_requires_valid_unchanged_device_and_backend_identity() {
        assert_eq!(
            report(10_000, 10_000, "sda4", 100)["served_from_ram_for_second_bounded_scan"],
            true
        );
        assert_eq!(
            report(10_000, 9_000, "sda4", 100)["second_scan_physical_read_delta_bytes"],
            serde_json::Value::Null
        );
        assert_eq!(
            report(10_000, 9_000, "sda4", 100)["served_from_ram_for_second_bounded_scan"],
            false
        );
        assert_eq!(
            residency_scan_report(
                &scan(),
                &scan(),
                &host_without_counter("sda4"),
                &host(10_000, "sda4"),
                &process(100, 500),
                &process(100, 500),
                true,
            )["served_from_ram_for_second_bounded_scan"],
            false
        );
        assert_eq!(
            report(10_000, 10_000, "sdb4", 100)["served_from_ram_for_second_bounded_scan"],
            false
        );
        assert_eq!(
            report(10_000, 10_000, "sda4", 101)["served_from_ram_for_second_bounded_scan"],
            false
        );
    }
}
