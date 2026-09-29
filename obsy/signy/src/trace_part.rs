use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use arrow::array::{ArrayRef, AsArray, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;
use serde::{Deserialize, Serialize};

use crate::bloom::BloomFilter;
use crate::part::partition_of;
use crate::tenant::TenantId;
use crate::trace::TraceSpan;

pub const TRACE_DATA_FILE: &str = "data.parquet";
pub const TRACE_BLOOM_FILE: &str = "trace.bloom";
pub const TRACE_META_FILE: &str = "meta.json";

const TRACE_BLOOM_MAGIC: &[u8; 4] = b"TBF1";

/// One tenant's contiguous run of row groups in a shared trace part. Same
/// role as `part::TenantSegment` on the log side.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceTenantSegment {
    pub tenant: TenantId,
    pub row_group_start: u32,
    pub row_group_end: u32,
    pub row_count: u64,
    /// Where this tenant's row groups sit in `data.parquet`.
    ///
    /// Recorded for accounting rather than for reading: nothing range-reads a
    /// trace part yet. It is what a storage quota charges in, and it has to
    /// come from the format because the alternative — the local file's size —
    /// disappears the moment the cache evicts the body.
    ///
    /// `default` so a part written before the field existed still parses,
    /// reporting an empty extent rather than failing to open.
    #[serde(default)]
    pub bytes: crate::part::ByteRange,
}

#[derive(Clone, Debug)]
pub struct TracePartMeta {
    pub id: String,
    pub partition: String,
    pub min_ts_ns: i64,
    pub max_ts_ns: i64,
    pub row_count: u64,
    pub row_group_count: u32,
    pub row_group_min_ts: Vec<i64>,
    pub row_group_max_ts: Vec<i64>,
    pub tenants: Vec<TraceTenantSegment>,
    #[allow(dead_code)]
    integrity: TracePartIntegrity,
}

impl TracePartMeta {
    pub fn tenant_row_groups(&self, tenant: &TenantId) -> Option<std::ops::Range<u32>> {
        self.tenants
            .binary_search_by(|segment| segment.tenant.cmp(tenant))
            .ok()
            .map(|index| self.tenants[index].row_group_start..self.tenants[index].row_group_end)
    }

    /// The tenant's row groups whose span timestamps can reach `[start_ns,
    /// end_ns]`.
    ///
    /// The tenant segment carries no timestamps of its own, but row groups do
    /// and a tenant owns a contiguous range of them, so the bounds are already
    /// on hand. Without this a tag lookup restores and scans every part the
    /// tenant has ever written, which is the whole history for one dropdown.
    pub fn tenant_row_groups_in_range(
        &self,
        tenant: &TenantId,
        start_ns: i64,
        end_ns: i64,
    ) -> Vec<usize> {
        let Some(groups) = self.tenant_row_groups(tenant) else {
            return Vec::new();
        };
        groups
            .filter(|row_group| {
                let index = *row_group as usize;
                match (
                    self.row_group_min_ts.get(index),
                    self.row_group_max_ts.get(index),
                ) {
                    (Some(min_ts), Some(max_ts)) => *max_ts >= start_ns && *min_ts <= end_ns,
                    // Metadata that cannot answer must not be able to hide
                    // data: an absent bound scans rather than prunes.
                    _ => true,
                }
            })
            .map(|row_group| row_group as usize)
            .collect()
    }

    /// Whether any of the tenant's row groups reaches the range at all.
    pub fn tenant_overlaps_range(&self, tenant: &TenantId, start_ns: i64, end_ns: i64) -> bool {
        !self
            .tenant_row_groups_in_range(tenant, start_ns, end_ns)
            .is_empty()
    }
}

/// Whether a span touches `[start_ns, end_ns]` at any point.
///
/// The one definition of "in the window" for spans, shared by the part reader
/// and the memtable side of a scan so the two halves of a query cannot answer
/// differently.
pub fn span_overlaps(span: &TraceSpan, start_ns: i64, end_ns: i64) -> bool {
    span.end_time_ns >= start_ns && span.start_time_ns <= end_ns
}

#[derive(Clone, Debug)]
pub struct TracePart {
    pub dir: PathBuf,
    pub meta: TracePartMeta,
}

impl TracePart {
    pub fn data_path(&self) -> PathBuf {
        self.dir.join(TRACE_DATA_FILE)
    }

    pub fn bloom_path(&self) -> PathBuf {
        self.dir.join(TRACE_BLOOM_FILE)
    }

