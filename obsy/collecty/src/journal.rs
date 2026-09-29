use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber};
use opentelemetry_proto::tonic::resource::v1::Resource;
use prost::Message;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::watch;

use crate::TENANT_ATTRIBUTE;
use crate::config::JournalConfig;
use crate::observe::SourceStats;
use crate::queue::{SenderId, Spool};
use crate::receive::{Intake, Refusal};
use crate::signal::Signal;

const MAX_BATCH_RECORDS: usize = 64;
const MAX_MESSAGE_BYTES: usize = 64 * 1024;
const MAX_ATTRIBUTE_BYTES: usize = 1024;
const MAX_ATTRIBUTES: usize = 32;
const CHECKPOINT_VERSION: &str = "v1";

pub struct JournalSource {
    config: JournalConfig,
    tenant_id: String,
    host_name: String,
    checkpoint: Checkpoint,
    intake: Arc<Intake>,
    spool: Spool,
    source_stats: Arc<SourceStats>,
}

pub struct JournalRuntime {
    pub intake: Arc<Intake>,
    pub spool: Spool,
    pub source_stats: Arc<SourceStats>,
}

struct JournalRecord {
    cursor: String,
    record: LogRecord,
}

struct Checkpoint {
    path: PathBuf,
    sender: String,
}

impl JournalSource {
    pub fn new(
        config: JournalConfig,
        root: impl Into<PathBuf>,
        data_dir: impl Into<PathBuf>,
        tenant_id: impl Into<String>,
        sender: SenderId,
        runtime: JournalRuntime,
    ) -> io::Result<Self> {
        let root = root.into();
        let host_name = fs::read_to_string(root.join("etc/hostname"))?
            .trim()
            .to_string();
        if host_name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "hostname is empty",
            ));
        }
        probe_journalctl(&config)?;
        Ok(Self {
            config,
            tenant_id: tenant_id.into(),
            host_name,
            checkpoint: Checkpoint {
                path: data_dir.into().join("sources/journal.cursor"),
                sender: sender.to_string(),
            },
            intake: runtime.intake,
            spool: runtime.spool,
            source_stats: runtime.source_stats,
        })
    }

    pub async fn run(self, mut shutdown: watch::Receiver<bool>) {
        let mut source = self;
        loop {
            if *shutdown.borrow() {
                return;
            }
            match source.follow(&mut shutdown).await {
                Ok(()) => return,
                Err(error) => {
                    source
                        .source_stats
                        .journal_errors
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    tracing::error!(%error, "the systemd journal source stopped");
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                        _ = shutdown.changed() => return,
                    }
                }
            }
        }
    }

    async fn follow(&mut self, shutdown: &mut watch::Receiver<bool>) -> io::Result<()> {
        let mut command = Command::new(&self.config.journalctl_path);
        command
            .arg("--output=json")
            .arg("--no-pager")
            .arg("--follow")
            .arg("--lines=0")
            .arg(format!("--priority={}", self.config.min_priority))
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(directory) = &self.config.directory {
            command.arg("--directory").arg(directory);
        }
        for unit in &self.config.units {
            command.arg("--unit").arg(unit);
        }
        if let Some(cursor) = self.checkpoint.load()? {
            command.arg("--after-cursor").arg(cursor);
        }

        let mut child = command.spawn().map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("cannot start {:?}: {error}", self.config.journalctl_path),
            )
        })?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("journalctl has no stdout"))?;
        let mut lines = BufReader::new(stdout).lines();
        let result = self.read_lines(&mut child, &mut lines, shutdown).await;
        let _ = child.kill().await;
        result
    }

    async fn read_lines(
        &mut self,
        child: &mut Child,
        lines: &mut Lines<BufReader<ChildStdout>>,
        shutdown: &mut watch::Receiver<bool>,
    ) -> io::Result<()> {
        let mut batch = Vec::new();
        let mut deadline = None;
        loop {
            let next = if batch.is_empty() {
                tokio::select! {
                    line = lines.next_line() => line,
                    _ = shutdown.changed() => return Ok(()),
                }
            } else {
                let wait = tokio::time::sleep_until(deadline.expect("a batch has a deadline"));
                tokio::pin!(wait);
                tokio::select! {
                    line = lines.next_line() => line,
                    _ = &mut wait => {
                        self.flush(&mut batch, shutdown).await?;
                        if *shutdown.borrow() {
                            return Ok(());
                        }
                        deadline = None;
                        continue;
                    }
                    _ = shutdown.changed() => return Ok(()),
                }
            };
            let Some(line) = next? else {
                let status = child.wait().await?;
                return Err(io::Error::other(format!("journalctl exited with {status}")));
            };
            let entry = parse_line(&line)?;
            if batch.len() >= MAX_BATCH_RECORDS {
                self.flush(&mut batch, shutdown).await?;
                if *shutdown.borrow() {
                    return Ok(());
                }
                deadline = None;
            }
            batch.push(entry);
            if batch.len() > 1 && self.payload_size(&batch) > self.intake.max_request_bytes() {
                let last = batch.pop().expect("a non-empty batch");
                self.flush(&mut batch, shutdown).await?;
                if *shutdown.borrow() {
                    return Ok(());
                }
                deadline = None;
                batch.push(last);
            }
            if deadline.is_none() {
                deadline = Some(tokio::time::Instant::now() + Duration::from_secs(1));
            }
        }
    }

    fn payload_size(&self, batch: &[JournalRecord]) -> usize {
        encode(batch, &self.host_name, &self.tenant_id)
            .encode_to_vec()
            .len()
    }

    async fn flush(
        &self,
        batch: &mut Vec<JournalRecord>,
        shutdown: &mut watch::Receiver<bool>,
    ) -> io::Result<()> {
        if batch.is_empty() {
            return Ok(());
        }
        let cursor = batch.last().expect("a non-empty batch").cursor.clone();
        let payload = encode(batch, &self.host_name, &self.tenant_id).encode_to_vec();
        if payload.len() > self.intake.max_request_bytes() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "journal batch is {} bytes, exceeding the {} byte maximum",
                    payload.len(),
                    self.intake.max_request_bytes()
                ),
            ));
        }
        self.intake
            .accept(Signal::Logs, vec![Bytes::from(payload)])
            .await
            .map_err(refusal_error)?;
        loop {
            match self.spool.seal().await {
                Ok(Ok(())) => break,
                Ok(Err(error)) => {
                    tracing::warn!(%error, "journal batch is queued but not durable; retrying the seal");
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                        _ = shutdown.changed() => return Ok(()),
                    }
                }
                Err(_) => return Err(io::Error::other("the spool thread stopped")),
            }
        }
        self.checkpoint.store(&cursor)?;
        self.source_stats
            .journal_exports
            .fetch_add(batch.len() as u64, std::sync::atomic::Ordering::Relaxed);
        batch.clear();
        Ok(())
    }
}

