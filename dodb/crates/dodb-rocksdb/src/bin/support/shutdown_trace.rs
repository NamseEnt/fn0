use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

const SOURCE_REVISION_LIMIT: usize = 128;

pub(crate) struct ShutdownTrace {
    identity: Option<TraceIdentity>,
}

struct TraceIdentity {
    process_id: u32,
    process_start_time_ticks: Option<u64>,
    source_revision: String,
}

#[derive(Serialize)]
struct ShutdownEvent<'event> {
    timestamp_unix_ns: u128,
    process_id: u32,
    process_start_time_ticks: Option<u64>,
    source_revision: &'event str,
    stage: &'event str,
}

impl ShutdownTrace {
    pub(crate) fn new(enabled: bool, source_revision: &str) -> Self {
        let identity = enabled.then(|| TraceIdentity {
            process_id: std::process::id(),
            process_start_time_ticks: process_start_time_ticks(),
            source_revision: source_revision
                .chars()
                .take(SOURCE_REVISION_LIMIT)
                .collect(),
        });
        Self { identity }
    }

    pub(crate) fn emit(&self, stage: &'static str) {
        if self.identity.is_none() {
            return;
        }
        let mut stderr = io::stderr().lock();
        let _ = self.emit_to(&mut stderr, stage, unix_time_ns());
    }

    fn emit_to(
        &self,
        writer: &mut impl Write,
        stage: &'static str,
        timestamp_unix_ns: u128,
    ) -> io::Result<()> {
        let Some(identity) = self.identity.as_ref() else {
            return Ok(());
        };
        let event = ShutdownEvent {
            timestamp_unix_ns,
            process_id: identity.process_id,
            process_start_time_ticks: identity.process_start_time_ticks,
            source_revision: &identity.source_revision,
            stage,
        };
        serde_json::to_writer(&mut *writer, &event).map_err(io::Error::other)?;
        writer.write_all(b"\n")
    }
}

fn process_start_time_ticks() -> Option<u64> {
    let contents = std::fs::read_to_string("/proc/self/stat").ok()?;
    let command_end = contents.rfind(')')?;
    contents[command_end + 1..]
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}

fn unix_time_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use serde_json::Value;

    use super::ShutdownTrace;

    #[test]
    fn enabled_trace_has_timestamp_identity_source_and_stage() {
        let trace = ShutdownTrace {
            identity: Some(super::TraceIdentity {
                process_id: 42,
                process_start_time_ticks: Some(123),
                source_revision: "revision".to_owned(),
            }),
        };
        let mut output = Vec::new();

        trace
            .emit_to(&mut output, "runtime_destruction_start", 456)
            .unwrap();

        let event: Value = serde_json::from_slice(output.strip_suffix(b"\n").unwrap()).unwrap();
        assert_eq!(event["timestamp_unix_ns"], 456);
        assert_eq!(event["process_id"], 42);
        assert_eq!(event["process_start_time_ticks"], 123);
        assert_eq!(event["source_revision"], "revision");
        assert_eq!(event["stage"], "runtime_destruction_start");
    }

    #[test]
    fn disabled_trace_writes_nothing() {
        let trace = ShutdownTrace { identity: None };
        trace
            .emit_to(&mut PanickingWriter, "command_received_shutdown", 456)
            .unwrap();
        trace.emit("command_received_shutdown");
    }

    #[test]
    fn trace_write_failure_is_returned_for_best_effort_callers_to_ignore() {
        struct FailedWriter;

        impl Write for FailedWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("closed"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let trace = ShutdownTrace {
            identity: Some(super::TraceIdentity {
                process_id: 42,
                process_start_time_ticks: Some(123),
                source_revision: "revision".to_owned(),
            }),
        };

        assert!(
            trace
                .emit_to(&mut FailedWriter, "async_server_shutdown_error", 456)
                .is_err()
        );
    }

    struct PanickingWriter;

    impl Write for PanickingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            panic!("disabled trace called its writer")
        }

        fn flush(&mut self) -> io::Result<()> {
            panic!("disabled trace flushed its writer")
        }
    }
}