    pub fn meta_path(&self) -> PathBuf {
        self.dir.join(TRACE_META_FILE)
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct TraceSpanFile {
    tenant: TenantId,
    trace_id: String,
    span_id: String,
    start_time_ns: i64,
    end_time_ns: i64,
    span: opentelemetry_proto::tonic::trace::v1::Span,
    resource: Option<opentelemetry_proto::tonic::resource::v1::Resource>,
    resource_schema_url: String,
    scope: Option<opentelemetry_proto::tonic::common::v1::InstrumentationScope>,
    scope_schema_url: String,
}

impl From<&TraceSpan> for TraceSpanFile {
    fn from(span: &TraceSpan) -> Self {
        Self {
            tenant: span.tenant.clone(),
            trace_id: span.trace_id.clone(),
            span_id: span.span_id.clone(),
            start_time_ns: span.start_time_ns,
            end_time_ns: span.end_time_ns,
            span: span.span.clone(),
            resource: span.resource.clone(),
            resource_schema_url: span.resource_schema_url.clone(),
            scope: span.scope.clone(),
            scope_schema_url: span.scope_schema_url.clone(),
        }
    }
}

impl From<TraceSpanFile> for TraceSpan {
    fn from(span: TraceSpanFile) -> Self {
        Self {
            tenant: span.tenant,
            trace_id: span.trace_id,
            span_id: span.span_id,
            start_time_ns: span.start_time_ns,
            end_time_ns: span.end_time_ns,
            span: span.span,
            resource: span.resource,
            resource_schema_url: span.resource_schema_url,
            scope: span.scope,
            scope_schema_url: span.scope_schema_url,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TracePartIntegrity {
    data_crc32: u32,
    bloom_crc32: u32,
    metadata_crc32: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct TraceMetaFile {
    #[serde(default)]
    id: String,
    partition: String,
    min_ts_ns: i64,
    max_ts_ns: i64,
    row_count: u64,
    row_group_count: u32,
    row_group_min_ts: Vec<i64>,
    row_group_max_ts: Vec<i64>,
    tenants: Vec<TraceTenantSegment>,
    integrity: TracePartIntegrity,
}

/// Writes the flushing buffer without taking ownership of it.
///
/// The buffer stays shared with the memtable until the flush commits, because
/// an abort has to put it back. Bucketing by reference is what keeps that from
/// costing a second copy of everything being flushed.
pub fn flush_trace_spans(
    spans: &[TraceSpan],
    traces_root: &Path,
    row_group_size: usize,
) -> io::Result<Vec<TracePart>> {
    flush_trace_spans_with_id(spans, traces_root, row_group_size, None)
}

/// Flush one compaction partition using an id that was recorded durably before
/// the replacement files were created. The regular flush path keeps generating
/// ids internally because it may produce one part per partition.
pub fn flush_trace_spans_with_id(
    spans: &[TraceSpan],
    traces_root: &Path,
    row_group_size: usize,
    forced_id: Option<&str>,
) -> io::Result<Vec<TracePart>> {
    if spans.is_empty() {
        return Ok(Vec::new());
    }
    if row_group_size == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "trace row group size must be positive",
        ));
    }
    fs::create_dir_all(traces_root.join(".tmp"))?;
    let mut by_partition: BTreeMap<String, Vec<&TraceSpan>> = BTreeMap::new();
    for span in spans {
        by_partition
            .entry(partition_of(span.start_time_ns))
            .or_default()
            .push(span);
    }
    if forced_id.is_some() && by_partition.len() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "a forced trace compaction id requires one partition",
        ));
    }