fn probe_journalctl(config: &JournalConfig) -> io::Result<()> {
    let mut command = std::process::Command::new(&config.journalctl_path);
    command
        .arg("--output=json")
        .arg("--no-pager")
        .arg("--lines=0")
        .arg(format!("--priority={}", config.min_priority))
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(directory) = &config.directory {
        command.arg("--directory").arg(directory);
    }
    for unit in &config.units {
        command.arg("--unit").arg(unit);
    }
    let output = command.output().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("cannot start {:?}: {error}", config.journalctl_path),
        )
    })?;
    if output.status.success() {
        return Ok(());
    }
    let explanation = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(io::Error::other(format!(
        "journalctl preflight failed with {}{}",
        output.status,
        if explanation.is_empty() {
            String::new()
        } else {
            format!(": {explanation}")
        }
    )))
}

fn parse_line(line: &str) -> io::Result<JournalRecord> {
    let value: Value = serde_json::from_str(line).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid journal JSON: {error}"),
        )
    })?;
    let object = value.as_object().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "journal JSON is not an object")
    })?;
    let cursor = object
        .get("__CURSOR")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "journal entry has no cursor"))?
        .to_string();
    let timestamp = object
        .get("__REALTIME_TIMESTAMP")
        .and_then(value_as_u64)
        .unwrap_or_else(unix_micros)
        .saturating_mul(1000);
    let priority = object
        .get("PRIORITY")
        .and_then(value_as_u64)
        .unwrap_or(6)
        .min(7) as u8;
    let message = bounded_string(
        object.get("MESSAGE").and_then(Value::as_str).unwrap_or(""),
        MAX_MESSAGE_BYTES,
    );
    let mut attributes = Vec::new();
    for (source_key, target_key) in [
        ("_SYSTEMD_UNIT", "systemd.unit"),
        ("_SYSTEMD_USER_UNIT", "systemd.user_unit"),
        ("SYSLOG_IDENTIFIER", "syslog.identifier"),
    ] {
        if let Some(value) = object.get(source_key).and_then(Value::as_str) {
            attributes.push(attribute(
                target_key,
                bounded_string(value, MAX_ATTRIBUTE_BYTES),
            ));
        }
    }
    attributes.push(attribute("journal.priority", priority.to_string()));
    let (severity_number, severity_text) = severity(priority);
    Ok(JournalRecord {
        cursor,
        record: LogRecord {
            time_unix_nano: timestamp,
            observed_time_unix_nano: unix_nanos(),
            severity_number: severity_number as i32,
            severity_text: severity_text.to_string(),
            body: Some(AnyValue {
                value: Some(any_value::Value::StringValue(message)),
            }),
            attributes: attributes.into_iter().take(MAX_ATTRIBUTES).collect(),
            ..Default::default()
        },
    })
}

fn encode(batch: &[JournalRecord], host_name: &str, tenant_id: &str) -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![
                    attribute(TENANT_ATTRIBUTE, tenant_id),
                    attribute("service.name", "collecty"),
                    attribute("host.name", host_name),
                ],
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                scope: Some(InstrumentationScope {
                    name: "collecty.journal".to_string(),
                    ..Default::default()
                }),
                log_records: batch.iter().map(|entry| entry.record.clone()).collect(),
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

