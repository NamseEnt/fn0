use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap};
use std::fs;

use std::io::{self, Read};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use arrow::array::{
    Array, ArrayRef, AsArray, Int64Array, RecordBatch, RecordBatchOptions, StringArray,
};
use arrow::datatypes::{DataType, Field, Int64Type, Schema};
use bytes::Bytes;
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::{
    ArrowReaderMetadata, ParquetRecordBatchReaderBuilder, RowSelection,
};
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::errors::Result as ParquetResult;
use parquet::file::properties::{EnabledStatistics, WriterProperties};
use parquet::file::reader::{ChunkReader, Length};
use parquet::schema::types::ColumnPath;
use serde::{Deserialize, Serialize};

use crate::bloom::BloomFilter;
use crate::logql::LineFilter;
use crate::memtable::{LogEntry, MemTableSnapshot, QueryResult, SharedLabels, StreamResult};
use crate::tenant::TenantId;

pub const DATA_FILE: &str = "data.parquet";
/// Trigram blooms and the stream index in one file.
///
/// They were two, and a part therefore cost two objects, two round trips on
/// restore and two checksum passes at startup. Startup at 10,099 parts is
/// dominated by exactly that validation (LOAD_RESULTS.md §8), and every object
/// per part is a Class A operation per flush (§9). Nothing ever read one of
/// these without the other, so being two files bought nothing.
pub const INDEX_FILE: &str = "index.bin";
/// Container header: magic plus the version of the layout inside it. Checked
/// before any section is read, so a file from another build is refused rather
/// than parsed as garbage.
pub const INDEX_MAGIC: &[u8; 5] = b"LTIX1";
pub const META_FILE: &str = "meta.json";
pub const MERGE_TOMBSTONE_FILE: &str = ".merge.tombstone";
const BLOOM_MAGIC: &[u8; 4] = b"BTF5";

/// Rows per exact-field sub-bloom window inside a row group.
///
/// Two couplings pin this value. It equals `part_writer_properties`'
/// `set_data_page_row_count_limit`, so a `RowSelection` built from admitted
/// windows skips whole Parquet pages through the offset index instead of
/// decoding into the middle of one. And `row_group_size` is capped at 65,536
/// (`config::validate`), so a group never holds more than 64 windows and a
/// window set fits a `u64` mask.
pub(crate) const BLOOM_WINDOW_ROWS: usize = 1024;

/// Per-window exact-field bloom false-positive rate. At 1% a unique token —
/// the rare-shape case — was admitted by ~1.5 windows it is not in across a
/// 150-window dataset, and each false window costs a ~0.5 ms narrow-pass
/// examination: the comparison bed measured `metadata_rare` cold reading
/// 2-3 windows for a one-row answer. At 0.1% the expected false admission
/// per query drops to ~0.15 window for ~1.5x the filter bytes (~14.4 vs
/// ~9.6 bits per token), which the sidecar budget absorbs. The filter
/// encodes its own bit count, so parts written at either rate read fine.
pub(crate) const EXACT_FIELD_WINDOW_FPP: f64 = 0.001;

/// Most exact-field tokens one window's bloom is sized for. The count is
/// attacker-shaped (a token per field per canonical variant per row), so past
/// this the window is stored saturated — admit-everything — instead of
/// letting an adversarial tenant size `index.bin` past `data.parquet`. At
/// [`EXACT_FIELD_WINDOW_FPP`]'s ~14.4 bits per token this caps a window's
/// filter near 118 KB; the bed corpus runs at ~7k tokens per window.
pub(crate) const MAX_EXACT_TOKENS_PER_WINDOW: usize = 65_536;

/// The window-slot length that encodes a saturated window in the `BTF5`
/// section. Distinct from 0, which means "no tokens — prune".
pub(crate) const SATURATED_WINDOW_SENTINEL: u32 = u32::MAX;

const EXACT_FIELD_TOKEN_MAGIC: &[u8; 4] = b"FEQ1";
const EXACT_FIELD_SCALAR_SCOPE: u8 = 0;

