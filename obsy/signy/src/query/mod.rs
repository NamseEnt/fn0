use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::Json;
use axum::extract::{Path, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use std::time::Duration;

use serde::Serialize;

use crate::AppState;
use crate::logql::{self};
use crate::memtable::{Labels, LogEntry, SharedLabels, StreamResult};
use crate::part;
use crate::series::SeriesLabels;
use crate::tenant::TenantId;

const MAX_HISTOGRAM_BUCKETS: usize = 10_000;
const MAX_LOG_LIMIT: usize = 100_000;
const MAX_LOG_SCAN_ROWS: usize = 5_000_000;

/// Marks a refusal that belongs to the tenant's own quota rather than to the
/// query. Carried in the message because the scan path reports `String`, and
/// the distinction matters to the client: a 429 is worth retrying later and a
/// 400 never is.
pub(crate) const TENANT_QUOTA_PREFIX: &str = "tenant quota: ";

fn metric_error_status(error: &str) -> StatusCode {
    if error.starts_with(TENANT_QUOTA_PREFIX) {
        StatusCode::TOO_MANY_REQUESTS
    } else if error.starts_with(crate::memory_budget::EXHAUSTED_PREFIX) {
        // This instance had no room, which is temporary by construction: the
        // account empties as the work holding it finishes, and the same request
        // a moment later succeeds. It reached a client as `500` until
        // 2026-08-13 — a working limit wearing a fault's code, which pages an
        // operator for a refusal and tells a client library to treat backoff as
        // pointless. The event is not thereby harmless, and it is not meant to
        // disappear into the throttling either: `MemoryAccount::exhausted`
        // counts it on its own so "this instance could not serve work it was
        // willing to do" stays visible after the client has been told the right
        // thing.
        StatusCode::TOO_MANY_REQUESTS
    } else if error.starts_with(crate::memory_budget::OVER_BUDGET_PREFIX) {
        // The other half of that refusal, and the opposite instruction. This
        // request wants more than the whole instance has, so no wait helps and
        // a `429` would loop a well-behaved client forever on something that
        // can never succeed. The message names the budget and says to narrow.
        StatusCode::BAD_REQUEST
    } else if error.starts_with("metric query exceeds") || error.starts_with("query exceeds") {
        StatusCode::BAD_REQUEST
    } else if error.starts_with("trace query exceeds") {
        // A refusal about the trace's size, not the request's shape: the same
        // request against a smaller trace succeeds, so 400 would blame the
        // client for the data and 429 would promise a retry can help.
        StatusCode::PAYLOAD_TOO_LARGE
    } else if error.starts_with("metric selection exceeds") {
        // The trace-413 rationale again: a statement about the data's
        // cardinality in the window, not the request's shape.
        StatusCode::PAYLOAD_TOO_LARGE
    } else if error == "query timed out"
        || error == "metric query timed out"
        || error == "object store restore timed out"
        || error == "trace query timed out"
        || error == "trace object store restore timed out"
        || error == "metric object store restore timed out"
    {
        StatusCode::GATEWAY_TIMEOUT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// A stream's label set on the wire, never materialized as one map.
///
/// The union of a row's stream labels and its structured metadata used to be
/// built per row — a deep clone of the label map, a `BTreeMap` insert per
/// metadata pair, then a second map for serialization — and grouped under a
/// `BTreeMap` whose key comparison walked the label pairs; with `trace_id`
/// both unique per row and last alphabetically, that walk compared every
/// label of every row against every probe. At thousands of returned rows
/// this was the response's dominant cost. The key now holds the label `Arc`
/// and the row's metadata as they are, and every consumer — equality,
/// hashing, serialization — reads the union through one sorted merge with
/// metadata shadowing same-named labels, which is the semantic the built
/// map had.
#[derive(Debug)]
pub struct StreamKey {
    labels: SharedLabels,
    /// Sorted by key, one entry per key (the row's last duplicate wins),
    /// pairs that merely repeat the label's own value dropped — so two rows
    /// whose unions are equal always compare equal however the union was
    /// split between labels and metadata.
    metadata: Vec<(String, String)>,
}

impl StreamKey {
    fn new(labels: SharedLabels, metadata: Vec<(String, String)>) -> Self {
        let mut normalized: Vec<(String, String)> = Vec::with_capacity(metadata.len());
        for (name, value) in metadata {
            match normalized
                .iter_mut()
                .find(|(existing, _)| *existing == name)
            {
                Some((_, slot)) => *slot = value,
                None => normalized.push((name, value)),
            }
        }
        normalized.sort_by(|a, b| a.0.cmp(&b.0));
        normalized.retain(|(name, value)| labels.get(name) != Some(value));
        Self {
            labels,
            metadata: normalized,
        }
    }

    /// The union, sorted by key, metadata shadowing labels.
    fn merged(&self) -> impl Iterator<Item = (&str, &str)> {
        let mut labels = self.labels.iter().peekable();
        let mut metadata = self.metadata.iter().peekable();
        std::iter::from_fn(move || match (labels.peek(), metadata.peek()) {
            (Some((ln, lv)), Some((mn, mv))) => match ln.as_str().cmp(mn.as_str()) {
                std::cmp::Ordering::Less => {
                    let out = (ln.as_str(), lv.as_str());
                    labels.next();
                    Some(out)
                }
                std::cmp::Ordering::Greater => {
                    let out = (mn.as_str(), mv.as_str());
                    metadata.next();
                    Some(out)
                }
                std::cmp::Ordering::Equal => {
                    labels.next();
                    let out = (mn.as_str(), mv.as_str());
                    metadata.next();
                    Some(out)
                }
            },
            (Some((ln, lv)), None) => {
                let out = (ln.as_str(), lv.as_str());
                labels.next();
                Some(out)
            }
            (None, Some((mn, mv))) => {
                let out = (mn.as_str(), mv.as_str());
                metadata.next();
                Some(out)
            }
            (None, None) => None,
        })
    }

    #[cfg(test)]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.merged()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value)
    }

    #[cfg(test)]
    pub fn to_map(&self) -> std::collections::BTreeMap<String, String> {
        self.merged()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
    }
}

impl Serialize for StreamKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        for (name, value) in self.merged() {
            map.serialize_entry(name, value)?;
        }
        map.end()
    }
}

