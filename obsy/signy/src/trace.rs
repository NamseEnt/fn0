use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;

pub use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::InstrumentationScope;
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::Span;

use crate::tenant::TenantId;

pub const TRACE_ID_BYTES: usize = 16;
pub const SPAN_ID_BYTES: usize = 8;

#[derive(Clone, Debug)]
pub struct TraceSpan {
    pub tenant: TenantId,
    pub trace_id: String,
    pub span_id: String,
    pub start_time_ns: i64,
    pub end_time_ns: i64,
    pub span: Span,
    pub resource: Option<Resource>,
    pub resource_schema_url: String,
    pub scope: Option<InstrumentationScope>,
    pub scope_schema_url: String,
}

impl TraceSpan {
    pub fn duration_ns(&self) -> u64 {
        u64::try_from(i128::from(self.end_time_ns) - i128::from(self.start_time_ns))
            .unwrap_or(u64::MAX)
    }

    pub fn service_name(&self) -> Option<&str> {
        self.resource.as_ref().and_then(|resource| {
            resource.attributes.iter().find_map(|attribute| {
                (attribute.key == "service.name").then(|| {
                    attribute
                        .value
                        .as_ref()
                        .and_then(any_value_string)
                        .unwrap_or("")
                })
            })
        })
    }

    pub fn attribute(&self, name: &str) -> Option<serde_json::Value> {
        self.span
            .attributes
            .iter()
            .find(|attribute| attribute.key == name)
            .and_then(|attribute| attribute.value.as_ref())
            .map(any_value_json)
    }