/// Parquet column holding the row's tenant. It is the leading sort key, so it
/// is also the leading column.
pub const TENANT_COLUMN: &str = "_tenant";

/// The most metadata keys one part stores as `_sm:` columns; the rest of a
/// row's pairs stay in the residual `structured_metadata` blob.
///
/// A cap because the schema must not scale with an adversarial tenant's key
/// churn — wide-JSON keys are unbounded where OTLP attribute names are not —
/// and 128 because every key the intended consumer sends fits in it several
/// times over. Which keys make the cut is by row count, not arrival order, so
/// churn cannot push `trace_id` out of a column; see
/// `format::select_metadata_columns`.
pub const MAX_METADATA_COLUMNS: usize = 128;

/// Which of a part's columns a scan decodes.
///
/// The tenant and timestamp columns are always read — they are the isolation
/// cross-check and the sort axis, and both are cheap dictionary runs. What a
/// query can spare is the line and the metadata: `sum(rate({app="x"}[5m]))`
/// needs neither, and decoding them anyway was most of what the metric path
/// paid per row.
///
/// `Named` is only sound when the caller can prove nothing downstream reads an
/// unlisted field. A parser stage cannot prove it — an extraction is shadowed
/// by same-named metadata, so dropping a metadata column would change which
/// value wins — and a template can name any field; those bail to `All`.
/// The log path always asks for `All`, because a log response returns every
/// pair it stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataProjection {
    All,
    Named(BTreeSet<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnSet {
    pub line: bool,
    pub metadata: MetadataProjection,
    /// Also decode the `_pf:` columns so the scan can hand the pipeline its
    /// `| json` extraction precomputed. Only worth asking for when the query
    /// has a `Json` stage; the columns are otherwise dead weight.
    pub parsed_fields: bool,
}

impl ColumnSet {
    pub fn all() -> Self {
        Self {
            line: true,
            metadata: MetadataProjection::All,
            parsed_fields: false,
        }
    }

    /// The log path's projection. Feeding `| json` from the `_pf:` columns is
    /// wired and correct, and **measured off**: decoding the parsed columns
    /// wide costs more than the per-survivor parse it saves — `json_field`
    /// went 21.5 → 25.7 ms and `json_field_rare` 3.05 → 3.77 in the bed with
    /// it on, because a scattered row selection still decodes most pages of
    /// every extra column. The machinery stays (the sinks take a precomputed
    /// map, the equality tests pin it) for a future where the survivor share
    /// of the decode is high enough to flip the sign; the projection is what
    /// stays off.
    pub fn for_log_query(query: &crate::logql::LogQuery) -> Self {
        let _ = query;
        Self::all()
    }

    /// The counting scan's projection: nothing is returned, so only what the
    /// filters read has to be decoded. A parser stage bails to the full set —
    /// an extraction is shadowed by same-named metadata, so dropping a
    /// metadata column would change which value wins.
    pub fn for_count_query(query: &crate::logql::LogQuery) -> Self {
        let mut line = !query.line_filters.is_empty();
        let mut named: BTreeSet<String> = BTreeSet::new();
        for matcher in &query.matchers {
            named.insert(matcher.name.clone());
        }
        for stage in &query.stages {
            match stage {
                crate::logql::PipelineStage::Json | crate::logql::PipelineStage::Logfmt => {
                    return Self::for_log_query(query);
                }
                crate::logql::PipelineStage::Line(_) => line = true,
                crate::logql::PipelineStage::Field(filter) => {
                    named.insert(filter.name.clone());
                }
            }
        }
        Self {
            line,
            metadata: MetadataProjection::Named(named),
            parsed_fields: false,
        }
    }
}

/// The time span a metadata lookup is allowed to see.
///
/// Replaces the bare retention floor that these lookups used to take. Grafana
/// sends `start`/`end` on every label and series call, and answering from the
/// whole history both returns labels that do not exist in the range and reads
/// every part to do it. The retention floor folds into `start_ns`, so one
/// bound expresses both "what the client asked for" and "what the tenant is
/// still entitled to".
///
/// Deliberately not a `QueryTimeRange`, and its `end` is deliberately still
/// inclusive. These endpoints answer "did this label exist in the window", not
/// "which rows are in the window", and no measurement establishes what Loki's
/// own bound is here. Closing an existence question one nanosecond wide is the
/// conservative error; a row set is what `query_range` promises exactly.
#[derive(Clone, Copy, Debug)]
pub struct MetadataWindow {
    pub start_ns: i64,
    pub end_ns: i64,
}

impl MetadataWindow {
    pub fn unbounded() -> Self {
        Self {
            start_ns: i64::MIN,
            end_ns: i64::MAX,
        }
    }

    pub fn clamped_to(self, retention_floor_ns: Option<i64>) -> Self {
        match retention_floor_ns {
            Some(floor_ns) => Self {
                start_ns: self.start_ns.max(floor_ns),
                ..self
            },
            None => self,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start_ns > self.end_ns
    }

    pub fn contains(&self, timestamp_ns: i64) -> bool {
        timestamp_ns >= self.start_ns && timestamp_ns <= self.end_ns
    }

    pub fn overlaps(&self, min_ts_ns: i64, max_ts_ns: i64) -> bool {
        max_ts_ns >= self.start_ns && min_ts_ns <= self.end_ns
    }
}

/// The time window a log scan is allowed to return rows from.
///
/// Every level of the read path — the memtable scan, the part-level and
/// row-group-level pruning, and the row-level reject inside a part — used to
/// spell its own comparison out, and four independent comparisons that have to
/// agree is how `end` came to be inclusive on the log path while Loki's
/// `query_range` is `[start, end)`. So the boundary lives here and nothing below
/// the query layer decides it: a site that prunes asks `overlaps`, a site that
/// rejects a row asks `contains`, and the two cannot drift apart. A pruning
/// bound tighter than the row bound silently drops rows, which is the failure
/// mode this type exists to make unrepresentable.
/// The fields are private so no caller can spell the comparison out again from
/// the bounds: the only way to ask whether a timestamp is in the window is to
/// ask this type.
#[derive(Clone, Copy, Debug)]
pub struct QueryTimeRange {
    start_ns: i64,
    end_ns: i64,
    include_end: bool,
}

impl QueryTimeRange {
    /// The window's identity for cache keys — an opaque tuple, not bounds to
    /// compare against; the privacy rationale above still holds.
    pub(crate) fn cache_identity(self) -> (i64, i64, bool) {
        (self.start_ns, self.end_ns, self.include_end)
    }

    /// `[start_ns, end_ns)` — what a Loki log query asks for. Both `query` and
    /// `query_range` include `start` and exclude `end`.
    pub fn half_open(start_ns: i64, end_ns: i64) -> Self {
        Self {
            start_ns,
            end_ns,
            include_end: false,
        }
    }

    /// `[start_ns, end_ns]`.
    ///
    /// Not a log-query window. This is for the scans whose `end` is not a
    /// client-supplied exclusive bound: a metric scan's `end` is its last
    /// evaluation point, and the range evaluator's own window closes on it
    /// (`timestamp_ns <= evaluation_ns`), so a half-open scan would starve the
    /// final point of the row that lands exactly on it.
    pub fn closed(start_ns: i64, end_ns: i64) -> Self {
        Self {
            start_ns,
            end_ns,
            include_end: true,
        }
    }

    /// Every row there is, for the callers that mean "all of them".
    pub fn unbounded() -> Self {
        Self::closed(i64::MIN, i64::MAX)
    }

    /// Whether a row at `timestamp_ns` belongs to the answer.
    pub fn contains(&self, timestamp_ns: i64) -> bool {
        timestamp_ns >= self.start_ns
            && (timestamp_ns < self.end_ns || (self.include_end && timestamp_ns == self.end_ns))
    }

    /// Whether a span `[min_ts_ns, max_ts_ns]` can hold a row this range
    /// contains. Exactly `contains` lifted to a span, so pruning never rejects
    /// a span holding a row the row-level test would have kept.
    pub fn overlaps(&self, min_ts_ns: i64, max_ts_ns: i64) -> bool {
        max_ts_ns >= self.start_ns
            && (min_ts_ns < self.end_ns || (self.include_end && min_ts_ns == self.end_ns))
    }
}

/// A positive exact equality over an entry's structured or parser-visible
/// scalar fields.
///
/// This type deliberately lives below LogQL. Callers may compile eligible
/// pipeline predicates into it without coupling the immutable part format to
/// a particular query AST. Missing field blooms (notably BTF1 parts) always
/// fall back to scanning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactFieldPredicate {
    pub name: String,
    pub value: String,
    /// A preceding parser may also produce this field from the log line. BTF2
    /// indexes parser-visible scalars as well as structured metadata; this bit
    /// remains explicit so older/alternate indexes can conservatively scan.
    pub may_be_extracted: bool,
    /// Whether `value` is a canonical numeric/duration representation. Older
    /// BTF2 indexes contain only raw values and must not prune such queries.
    pub canonical: bool,
    /// True when the only parser that can produce this field is `| json` with
    /// the stored line intact — no logfmt stage, no `line_format` rewriting
    /// the line first. Exactly then the `_pf:` column *is* what the stage
    /// would extract, and the two-pass scan may answer the predicate from it.
    pub json_only_extraction: bool,
}

impl ExactFieldPredicate {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            may_be_extracted: false,
            canonical: false,
            json_only_extraction: false,
        }
    }

    pub fn new_with_extraction(
        name: impl Into<String>,
        value: impl Into<String>,
        may_be_extracted: bool,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            may_be_extracted,
            canonical: false,
            json_only_extraction: false,
        }
    }

    pub fn new_canonical_with_extraction(
        name: impl Into<String>,
        value: impl Into<String>,
        may_be_extracted: bool,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            may_be_extracted,
            canonical: true,
            json_only_extraction: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ExactFieldPruning<'a> {
    pub line_filters: &'a [LineFilter],
    pub exact_fields: &'a [ExactFieldPredicate],
}