    let mut parts = Vec::new();
    let mut committed_dirs = Vec::new();
    for (partition, mut partition_spans) in by_partition {
        // Tenant leads the sort key so a tenant occupies a contiguous run of
        // row groups; trace_id still leads within a tenant so the per-row-group
        // trace-id bloom stays selective.
        partition_spans.sort_by(|left, right| {
            left.tenant
                .cmp(&right.tenant)
                .then_with(|| left.trace_id.cmp(&right.trace_id))
                .then_with(|| left.start_time_ns.cmp(&right.start_time_ns))
                .then_with(|| left.span_id.cmp(&right.span_id))
        });
        let id = forced_id
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{}-{}", partition.replace('-', ""), uuid::Uuid::new_v4()));
        if let Some(forced_id) = forced_id {
            let expected_prefix = format!("{}-", partition.replace('-', ""));
            if !forced_id.starts_with(&expected_prefix) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "trace compaction output id does not match its partition",
                ));
            }
        }
        let tmp_dir = traces_root.join(".tmp").join(&id);
        let final_dir = traces_root.join(&partition).join(&id);
        let result = (|| -> io::Result<TracePart> {
            if tmp_dir.exists() {
                fs::remove_dir_all(&tmp_dir)?;
            }
            fs::create_dir_all(&tmp_dir)?;
            write_trace_part_files(&tmp_dir, &id, &partition, &partition_spans, row_group_size)?;
            if let Some(parent) = final_dir.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(&tmp_dir, &final_dir)?;
            committed_dirs.push(final_dir.clone());
            sync_dir(final_dir.parent().unwrap_or(traces_root))?;
            sync_dir(traces_root)?;
            load_trace_part(&final_dir).map_err(io::Error::other)
        })();

        match result {
            Ok(part) => parts.push(part),
            Err(error) => {
                let _ = fs::remove_dir_all(&tmp_dir);
                let cleanup = crate::part::remove_part_dirs(&committed_dirs);
                return Err(match cleanup {
                    Ok(()) => error,
                    Err(cleanup_error) => io::Error::other(format!(
                        "trace part flush failed: {error}; rollback failed: {cleanup_error}"
                    )),
                });
            }
        }
    }
    Ok(parts)
}

fn write_trace_part_files(
    dir: &Path,
    id: &str,
    partition: &str,
    spans: &[&TraceSpan],
    row_group_size: usize,
) -> io::Result<()> {
    let row_group_ranges = write_trace_parquet(&dir.join(TRACE_DATA_FILE), spans, row_group_size)?;
    write_trace_bloom(&dir.join(TRACE_BLOOM_FILE), spans, row_group_size)?;

    let bounds = row_group_bounds(spans, row_group_size);
    let min_ts = spans.iter().map(|span| span.start_time_ns).min().unwrap();
    let max_ts = spans.iter().map(|span| span.end_time_ns).max().unwrap();
    let meta_without_crc = TraceMetaFile {
        id: id.to_string(),
        partition: partition.to_string(),
        min_ts_ns: min_ts,
        max_ts_ns: max_ts,
        row_count: spans.len() as u64,
        row_group_count: bounds.len() as u32,
        row_group_min_ts: bounds
            .iter()
            .map(|(start, end)| {
                spans[*start..*end]
                    .iter()
                    .map(|span| span.start_time_ns)
                    .min()
                    .unwrap()
            })
            .collect(),
        row_group_max_ts: bounds
            .iter()
            .map(|(start, end)| {
                spans[*start..*end]
                    .iter()
                    .map(|span| span.end_time_ns)
                    .max()
                    .unwrap()
            })
            .collect(),
        tenants: trace_tenant_segments(spans, &bounds, &row_group_ranges),
        integrity: TracePartIntegrity {
            data_crc32: file_crc32(&dir.join(TRACE_DATA_FILE))?,
            bloom_crc32: file_crc32(&dir.join(TRACE_BLOOM_FILE))?,
            metadata_crc32: 0,
        },
    };
    let mut meta = meta_without_crc;
    meta.integrity.metadata_crc32 = metadata_crc32(&meta).map_err(io::Error::other)?;
    let encoded = serde_json::to_vec_pretty(&meta).map_err(io::Error::other)?;
    fs::write(dir.join(TRACE_META_FILE), encoded)?;
    sync_file(&dir.join(TRACE_META_FILE))?;
    Ok(())
}

fn trace_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("tenant", DataType::Utf8, false),
        Field::new("start_time_ns", DataType::Int64, false),
        Field::new("end_time_ns", DataType::Int64, false),
        Field::new("trace_id", DataType::Utf8, false),
        Field::new("span_id", DataType::Utf8, false),
        Field::new("payload", DataType::Utf8, false),
    ]))
}

fn trace_row_group_batch(schema: &Arc<Schema>, spans: &[&TraceSpan]) -> io::Result<RecordBatch> {
    let tenants: Vec<&str> = spans.iter().map(|span| span.tenant.as_str()).collect();
    let starts: Vec<i64> = spans.iter().map(|span| span.start_time_ns).collect();
    let ends: Vec<i64> = spans.iter().map(|span| span.end_time_ns).collect();
    let trace_ids: Vec<&str> = spans.iter().map(|span| span.trace_id.as_str()).collect();
    let span_ids: Vec<&str> = spans.iter().map(|span| span.span_id.as_str()).collect();
    let payloads: Vec<String> = spans
        .iter()
        .map(|span| serde_json::to_string(&TraceSpanFile::from(*span)).map_err(io::Error::other))
        .collect::<io::Result<_>>()?;
    let payload_refs: Vec<&str> = payloads.iter().map(String::as_str).collect();
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(tenants)) as ArrayRef,
            Arc::new(Int64Array::from(starts)) as ArrayRef,
            Arc::new(Int64Array::from(ends)) as ArrayRef,
            Arc::new(StringArray::from(trace_ids)) as ArrayRef,
            Arc::new(StringArray::from(span_ids)) as ArrayRef,
            Arc::new(StringArray::from(payload_refs)) as ArrayRef,
        ],
    )
    .map_err(io::Error::other)
}