    pub fn tag_value(&self, name: &str) -> Option<String> {
        if name == "name" {
            return Some(self.span.name.clone());
        }
        if name == "duration" {
            return Some(self.duration_ns().to_string());
        }
        if name == "status" {
            use opentelemetry_proto::tonic::trace::v1::status::StatusCode;
            let status = self
                .span
                .status
                .as_ref()
                .and_then(|status| StatusCode::try_from(status.code).ok())
                .unwrap_or(StatusCode::Unset);
            return Some(
                match status {
                    StatusCode::Unset => "unset",
                    StatusCode::Ok => "ok",
                    StatusCode::Error => "error",
                }
                .to_string(),
            );
        }
        if name == "service.name" {
            return self.service_name().map(str::to_string);
        }
        if let Some(value) = self.attribute(name) {
            return json_scalar_string(&value);
        }
        self.resource.as_ref().and_then(|resource| {
            resource
                .attributes
                .iter()
                .find(|attribute| attribute.key == name)
                .and_then(|attribute| attribute.value.as_ref())
                .map(any_value_json)
                .and_then(|value| json_scalar_string(&value))
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TraceError {
    EmptyRequest,
    InvalidTraceId,
    InvalidSpanId,
    InvalidParentSpanId,
    TimestampOutOfRange,
    EndBeforeStart,
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyRequest => "trace request contains no spans",
            Self::InvalidTraceId => "trace_id must be exactly 16 non-zero bytes",
            Self::InvalidSpanId => "span_id must be exactly 8 non-zero bytes",
            Self::InvalidParentSpanId => "parent_span_id must be empty or exactly 8 non-zero bytes",
            Self::TimestampOutOfRange => "span timestamp is outside the supported range",
            Self::EndBeforeStart => "span end_time must not precede start_time",
        };
        f.write_str(message)
    }
}

impl std::error::Error for TraceError {}

pub fn normalize_request(
    tenant: &TenantId,
    request: ExportTraceServiceRequest,
) -> Result<Vec<TraceSpan>, TraceError> {
    let mut spans = Vec::new();
    for resource_spans in request.resource_spans {
        // Stripped here rather than at the split, because the split hands the
        // request on whole and the WAL keeps the bytes that arrived. Replay
        // comes back through this function, so one removal covers both.
        let resource = resource_spans.resource.map(|mut resource| {
            resource
                .attributes
                .retain(|attribute| !crate::otlp_tenant::is_tenant_attribute(&attribute.key));
            resource
        });
        for scope_spans in resource_spans.scope_spans {
            for span in scope_spans.spans {
                spans.push(normalize_span(
                    tenant,
                    span,
                    resource.clone(),
                    resource_spans.schema_url.clone(),
                    scope_spans.scope.clone(),
                    scope_spans.schema_url.clone(),
                )?);
            }
        }
    }
    if spans.is_empty() {
        return Err(TraceError::EmptyRequest);
    }
    Ok(spans)
}

/// A trace id as a URL path segment, parsed into the canonical stored form.
/// The stored form is what ingest's `normalize_id` produced, so a lookup that
/// goes through this can compare strings byte-for-byte.
pub fn canonical_trace_id(input: &str) -> Result<String, String> {
    let input = input.trim();
    if input.len() != TRACE_ID_BYTES * 2 {
        return Err("trace ID must contain exactly 32 hexadecimal characters".to_string());
    }
    let mut bytes = Vec::with_capacity(TRACE_ID_BYTES);
    let chars: Vec<char> = input.chars().collect();
    for pair in chars.chunks_exact(2) {
        let high = pair[0]
            .to_digit(16)
            .ok_or_else(|| "trace ID must contain only hexadecimal characters".to_string())?;
        let low = pair[1]
            .to_digit(16)
            .ok_or_else(|| "trace ID must contain only hexadecimal characters".to_string())?;
        bytes.push(((high << 4) | low) as u8);
    }
    normalize_id(&bytes, TRACE_ID_BYTES, TraceError::InvalidTraceId)
        .map_err(|error| error.to_string())
}

fn normalize_span(
    tenant: &TenantId,
    span: Span,
    resource: Option<Resource>,
    resource_schema_url: String,
    scope: Option<InstrumentationScope>,
    scope_schema_url: String,
) -> Result<TraceSpan, TraceError> {
    let trace_id = normalize_id(&span.trace_id, TRACE_ID_BYTES, TraceError::InvalidTraceId)?;
    let span_id = normalize_id(&span.span_id, SPAN_ID_BYTES, TraceError::InvalidSpanId)?;
    if !span.parent_span_id.is_empty() {
        normalize_id(
            &span.parent_span_id,
            SPAN_ID_BYTES,
            TraceError::InvalidParentSpanId,
        )?;
    }

    let start_time_ns =
        i64::try_from(span.start_time_unix_nano).map_err(|_| TraceError::TimestampOutOfRange)?;
    let end_time_ns =
        i64::try_from(span.end_time_unix_nano).map_err(|_| TraceError::TimestampOutOfRange)?;
    if end_time_ns < start_time_ns {
        return Err(TraceError::EndBeforeStart);
    }

    Ok(TraceSpan {
        tenant: tenant.clone(),
        trace_id,
        span_id,
        start_time_ns,
        end_time_ns,
        span,
        resource,
        resource_schema_url,
        scope,
        scope_schema_url,
    })
}

fn normalize_id(
    bytes: &[u8],
    expected_len: usize,
    error: TraceError,
) -> Result<String, TraceError> {
    if bytes.len() != expected_len || bytes.iter().all(|byte| *byte == 0) {
        return Err(error);
    }
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn any_value_string(value: &opentelemetry_proto::tonic::common::v1::AnyValue) -> Option<&str> {
    use opentelemetry_proto::tonic::common::v1::any_value::Value;
    match value.value.as_ref()? {
        Value::StringValue(value) => Some(value.as_str()),
        _ => None,
    }
}

fn any_value_json(value: &opentelemetry_proto::tonic::common::v1::AnyValue) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}

/// One attribute value as the string a viewer shows: scalars the way
/// `tag_value` stringifies them, everything else (arrays, kvlists) as compact
/// JSON — so no attribute silently disappears from a timeline.
pub(crate) fn attr_display_string(
    value: &opentelemetry_proto::tonic::common::v1::AnyValue,
) -> String {
    let json = any_value_json(value);
    json_scalar_string(&json).unwrap_or_else(|| json.to_string())
}

fn json_scalar_string(value: &serde_json::Value) -> Option<String> {
    if let Some(object) = value.as_object() {
        if let Some(value) = object.get("stringValue").and_then(|v| v.as_str()) {
            return Some(value.to_string());
        }
        if let Some(value) = object.get("boolValue").and_then(|v| v.as_bool()) {
            return Some(value.to_string());
        }
        if let Some(value) = object.get("intValue").and_then(|v| v.as_str()) {
            return Some(value.to_string());
        }
        if let Some(value) = object.get("doubleValue") {
            return Some(value.to_string());
        }
    }
    None
}

pub struct TraceMemTable {
    inner: RwLock<Vec<TraceSpan>>,
    flushing: RwLock<Option<Arc<Vec<TraceSpan>>>>,
    /// Maintained by every mutation for the same reason as the log memtable's:
    /// the flush loop and the ingest gate both ask for this several times a
    /// second, and walking the spans gets slower exactly as the buffer grows.
    inner_bytes: AtomicU64,
    flushing_bytes: AtomicU64,
}

fn span_bytes(span: &TraceSpan) -> u64 {
    (span.trace_id.len()
        + span.span_id.len()
        + span.span.name.len()
        + span.span.attributes.len() * 32) as u64
}

fn spans_bytes(spans: &[TraceSpan]) -> u64 {
    spans.iter().map(span_bytes).sum()
}

/// What one span costs a query that holds it materialized — an estimate in
/// the `span_bytes` spirit, but sized for the read side, where the whole
/// decoded span is resident. Charged against the shared query memory pool,
/// whose chunk granularity is what the estimate has to stay under, not the
/// allocator's truth.
pub(crate) fn span_query_bytes(span: &TraceSpan) -> u64 {
    fn attr_bytes(attributes: &[opentelemetry_proto::tonic::common::v1::KeyValue]) -> usize {
        attributes
            .iter()
            .map(|attribute| attribute.key.len() + 16)
            .sum()
    }
    (std::mem::size_of::<TraceSpan>()
        + span.trace_id.len()
        + span.span_id.len()
        + span.span.parent_span_id.len()
        + span.span.name.len()
        + attr_bytes(&span.span.attributes)
        + span
            .resource
            .as_ref()
            .map(|resource| attr_bytes(&resource.attributes))
            .unwrap_or(0)
        + span.span.events.len() * 64) as u64
}

/// See `memtable::unwrap_snapshot`.
fn unwrap_spans(spans: Arc<Vec<TraceSpan>>) -> Vec<TraceSpan> {
    Arc::try_unwrap(spans).unwrap_or_else(|shared| (*shared).clone())
}

impl TraceMemTable {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Vec::new()),
            flushing: RwLock::new(None),
            inner_bytes: AtomicU64::new(0),
            flushing_bytes: AtomicU64::new(0),
        }
    }

    pub fn insert(&self, spans: Vec<TraceSpan>) {
        let added = spans_bytes(&spans);
        self.inner.write().extend(spans);
        self.inner_bytes.fetch_add(added, Ordering::Relaxed);
    }

    /// Shared with the flush rather than copied to it, for the same reason the
    /// log memtable shares its snapshot: the copy doubled buffered memory at
    /// the moment the buffer is largest.
    pub fn begin_flush(&self) -> Arc<Vec<TraceSpan>> {
        let mut inner = self.inner.write();
        let mut flushing = self.flushing.write();
        let mut snapshot = std::mem::take(&mut *inner);
        let moved = self.inner_bytes.swap(0, Ordering::Relaxed);
        if let Some(previous) = flushing.take() {
            snapshot.extend(unwrap_spans(previous));
        }
        let snapshot = Arc::new(snapshot);
        *flushing = Some(snapshot.clone());
        self.flushing_bytes.fetch_add(moved, Ordering::Relaxed);
        snapshot
    }

    pub fn commit_flush(&self) {
        *self.flushing.write() = None;
        self.flushing_bytes.store(0, Ordering::Relaxed);
    }

    pub fn abort_flush(&self, snapshot: Arc<Vec<TraceSpan>>) {
        // Cleared first so the unwrap below takes ownership rather than copying.
        *self.flushing.write() = None;
        let mut snapshot = unwrap_spans(snapshot);
        self.inner.write().append(&mut snapshot);
        let returned = self.flushing_bytes.swap(0, Ordering::Relaxed);
        self.inner_bytes.fetch_add(returned, Ordering::Relaxed);
    }

    pub fn is_empty(&self) -> bool {
        if !self.inner.read().is_empty() {
            return false;
        }
        self.flushing
            .read()
            .as_ref()
            .map(|spans| spans.is_empty())
            .unwrap_or(true)
    }

    /// Every tenant with spans that have not been flushed yet, including the
    /// ones in a flush that is still in flight. See `MemTable::tenants`.
    pub fn tenants(&self) -> std::collections::BTreeSet<TenantId> {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        inner
            .iter()
            .chain(flushing.as_deref().into_iter().flatten())
            .map(|span| span.tenant.clone())
            .collect()
    }

    pub fn approximate_size(&self) -> usize {
        self.inner_bytes
            .load(Ordering::Relaxed)
            .saturating_add(self.flushing_bytes.load(Ordering::Relaxed)) as usize
    }

    pub fn query_trace_id(&self, tenant: &TenantId, trace_id: &str) -> Vec<TraceSpan> {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        inner
            .iter()
            .chain(flushing.as_deref().into_iter().flatten())
            .filter(|span| span.tenant == *tenant && span.trace_id == trace_id)
            .cloned()
            .collect()
    }

    pub fn query_trace_id_limited(
        &self,
        tenant: &TenantId,
        trace_id: &str,
        limit: usize,
    ) -> Result<Vec<TraceSpan>, String> {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        let mut spans = Vec::new();
        for span in inner
            .iter()
            .chain(flushing.as_deref().into_iter().flatten())
            .filter(|span| span.tenant == *tenant && span.trace_id == trace_id)
        {
            if spans.len() >= limit {
                return Err(format!("trace query exceeds the maximum of {limit} spans"));
            }
            spans.push(span.clone());
        }
        Ok(spans)
    }

    /// Every buffered span, across all tenants. Only the flush path may use
    /// this; query paths must go through the tenant-scoped variants.
    pub fn flush_snapshot(&self) -> Vec<TraceSpan> {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        inner
            .iter()
            .chain(flushing.as_deref().into_iter().flatten())
            .cloned()
            .collect()
    }

    pub fn snapshot(&self, tenant: &TenantId) -> Vec<TraceSpan> {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        inner
            .iter()
            .chain(flushing.as_deref().into_iter().flatten())
            .filter(|span| span.tenant == *tenant)
            .cloned()
            .collect()
    }

    pub fn snapshot_limited(
        &self,
        tenant: &TenantId,
        limit: usize,
    ) -> Result<Vec<TraceSpan>, String> {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        let mut spans = Vec::new();
        for span in inner
            .iter()
            .chain(flushing.as_deref().into_iter().flatten())
            .filter(|span| span.tenant == *tenant)
        {
            if spans.len() >= limit {
                return Err(format!("trace query exceeds the maximum of {limit} spans"));
            }
            spans.push(span.clone());
        }
        Ok(spans)
    }

    /// The window scan's memtable half. The overlap predicate is
    /// [`crate::trace_part::span_overlaps`] — the same one the part side and
    /// its row-group pruning answer — applied before the limit is charged, so
    /// an out-of-window span costs neither budget nor a clone.
    pub fn snapshot_range_limited(
        &self,
        tenant: &TenantId,
        start_ns: i64,
        end_ns: i64,
        limit: usize,
    ) -> Result<Vec<TraceSpan>, String> {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        let mut spans = Vec::new();
        for span in inner
            .iter()
            .chain(flushing.as_deref().into_iter().flatten())
            .filter(|span| {
                span.tenant == *tenant && crate::trace_part::span_overlaps(span, start_ns, end_ns)
            })
        {
            if spans.len() >= limit {
                return Err(format!("trace query exceeds the maximum of {limit} spans"));
            }
            spans.push(span.clone());
        }
        Ok(spans)
    }

    pub fn trace_ids(&self, tenant: &TenantId) -> BTreeMap<String, usize> {
        let mut ids = BTreeMap::new();
        for span in self.snapshot(tenant) {
            *ids.entry(span.trace_id).or_default() += 1;
        }
        ids
    }
}