impl<'a> ExactFieldPruning<'a> {
    pub fn new(line_filters: &'a [LineFilter], exact_fields: &'a [ExactFieldPredicate]) -> Self {
        Self {
            line_filters,
            exact_fields,
        }
    }
}

/// A half-open `[start, end)` extent within `data.parquet`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }
}

/// One tenant's contiguous slice of a shared part.
///
/// Rows are sorted by `(tenant, timestamp_ns)` and row groups never straddle
/// a tenant boundary, so a tenant occupies a whole run of row groups. A query
/// is confined to its own run, which is what makes a shared object as
/// fail-closed as a per-tenant file: rows outside the run are not addressable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantSegment {
    pub tenant: TenantId,
    /// Inclusive first row group.
    pub row_group_start: u32,
    /// Exclusive last row group.
    pub row_group_end: u32,
    pub row_count: u64,
    pub min_ts_ns: i64,
    pub max_ts_ns: i64,
    /// Where this tenant's row groups sit in `data.parquet`.
    ///
    /// Recorded so a reader can fetch one tenant's bytes instead of the whole
    /// shared object. A part is up to `merge_target_part_rows` across every
    /// tenant in it, so serving one tenant from the whole object is an
    /// amplification proportional to tenant breadth.
    pub bytes: ByteRange,
    /// CRC32 of exactly that range.
    ///
    /// The part-wide `data_crc32` cannot verify a slice, and a slice fetched on
    /// its own is the only thing a range read has. Recording it per segment
    /// keeps a partial read as checkable as a whole one — which is the property
    /// every other file here already has.
    pub crc32: u32,
}

