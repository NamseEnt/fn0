use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::RwLock;

use crate::object_storage::TraceManifest;
use crate::tenant::TenantId;
use crate::trace::TraceSpan;
use crate::trace_part::{TracePart, TracePartReader, discover_trace_parts};

/// How much larger a decoded span is than its stored bytes.
///
/// Stored trace bytes are zstd-compressed Parquet columns; a live `TraceSpan`
/// is the protobuf message plus every id, name and attribute string owned
/// separately on the heap. The factor is a deliberately pessimistic round
/// number rather than a measurement — it prices an admission, and a request
/// that overshoots it still meets `max_query_memory_bytes` inside the scan.
const SPAN_DECODE_EXPANSION: u64 = 8;

/// The floor under that estimate: a span costs at least its own struct plus
/// the two ids, whatever the compression achieved on a run of similar spans.
pub const MIN_SPAN_BYTES: u64 = 512;

pub struct TraceRegistry {
    inner: RwLock<HashMap<String, Arc<TracePartReader>>>,
    /// Bytes each tenant holds across the registered trace parts, maintained as
    /// the set changes for the same reason the log registry keeps its own: a
    /// storage quota is read on the ingest path, and summing tenant segments
    /// per write is work proportional to the part count.
    stored_bytes: RwLock<HashMap<TenantId, u64>>,
    operation_lock: Arc<tokio::sync::RwLock<()>>,
}

/// One trace part's contribution to the per-tenant totals.
fn reader_tenant_bytes(reader: &TracePartReader) -> Vec<(TenantId, u64)> {
    reader
        .part()
        .meta
        .tenants
        .iter()
        .map(|segment| (segment.tenant.clone(), segment.bytes.len()))
        .collect()
}

fn census_of(readers: &HashMap<String, Arc<TracePartReader>>) -> HashMap<TenantId, u64> {
    let mut totals: HashMap<TenantId, u64> = HashMap::new();
    for reader in readers.values() {
        for (tenant, bytes) in reader_tenant_bytes(reader) {
            *totals.entry(tenant).or_insert(0) += bytes;
        }
    }
    totals
}

impl TraceRegistry {
    pub fn standalone() -> Self {
        Self::new(Arc::new(tokio::sync::RwLock::new(())))
    }