impl Default for TraceMemTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::test_tenant;
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans};

    fn request(trace_id: Vec<u8>, span_id: Vec<u8>) -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("api".to_string())),
                        }),
                        key_strindex: 0,
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id,
                        span_id,
                        start_time_unix_nano: 100,
                        end_time_unix_nano: 250,
                        name: "GET /items".to_string(),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    #[test]
    fn normalizes_ids_and_preserves_resource() {
        let spans = normalize_request(&test_tenant(), request(vec![1; 16], vec![2; 8])).unwrap();
        assert_eq!(spans[0].trace_id, "01".repeat(16));
        assert_eq!(spans[0].span_id, "02".repeat(8));
        assert_eq!(spans[0].service_name(), Some("api"));
        assert_eq!(spans[0].duration_ns(), 150);
    }

    #[test]
    fn rejects_invalid_ids_and_end_before_start() {
        assert_eq!(
            normalize_request(&test_tenant(), request(vec![0; 16], vec![2; 8])).unwrap_err(),
            TraceError::InvalidTraceId
        );
        assert_eq!(
            normalize_request(&test_tenant(), request(vec![1; 16], vec![2; 7])).unwrap_err(),
            TraceError::InvalidSpanId
        );
        let mut invalid = request(vec![1; 16], vec![2; 8]);
        invalid.resource_spans[0].scope_spans[0].spans[0].end_time_unix_nano = 99;
        assert_eq!(
            normalize_request(&test_tenant(), invalid).unwrap_err(),
            TraceError::EndBeforeStart
        );
    }

    #[test]
    fn duration_handles_the_maximum_supported_timestamp_range() {
        let mut extreme = request(vec![1; 16], vec![2; 8]);
        extreme.resource_spans[0].scope_spans[0].spans[0].start_time_unix_nano = 0;
        extreme.resource_spans[0].scope_spans[0].spans[0].end_time_unix_nano = i64::MAX as u64;

        let span = normalize_request(&test_tenant(), extreme)
            .unwrap()
            .remove(0);

        assert_eq!(span.duration_ns(), i64::MAX as u64);
        assert_eq!(
            span.tag_value("duration"),
            Some((i64::MAX as u64).to_string())
        );
    }

    #[test]
    fn memtable_queries_across_active_and_flushing_buffers() {
        let memtable = TraceMemTable::new();
        memtable
            .insert(normalize_request(&test_tenant(), request(vec![1; 16], vec![2; 8])).unwrap());
        let snapshot = memtable.begin_flush();
        memtable
            .insert(normalize_request(&test_tenant(), request(vec![1; 16], vec![3; 8])).unwrap());
        assert_eq!(
            memtable
                .query_trace_id(&test_tenant(), &"01".repeat(16))
                .len(),
            2
        );
        memtable.abort_flush(snapshot);
        assert_eq!(
            memtable
                .query_trace_id(&test_tenant(), &"01".repeat(16))
                .len(),
            2
        );
    }
}