#[derive(Clone, Debug)]
pub struct PartMeta {
    pub id: String,
    pub partition: String,
    pub min_ts_ns: i64,
    pub max_ts_ns: i64,
    pub row_count: u64,
    pub row_group_count: u32,
    pub row_group_min_ts: Vec<i64>,
    pub row_group_max_ts: Vec<i64>,
    pub row_group_rows: Vec<u32>,
    /// Whether each group is one non-decreasing timestamp run. Only such a
    /// group may be read partially: the scan's early exits are sound exactly
    /// when a row's position bounds the rows after it.
    pub row_group_ts_monotonic: Vec<bool>,
    /// Sorted by tenant, non-overlapping, and covering every row group.
    pub tenants: Vec<TenantSegment>,
    /// Memory a full read of this part materializes, recorded at write time.
    /// Merge compares this against its budgets instead of the compressed file
    /// size, which is smaller by whatever zstd achieved on the data.
    pub materialized_bytes: u64,
    /// The metadata keys stored as `_sm:` columns, sorted, with the number of
    /// rows carrying each. See `MAX_METADATA_COLUMNS`.
    pub metadata_columns: Vec<(String, u64)>,
    /// The `| json`-extracted fields stored as `_pf:` columns, same shape and
    /// cap. The line itself stays; these are the schema-on-write copy of what
    /// the query-time parser would produce, exact by construction because the
    /// writer runs the same extraction.
    pub parsed_columns: Vec<(String, u64)>,
    /// Size of `meta.json` on disk, recorded when it was read.
    ///
    /// Startup parses this file for every part before serving anything, and
    /// `tenants` makes its size scale with tenant breadth, so this is the
    /// concrete cost of the per-tenant index rather than an estimate of it.
    pub meta_bytes: u64,
    integrity: PartIntegrity,
}