    pub fn new(operation_lock: Arc<tokio::sync::RwLock<()>>) -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            stored_bytes: RwLock::new(HashMap::new()),
            operation_lock,
        }
    }

    pub fn operation_lock(&self) -> Arc<tokio::sync::RwLock<()>> {
        self.operation_lock.clone()
    }

    /// Every tenant that owns a segment in some trace part. See
    /// `PartRegistry::visit_tenants`.
    pub fn visit_tenants(&self, mut visit: impl FnMut(&TenantId)) {
        for reader in self.inner.read().values() {
            for segment in &reader.part().meta.tenants {
                visit(&segment.tenant);
            }
        }
    }

    pub fn load_from_disk(
        traces_root: &Path,
        operation_lock: Arc<tokio::sync::RwLock<()>>,
    ) -> Result<Self, String> {
        let registry = Self::new(operation_lock);
        registry.reload_from_disk(traces_root)?;
        Ok(registry)
    }

    pub fn reload_from_disk(&self, traces_root: &Path) -> Result<(), String> {
        let parts = discover_trace_parts(traces_root)?;
        let mut readers = HashMap::new();
        for part in parts {
            let id = part.meta.id.clone();
            let reader = TracePartReader::open(part)
                .map_err(|error| format!("failed to open trace part {id}: {error}"))?;
            readers.insert(id, Arc::new(reader));
        }
        let mut inner = self.inner.write();
        *self.stored_bytes.write() = census_of(&readers);
        *inner = readers;
        Ok(())
    }

    pub fn load_from_manifest(
        traces_root: &Path,
        manifest: &TraceManifest,
        operation_lock: Arc<tokio::sync::RwLock<()>>,
    ) -> Result<Self, String> {
        let registry = Self::new(operation_lock);
        let mut readers = HashMap::new();
        for descriptor in &manifest.parts {
            let dir = traces_root.join(&descriptor.partition).join(&descriptor.id);
            let part = crate::trace_part::load_trace_part(&dir).map_err(|error| {
                format!(
                    "failed to load trace manifest part {}: {error}",
                    descriptor.id
                )
            })?;
            if part.meta.id != descriptor.id || part.meta.partition != descriptor.partition {
                return Err(format!(
                    "cached trace part metadata does not match manifest descriptor {}/{}",
                    descriptor.partition, descriptor.id
                ));
            }
            let reader = TracePartReader::open_cached(part).map_err(|error| {
                format!(
                    "failed to open trace manifest part {}: {error}",
                    descriptor.id
                )
            })?;
            readers.insert(descriptor.id.clone(), Arc::new(reader));
        }
        *registry.stored_bytes.write() = census_of(&readers);
        *registry.inner.write() = readers;
        Ok(registry)
    }

    /// Open readers for freshly written trace parts, without touching the
    /// registry — the same open-outside-the-lock split as
    /// [`crate::part_registry::PartRegistry::open_parts`].
    pub fn open_parts(
        parts: Vec<TracePart>,
    ) -> Result<Vec<(String, Arc<TracePartReader>)>, String> {
        let mut readers = Vec::with_capacity(parts.len());
        for part in parts {
            let id = part.meta.id.clone();
            let reader = TracePartReader::open(part)
                .map_err(|error| format!("failed to open trace part {id}: {error}"))?;
            readers.push((id, Arc::new(reader)));
        }
        Ok(readers)
    }

    pub fn register(&self, parts: Vec<TracePart>) -> Result<Vec<String>, String> {
        Ok(self.register_opened(Self::open_parts(parts)?))
    }

    pub fn register_opened(&self, readers: Vec<(String, Arc<TracePartReader>)>) -> Vec<String> {
        let ids = readers.iter().map(|(id, _)| id.clone()).collect();
        let mut inner = self.inner.write();
        let mut census = self.stored_bytes.write();
        for (id, reader) in readers {
            // Registering an id already present replaces it, so its predecessor
            // leaves the census with it.
            if let Some(previous) = inner.insert(id, reader.clone()) {
                subtract(&mut census, &previous);
            }
            for (tenant, bytes) in reader_tenant_bytes(&reader) {
                *census.entry(tenant).or_insert(0) += bytes;
            }
        }
        ids
    }

    pub fn unregister(&self, ids: &[String]) {
        let mut inner = self.inner.write();
        let mut census = self.stored_bytes.write();
        for id in ids {
            if let Some(removed) = inner.remove(id) {
                subtract(&mut census, &removed);
            }
        }
    }

    /// Bytes the tenant's row groups occupy across every registered trace part.
    pub fn tenant_stored_bytes(&self, tenant: &TenantId) -> u64 {
        self.stored_bytes.read().get(tenant).copied().unwrap_or(0)
    }

    pub fn snapshot(&self) -> Vec<Arc<TracePartReader>> {
        self.inner.read().values().cloned().collect()
    }

    /// What one materialized span of this tenant's costs, in bytes.
    ///
    /// Unlike a log part, a trace part records no materialized size — the
    /// format has a stored byte extent per tenant and nothing that says what
    /// decoding it costs. So the figure is the stored average
    /// (`bytes ÷ row_count`, both per tenant, straight out of `meta.json`)
    /// scaled by [`SPAN_DECODE_EXPANSION`]. That constant is the estimate's
    /// one unmeasured term, which is why nothing rests on it alone: it prices
    /// a request for admission, and `max_query_memory_bytes` still refuses the
    /// scan if the real spans come out larger.
    ///
    /// `None` when the tenant has no stored spans, which is the caller's cue
    /// to price the memtable's spans alone.
    pub fn average_span_bytes(&self, tenant: &TenantId) -> Option<u64> {
        let mut rows = 0u64;
        let mut bytes = 0u64;
        for reader in self.inner.read().values() {
            let Some(segment) = reader
                .part()
                .meta
                .tenants
                .iter()
                .find(|segment| segment.tenant == *tenant)
            else {
                continue;
            };
            rows = rows.saturating_add(segment.row_count);
            bytes = bytes.saturating_add(segment.bytes.len());
        }
        (rows > 0).then(|| {
            bytes
                .div_ceil(rows)
                .saturating_mul(SPAN_DECODE_EXPANSION)
                .max(MIN_SPAN_BYTES)
        })
    }

    pub fn candidate_part_ids(
        &self,
        tenant: &TenantId,
        trace_id: &str,
    ) -> std::collections::HashSet<String> {
        self.inner
            .read()
            .iter()
            .filter(|(_, reader)| reader.may_match_trace_id(tenant, trace_id))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Parts that hold any row for `tenant`. Used to pin the restore set for
    /// an unfiltered search, which would otherwise restore every tenant's
    /// object bodies.
    pub fn tenant_part_ids(&self, tenant: &TenantId) -> std::collections::HashSet<String> {
        self.inner
            .read()
            .iter()
            .filter(|(_, reader)| reader.part().meta.tenant_row_groups(tenant).is_some())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// The same set narrowed to a time range. Restoring a part is a download,
    /// so this decides the cost of the request before any of it is paid.
    pub fn tenant_part_ids_in_range(
        &self,
        tenant: &TenantId,
        start_ns: i64,
        end_ns: i64,
    ) -> std::collections::HashSet<String> {
        self.inner
            .read()
            .iter()
            .filter(|(_, reader)| {
                reader
                    .part()
                    .meta
                    .tenant_overlaps_range(tenant, start_ns, end_ns)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn missing_data_ids(
        &self,
        ids: &std::collections::HashSet<String>,
    ) -> std::collections::HashSet<String> {
        self.inner
            .read()
            .iter()
            .filter(|(id, reader)| ids.contains(*id) && !reader.part().data_path().exists())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// See `PartRegistry::part_dirs`.
    pub fn part_dirs(&self) -> Vec<std::path::PathBuf> {
        self.inner
            .read()
            .values()
            .map(|reader| reader.part().dir.clone())
            .collect()
    }

    pub fn part_ids(&self) -> std::collections::HashSet<String> {
        self.inner.read().keys().cloned().collect()
    }

    pub fn query_trace_id(
        &self,
        tenant: &TenantId,
        trace_id: &str,
        scan_limit: Option<usize>,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Vec<TraceSpan>, String> {
        let mut readers = self.snapshot();
        readers.sort_by_key(|reader| reader.part().meta.min_ts_ns);
        let mut spans = Vec::new();
        for reader in readers {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                return Err("trace query timed out".to_string());
            }
            if !reader.may_match_trace_id(tenant, trace_id) {
                continue;
            }
            let remaining = scan_limit
                .map(|limit| limit.saturating_sub(spans.len()))
                .unwrap_or(usize::MAX);
            spans.extend(reader.query_trace_id_limited(
                tenant,
                trace_id,
                remaining,
                cancellation,
            )?);
        }
        spans.sort_by(|left, right| {
            left.start_time_ns
                .cmp(&right.start_time_ns)
                .then_with(|| left.span_id.cmp(&right.span_id))
        });
        Ok(spans)
    }

    pub fn query_all(
        &self,
        tenant: &TenantId,
        scan_limit: Option<usize>,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Vec<TraceSpan>, String> {
        self.query_range(tenant, None, scan_limit, cancellation)
    }

    /// Every span for the tenant, or only those starting inside `range`.
    pub fn query_range(
        &self,
        tenant: &TenantId,
        range: Option<(i64, i64)>,
        scan_limit: Option<usize>,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Vec<TraceSpan>, String> {
        let mut spans = Vec::new();
        for reader in self.snapshot() {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                return Err("trace query timed out".to_string());
            }
            let remaining = scan_limit
                .map(|limit| limit.saturating_sub(spans.len()))
                .unwrap_or(usize::MAX);
            let part_spans = match range {
                Some((start_ns, end_ns)) => {
                    reader.query_range_limited(tenant, start_ns, end_ns, remaining, cancellation)?
                }
                None => reader.query_all_limited(tenant, remaining, cancellation)?,
            };
            spans.extend(part_spans);
        }
        spans.sort_by(|left, right| {
            left.start_time_ns
                .cmp(&right.start_time_ns)
                .then_with(|| left.span_id.cmp(&right.span_id))
        });
        Ok(spans)
    }

    pub fn part_count(&self) -> usize {
        self.inner.read().len()
    }
}

fn subtract(census: &mut HashMap<TenantId, u64>, reader: &TracePartReader) {
    for (tenant, bytes) in reader_tenant_bytes(reader) {
        if let Some(total) = census.get_mut(&tenant) {
            *total = total.saturating_sub(bytes);
            if *total == 0 {
                census.remove(&tenant);
            }
        }
    }
}