fn write_trace_parquet(
    path: &Path,
    spans: &[&TraceSpan],
    row_group_size: usize,
) -> io::Result<Vec<crate::part::ByteRange>> {
    let schema = trace_schema();
    let bounds = row_group_bounds(spans, row_group_size);
    let file = fs::File::create(path)?;
    let properties = WriterProperties::builder()
        .set_max_row_group_row_count(Some(row_group_size))
        .set_compression(Compression::ZSTD(ZstdLevel::default()))
        .build();
    let mut writer =
        ArrowWriter::try_new(file, schema.clone(), Some(properties)).map_err(io::Error::other)?;
    // The bloom sidecar and the tenant index address row groups by ordinal, so
    // the boundaries must be exactly `bounds`.
    for (start, end) in &bounds {
        let batch = trace_row_group_batch(&schema, &spans[*start..*end])?;
        writer.write(&batch).map_err(io::Error::other)?;
        writer.flush().map_err(io::Error::other)?;
    }
    let metadata = writer.close().map_err(io::Error::other)?;
    sync_file(path)?;

    // Same extent arithmetic as the log side: a row group's column chunks are
    // contiguous, but they are reported in schema order rather than file order.
    let ranges: Vec<crate::part::ByteRange> = metadata
        .row_groups()
        .iter()
        .map(|row_group| {
            let mut start = u64::MAX;
            let mut end = 0u64;
            for column in row_group.columns() {
                let (offset, length) = column.byte_range();
                start = start.min(offset);
                end = end.max(offset.saturating_add(length));
            }
            crate::part::ByteRange { start, end }
        })
        .collect();
    if ranges.len() != bounds.len() {
        return Err(io::Error::other(format!(
            "trace parquet wrote {} row groups for {} planned boundaries",
            ranges.len(),
            bounds.len()
        )));
    }
    Ok(ranges)
}

fn write_trace_bloom(path: &Path, spans: &[&TraceSpan], row_group_size: usize) -> io::Result<()> {
    let bounds = row_group_bounds(spans, row_group_size);
    let mut encoded = Vec::new();
    encoded.extend_from_slice(TRACE_BLOOM_MAGIC);
    encoded.extend_from_slice(&(bounds.len() as u32).to_le_bytes());
    for (start, end) in bounds {
        let mut ids = std::collections::BTreeSet::new();
        for span in &spans[start..end] {
            ids.insert(span.trace_id.as_bytes());
        }
        let mut bloom = BloomFilter::with_capacity(ids.len().max(1), 0.01);
        for id in ids {
            bloom.insert(id);
        }
        let bytes = bloom.encode();
        encoded.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        encoded.extend_from_slice(&bytes);
    }
    fs::write(path, encoded)?;
    sync_file(path)
}

/// Row-group boundaries that never straddle a tenant.
fn row_group_bounds(spans: &[&TraceSpan], row_group_size: usize) -> Vec<(usize, usize)> {
    let mut bounds = Vec::new();
    let mut segment_start = 0usize;
    while segment_start < spans.len() {
        let mut segment_end = segment_start;
        while segment_end < spans.len() && spans[segment_end].tenant == spans[segment_start].tenant
        {
            segment_end += 1;
        }
        let mut start = segment_start;
        while start < segment_end {
            let end = (start + row_group_size).min(segment_end);
            bounds.push((start, end));
            start = end;
        }
        segment_start = segment_end;
    }
    bounds
}

fn trace_tenant_segments(
    spans: &[&TraceSpan],
    bounds: &[(usize, usize)],
    row_group_ranges: &[crate::part::ByteRange],
) -> Vec<TraceTenantSegment> {
    let mut segments: Vec<TraceTenantSegment> = Vec::new();
    for (row_group, (start, end)) in bounds.iter().enumerate() {
        let tenant = &spans[*start].tenant;
        let row_count = (end - start) as u64;
        let range = row_group_ranges.get(row_group).copied().unwrap_or_default();
        match segments.last_mut() {
            Some(segment) if segment.tenant == *tenant => {
                segment.row_group_end = row_group as u32 + 1;
                segment.row_count += row_count;
                segment.bytes.end = segment.bytes.end.max(range.end);
            }
            _ => segments.push(TraceTenantSegment {
                tenant: tenant.clone(),
                row_group_start: row_group as u32,
                row_group_end: row_group as u32 + 1,
                row_count,
                bytes: range,
            }),
        }
    }
    segments
}