pub(crate) fn parse_time_ns(s: &str) -> Result<i64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty timestamp".to_string());
    }
    let numeric_digits = s.strip_prefix('-').unwrap_or(s);
    if !numeric_digits.is_empty() && numeric_digits.bytes().all(|b| b.is_ascii_digit()) {
        let n: i64 = s
            .parse()
            .map_err(|e| format!("invalid timestamp '{}': {}", s, e))?;
        let multiplier = match numeric_digits.len() {
            1..=10 => 1_000_000_000i64,
            13 => 1_000_000i64,
            16 => 1_000i64,
            19 => 1i64,
            _ => {
                return Err(format!(
                    "unsupported numeric timestamp precision (expected seconds, milliseconds, microseconds, or nanoseconds): '{}'",
                    s
                ));
            }
        };
        n.checked_mul(multiplier)
            .ok_or_else(|| format!("timestamp '{}' is out of range", s))
    } else if s.contains(['.', 'e', 'E'])
        && s.bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'.' | b'e' | b'E' | b'+' | b'-'))
    {
        logql::decimal_seconds_to_ns(s).map_err(|_| {
            format!(
                "invalid decimal Unix timestamp '{}': expected finite seconds with nanosecond precision",
                s
            )
        })
    } else {
        let dt = chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|e| format!("invalid RFC3339 timestamp '{}': {}", s, e))?;
        dt.timestamp_nanos_opt()
            .ok_or_else(|| format!("timestamp '{}' out of range", s))
    }
}

fn parse_direction(direction: &Option<String>) -> Result<bool, String> {
    match direction.as_deref() {
        None => Ok(false),
        Some(value) if value.eq_ignore_ascii_case("forward") => Ok(true),
        Some(value) if value.eq_ignore_ascii_case("backward") => Ok(false),
        Some(value) => Err(format!(
            "invalid direction '{value}': expected 'forward' or 'backward'"
        )),
    }
}

fn parse_limit(limit: Option<usize>, max_limit: usize) -> Result<usize, String> {
    let limit = limit.unwrap_or(100);
    if limit > max_limit {
        return Err(format!("query limit exceeds the maximum of {max_limit}"));
    }
    Ok(limit)
}

pub(crate) fn validate_query_range(
    config: &crate::config::Config,
    start_ns: i64,
    end_ns: i64,
) -> Result<(), String> {
    if start_ns > end_ns {
        return Err("query start must not be after end".to_string());
    }
    let Some(max_range) = config.max_query_range else {
        return Ok(());
    };
    let range_ns = (i128::from(end_ns) - i128::from(start_ns)) as u128;
    if range_ns > max_range.as_nanos() {
        return Err(format!(
            "query range exceeds the maximum of {} seconds",
            max_range.as_secs()
        ));
    }
    Ok(())
}

fn estimated_query_memory_bytes(results: &[StreamResult]) -> u64 {
    results
        .iter()
        .flat_map(|stream| {
            stream.entries.iter().map(|entry| {
                let labels = stream
                    .labels
                    .iter()
                    .map(|(name, value)| name.len().saturating_add(value.len()))
                    .sum::<usize>();
                let metadata = entry
                    .structured_metadata
                    .iter()
                    .map(|(name, value)| name.len().saturating_add(value.len()))
                    .sum::<usize>();
                labels
                    .saturating_add(metadata)
                    .saturating_add(entry.line.len())
                    .saturating_add(std::mem::size_of::<LogEntry>())
            })
        })
        .try_fold(0u64, |total, bytes| total.checked_add(bytes as u64))
        .unwrap_or(u64::MAX)
}

include!("execution.rs");
include!("params.rs");
include!("logs.rs");
include!("attributes.rs");
include!("restore.rs");
include!("prometheus.rs");
include!("delete_api.rs");
include!("tail.rs");
include!("traces.rs");
include!("trace_scan.rs");
include!("metric_scan.rs");
include!("metrics_query.rs");
include!("metrics_metadata.rs");

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