impl Checkpoint {
    fn load(&self) -> io::Result<Option<String>> {
        let value = match fs::read_to_string(&self.path) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let mut fields = value.lines();
        if fields.next() != Some(CHECKPOINT_VERSION) || fields.next() != Some(self.sender.as_str())
        {
            return Ok(None);
        }
        let cursor = fields.next().unwrap_or("").trim();
        if cursor.is_empty() {
            return Ok(None);
        }
        Ok(Some(cursor.to_string()))
    }

    fn store(&self, cursor: &str) -> io::Result<()> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| io::Error::other("journal checkpoint has no parent"))?;
        fs::create_dir_all(parent)?;
        let temporary = self.path.with_extension("tmp");
        fs::write(
            &temporary,
            format!("{CHECKPOINT_VERSION}\n{}\n{cursor}\n", self.sender),
        )?;
        File::open(&temporary)?.sync_all()?;
        fs::rename(&temporary, &self.path)?;
        File::open(parent)?.sync_all()
    }
}

fn refusal_error(refusal: Refusal) -> io::Error {
    io::Error::other(refusal.to_string())
}

fn value_as_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse::<u64>().ok())
}

fn severity(priority: u8) -> (SeverityNumber, &'static str) {
    match priority {
        0..=2 => (SeverityNumber::Fatal, "FATAL"),
        3 => (SeverityNumber::Error, "ERROR"),
        4..=5 => (SeverityNumber::Warn, "WARN"),
        6 => (SeverityNumber::Info, "INFO"),
        _ => (SeverityNumber::Debug, "DEBUG"),
    }
}

fn bounded_string(value: &str, maximum: usize) -> String {
    let mut end = value.len().min(maximum);
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

fn attribute(key: &str, value: impl Into<String>) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.into())),
        }),
        ..Default::default()
    }
}

fn unix_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros() as u64)
        .unwrap_or(0)
}

fn unix_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::Scratch;

    #[test]
    fn journal_json_maps_to_an_otlp_record() {
        let line = r#"{"__CURSOR":"s=abc;i=1;b=2;m=3;t=4;x=5","__REALTIME_TIMESTAMP":"1700000000000000","PRIORITY":"3","MESSAGE":"failed","_SYSTEMD_UNIT":"demo.service","SYSLOG_IDENTIFIER":"demo"}"#;
        let parsed = parse_line(line).expect("journal record");
        assert_eq!(parsed.cursor, "s=abc;i=1;b=2;m=3;t=4;x=5");
        assert_eq!(parsed.record.time_unix_nano, 1_700_000_000_000_000_000);
        assert_eq!(parsed.record.severity_number, SeverityNumber::Error as i32);
        assert_eq!(parsed.record.severity_text, "ERROR");
        assert_eq!(parsed.record.attributes.len(), 3);
    }

    #[test]
    fn checkpoint_is_invalidated_by_a_new_queue_identity() {
        let scratch = Scratch::new("journal-checkpoint");
        let checkpoint = Checkpoint {
            path: scratch.path().join("sources/journal.cursor"),
            sender: "sender-a".to_string(),
        };
        checkpoint.store("cursor").expect("checkpoint");
        assert_eq!(checkpoint.load().expect("load"), Some("cursor".to_string()));
        let replacement = Checkpoint {
            path: checkpoint.path.clone(),
            sender: "sender-b".to_string(),
        };
        assert_eq!(replacement.load().expect("load"), None);
    }

    #[tokio::test]
    async fn a_batch_is_synced_before_its_cursor_is_stored() {
        let scratch = Scratch::new("journal-flush");
        let queue = std::sync::Arc::new(
            crate::queue::Queue::open(
                &scratch.path().join("queue"),
                crate::queue::QueueLimits {
                    max_segment_age: Duration::from_secs(60),
                    ..crate::queue::QueueLimits::default()
                },
                crate::wire::ZSTD_LEVEL,
            )
            .expect("queue"),
        );
        let spool = Spool::new(queue.clone());
        let intake = Intake::new(spool.clone(), 1024 * 1024, 2 * 1024 * 1024);
        let source = JournalSource {
            config: JournalConfig {
                directory: None,
                journalctl_path: PathBuf::from("journalctl"),
                units: Vec::new(),
                min_priority: 6,
            },
            tenant_id: "tenant".to_string(),
            host_name: "host".to_string(),
            checkpoint: Checkpoint {
                path: scratch.path().join("sources/journal.cursor"),
                sender: queue.sender_id().to_string(),
            },
            intake,
            spool,
            source_stats: Arc::new(SourceStats::default()),
        };
        let mut batch = vec![parse_line(
            r#"{"__CURSOR":"cursor-1","__REALTIME_TIMESTAMP":"1700000000000000","MESSAGE":"hello"}"#,
        )
        .expect("record")];
        let (_, mut shutdown) = watch::channel(false);
        source
            .flush(&mut batch, &mut shutdown)
            .await
            .expect("durable batch");
        assert!(batch.is_empty());
        assert_eq!(
            source.checkpoint.load().expect("checkpoint"),
            Some("cursor-1".to_string())
        );
        assert_eq!(queue.oldest_sealed(), Some((Signal::Logs, 1)));
    }
}