fn validate_trace_tenant_segments(meta: &TraceMetaFile) -> Result<(), String> {
    if meta.tenants.is_empty() {
        return Err("trace part metadata has no tenant segments".to_string());
    }
    let mut expected_start = 0u32;
    let mut total_rows = 0u64;
    for (index, segment) in meta.tenants.iter().enumerate() {
        if index > 0 && meta.tenants[index - 1].tenant >= segment.tenant {
            return Err("trace tenant segments are not sorted by tenant".to_string());
        }
        if segment.row_group_start != expected_start
            || segment.row_group_end <= segment.row_group_start
            || segment.row_group_end > meta.row_group_count
        {
            return Err("trace tenant segments do not tile the row groups".to_string());
        }
        if segment.row_count == 0 {
            return Err("trace tenant segment is empty".to_string());
        }
        total_rows = total_rows.saturating_add(segment.row_count);
        expected_start = segment.row_group_end;
    }
    if expected_start != meta.row_group_count {
        return Err("trace tenant segments do not cover every row group".to_string());
    }
    if total_rows != meta.row_count {
        return Err("trace tenant segment row counts do not sum to the part row count".to_string());
    }
    Ok(())
}

pub fn load_trace_part(dir: &Path) -> Result<TracePart, String> {
    let dir_metadata = fs::symlink_metadata(dir).map_err(|error| error.to_string())?;
    if dir_metadata.file_type().is_symlink() {
        return Err(format!(
            "refusing symlinked trace part directory {}",
            dir.display()
        ));
    }
    for file in [TRACE_META_FILE, TRACE_BLOOM_FILE, TRACE_DATA_FILE] {
        let path = dir.join(file);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("refusing symlinked trace file {}", path.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    let bytes = fs::read(dir.join(TRACE_META_FILE)).map_err(|error| error.to_string())?;
    let meta: TraceMetaFile = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if metadata_crc32(&meta)? != meta.integrity.metadata_crc32 {
        return Err(format!(
            "trace metadata checksum mismatch in {}",
            dir.display()
        ));
    }
    if dir.join(TRACE_DATA_FILE).exists() {
        validate_file_crc(
            &dir.join(TRACE_DATA_FILE),
            meta.integrity.data_crc32,
            "trace parquet",
        )?;
    }
    validate_file_crc(
        &dir.join(TRACE_BLOOM_FILE),
        meta.integrity.bloom_crc32,
        "trace bloom",
    )?;
    if meta.row_group_count as usize != meta.row_group_min_ts.len()
        || meta.row_group_count as usize != meta.row_group_max_ts.len()
    {
        return Err(format!(
            "trace row-group metadata mismatch in {}",
            dir.display()
        ));
    }
    validate_trace_tenant_segments(&meta)?;
    Ok(TracePart {
        dir: dir.to_path_buf(),
        meta: TracePartMeta {
            id: meta.id,
            partition: meta.partition,
            min_ts_ns: meta.min_ts_ns,
            max_ts_ns: meta.max_ts_ns,
            row_count: meta.row_count,
            row_group_count: meta.row_group_count,
            row_group_min_ts: meta.row_group_min_ts,
            row_group_max_ts: meta.row_group_max_ts,
            tenants: meta.tenants,
            integrity: meta.integrity,
        },
    })
}

pub fn discover_trace_parts(root: &Path) -> Result<Vec<TracePart>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut parts = Vec::new();
    for partition in fs::read_dir(root).map_err(|error| error.to_string())? {
        let partition = partition.map_err(|error| error.to_string())?;
        if !partition
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
            || partition.file_name() == ".tmp"
            || partition.file_name() == ".compact"
        {
            continue;
        }
        for part_dir in fs::read_dir(partition.path()).map_err(|error| error.to_string())? {
            let part_dir = part_dir.map_err(|error| error.to_string())?;
            if part_dir
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                parts.push(load_trace_part(&part_dir.path())?);
            }
        }
    }
    Ok(parts)
}

pub struct TracePartReader {
    part: TracePart,
    blooms: Vec<BloomFilter>,
}

impl TracePartReader {
    pub fn open(part: TracePart) -> Result<Self, String> {
        Self::open_internal(part, true)
    }

