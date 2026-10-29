use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use serde::Serialize;
use serde_json::Value;
use serde_json::value::{RawValue, to_raw_value};

pub const SAMPLE_LIMIT: usize = 4_096;
pub const SERIALIZED_BYTE_LIMIT: usize = 8 * 1024 * 1024;

#[derive(Default)]
pub struct PressureObservations {
    samples: Vec<Box<RawValue>>,
    serialized_bytes: usize,
    truncated: bool,
}

impl PressureObservations {
    pub fn can_record(&self) -> bool {
        !self.truncated
            && self.samples.len() < SAMPLE_LIMIT
            && self.serialized_bytes < SERIALIZED_BYTE_LIMIT
    }

    pub fn record(&mut self, sample: &Value) -> serde_json::Result<()> {
        if !self.can_record() {
            self.truncated = true;
            return Ok(());
        }
        let serialized = to_raw_value(sample)?;
        let serialized_bytes = serialized.get().len();
        if self.serialized_bytes + serialized_bytes > SERIALIZED_BYTE_LIMIT {
            self.truncated = true;
            return Ok(());
        }
        self.serialized_bytes += serialized_bytes;
        self.samples.push(serialized);
        Ok(())
    }

    pub fn mark_truncated(&mut self) {
        self.truncated = true;
    }

    pub fn serialized_bytes(&self) -> usize {
        self.serialized_bytes
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }
}

#[derive(Serialize)]
struct PeriodicPressureReport<'report> {
    #[serde(flatten)]
    metadata: &'report Value,
    samples: &'report [Box<RawValue>],
}

#[derive(Serialize)]
struct WorkerHealthReport<'report> {
    #[serde(flatten)]
    metadata: &'report Value,
    periodic_pressure: PeriodicPressureReport<'report>,
}

fn serialize_report(
    writer: impl Write,
    mut metadata: Value,
    observations: &PressureObservations,
) -> io::Result<()> {
    let pressure_metadata = metadata
        .as_object_mut()
        .and_then(|fields| fields.remove("periodic_pressure"))
        .ok_or_else(|| {
            io::Error::other("worker health report requires periodic pressure metadata")
        })?;
    let pressure_fields = pressure_metadata
        .as_object()
        .ok_or_else(|| io::Error::other("periodic pressure metadata must be an object"))?;
    if pressure_fields.contains_key("samples") {
        return Err(io::Error::other(
            "periodic pressure metadata must not contain a sample DOM",
        ));
    }
    let report = WorkerHealthReport {
        metadata: &metadata,
        periodic_pressure: PeriodicPressureReport {
            metadata: &pressure_metadata,
            samples: &observations.samples,
        },
    };
    serde_json::to_writer_pretty(writer, &report).map_err(io::Error::other)
}

pub fn write_health_report(
    path: &Path,
    metadata: Value,
    observations: &PressureObservations,
) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    serialize_report(&mut writer, metadata, observations)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_pressure_report_preserves_legacy_schema() {
        let metadata = json!({"backend": "blink", "periodic_pressure": {"enabled": false}});
        let observations = PressureObservations::default();
        let mut output = Vec::new();
        serialize_report(&mut output, metadata, &observations).unwrap();
        let report: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(
            report,
            json!({"backend": "blink", "periodic_pressure": {"enabled": false, "samples": []}})
        );
    }

    #[test]
    fn raw_frames_preserve_nested_values_phases_and_unavailable_properties() {
        let samples = [
            json!({"phase": "before-measurement", "properties": {"status": "unavailable", "value": null}}),
            json!({"phase": "measurement-including-drain", "shared_resources": {"usage": 37}, "payload": "quote:\" newline:\n unicode:\u{03bb}"}),
        ];
        let mut observations = PressureObservations::default();
        for sample in &samples {
            observations.record(sample).unwrap();
        }
        assert_eq!(
            observations.serialized_bytes(),
            samples
                .iter()
                .map(|sample| serde_json::to_vec(sample).unwrap().len())
                .sum::<usize>()
        );
        let metadata = json!({"worker_pid": 17, "periodic_pressure": {"enabled": true, "samples_truncated": false}});
        let mut output = Vec::new();
        serialize_report(&mut output, metadata, &observations).unwrap();
        let report: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(report["periodic_pressure"]["samples"], json!(samples));
        assert_eq!(report["worker_pid"], 17);
        assert!(!observations.truncated());
    }

    #[test]
    fn sample_count_bound_preserves_the_complete_accepted_prefix() {
        let mut observations = PressureObservations::default();
        for sample_number in 0..SAMPLE_LIMIT {
            observations
                .record(&json!({"sample_number": sample_number}))
                .unwrap();
        }
        let accepted_bytes = observations.serialized_bytes();
        assert!(!observations.can_record());
        assert!(!observations.truncated());
        observations.record(&json!({"rejected": true})).unwrap();
        assert!(observations.truncated());
        assert_eq!(observations.samples.len(), SAMPLE_LIMIT);
        assert_eq!(observations.serialized_bytes(), accepted_bytes);
    }

    #[test]
    fn serialized_byte_bound_accepts_the_boundary_and_rejects_an_oversized_frame() {
        let mut exact = PressureObservations::default();
        exact
            .record(&json!("x".repeat(SERIALIZED_BYTE_LIMIT - 2)))
            .unwrap();
        assert_eq!(exact.serialized_bytes(), SERIALIZED_BYTE_LIMIT);
        assert_eq!(exact.samples.len(), 1);
        assert!(!exact.can_record());
        let mut oversized = PressureObservations::default();
        oversized
            .record(&json!("x".repeat(SERIALIZED_BYTE_LIMIT)))
            .unwrap();
        assert!(oversized.truncated());
        assert_eq!(oversized.serialized_bytes(), 0);
        assert!(oversized.samples.is_empty());
    }

    #[test]
    fn report_rejects_missing_metadata_and_prebuilt_sample_dom() {
        let observations = PressureObservations::default();
        for metadata in [
            json!({}),
            json!({"periodic_pressure": null}),
            json!({"periodic_pressure": {"samples": []}}),
        ] {
            assert!(serialize_report(Vec::new(), metadata, &observations).is_err());
        }
    }

    #[test]
    fn report_propagates_output_failures() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("test output failure"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let observations = PressureObservations::default();
        assert!(
            serialize_report(
                FailingWriter,
                json!({"periodic_pressure": {"enabled": false}}),
                &observations,
            )
            .is_err()
        );
    }
}