impl PartMeta {
    pub fn tenant_segment(&self, tenant: &TenantId) -> Option<&TenantSegment> {
        self.tenants
            .binary_search_by(|segment| segment.tenant.cmp(tenant))
            .ok()
            .map(|index| &self.tenants[index])
    }
}

#[derive(Clone, Debug)]
pub struct Part {
    pub dir: PathBuf,
    pub meta: PartMeta,
}

impl Part {
    pub fn data_path(&self) -> PathBuf {
        self.dir.join(DATA_FILE)
    }
    pub fn index_path(&self) -> PathBuf {
        self.dir.join(INDEX_FILE)
    }
    pub fn meta_path(&self) -> PathBuf {
        self.dir.join(META_FILE)
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub tenant: TenantId,
    pub timestamp_ns: i64,
    pub line: String,
    pub structured_metadata: Vec<(String, String)>,
}

impl Row {
    /// What this row costs to hold in memory once read back.
    ///
    /// Merge sizes both its group selection and its read budget with this, so
    /// the two are in the same unit by construction. Comparing a compressed
    /// on-disk size against a materialized budget is what made large groups
    /// select successfully and then always fail to read.
    pub fn materialized_bytes(&self) -> u64 {
        let metadata_bytes: usize = self
            .structured_metadata
            .iter()
            .map(|(name, value)| name.len() + value.len())
            .sum();
        (self.line.len() + metadata_bytes + std::mem::size_of::<Row>()) as u64
    }

    pub fn from_entry(tenant: &TenantId, e: &LogEntry) -> Self {
        Self {
            tenant: tenant.clone(),
            timestamp_ns: e.timestamp_ns,
            line: e.line.clone(),
            structured_metadata: e.structured_metadata.clone(),
        }
    }

    /// The part sort key. `(tenant, timestamp)` is what the layout needs —
    /// tenant-aligned row groups and time order within a tenant, so a part is
    /// one timestamp-ordered run per tenant and the reader's early termination
    /// and row-group time pruning work directly. Everything after the
    /// timestamp is a tie-break, which the ordering never cared about, and
    /// which makes two copies of one entry sort adjacent so a duplicate can be
    /// recognised.
    fn sort_key(&self) -> (&str, i64, &str, &[(String, String)]) {
        (
            self.tenant.as_str(),
            self.timestamp_ns,
            self.line.as_str(),
            &self.structured_metadata,
        )
    }
}

include!("sink.rs");
include!("group_cache.rs");
include!("bloom_cache.rs");
include!("format.rs");
include!("indexes.rs");
include!("metadata.rs");
include!("tombstone.rs");
include!("reader.rs");
include!("row_stream.rs");
include!("stream_writer.rs");

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