    pub fn open_cached(part: TracePart) -> Result<Self, String> {
        Self::open_internal(part, false)
    }

    fn open_internal(part: TracePart, require_data: bool) -> Result<Self, String> {
        if require_data && !part.data_path().exists() {
            return Err(format!(
                "trace parquet is missing: {}",
                part.data_path().display()
            ));
        }
        let bytes = fs::read(part.bloom_path()).map_err(|error| error.to_string())?;
        let blooms = decode_trace_blooms(&bytes, part.meta.row_group_count as usize)?;
        Ok(Self { part, blooms })
    }

    pub fn part(&self) -> &TracePart {
        &self.part
    }

    pub fn may_match_trace_id(&self, tenant: &TenantId, trace_id: &str) -> bool {
        let Some(groups) = self.part.meta.tenant_row_groups(tenant) else {
            return false;
        };
        groups
            .clone()
            .any(|row_group| self.blooms[row_group as usize].contains(trace_id.as_bytes()))
    }

    #[cfg(test)]
    pub fn query_trace_id(
        &self,
        tenant: &TenantId,
        trace_id: &str,
    ) -> Result<Vec<TraceSpan>, String> {
        self.query_trace_id_limited(tenant, trace_id, usize::MAX, None)
    }

    pub fn query_trace_id_limited(
        &self,
        tenant: &TenantId,
        trace_id: &str,
        limit: usize,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Vec<TraceSpan>, String> {
        let Some(groups) = self.part.meta.tenant_row_groups(tenant) else {
            return Ok(Vec::new());
        };
        let selected: Vec<usize> = groups
            .filter(|row_group| self.blooms[*row_group as usize].contains(trace_id.as_bytes()))
            .map(|row_group| row_group as usize)
            .collect();
        self.query_selected(tenant, selected, Some(trace_id), limit, cancellation)
    }

    #[cfg(test)]
    pub fn query_all(&self, tenant: &TenantId) -> Result<Vec<TraceSpan>, String> {
        self.query_all_limited(tenant, usize::MAX, None)
    }

    pub fn query_all_limited(
        &self,
        tenant: &TenantId,
        limit: usize,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Vec<TraceSpan>, String> {
        let Some(groups) = self.part.meta.tenant_row_groups(tenant) else {
            return Ok(Vec::new());
        };
        let selected: Vec<usize> = groups.map(|row_group| row_group as usize).collect();
        self.query_selected(tenant, selected, None, limit, cancellation)
    }

    /// The tenant's spans that overlap `[start_ns, end_ns]`.
    ///
    /// A span is in the window when any part of it is, not only when it
    /// started there: a request that began before the window and was still
    /// running inside it is exactly the one an operator is looking for. That
    /// also makes this the same predicate the row-group bounds already answer,
    /// since those record the earliest start and the latest end, so the filter
    /// and the pruning cannot disagree.
    ///
    /// Row groups outside the range are never read, and the surviving spans
    /// are filtered exactly, so the answer does not depend on where the flush
    /// happened to cut a row group.
    pub fn query_range_limited(
        &self,
        tenant: &TenantId,
        start_ns: i64,
        end_ns: i64,
        limit: usize,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Vec<TraceSpan>, String> {
        let selected = self
            .part
            .meta
            .tenant_row_groups_in_range(tenant, start_ns, end_ns);
        let mut spans = self.query_selected(tenant, selected, None, limit, cancellation)?;
        spans.retain(|span| span_overlaps(span, start_ns, end_ns));
        Ok(spans)
    }

    fn query_selected(
        &self,
        tenant: &TenantId,
        selected: Vec<usize>,
        trace_id: Option<&str>,
        limit: usize,
        cancellation: Option<&AtomicBool>,
    ) -> Result<Vec<TraceSpan>, String> {
        if selected.is_empty() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(self.part.data_path()).map_err(|error| error.to_string())?;
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|error| error.to_string())?
            .with_batch_size(1024)
            .with_row_groups(selected)
            .build()
            .map_err(|error| error.to_string())?;
        let mut spans: Vec<TraceSpan> = Vec::new();
        for batch in reader {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                return Err("trace query timed out".to_string());
            }
            let batch = batch.map_err(|error| error.to_string())?;
            let row_tenants = batch.column(0).as_string::<i32>();
            let trace_ids = batch.column(3).as_string::<i32>();
            let payloads = batch.column(5).as_string::<i32>();
            for index in 0..batch.num_rows() {
                if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                    return Err("trace query timed out".to_string());
                }
                // Row groups are tenant-aligned, so this is unreachable in a
                // well-formed part; it keeps a metadata bug from becoming a
                // cross-tenant read.
                if row_tenants.value(index) != tenant.as_str() {
                    return Err(format!(
                        "trace part {} contains rows outside tenant {tenant}",
                        self.part.meta.id
                    ));
                }
                if trace_id.is_some_and(|wanted| trace_ids.value(index) != wanted) {
                    continue;
                }
                if spans.len() >= limit {
                    return Err(format!("trace query exceeds the maximum of {limit} spans"));
                }
                let payload: TraceSpanFile =
                    serde_json::from_str(payloads.value(index)).map_err(|error| {
                        format!("invalid trace payload in {}: {error}", self.part.meta.id)
                    })?;
                spans.push(payload.into());
            }
        }
        spans.sort_by(|left, right| {
            left.start_time_ns
                .cmp(&right.start_time_ns)
                .then_with(|| left.span_id.cmp(&right.span_id))
        });
        Ok(spans)
    }
}

