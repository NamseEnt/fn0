use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::LogRecord;

use crate::memtable::LogEntry;

#[derive(Debug, PartialEq, Eq)]
pub enum OtlpLogError {
    EmptyRequest,
    TimestampOutOfRange,
}

impl std::fmt::Display for OtlpLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyRequest => "OTLP log request contains no records",
            Self::TimestampOutOfRange => "log record timestamp is outside the supported range",
        };
        f.write_str(message)
    }
}

/// Turn an OTLP export into flat log entries.
///
/// There is no stream concept: nothing is promoted to a label, and every
/// resource attribute, scope name and record attribute lands in the entry's
/// structured metadata. Attribute keys are normalized to the LogQL identifier
/// grammar (`service.name` → `service_name`) so every stored key is one a
/// query can name — Loki's own OTLP intake normalizes the same way.
///
/// Consumes the request: the decoded protobuf serves nothing after this — the
/// WAL stores the received bytes, not the message — so every body string,
/// attribute key and value moves into its `LogEntry` instead of being cloned
/// per row on the ingest hot path.
pub fn normalize_request(request: ExportLogsServiceRequest) -> Result<Vec<LogEntry>, OtlpLogError> {
    let mut entries = Vec::new();
    for resource_logs in request.resource_logs {
        let resource_attributes = resource_logs
            .resource
            .map(|resource| resource.attributes)
            .unwrap_or_default();
        let mut resource_metadata = Vec::with_capacity(resource_attributes.len());
        for attribute in resource_attributes {
            // The tenant is how this row was filed, not something it is about.
            // Left in, it would be a second copy of the isolation the `_tenant`
            // column already enforces, and one a query could select on.
            if crate::otlp_tenant::is_tenant_attribute(&attribute.key) {
                continue;
            }
            if let Some(value) = attribute.value {
                resource_metadata.push((
                    normalize_attribute_key(&attribute.key),
                    scalar_string_owned(value).unwrap_or_else(|value| value_json(&value)),
                ));
            }
        }
        for scope_logs in resource_logs.scope_logs {
            let scope_name = scope_logs.scope.map(|scope| scope.name).unwrap_or_default();
            for record in scope_logs.log_records {
                entries.push(normalize_record(record, &resource_metadata, &scope_name)?);
            }
        }
    }
    if entries.is_empty() {
        return Err(OtlpLogError::EmptyRequest);
    }
    Ok(entries)
}

fn normalize_record(
    record: LogRecord,
    resource_metadata: &[(String, String)],
    scope_name: &str,
) -> Result<LogEntry, OtlpLogError> {
    // `time_unix_nano` is optional in OTLP and collectors leave it at zero when
    // the source had no timestamp; `observed_time_unix_nano` is when the
    // collector saw it, which is the best answer available then.
    let raw_timestamp = if record.time_unix_nano != 0 {
        record.time_unix_nano
    } else {
        record.observed_time_unix_nano
    };
    let timestamp_ns =
        i64::try_from(raw_timestamp).map_err(|_| OtlpLogError::TimestampOutOfRange)?;

    let line = match record.body {
        Some(body) => scalar_string_owned(body).unwrap_or_else(|value| value_json(&value)),
        None => String::new(),
    };

    let mut structured_metadata: Vec<(String, String)> = resource_metadata.to_vec();
    if !scope_name.is_empty() {
        structured_metadata.push(("scope_name".to_string(), scope_name.to_string()));
    }
    if !record.severity_text.is_empty() {
        structured_metadata.push(("severity_text".to_string(), record.severity_text));
    }
    if record.severity_number != 0 {
        structured_metadata.push((
            "severity_number".to_string(),
            record.severity_number.to_string(),
        ));
    }
    // Kept as hex, the form every other surface here uses for these: the Tempo
    // handlers, the trace part format and the trace-to-logs link in Grafana all
    // speak hex, so a raw byte string would be the one place that does not.
    if !record.trace_id.is_empty() {
        structured_metadata.push(("trace_id".to_string(), hex(&record.trace_id)));
    }
    if !record.span_id.is_empty() {
        structured_metadata.push(("span_id".to_string(), hex(&record.span_id)));
    }
    for attribute in record.attributes {
        if let Some(value) = attribute.value {
            structured_metadata.push((
                normalize_attribute_key(&attribute.key),
                scalar_string_owned(value).unwrap_or_else(|value| value_json(&value)),
            ));
        }
    }

    Ok(LogEntry {
        timestamp_ns,
        line,
        structured_metadata,
    })
}

/// Attribute keys are matched by LogQL, whose grammar is Prometheus-shaped, so
/// the dots OTLP uses have to become underscores. This is what Loki's OTLP
/// intake does too, which is why `service.name` is queried as `service_name`.
pub fn normalize_attribute_key(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

/// The moving form of the scalar check: a string body or attribute is the
/// common case and the bytes are handed over rather than cloned. A non-scalar
/// comes back whole so the caller can render it as JSON.
fn scalar_string_owned(value: AnyValue) -> Result<String, AnyValue> {
    match value.value {
        Some(any_value::Value::StringValue(value)) => Ok(value),
        Some(any_value::Value::BoolValue(value)) => Ok(value.to_string()),
        Some(any_value::Value::IntValue(value)) => Ok(value.to_string()),
        Some(any_value::Value::DoubleValue(value)) => Ok(value.to_string()),
        _ => Err(value),
    }
}

/// A composite attribute or body, rendered as JSON so the parser filters that
/// already exist for structured lines can reach into it.
fn value_json(value: &AnyValue) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    include!("tests/otlp_log.rs");
}