fn decode_trace_blooms(bytes: &[u8], expected_count: usize) -> Result<Vec<BloomFilter>, String> {
    if bytes.len() < 8 || &bytes[..4] != TRACE_BLOOM_MAGIC {
        return Err("trace bloom magic or header mismatch".to_string());
    }
    let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if count != expected_count {
        return Err(format!(
            "trace bloom row-group count mismatch: {count} != {expected_count}"
        ));
    }
    let mut offset: usize = 8;
    let mut blooms = Vec::with_capacity(count);
    for _ in 0..count {
        let end = offset
            .checked_add(4)
            .ok_or_else(|| "trace bloom length overflow".to_string())?;
        if end > bytes.len() {
            return Err("trace bloom length is truncated".to_string());
        }
        let length = u32::from_le_bytes(bytes[offset..end].try_into().unwrap()) as usize;
        offset = end;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| "trace bloom payload overflow".to_string())?;
        if end > bytes.len() {
            return Err("trace bloom payload is truncated".to_string());
        }
        blooms.push(BloomFilter::decode(&bytes[offset..end])?);
        offset = end;
    }
    if offset != bytes.len() {
        return Err("trace bloom has trailing bytes".to_string());
    }
    Ok(blooms)
}

fn metadata_crc32(meta: &TraceMetaFile) -> Result<u32, String> {
    let mut canonical = meta.clone();
    canonical.integrity.metadata_crc32 = 0;
    let bytes = serde_json::to_vec(&canonical).map_err(|error| error.to_string())?;
    Ok(crc32fast::hash(&bytes))
}

fn file_crc32(path: &Path) -> io::Result<u32> {
    let mut file = fs::File::open(path)?;
    let mut hasher = crc32fast::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize())
}

fn validate_file_crc(path: &Path, expected: u32, label: &str) -> Result<(), String> {
    let actual = file_crc32(path).map_err(|error| format!("failed to read {label}: {error}"))?;
    if actual != expected {
        return Err(format!("{label} checksum mismatch: {}", path.display()));
    }
    Ok(())
}

fn sync_file(path: &Path) -> io::Result<()> {
    fs::File::open(path)?.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_dir(parent)?;
    }
    Ok(())
}

fn sync_dir(path: &Path) -> io::Result<()> {
    fs::File::open(path)?.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::test_tenant;
    use crate::trace::normalize_request;
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};

    fn spans() -> Vec<TraceSpan> {
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![
                        Span {
                            trace_id: vec![1; 16],
                            span_id: vec![2; 8],
                            start_time_unix_nano: 100,
                            end_time_unix_nano: 200,
                            ..Default::default()
                        },
                        Span {
                            trace_id: vec![3; 16],
                            span_id: vec![4; 8],
                            start_time_unix_nano: 300,
                            end_time_unix_nano: 400,
                            ..Default::default()
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        normalize_request(&test_tenant(), request).unwrap()
    }

    /// Segments tile the object: each tenant's extent is non-empty and starts
    /// where the previous one ended, which is what makes the sum of them the
    /// tenant-attributable size of the part.
    #[test]
    fn tenant_segments_record_disjoint_byte_extents() {
        let root = crate::test_support::temp_dir("trace-bytes");
        let mut all = Vec::new();
        for tenant in ["acme", "globex"] {
            for span in spans() {
                all.push(TraceSpan {
                    tenant: TenantId::parse(tenant).unwrap(),
                    ..span
                });
            }
        }
        let parts = flush_trace_spans(&all, &root, 1).unwrap();
        assert_eq!(parts.len(), 1);
        let segments = &parts[0].meta.tenants;
        assert_eq!(segments.len(), 2);

        let mut previous_end = 0u64;
        for segment in segments {
            assert!(
                !segment.bytes.is_empty(),
                "tenant {} has no byte extent",
                segment.tenant
            );
            assert!(
                segment.bytes.start >= previous_end,
                "tenant extents overlap"
            );
            previous_end = segment.bytes.end;
        }
        let total: u64 = segments.iter().map(|segment| segment.bytes.len()).sum();
        let file_len = std::fs::metadata(
            root.join(&parts[0].meta.partition)
                .join(&parts[0].meta.id)
                .join(TRACE_DATA_FILE),
        )
        .unwrap()
        .len();
        assert!(
            total < file_len,
            "the footer belongs to no tenant, so the extents cannot cover the file"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn trace_part_round_trip_and_bloom_pruning() {
        let root = std::env::temp_dir().join(format!("signy-trace-part-{}", uuid::Uuid::new_v4()));
        let parts = flush_trace_spans(&spans(), &root, 1).unwrap();
        assert_eq!(parts.len(), 1);
        let reader = TracePartReader::open(parts.into_iter().next().unwrap()).unwrap();
        assert!(reader.may_match_trace_id(&test_tenant(), &"01".repeat(16)));
        let negative = ["09", "0a", "0b", "0c", "f0", "ff"]
            .into_iter()
            .map(|prefix| prefix.repeat(16))
            .find(|id| !reader.may_match_trace_id(&test_tenant(), id))
            .expect("test ID should be absent from the small bloom");
        assert!(
            reader
                .query_trace_id(&test_tenant(), &negative)
                .unwrap()
                .is_empty()
        );
        let result = reader
            .query_trace_id(&test_tenant(), &"01".repeat(16))
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].span_id, "02".repeat(8));
    }

    #[test]
    fn trace_catalog_can_open_without_parquet_body() {
        let root =
            std::env::temp_dir().join(format!("signy-trace-catalog-{}", uuid::Uuid::new_v4()));
        let parts = flush_trace_spans(&spans(), &root, 2).unwrap();
        let part = parts.into_iter().next().unwrap();
        fs::remove_file(part.data_path()).unwrap();
        let reader = TracePartReader::open_cached(load_trace_part(&part.dir).unwrap()).unwrap();
        assert!(reader.may_match_trace_id(&test_tenant(), &"01".repeat(16)));
        assert!(
            reader
                .query_trace_id(&test_tenant(), &"01".repeat(16))
                .is_err()
        );
    }

    fn tenant_spans(tenant: &str, trace_byte: u8, start_ns: u64) -> Vec<TraceSpan> {
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![trace_byte; 16],
                        span_id: vec![trace_byte; 8],
                        start_time_unix_nano: start_ns,
                        end_time_unix_nano: start_ns + 10,
                        name: format!("{tenant}-span"),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        normalize_request(&TenantId::parse(tenant).unwrap(), request).unwrap()
    }

    #[test]
    fn a_shared_trace_part_confines_every_read_to_the_querying_tenant() {
        let root =
            std::env::temp_dir().join(format!("signy-trace-tenant-{}", uuid::Uuid::new_v4()));
        let mut spans = tenant_spans("globex", 9, 300);
        spans.extend(tenant_spans("acme", 1, 100));
        let part = flush_trace_spans(&spans, &root, 1).unwrap().remove(0);

        let acme = TenantId::parse("acme").unwrap();
        let globex = TenantId::parse("globex").unwrap();
        let outsider = TenantId::parse("initech").unwrap();
        assert_eq!(part.meta.tenants.len(), 2);
        assert!(part.meta.tenant_row_groups(&outsider).is_none());

        let reader = TracePartReader::open(part).unwrap();
        let acme_trace = "01".repeat(16);
        let globex_trace = "09".repeat(16);

        assert_eq!(reader.query_all(&acme).unwrap().len(), 1);
        assert_eq!(reader.query_all(&globex).unwrap().len(), 1);
        assert!(reader.query_all(&outsider).unwrap().is_empty());

        // Knowing another tenant's trace ID must not be enough to read it.
        // (`may_match_trace_id` is a bloom hint and may say yes; the read
        // itself is what has to be empty.)
        assert!(!reader.may_match_trace_id(&outsider, &globex_trace));
        assert!(
            reader
                .query_trace_id(&acme, &globex_trace)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            reader.query_trace_id(&globex, &globex_trace).unwrap().len(),
            1
        );
        assert_eq!(reader.query_trace_id(&acme, &acme_trace).unwrap().len(), 1);
    }
}
