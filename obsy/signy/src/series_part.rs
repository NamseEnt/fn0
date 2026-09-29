//! The metric part format (M14, issue #8): one flushed batch of series
//! samples as four artifacts, modeled file-for-file on `trace_part.rs` so the
//! object-storage lifecycle generalizes without redesign.
//!
//! * **`data.bin`** — magic `LMS1`, then one Gorilla chunk per series,
//!   concatenated in catalog order. A series' samples are merged (chunks +
//!   out-of-order spill), time-sorted, and re-encoded into a single chunk at
//!   flush, so a part's chunk is always sorted and self-describing.
//! * **`index.bin`** — magic `LMI1`: the tenant segment table and the series
//!   catalog (canonical labels, chunk byte range, sample count, min/max
//!   timestamp). Everything selection needs without touching `data.bin`.
//! * **`series.bloom`** — one bloom over the part's `key\0value` label pair
//!   tokens, for pruning a part on an equality selector without reading
//!   `index.bin`.
//! * **`meta.json`** — identity, time bounds, per-tenant segments with byte
//!   extents (the storage quota's census), and integrity checksums.
//!
//! Parts are tenant-major: each tenant owns a contiguous ordinal range of the
//! catalog, recorded in its segment, so a read can never cross tenants.

use memmap2::Mmap;
use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::bloom::BloomFilter;
use crate::gorilla;
use crate::part::partition_of;
use crate::series::{SeriesLabels, SeriesSnapshot};
use crate::tenant::TenantId;

pub const SERIES_DATA_FILE: &str = "data.bin";
pub const SERIES_INDEX_FILE: &str = "index.bin";
pub const SERIES_LABELS_FILE: &str = "labels.bin";
pub const SERIES_BLOOM_FILE: &str = "series.bloom";
pub const SERIES_META_FILE: &str = "meta.json";

const SERIES_DATA_MAGIC: &[u8; 4] = b"LMS1";
const SERIES_INDEX_MAGIC: &[u8; 4] = b"LMI1";
const SERIES_LABELS_MAGIC: &[u8; 4] = b"LML1";
const SERIES_BLOOM_MAGIC: &[u8; 4] = b"LMB1";

/// `index.bin` is a fixed-stride array so a reader can address the nth catalog
/// row without decoding the ones before it, and so selection walks only the
/// row it needs rather than stepping over inline label payloads. The labels
/// live in their own file, touched only for a row that survived the time and
/// name filters.
///
/// Header: magic, row count, and the base timestamp the row's millisecond
/// deltas are measured from. The base is the partition's midnight, which is
/// known before the first row is written — a part never spans two partitions,
/// so every sample in it sits inside one day and its offset fits `u32`.
const SERIES_INDEX_HEADER_BYTES: usize = 16;
const SERIES_INDEX_ENTRY_BYTES: usize = 32;

/// What a row's chunk holds. A scalar series is a Gorilla stream of
/// `(timestamp, value)`; a histogram series is one instrument's observations,
/// bucket vectors and all, in the form [`crate::histogram_chunk`] writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeriesRowKind {
    Scalar,
    Histogram,
}

impl SeriesRowKind {
    fn tag(self) -> u8 {
        match self {
            Self::Scalar => 0,
            Self::Histogram => 1,
        }
    }

    fn from_tag(tag: u8) -> Result<Self, String> {
        match tag {
            0 => Ok(Self::Scalar),
            1 => Ok(Self::Histogram),
            other => Err(format!("metric catalog row has unknown kind {other}")),
        }
    }
}

/// Midnight UTC of a partition key, in nanoseconds — the base every catalog
/// row's timestamp delta is relative to.
fn partition_base_ns(partition: &str) -> Result<i64, String> {
    chrono::NaiveDate::parse_from_str(partition, "%Y-%m-%d")
        .map_err(|error| format!("metric partition {partition} is not a date: {error}"))?
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| naive.and_utc().timestamp_nanos_opt())
        .ok_or_else(|| format!("metric partition {partition} is outside the nanosecond range"))
}

/// Milliseconds from `base`, rounded **outward** so a stored range is never
/// narrower than the samples it describes. Metadata that cannot answer must
/// not be able to hide data, which is the rule the bloom follows too.
fn delta_floor_ms(base: i64, ts_ns: i64) -> u32 {
    let delta = ts_ns.saturating_sub(base).max(0);
    (delta / 1_000_000).min(u32::MAX as i64) as u32
}

fn delta_ceil_ms(base: i64, ts_ns: i64) -> u32 {
    let delta = ts_ns.saturating_sub(base).max(0);
    (delta.saturating_add(999_999) / 1_000_000).min(u32::MAX as i64) as u32
}

/// The false-positive rate of the label-pair bloom. The trace bloom's 1% is
/// kept: a false positive costs one `index.bin` read, not a scan.
const BLOOM_FPP: f64 = 0.01;

/// One tenant's contiguous run of catalog ordinals in a shared metric part.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeriesTenantSegment {
    pub tenant: TenantId,
    pub series_start: u32,
    pub series_end: u32,
    pub sample_count: u64,
    /// Where this tenant's chunks sit in `data.bin` — what the storage quota
    /// charges, from the format rather than the evictable local file.
    pub bytes: crate::part::ByteRange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SeriesPartIntegrity {
    data_crc32: u32,
    index_crc32: u32,
    labels_crc32: u32,
    bloom_crc32: u32,
    metadata_crc32: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct SeriesMetaFile {
    id: String,
    partition: String,
    min_ts_ns: i64,
    max_ts_ns: i64,
    series_count: u32,
    sample_count: u64,
    tenants: Vec<SeriesTenantSegment>,
    integrity: SeriesPartIntegrity,
}

#[derive(Clone, Debug)]
pub struct SeriesPartMeta {
    pub id: String,
    pub partition: String,
    pub min_ts_ns: i64,
    pub max_ts_ns: i64,
    pub series_count: u32,
    pub sample_count: u64,
    pub tenants: Vec<SeriesTenantSegment>,
    // Mirrors the trace part: carried so a future re-serialization keeps the
    // checksums beside what they cover.
    #[allow(dead_code)]
    integrity: SeriesPartIntegrity,
}

impl SeriesPartMeta {
    pub fn tenant_segment(&self, tenant: &TenantId) -> Option<&SeriesTenantSegment> {
        self.tenants
            .binary_search_by(|segment| segment.tenant.cmp(tenant))
            .ok()
            .map(|index| &self.tenants[index])
    }

    pub fn overlaps_range(&self, start_ns: i64, end_ns: i64) -> bool {
        self.max_ts_ns >= start_ns && self.min_ts_ns <= end_ns
    }
}

#[derive(Clone, Debug)]
pub struct SeriesPart {
    pub dir: PathBuf,
    pub meta: SeriesPartMeta,
}

impl SeriesPart {
    pub fn data_path(&self) -> PathBuf {
        self.dir.join(SERIES_DATA_FILE)
    }

    pub fn index_path(&self) -> PathBuf {
        self.dir.join(SERIES_INDEX_FILE)
    }

    pub fn labels_path(&self) -> PathBuf {
        self.dir.join(SERIES_LABELS_FILE)
    }

    pub fn bloom_path(&self) -> PathBuf {
        self.dir.join(SERIES_BLOOM_FILE)
    }

    pub fn meta_path(&self) -> PathBuf {
        self.dir.join(SERIES_META_FILE)
    }
}

/// One catalog row: a series and where its chunk lives.
#[derive(Clone, Debug)]
pub struct CatalogRow<'a> {
    pub kind: SeriesRowKind,
    /// Canonical label bytes, pointing into the part's mapped label file. A
    /// row becomes an owned [`SeriesLabels`] only once a selector has matched
    /// it; the walk that decides costs no allocation at all.
    pub labels: &'a [u8],
    pub chunk: ChunkRef,
    /// Inclusive sample range as milliseconds from the part's base, rounded
    /// outward. Absolute nanoseconds cost sixteen bytes a row and buy nothing
    /// a part-relative delta does not: a part never leaves its partition, so
    /// one day is the widest range either end can hold.
    min_delta_ms: u32,
    max_delta_ms: u32,
}

/// Where one series' chunk lives in a part's body — the half of a row a
/// caller keeps after the row itself has gone out of scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkRef {
    pub offset: u64,
    pub length: u32,
}

/// A query window expressed in one part's delta space, so a catalog walk
/// compares two `u32`s per row instead of rebuilding absolute timestamps.
#[derive(Clone, Copy, Debug)]
pub struct CatalogWindow {
    start_delta_ms: u32,
    end_delta_ms: u32,
}

impl CatalogRow<'_> {
    pub fn overlaps(&self, window: CatalogWindow) -> bool {
        self.max_delta_ms >= window.start_delta_ms && self.min_delta_ms <= window.end_delta_ms
    }
}

/// A contiguous run of catalog rows over one part's mapped files — usually
/// one tenant's segment. Rows are decoded on access, so holding this holds
/// nothing but two slices.
#[derive(Clone, Copy)]
pub struct CatalogRows<'a> {
    index: &'a [u8],
    labels: &'a [u8],
    start: usize,
    end: usize,
}

impl<'a> CatalogRows<'a> {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// The row at `ordinal` within this run. Every row's label range was
    /// bounds-checked when the part opened, so the slicing here cannot fail
    /// and the hot path does not branch on it.
    pub fn get(&self, ordinal: usize) -> Option<CatalogRow<'a>> {
        let row = self
            .start
            .checked_add(ordinal)
            .filter(|row| *row < self.end)?;
        let at = SERIES_INDEX_HEADER_BYTES + row * SERIES_INDEX_ENTRY_BYTES;
        let record = self.index.get(at..at + SERIES_INDEX_ENTRY_BYTES)?;
        let labels_at = u32::from_le_bytes(record[0..4].try_into().ok()?) as usize;
        let labels_len = u32::from_le_bytes(record[4..8].try_into().ok()?) as usize;
        Some(CatalogRow {
            kind: SeriesRowKind::from_tag(record[28]).ok()?,
            labels: self.labels.get(labels_at..labels_at + labels_len)?,
            chunk: ChunkRef {
                offset: u64::from_le_bytes(record[8..16].try_into().ok()?),
                length: u32::from_le_bytes(record[16..20].try_into().ok()?),
            },
            min_delta_ms: u32::from_le_bytes(record[20..24].try_into().ok()?),
            max_delta_ms: u32::from_le_bytes(record[24..28].try_into().ok()?),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = CatalogRow<'a>> + use<'a> {
        let rows = *self;
        (0..rows.len()).filter_map(move |ordinal| rows.get(ordinal))
    }
}

/// The bloom's token for one label pair. NUL-separated because a label value
/// may contain `=`; keys cannot contain NUL (they pass through attribute-key
/// normalization or are this crate's own reserved names).
pub fn pair_token(key: &str, value: &str) -> Vec<u8> {
    let mut token = Vec::with_capacity(key.len() + 1 + value.len());
    token.extend_from_slice(key.as_bytes());
    token.push(0);
    token.extend_from_slice(value.as_bytes());
    token
}

/// Writes the flushing snapshot without taking ownership of it — the buffer
/// stays shared with the memtable until the flush commits, exactly like the
/// trace writer.
#[allow(dead_code)]
pub fn flush_series_snapshot(
    snapshot: &SeriesSnapshot,
    metrics_root: &Path,
) -> io::Result<Vec<SeriesPart>> {
    if snapshot.is_empty() {
        return Ok(Vec::new());
    }
    // Charged like the log flush: the partition map, the re-encoded chunks and
    // the catalog this builds are the writer's, not its caller's — and one of
    // its callers is compaction.
    let _arena = crate::memprof::enter(crate::memprof::Arena::Flush);
    fs::create_dir_all(metrics_root.join(".tmp"))?;

    // Merge the snapshot's per-series entries (an aborted flush leaves more
    // than one per series), sort, and split by partition. The map is ordered
    // by (partition, tenant, labels), which is exactly the catalog order.
    let mut partitions: BTreeMap<String, PartitionSeries> = BTreeMap::new();
    for (tenant, list) in &snapshot.tenants {
        for series in list {
            if series.is_histogram() {
                for (ts, point) in &series.points {
                    match partitions
                        .entry(partition_of(*ts))
                        .or_default()
                        .entry(tenant.clone())
                        .or_default()
                        .entry(series.labels.clone())
                        .or_insert_with(|| SeriesPayload::Histogram(Vec::new()))
                    {
                        SeriesPayload::Histogram(points) => points.push((*ts, point.clone())),
                        SeriesPayload::Scalar(_) => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "one series cannot be scalar and histogram in one part",
                            ));
                        }
                    }
                }
                continue;
            }
            let samples = series.sorted_samples().map_err(io::Error::other)?;
            for (ts, value) in samples {
                match partitions
                    .entry(partition_of(ts))
                    .or_default()
                    .entry(tenant.clone())
                    .or_default()
                    .entry(series.labels.clone())
                    .or_insert_with(|| SeriesPayload::Scalar(Vec::new()))
                {
                    SeriesPayload::Scalar(values) => values.push((ts, value)),
                    SeriesPayload::Histogram(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "one series cannot be scalar and histogram in one part",
                        ));
                    }
                }
            }
        }
    }

    let mut parts = Vec::new();
    let mut committed_dirs = Vec::new();
    for (partition, tenants) in partitions {
        let id = format!("{}-{}", partition.replace('-', ""), uuid::Uuid::new_v4());
        let tmp_dir = metrics_root.join(".tmp").join(&id);
        let final_dir = metrics_root.join(&partition).join(&id);
        let result = (|| -> io::Result<SeriesPart> {
            if tmp_dir.exists() {
                fs::remove_dir_all(&tmp_dir)?;
            }
            fs::create_dir_all(&tmp_dir)?;
            write_series_part_files(&tmp_dir, &id, &partition, &tenants)?;
            if let Some(parent) = final_dir.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(&tmp_dir, &final_dir)?;
            committed_dirs.push(final_dir.clone());
            sync_dir(final_dir.parent().unwrap_or(metrics_root))?;
            sync_dir(metrics_root)?;
            load_series_part(&final_dir).map_err(io::Error::other)
        })();

        match result {
            Ok(part) => parts.push(part),
            Err(error) => {
                let _ = fs::remove_dir_all(&tmp_dir);
                let cleanup = crate::part::remove_part_dirs(&committed_dirs);
                return Err(match cleanup {
                    Ok(()) => error,
                    Err(cleanup_error) => io::Error::other(format!(
                        "metric part flush failed: {error}; rollback failed: {cleanup_error}"
                    )),
                });
            }
        }
    }
    Ok(parts)
}

/// Re-flush a set of parts without materialising the complete merge in a
/// `tenant -> labels -> samples` map.
///
/// Parts are already ordered by tenant and labels. A small k-way heap merges
/// those catalog streams, and a second heap merges the current series' sample
/// streams. Consequently the live sample state is one Gorilla chunk and one
/// sample per input part, independent of the number of series in the group.
/// The output format is the same as [`flush_series_snapshot`].
#[allow(dead_code)]
pub fn compact_series_parts(
    readers: &[std::sync::Arc<SeriesPartReader>],
    metrics_root: &Path,
) -> io::Result<Vec<SeriesPart>> {
    if readers.is_empty() {
        return Ok(Vec::new());
    }
    let partition = readers[0].part().meta.partition.clone();
    let id = format!("{}-{}", partition.replace('-', ""), uuid::Uuid::new_v4());
    compact_series_parts_with_id(readers, metrics_root, &id)
}

/// Re-flush a compaction group using an id that was recorded durably before
/// any replacement files were created. That ordering makes a restart able to
/// distinguish an unfinished output from a committed one.
pub fn compact_series_parts_with_id(
    readers: &[std::sync::Arc<SeriesPartReader>],
    metrics_root: &Path,
    id: &str,
) -> io::Result<Vec<SeriesPart>> {
    if readers.is_empty() {
        return Ok(Vec::new());
    }
    let partition = readers[0].part().meta.partition.clone();
    if readers
        .iter()
        .any(|reader| reader.part().meta.partition != partition)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "metric compaction group spans partitions",
        ));
    }

    let expected_prefix = format!("{}-", partition.replace('-', ""));
    if !id.starts_with(&expected_prefix) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "metric compaction output id does not match its partition",
        ));
    }
    fs::create_dir_all(metrics_root.join(".tmp"))?;
    let tmp_dir = metrics_root.join(".tmp").join(id);
    let final_dir = metrics_root.join(&partition).join(id);
    let result = (|| -> io::Result<SeriesPart> {
        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir)?;
        }
        fs::create_dir_all(&tmp_dir)?;
        write_streaming_series_part_files(&tmp_dir, id, &partition, readers)?;
        if let Some(parent) = final_dir.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&tmp_dir, &final_dir)?;
        sync_dir(final_dir.parent().unwrap_or(metrics_root))?;
        sync_dir(metrics_root)?;
        load_series_part(&final_dir).map_err(io::Error::other)
    })();

    match result {
        Ok(part) => Ok(vec![part]),
        Err(error) => {
            let _ = fs::remove_dir_all(&tmp_dir);
            let _ = crate::part::remove_part_dirs(std::slice::from_ref(&final_dir));
            Err(error)
        }
    }
}

/// A cursor over one part's tenant-major catalog. It stores only its current
/// ordinal; the catalog itself remains owned by the part reader.
struct CatalogCursor {
    reader: std::sync::Arc<SeriesPartReader>,
    tenant: usize,
    entry: usize,
}

impl CatalogCursor {
    fn new(reader: std::sync::Arc<SeriesPartReader>) -> Self {
        Self {
            reader,
            tenant: 0,
            entry: 0,
        }
    }

    fn current(&self, stream: usize) -> Option<CatalogHead> {
        let segment = self.reader.part().meta.tenants.get(self.tenant)?;
        let row = self
            .reader
            .tenant_catalog(&segment.tenant)
            .get(self.entry)?;
        Some(CatalogHead {
            stream,
            reader: self.reader.clone(),
            tenant: segment.tenant.clone(),
            // The merge orders by identity and writes it out again, so this
            // head is one of the few places a catalog row still has to become
            // an owned label — at most one per input stream at a time.
            labels: SeriesLabels::from_canonical(row.labels.to_vec()),
            chunk: row.chunk,
            kind: row.kind,
        })
    }

    fn advance(&mut self) {
        let Some(segment) = self.reader.part().meta.tenants.get(self.tenant) else {
            return;
        };
        self.entry += 1;
        if self.entry >= (segment.series_end - segment.series_start) as usize {
            self.tenant += 1;
            self.entry = 0;
        }
    }
}

/// One heap head. The input stream index is the final tie-breaker so equal
/// labels retain the old compactor's input order, which also preserves stable
/// ordering for duplicate timestamps.
struct CatalogHead {
    stream: usize,
    reader: std::sync::Arc<SeriesPartReader>,
    tenant: TenantId,
    labels: SeriesLabels,
    chunk: ChunkRef,
    kind: SeriesRowKind,
}

impl PartialEq for CatalogHead {
    fn eq(&self, other: &Self) -> bool {
        self.tenant == other.tenant && self.labels == other.labels && self.stream == other.stream
    }
}

impl Eq for CatalogHead {}

impl PartialOrd for CatalogHead {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CatalogHead {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .tenant
            .cmp(&self.tenant)
            .then_with(|| other.labels.cmp(&self.labels))
            .then_with(|| other.stream.cmp(&self.stream))
    }
}

struct SampleHead {
    stream: usize,
    decoder: gorilla::OwnedDecoder,
    sample: (i64, f64),
}

impl PartialEq for SampleHead {
    fn eq(&self, other: &Self) -> bool {
        self.sample.0 == other.sample.0 && self.stream == other.stream
    }
}

impl Eq for SampleHead {}

impl PartialOrd for SampleHead {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SampleHead {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .sample
            .0
            .cmp(&self.sample.0)
            .then_with(|| other.stream.cmp(&self.stream))
    }
}

/// One identity's histogram observations across the inputs, merged by
/// timestamp. Unlike the scalar merge this materializes: a histogram chunk
/// decodes whole, and one series' points are the unit either way.
fn merged_series_points(
    heads: &[CatalogHead],
) -> io::Result<Vec<(i64, crate::series::HistogramPoint)>> {
    let mut merged: Vec<(usize, i64, crate::series::HistogramPoint)> = Vec::new();
    for head in heads {
        let points = head
            .reader
            .read_histogram_points(head.chunk)
            .map_err(io::Error::other)?;
        merged.extend(
            points
                .into_iter()
                .map(|(ts, point)| (head.stream, ts, point)),
        );
    }
    // The input stream index breaks a timestamp tie, which is the same rule
    // the scalar merge follows so a duplicate keeps its input order.
    merged.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
    Ok(merged
        .into_iter()
        .map(|(_, ts, point)| (ts, point))
        .collect())
}

struct MergedSeriesSamples {
    heap: std::collections::BinaryHeap<SampleHead>,
}

impl MergedSeriesSamples {
    fn new(heads: &[CatalogHead]) -> io::Result<Self> {
        let mut heap = std::collections::BinaryHeap::with_capacity(heads.len());
        for head in heads {
            let mut decoder = head
                .reader
                .read_series_decoder(head.chunk)
                .map_err(io::Error::other)?;
            let Some(sample) = decoder.next().transpose().map_err(io::Error::other)? else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "metric catalog entry has no samples",
                ));
            };
            heap.push(SampleHead {
                stream: head.stream,
                decoder,
                sample,
            });
        }
        Ok(Self { heap })
    }

    fn next_sample(&mut self) -> io::Result<Option<(i64, f64)>> {
        let Some(mut head) = self.heap.pop() else {
            return Ok(None);
        };
        let sample = head.sample;
        if let Some(next) = head.decoder.next() {
            head.sample = next.map_err(io::Error::other)?;
            self.heap.push(head);
        }
        Ok(Some(sample))
    }
}

struct WrittenRow<'a> {
    kind: SeriesRowKind,
    chunk: &'a [u8],
    count: u64,
    min_ts: i64,
    max_ts: i64,
}

struct StreamingSeriesPartWriter {
    data: BufWriter<fs::File>,
    index: BufWriter<fs::File>,
    labels: BufWriter<fs::File>,
    data_path: PathBuf,
    index_path: PathBuf,
    labels_path: PathBuf,
    bloom_path: PathBuf,
    bloom: BloomFilter,
    data_offset: u64,
    labels_offset: u32,
    base_ts_ns: i64,
    series_count: u32,
    sample_count: u64,
    part_min: i64,
    part_max: i64,
    segments: Vec<SeriesTenantSegment>,
    current_tenant: Option<TenantId>,
    current_series_start: u32,
    current_segment_start: u64,
    current_segment_samples: u64,
}

impl StreamingSeriesPartWriter {
    fn new(dir: &Path, partition: &str, bloom_items: usize) -> io::Result<Self> {
        let base_ts_ns = partition_base_ns(partition).map_err(io::Error::other)?;
        let data_path = dir.join(SERIES_DATA_FILE);
        let index_path = dir.join(SERIES_INDEX_FILE);
        let labels_path = dir.join(SERIES_LABELS_FILE);
        let bloom_path = dir.join(SERIES_BLOOM_FILE);
        let mut data = BufWriter::new(fs::File::create(&data_path)?);
        data.write_all(SERIES_DATA_MAGIC)?;
        let mut index = BufWriter::new(fs::File::create(&index_path)?);
        index.write_all(SERIES_INDEX_MAGIC)?;
        // The row count is not known until the stream reaches EOF. It is
        // patched after the bounded writer has flushed its bytes; the base is
        // the partition's, so it is final before the first row.
        index.write_all(&0u32.to_le_bytes())?;
        index.write_all(&base_ts_ns.to_le_bytes())?;
        let mut labels = BufWriter::new(fs::File::create(&labels_path)?);
        labels.write_all(SERIES_LABELS_MAGIC)?;
        Ok(Self {
            data,
            index,
            labels,
            data_path,
            index_path,
            labels_path,
            bloom_path,
            bloom: BloomFilter::with_capacity(bloom_items.max(1), BLOOM_FPP),
            data_offset: SERIES_DATA_MAGIC.len() as u64,
            labels_offset: 0,
            base_ts_ns,
            series_count: 0,
            sample_count: 0,
            part_min: i64::MAX,
            part_max: i64::MIN,
            segments: Vec::new(),
            current_tenant: None,
            current_series_start: 0,
            current_segment_start: 0,
            current_segment_samples: 0,
        })
    }

    fn start_tenant(&mut self, tenant: &TenantId) {
        if self.current_tenant.as_ref() == Some(tenant) {
            return;
        }
        self.finish_tenant();
        self.current_tenant = Some(tenant.clone());
        self.current_series_start = self.series_count;
        self.current_segment_start = self.data_offset;
        self.current_segment_samples = 0;
    }

    fn finish_tenant(&mut self) {
        let Some(tenant) = self.current_tenant.take() else {
            return;
        };
        self.segments.push(SeriesTenantSegment {
            tenant,
            series_start: self.current_series_start,
            series_end: self.series_count,
            sample_count: self.current_segment_samples,
            bytes: crate::part::ByteRange {
                start: self.current_segment_start,
                end: self.data_offset,
            },
        });
    }

    fn write_series(
        &mut self,
        tenant: &TenantId,
        labels: &SeriesLabels,
        samples: &mut MergedSeriesSamples,
    ) -> io::Result<()> {
        self.write_series_values(
            tenant,
            labels,
            std::iter::from_fn(|| samples.next_sample().transpose()),
        )
    }

    fn write_series_values<I>(
        &mut self,
        tenant: &TenantId,
        labels: &SeriesLabels,
        samples: I,
    ) -> io::Result<()>
    where
        I: IntoIterator<Item = io::Result<(i64, f64)>>,
    {
        let mut encoder = gorilla::Encoder::new();
        let mut count = 0u64;
        let mut min_ts = i64::MAX;
        let mut max_ts = i64::MIN;
        for sample in samples {
            let (ts, value) = sample?;
            encoder.append(ts, value);
            count += 1;
            min_ts = min_ts.min(ts);
            max_ts = max_ts.max(ts);
        }
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "metric writer encountered an empty series",
            ));
        }
        self.write_row(
            tenant,
            labels,
            WrittenRow {
                kind: SeriesRowKind::Scalar,
                chunk: &encoder.close(),
                count,
                min_ts,
                max_ts,
            },
        )
    }

    /// One histogram series: its observations become a single chunk whose
    /// runs carry the boundary schemas they were counted against.
    fn write_histogram_series(
        &mut self,
        tenant: &TenantId,
        labels: &SeriesLabels,
        points: &[(i64, crate::series::HistogramPoint)],
    ) -> io::Result<()> {
        if points.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "metric writer encountered an empty series",
            ));
        }
        let min_ts = points.iter().map(|(ts, _)| *ts).min().expect("non-empty");
        let max_ts = points.iter().map(|(ts, _)| *ts).max().expect("non-empty");
        self.write_row(
            tenant,
            labels,
            WrittenRow {
                kind: SeriesRowKind::Histogram,
                chunk: &crate::histogram_chunk::encode(points),
                count: points.len() as u64,
                min_ts,
                max_ts,
            },
        )
    }

    /// What a finished chunk needs recorded about it.
    ///
    /// The half both kinds share: append the chunk, append the catalog row
    /// that points at it, and move the counters the metadata is built from.
    fn write_row(
        &mut self,
        tenant: &TenantId,
        labels: &SeriesLabels,
        row: WrittenRow<'_>,
    ) -> io::Result<()> {
        let WrittenRow {
            kind,
            chunk,
            count,
            min_ts,
            max_ts,
        } = row;
        self.start_tenant(tenant);
        let offset = self.data_offset;
        self.data.write_all(chunk)?;
        self.data_offset = self
            .data_offset
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| io::Error::other("metric data offset overflow"))?;

        let labels_len = u32::try_from(labels.as_bytes().len())
            .map_err(|_| io::Error::other("metric labels exceed index field width"))?;
        let chunk_len = u32::try_from(chunk.len())
            .map_err(|_| io::Error::other("metric chunk exceeds index field width"))?;
        self.index.write_all(&self.labels_offset.to_le_bytes())?;
        self.index.write_all(&labels_len.to_le_bytes())?;
        self.index.write_all(&offset.to_le_bytes())?;
        self.index.write_all(&chunk_len.to_le_bytes())?;
        self.index
            .write_all(&delta_floor_ms(self.base_ts_ns, min_ts).to_le_bytes())?;
        self.index
            .write_all(&delta_ceil_ms(self.base_ts_ns, max_ts).to_le_bytes())?;
        self.index.write_all(&[kind.tag(), 0, 0, 0])?;
        self.labels.write_all(labels.as_bytes())?;
        self.labels_offset = self
            .labels_offset
            .checked_add(labels_len)
            .ok_or_else(|| io::Error::other("metric label region offset overflow"))?;

        for (key, value) in labels.pairs().map_err(io::Error::other)? {
            self.bloom.insert(&pair_token(&key, &value));
        }
        self.series_count = self
            .series_count
            .checked_add(1)
            .ok_or_else(|| io::Error::other("metric series count overflow"))?;
        self.sample_count = self
            .sample_count
            .checked_add(count)
            .ok_or_else(|| io::Error::other("metric sample count overflow"))?;
        self.current_segment_samples = self
            .current_segment_samples
            .checked_add(count)
            .ok_or_else(|| io::Error::other("metric tenant sample count overflow"))?;
        self.part_min = self.part_min.min(min_ts);
        self.part_max = self.part_max.max(max_ts);
        Ok(())
    }

    fn finish(mut self, id: &str, partition: &str, metrics_root: &Path) -> io::Result<()> {
        self.finish_tenant();
        let data_path = self.data_path.clone();
        let index_path = self.index_path.clone();
        let labels_path = self.labels_path.clone();
        self.data.flush()?;
        self.index.flush()?;
        self.labels.flush()?;
        close_writer(self.data, &data_path)?;
        close_writer(self.index, &index_path)?;
        close_writer(self.labels, &labels_path)?;

        let mut index = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&index_path)?;
        index.seek(SeekFrom::Start(SERIES_INDEX_MAGIC.len() as u64))?;
        index.write_all(&self.series_count.to_le_bytes())?;
        index.sync_all()?;
        sync_dir(index_path.parent().unwrap_or(metrics_root))?;

        let mut bloom_file = BufWriter::new(fs::File::create(&self.bloom_path)?);
        bloom_file.write_all(SERIES_BLOOM_MAGIC)?;
        let bloom_len = u32::try_from(self.bloom.encoded_len())
            .map_err(|_| io::Error::other("metric bloom exceeds file field width"))?;
        bloom_file.write_all(&bloom_len.to_le_bytes())?;
        self.bloom.write_encoded(&mut bloom_file)?;
        bloom_file.flush()?;
        close_writer(bloom_file, &self.bloom_path)?;

        let mut meta = SeriesMetaFile {
            id: id.to_string(),
            partition: partition.to_string(),
            min_ts_ns: self.part_min,
            max_ts_ns: self.part_max,
            series_count: self.series_count,
            sample_count: self.sample_count,
            tenants: self.segments,
            integrity: SeriesPartIntegrity {
                data_crc32: crc32_file(&data_path)?,
                index_crc32: crc32_file(&index_path)?,
                labels_crc32: crc32_file(&labels_path)?,
                bloom_crc32: crc32_file(&self.bloom_path)?,
                metadata_crc32: 0,
            },
        };
        meta.integrity.metadata_crc32 = metadata_crc32(&meta).map_err(io::Error::other)?;
        let encoded = serde_json::to_vec_pretty(&meta).map_err(io::Error::other)?;
        let meta_path = self.bloom_path.with_file_name(SERIES_META_FILE);
        fs::write(&meta_path, encoded)?;
        sync_file(&meta_path)?;
        Ok(())
    }
}

fn close_writer(writer: BufWriter<fs::File>, path: &Path) -> io::Result<()> {
    let file = writer.into_inner().map_err(|error| error.into_error())?;
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_dir(parent)?;
    }
    Ok(())
}

fn crc32_file(path: &Path) -> io::Result<u32> {
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

fn write_streaming_series_part_files(
    dir: &Path,
    id: &str,
    partition: &str,
    readers: &[std::sync::Arc<SeriesPartReader>],
) -> io::Result<()> {
    // The upper bound is intentionally the total number of label pairs in the
    // inputs. Duplicate series/pairs only make this filter less full than its
    // sizing estimate; no set of all labels is retained just to size a bloom.
    let mut bloom_items = 0usize;
    for reader in readers {
        for segment in &reader.part().meta.tenants {
            for row in reader.tenant_catalog(&segment.tenant).iter() {
                let pairs = crate::series::canonical_pairs(row.labels).count();
                bloom_items = bloom_items
                    .checked_add(pairs)
                    .ok_or_else(|| io::Error::other("metric bloom item count overflow"))?;
            }
        }
    }
    let mut writer = StreamingSeriesPartWriter::new(dir, partition, bloom_items)?;
    let mut cursors: Vec<_> = readers.iter().cloned().map(CatalogCursor::new).collect();
    let mut heap = std::collections::BinaryHeap::with_capacity(cursors.len());
    for (stream, cursor) in cursors.iter().enumerate() {
        if let Some(head) = cursor.current(stream) {
            heap.push(head);
        }
    }

    while let Some(first) = heap.pop() {
        let tenant = first.tenant.clone();
        let labels = first.labels.clone();
        let mut heads = vec![first];
        while heap
            .peek()
            .is_some_and(|head| head.tenant == tenant && head.labels == labels)
        {
            heads.push(heap.pop().expect("heap head was present"));
        }
        if heads
            .iter()
            .any(|head| head.kind == SeriesRowKind::Histogram)
        {
            if !heads
                .iter()
                .all(|head| head.kind == SeriesRowKind::Histogram)
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "one series is scalar in one input part and histogram in another",
                ));
            }
            let points = merged_series_points(&heads)?;
            writer.write_histogram_series(&tenant, &labels, &points)?;
        } else {
            let mut samples = MergedSeriesSamples::new(&heads)?;
            writer.write_series(&tenant, &labels, &mut samples)?;
        }
        for head in heads {
            let cursor = &mut cursors[head.stream];
            cursor.advance();
            if let Some(next) = cursor.current(head.stream) {
                heap.push(next);
            }
        }
    }
    if writer.series_count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "metric compaction produced an empty part",
        ));
    }
    writer.finish(id, partition, dir)
}

/// One partition's samples, grouped `tenant -> series -> sorted samples` —
/// which is exactly the catalog order the files are written in.
/// One series' contribution to a part being written. Both kinds live in the
/// same map so a tenant's rows stay in label order whatever their shape —
/// which is the ordering the compactor's k-way merge is built on.
enum SeriesPayload {
    Scalar(Vec<(i64, f64)>),
    Histogram(Vec<(i64, crate::series::HistogramPoint)>),
}

impl SeriesPayload {
    fn is_empty(&self) -> bool {
        match self {
            Self::Scalar(samples) => samples.is_empty(),
            Self::Histogram(points) => points.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Scalar(samples) => samples.len(),
            Self::Histogram(points) => points.len(),
        }
    }
}

type PartitionSeries = BTreeMap<TenantId, BTreeMap<SeriesLabels, SeriesPayload>>;

/// Flush a snapshot in bounded batches of series/sample bytes.
///
/// Unlike the legacy [`flush_series_snapshot`], this never constructs a map
/// for the complete snapshot. A source series is decoded and sorted once,
/// grouped by day, then appended to the current batch. Once the batch reaches
/// `chunk_bytes`, each partition is written and committed before the next
/// batch is assembled. One very large series can still occupy one batch; that
/// is the indivisible unit of the metric format, but cardinality elsewhere in
/// the snapshot no longer multiplies its memory.
pub fn flush_series_snapshot_chunked(
    snapshot: &SeriesSnapshot,
    metrics_root: &Path,
    chunk_bytes: u64,
) -> io::Result<Vec<SeriesPart>> {
    if snapshot.is_empty() {
        return Ok(Vec::new());
    }
    let _arena = crate::memprof::enter(crate::memprof::Arena::Flush);
    fs::create_dir_all(metrics_root.join(".tmp"))?;

    let mut batches: BTreeMap<String, PartitionSeries> = BTreeMap::new();
    let mut batch_bytes = 0u64;
    let mut parts = Vec::new();
    let mut committed_dirs = Vec::new();
    let chunk_bytes = chunk_bytes.max(1);

    for (tenant, list) in &snapshot.tenants {
        for series in list {
            let by_partition: BTreeMap<String, SeriesPayload> = if series.is_histogram() {
                let mut split: BTreeMap<String, Vec<(i64, crate::series::HistogramPoint)>> =
                    BTreeMap::new();
                for (ts, point) in &series.points {
                    split
                        .entry(partition_of(*ts))
                        .or_default()
                        .push((*ts, point.clone()));
                }
                split
                    .into_iter()
                    .map(|(partition, points)| (partition, SeriesPayload::Histogram(points)))
                    .collect()
            } else {
                let samples = series.sorted_samples().map_err(io::Error::other)?;
                let mut split: BTreeMap<String, Vec<(i64, f64)>> = BTreeMap::new();
                for sample in samples {
                    split
                        .entry(partition_of(sample.0))
                        .or_default()
                        .push(sample);
                }
                split
                    .into_iter()
                    .map(|(partition, samples)| (partition, SeriesPayload::Scalar(samples)))
                    .collect()
            };
            for (partition, payload) in by_partition {
                if payload.is_empty() {
                    continue;
                }
                batch_bytes = batch_bytes.saturating_add(
                    (payload.len() as u64)
                        .saturating_mul(16)
                        .saturating_add(series.labels.byte_len() as u64),
                );
                let slot = batches
                    .entry(partition)
                    .or_default()
                    .entry(tenant.clone())
                    .or_default()
                    .entry(series.labels.clone())
                    .or_insert_with(|| match payload {
                        SeriesPayload::Scalar(_) => SeriesPayload::Scalar(Vec::new()),
                        SeriesPayload::Histogram(_) => SeriesPayload::Histogram(Vec::new()),
                    });
                match (slot, payload) {
                    (SeriesPayload::Scalar(into), SeriesPayload::Scalar(from)) => into.extend(from),
                    (SeriesPayload::Histogram(into), SeriesPayload::Histogram(from)) => {
                        into.extend(from)
                    }
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "one series cannot be scalar and histogram in one part",
                        ));
                    }
                }
            }
            if batch_bytes >= chunk_bytes {
                commit_snapshot_batch(&mut batches, metrics_root, &mut parts, &mut committed_dirs)?;
                batch_bytes = 0;
            }
        }
    }
    if !batches.is_empty() {
        commit_snapshot_batch(&mut batches, metrics_root, &mut parts, &mut committed_dirs)?;
    }
    Ok(parts)
}

fn commit_snapshot_batch(
    batches: &mut BTreeMap<String, PartitionSeries>,
    metrics_root: &Path,
    parts: &mut Vec<SeriesPart>,
    committed_dirs: &mut Vec<PathBuf>,
) -> io::Result<()> {
    let batch = std::mem::take(batches);
    for (partition, tenants) in batch {
        let id = format!("{}-{}", partition.replace('-', ""), uuid::Uuid::new_v4());
        let tmp_dir = metrics_root.join(".tmp").join(&id);
        let final_dir = metrics_root.join(&partition).join(&id);
        let result = (|| -> io::Result<SeriesPart> {
            fs::create_dir_all(&tmp_dir)?;
            write_streaming_partition_series(&tmp_dir, &id, &partition, &tenants)?;
            if let Some(parent) = final_dir.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(&tmp_dir, &final_dir)?;
            committed_dirs.push(final_dir.clone());
            sync_dir(final_dir.parent().unwrap_or(metrics_root))?;
            sync_dir(metrics_root)?;
            load_series_part(&final_dir).map_err(io::Error::other)
        })();
        match result {
            Ok(part) => parts.push(part),
            Err(error) => {
                let _ = fs::remove_dir_all(&tmp_dir);
                rollback_series_dirs(committed_dirs);
                return Err(error);
            }
        }
    }
    Ok(())
}

fn rollback_series_dirs(dirs: &[PathBuf]) {
    for dir in dirs.iter().rev() {
        if let Err(error) = crate::part::remove_part_dirs(std::slice::from_ref(dir)) {
            tracing::warn!(%error, ?dir, "metric flush rollback failed");
        }
    }
}

fn write_streaming_partition_series(
    dir: &Path,
    id: &str,
    partition: &str,
    tenants: &PartitionSeries,
) -> io::Result<()> {
    let mut bloom_items = 0usize;
    for series_map in tenants.values() {
        for labels in series_map.keys() {
            let count = labels.pairs().map_err(io::Error::other)?.len();
            bloom_items = bloom_items
                .checked_add(count)
                .ok_or_else(|| io::Error::other("metric bloom item count overflow"))?;
        }
    }
    let mut writer = StreamingSeriesPartWriter::new(dir, partition, bloom_items)?;
    for (tenant, series_map) in tenants {
        for (labels, payload) in series_map {
            match payload {
                SeriesPayload::Scalar(samples) => writer.write_series_values(
                    tenant,
                    labels,
                    samples.iter().copied().map(Ok::<_, io::Error>),
                )?,
                SeriesPayload::Histogram(points) => {
                    writer.write_histogram_series(tenant, labels, points)?
                }
            }
        }
    }
    writer.finish(id, partition, dir)
}

#[allow(dead_code)]
fn write_series_part_files(
    dir: &Path,
    id: &str,
    partition: &str,
    tenants: &PartitionSeries,
) -> io::Result<()> {
    let base_ts_ns = partition_base_ns(partition).map_err(io::Error::other)?;
    let mut data = Vec::new();
    data.extend_from_slice(SERIES_DATA_MAGIC);
    let mut label_region = Vec::new();
    label_region.extend_from_slice(SERIES_LABELS_MAGIC);

    struct IndexRow {
        labels_offset: u32,
        labels_len: u32,
        chunk_offset: u64,
        chunk_len: u32,
        min_delta_ms: u32,
        max_delta_ms: u32,
        kind: SeriesRowKind,
    }
    let mut catalog: Vec<IndexRow> = Vec::new();
    let mut segments: Vec<SeriesTenantSegment> = Vec::new();
    let mut bloom_tokens: std::collections::BTreeSet<Vec<u8>> = std::collections::BTreeSet::new();
    let mut part_min = i64::MAX;
    let mut part_max = i64::MIN;
    let mut part_samples = 0u64;

    for (tenant, series_map) in tenants {
        let series_start = catalog.len() as u32;
        let segment_bytes_start = data.len() as u64;
        let mut segment_samples = 0u64;
        for (labels, payload) in series_map {
            debug_assert!(
                !payload.is_empty(),
                "the partition split never leaves an empty series"
            );
            let mut min_ts = i64::MAX;
            let mut max_ts = i64::MIN;
            let (kind, chunk) = match payload {
                SeriesPayload::Scalar(samples) => {
                    let mut encoder = gorilla::Encoder::new();
                    for (ts, value) in samples {
                        encoder.append(*ts, *value);
                        min_ts = min_ts.min(*ts);
                        max_ts = max_ts.max(*ts);
                    }
                    (SeriesRowKind::Scalar, encoder.close())
                }
                SeriesPayload::Histogram(points) => {
                    for (ts, _) in points {
                        min_ts = min_ts.min(*ts);
                        max_ts = max_ts.max(*ts);
                    }
                    (
                        SeriesRowKind::Histogram,
                        crate::histogram_chunk::encode(points),
                    )
                }
            };
            let offset = data.len() as u64;
            data.extend_from_slice(&chunk);
            let labels_offset = u32::try_from(label_region.len() - SERIES_LABELS_MAGIC.len())
                .map_err(|_| io::Error::other("metric label region offset overflow"))?;
            label_region.extend_from_slice(labels.as_bytes());
            catalog.push(IndexRow {
                labels_offset,
                labels_len: u32::try_from(labels.as_bytes().len())
                    .map_err(|_| io::Error::other("metric labels exceed index field width"))?,
                chunk_offset: offset,
                chunk_len: u32::try_from(chunk.len())
                    .map_err(|_| io::Error::other("metric chunk exceeds index field width"))?,
                min_delta_ms: delta_floor_ms(base_ts_ns, min_ts),
                max_delta_ms: delta_ceil_ms(base_ts_ns, max_ts),
                kind,
            });
            for (key, value) in labels.pairs().map_err(io::Error::other)? {
                bloom_tokens.insert(pair_token(&key, &value));
            }
            part_min = part_min.min(min_ts);
            part_max = part_max.max(max_ts);
            segment_samples += payload.len() as u64;
            part_samples += payload.len() as u64;
        }
        segments.push(SeriesTenantSegment {
            tenant: tenant.clone(),
            series_start,
            series_end: catalog.len() as u32,
            sample_count: segment_samples,
            bytes: crate::part::ByteRange {
                start: segment_bytes_start,
                end: data.len() as u64,
            },
        });
    }

    fs::write(dir.join(SERIES_DATA_FILE), &data)?;
    sync_file(&dir.join(SERIES_DATA_FILE))?;

    let mut index = Vec::new();
    index.extend_from_slice(SERIES_INDEX_MAGIC);
    index.extend_from_slice(&(catalog.len() as u32).to_le_bytes());
    index.extend_from_slice(&base_ts_ns.to_le_bytes());
    for row in &catalog {
        index.extend_from_slice(&row.labels_offset.to_le_bytes());
        index.extend_from_slice(&row.labels_len.to_le_bytes());
        index.extend_from_slice(&row.chunk_offset.to_le_bytes());
        index.extend_from_slice(&row.chunk_len.to_le_bytes());
        index.extend_from_slice(&row.min_delta_ms.to_le_bytes());
        index.extend_from_slice(&row.max_delta_ms.to_le_bytes());
        index.extend_from_slice(&[row.kind.tag(), 0, 0, 0]);
    }
    fs::write(dir.join(SERIES_INDEX_FILE), &index)?;
    sync_file(&dir.join(SERIES_INDEX_FILE))?;
    fs::write(dir.join(SERIES_LABELS_FILE), &label_region)?;
    sync_file(&dir.join(SERIES_LABELS_FILE))?;

    let mut bloom = BloomFilter::with_capacity(bloom_tokens.len().max(1), BLOOM_FPP);
    for token in &bloom_tokens {
        bloom.insert(token);
    }
    let mut bloom_bytes = Vec::new();
    bloom_bytes.extend_from_slice(SERIES_BLOOM_MAGIC);
    let encoded = bloom.encode();
    bloom_bytes.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
    bloom_bytes.extend_from_slice(&encoded);
    fs::write(dir.join(SERIES_BLOOM_FILE), &bloom_bytes)?;
    sync_file(&dir.join(SERIES_BLOOM_FILE))?;

    let mut meta = SeriesMetaFile {
        id: id.to_string(),
        partition: partition.to_string(),
        min_ts_ns: part_min,
        max_ts_ns: part_max,
        series_count: catalog.len() as u32,
        sample_count: part_samples,
        tenants: segments,
        integrity: SeriesPartIntegrity {
            data_crc32: crc32fast::hash(&data),
            index_crc32: crc32fast::hash(&index),
            labels_crc32: crc32fast::hash(&label_region),
            bloom_crc32: crc32fast::hash(&bloom_bytes),
            metadata_crc32: 0,
        },
    };
    meta.integrity.metadata_crc32 = metadata_crc32(&meta).map_err(io::Error::other)?;
    let encoded = serde_json::to_vec_pretty(&meta).map_err(io::Error::other)?;
    fs::write(dir.join(SERIES_META_FILE), encoded)?;
    sync_file(&dir.join(SERIES_META_FILE))?;
    Ok(())
}

fn validate_series_tenant_segments(meta: &SeriesMetaFile) -> Result<(), String> {
    if meta.tenants.is_empty() {
        return Err("metric part metadata has no tenant segments".to_string());
    }
    let mut expected_start = 0u32;
    let mut total_samples = 0u64;
    for (index, segment) in meta.tenants.iter().enumerate() {
        if index > 0 && meta.tenants[index - 1].tenant >= segment.tenant {
            return Err("metric tenant segments are not sorted by tenant".to_string());
        }
        if segment.series_start != expected_start
            || segment.series_end <= segment.series_start
            || segment.series_end > meta.series_count
        {
            return Err("metric tenant segments do not tile the catalog".to_string());
        }
        if segment.sample_count == 0 {
            return Err("metric tenant segment is empty".to_string());
        }
        total_samples = total_samples.saturating_add(segment.sample_count);
        expected_start = segment.series_end;
    }
    if expected_start != meta.series_count {
        return Err("metric tenant segments do not cover every series".to_string());
    }
    if total_samples != meta.sample_count {
        return Err(
            "metric tenant segment sample counts do not sum to the part sample count".to_string(),
        );
    }
    Ok(())
}

pub fn load_series_part(dir: &Path) -> Result<SeriesPart, String> {
    let dir_metadata = fs::symlink_metadata(dir).map_err(|error| error.to_string())?;
    if dir_metadata.file_type().is_symlink() {
        return Err(format!(
            "refusing symlinked metric part directory {}",
            dir.display()
        ));
    }
    for file in [
        SERIES_META_FILE,
        SERIES_INDEX_FILE,
        SERIES_LABELS_FILE,
        SERIES_BLOOM_FILE,
        SERIES_DATA_FILE,
    ] {
        let path = dir.join(file);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("refusing symlinked metric file {}", path.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    let bytes = fs::read(dir.join(SERIES_META_FILE)).map_err(|error| error.to_string())?;
    let meta: SeriesMetaFile = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if metadata_crc32(&meta)? != meta.integrity.metadata_crc32 {
        return Err(format!(
            "metric metadata checksum mismatch in {}",
            dir.display()
        ));
    }
    if dir.join(SERIES_DATA_FILE).exists() {
        validate_file_crc(
            &dir.join(SERIES_DATA_FILE),
            meta.integrity.data_crc32,
            "metric data",
        )?;
    }
    validate_file_crc(
        &dir.join(SERIES_INDEX_FILE),
        meta.integrity.index_crc32,
        "metric index",
    )?;
    validate_file_crc(
        &dir.join(SERIES_LABELS_FILE),
        meta.integrity.labels_crc32,
        "metric label region",
    )?;
    validate_file_crc(
        &dir.join(SERIES_BLOOM_FILE),
        meta.integrity.bloom_crc32,
        "metric bloom",
    )?;
    validate_series_tenant_segments(&meta)?;
    Ok(SeriesPart {
        dir: dir.to_path_buf(),
        meta: SeriesPartMeta {
            id: meta.id,
            partition: meta.partition,
            min_ts_ns: meta.min_ts_ns,
            max_ts_ns: meta.max_ts_ns,
            series_count: meta.series_count,
            sample_count: meta.sample_count,
            tenants: meta.tenants,
            integrity: meta.integrity,
        },
    })
}

pub fn discover_series_parts(root: &Path) -> Result<Vec<SeriesPart>, String> {
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
                parts.push(load_series_part(&part_dir.path())?);
            }
        }
    }
    Ok(parts)
}

pub struct SeriesPartReader {
    part: SeriesPart,
    bloom: BloomFilter,
    /// The catalog is mapped, not read. It is resident for as long as the
    /// registry holds this reader — offloaded body or not — and at one row per
    /// series per part that residency is the largest thing a stored series
    /// costs. Page cache the kernel can reclaim under pressure is a different
    /// resource from anonymous memory it can only OOM on.
    index: Mmap,
    labels: Mmap,
    base_ts_ns: i64,
}

impl SeriesPartReader {
    pub fn open(part: SeriesPart) -> Result<Self, String> {
        Self::open_internal(part, true)
    }

    /// Open from the catalog artifacts alone: the data body may be evicted to
    /// the object store, and selection must not need it back.
    pub fn open_cached(part: SeriesPart) -> Result<Self, String> {
        Self::open_internal(part, false)
    }

    fn open_internal(part: SeriesPart, require_data: bool) -> Result<Self, String> {
        if require_data && !part.data_path().exists() {
            return Err(format!(
                "metric data body is missing: {}",
                part.data_path().display()
            ));
        }
        // The bloom outlives this call — a reader holds it for as long as the
        // registry holds the reader, offloaded body or not — so it is charged
        // to its own arena rather than to whoever happened to open the part.
        // The catalog is mapped and so is charged to nobody's heap.
        let _arena = crate::memprof::enter(crate::memprof::Arena::SeriesCatalog);
        let bloom_bytes = fs::read(part.bloom_path()).map_err(|error| error.to_string())?;
        let bloom = decode_series_bloom(&bloom_bytes)?;
        let index = map_part_file(&part.index_path())?;
        let labels = map_part_file(&part.labels_path())?;
        let base_ts_ns =
            validate_catalog(&index, &labels, part.meta.series_count as usize, &part.meta)?;
        Ok(Self {
            part,
            bloom,
            index,
            labels,
            base_ts_ns,
        })
    }

    /// Translate an absolute query range into this part's delta space once,
    /// so the catalog walk that follows compares two `u32`s a row. Both ends
    /// round outward, matching how the rows themselves were stored.
    pub fn window(&self, start_ns: i64, end_ns: i64) -> CatalogWindow {
        CatalogWindow {
            start_delta_ms: delta_floor_ms(self.base_ts_ns, start_ns),
            end_delta_ms: delta_ceil_ms(self.base_ts_ns, end_ns),
        }
    }

    pub fn part(&self) -> &SeriesPart {
        &self.part
    }

    /// Whether any series in this part might carry `key=value`. A false
    /// positive costs a catalog walk; a false negative is impossible.
    pub fn may_match_pair(&self, key: &str, value: &str) -> bool {
        self.bloom.contains(&pair_token(key, value))
    }

    /// The tenant's catalog rows — its contiguous ordinal range, so no other
    /// tenant's series are ever offered.
    pub fn tenant_catalog(&self, tenant: &TenantId) -> CatalogRows<'_> {
        let (start, end) = match self.part.meta.tenant_segment(tenant) {
            Some(segment) => (segment.series_start as usize, segment.series_end as usize),
            None => (0, 0),
        };
        CatalogRows {
            index: &self.index,
            labels: &self.labels[SERIES_LABELS_MAGIC.len()..],
            start,
            end,
        }
    }

    /// One series' samples, time-sorted as written.
    pub fn read_series(&self, chunk: ChunkRef) -> Result<Vec<(i64, f64)>, String> {
        self.read_series_decoder(chunk)?.collect()
    }

    /// One histogram series' observations. The caller reaches here because
    /// the row said [`SeriesRowKind::Histogram`]; a scalar chunk decoded this
    /// way fails rather than answering nonsense.
    pub fn read_histogram_points(
        &self,
        chunk: ChunkRef,
    ) -> Result<Vec<(i64, crate::series::HistogramPoint)>, String> {
        crate::histogram_chunk::decode(&self.read_chunk_bytes(chunk)?)
    }

    fn read_chunk_bytes(&self, location: ChunkRef) -> Result<Vec<u8>, String> {
        // The body is opened by path on every chunk read rather than held
        // open, because it is evictable and an open descriptor would keep the
        // bytes on disk after the cache had given them up. What that costs is
        // that a caller who did not pin the part gets an `io::Error` here,
        // whose own message says only "No such file or directory" -- which
        // once reached a client as a 500 with nothing to chase. So every
        // failure names the operation, the part, the path and the kind.
        let path = self.part.data_path();
        let failed = |what: &str, error: std::io::Error| {
            format!(
                "metric part {} chunk read: {what} {} at offset {} for {} bytes: {error} ({:?})",
                self.part.meta.id,
                path.display(),
                location.offset,
                location.length,
                error.kind(),
            )
        };
        let mut file = fs::File::open(&path).map_err(|error| failed("open", error))?;
        file.seek(SeekFrom::Start(location.offset))
            .map_err(|error| failed("seek", error))?;
        let mut chunk = vec![0u8; location.length as usize];
        file.read_exact(&mut chunk)
            .map_err(|error| failed("read", error))?;
        Ok(chunk)
    }

    /// Open one series' chunk as an owned decoder. Keeping the decoder's
    /// chunk and cursor together lets a compaction merge one sample at a time
    /// without retaining a `Vec` for every series in the input group.
    pub fn read_series_decoder(&self, location: ChunkRef) -> Result<gorilla::OwnedDecoder, String> {
        // The chunk carries its own sample count in its first four bytes, and
        // the file carries a checksum. A third copy in the catalog was a row
        // of every part paying for a cross-check both of those already make.
        gorilla::OwnedDecoder::new(self.read_chunk_bytes(location)?)
    }
}

fn decode_series_bloom(bytes: &[u8]) -> Result<BloomFilter, String> {
    if bytes.len() < 8 || &bytes[..4] != SERIES_BLOOM_MAGIC {
        return Err("metric bloom magic or header mismatch".to_string());
    }
    let length = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let end = 8usize
        .checked_add(length)
        .ok_or_else(|| "metric bloom length overflow".to_string())?;
    if end != bytes.len() {
        return Err("metric bloom length mismatch".to_string());
    }
    BloomFilter::decode(&bytes[8..end])
}

/// Map one of a part's catalog files.
///
/// A committed part is immutable, which is what makes a mapping safe: nothing
/// rewrites these bytes in place. Compaction and retention unlink a directory
/// only after the registry has dropped its reader, and an unlinked file that
/// is still mapped keeps its inode until the mapping goes, so a query holding
/// an `Arc` through a retirement reads what it opened.
fn map_part_file(path: &Path) -> Result<Mmap, String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    // SAFETY: parts are write-once. See the note above.
    unsafe { Mmap::map(&file) }.map_err(|error| format!("{}: {error}", path.display()))
}

/// Check a mapped catalog's shape once, at open, so every later row access is
/// infallible slicing. Returns the base timestamp its deltas are measured
/// from.
///
/// This walks the index but never the labels: the row array is 28 bytes a
/// series and sequential, while the payloads it points at are what a query
/// avoids touching until a selector has matched.
fn validate_catalog(
    index: &[u8],
    labels: &[u8],
    expected_count: usize,
    meta: &SeriesPartMeta,
) -> Result<i64, String> {
    if index.len() < SERIES_INDEX_HEADER_BYTES || &index[..4] != SERIES_INDEX_MAGIC {
        return Err("metric index magic or header mismatch".to_string());
    }
    if labels.len() < SERIES_LABELS_MAGIC.len()
        || &labels[..SERIES_LABELS_MAGIC.len()] != SERIES_LABELS_MAGIC
    {
        return Err("metric label region magic mismatch".to_string());
    }
    let region_len = labels.len() - SERIES_LABELS_MAGIC.len();
    let count = u32::from_le_bytes(index[4..8].try_into().unwrap()) as usize;
    if count != expected_count {
        return Err(format!(
            "metric index series count mismatch: {count} != {expected_count}"
        ));
    }
    let expected_len = SERIES_INDEX_HEADER_BYTES
        + count
            .checked_mul(SERIES_INDEX_ENTRY_BYTES)
            .ok_or_else(|| "metric index row count overflows its byte length".to_string())?;
    if index.len() != expected_len {
        return Err(format!(
            "metric index is {} bytes for {count} rows, expected {expected_len}",
            index.len()
        ));
    }
    let mut next = 0usize;
    for segment in &meta.tenants {
        let start = segment.series_start as usize;
        let end = segment.series_end as usize;
        if start != next || end < start || end > count {
            return Err("metric index tenant segments do not tile catalog".to_string());
        }
        next = end;
    }
    if next != count {
        return Err("metric index tenant segments do not cover catalog".to_string());
    }
    for row in 0..count {
        let at = SERIES_INDEX_HEADER_BYTES + row * SERIES_INDEX_ENTRY_BYTES;
        let record = &index[at..at + SERIES_INDEX_ENTRY_BYTES];
        let labels_at = u32::from_le_bytes(record[0..4].try_into().unwrap()) as usize;
        let labels_len = u32::from_le_bytes(record[4..8].try_into().unwrap()) as usize;
        if labels_at
            .checked_add(labels_len)
            .is_none_or(|end| end > region_len)
        {
            return Err("metric index points past its label region".to_string());
        }
    }
    Ok(i64::from_le_bytes(index[8..16].try_into().unwrap()))
}

fn metadata_crc32(meta: &SeriesMetaFile) -> Result<u32, String> {
    let mut canonical = meta.clone();
    canonical.integrity.metadata_crc32 = 0;
    let bytes = serde_json::to_vec(&canonical).map_err(|error| error.to_string())?;
    Ok(crc32fast::hash(&bytes))
}

fn validate_file_crc(path: &Path, expected: u32, label: &str) -> Result<(), String> {
    if crc32_file(path).map_err(|error| format!("failed to read {label}: {error}"))? != expected {
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
    use crate::series::{METRIC_NAME_LABEL, MetricSample, MetricValue, SampleKind, SeriesMemTable};
    use crate::tenant::test_tenant;

    fn labels(name: &str, instance: &str) -> SeriesLabels {
        SeriesLabels::from_pairs(vec![
            (METRIC_NAME_LABEL.to_string(), name.to_string()),
            ("instance".to_string(), instance.to_string()),
        ])
    }

    fn sample(tenant: &str, series: &SeriesLabels, ts: i64, value: f64) -> MetricSample {
        MetricSample {
            tenant: TenantId::parse(tenant).unwrap(),
            labels: series.clone(),
            ts_ns: ts,
            value: MetricValue::Scalar(value),
            kind: SampleKind::Gauge,
            datapoint_index: 0,
        }
    }

    fn temp_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("signy-series-part-{tag}-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn a_snapshot_round_trips_through_a_part_with_spill_merged_in_order() {
        let memtable = SeriesMemTable::new();
        let series = labels("queue_depth", "a");
        memtable.insert(vec![
            sample("test-tenant", &series, 1_772_000_000_000_000_000, 1.0),
            sample("test-tenant", &series, 1_772_000_020_000_000_000, 3.0),
            // Out of order: lands in the spill, must come back sorted.
            sample("test-tenant", &series, 1_772_000_010_000_000_000, 2.0),
        ]);
        let snapshot = memtable.begin_flush();
        let root = temp_root("roundtrip");
        let parts = flush_series_snapshot(&snapshot, &root).unwrap();
        memtable.commit_flush();
        assert_eq!(parts.len(), 1);
        let reader = SeriesPartReader::open(parts.into_iter().next().unwrap()).unwrap();
        let catalog = reader.tenant_catalog(&test_tenant());
        assert_eq!(catalog.len(), 1);
        assert_eq!(
            reader.read_series(catalog.get(0).unwrap().chunk).unwrap(),
            vec![
                (1_772_000_000_000_000_000, 1.0),
                (1_772_000_010_000_000_000, 2.0),
                (1_772_000_020_000_000_000, 3.0)
            ]
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn chunked_snapshot_flush_matches_batch_output() {
        let memtable = SeriesMemTable::new();
        let mut samples = Vec::new();
        for series_index in 0..9 {
            let series = labels("queue_depth", &format!("instance-{series_index}"));
            for sample_index in 0..3 {
                samples.push(sample(
                    "test-tenant",
                    &series,
                    1_772_000_000_000_000_000 + sample_index * 1_000_000_000,
                    (series_index * 10 + sample_index) as f64,
                ));
            }
        }
        memtable.insert(samples);
        let snapshot = memtable.begin_flush();
        let batch_root = temp_root("batch-equivalent");
        let chunked_root = temp_root("chunked-equivalent");
        let batch_parts = flush_series_snapshot(&snapshot, &batch_root).unwrap();
        let chunked_parts = flush_series_snapshot_chunked(&snapshot, &chunked_root, 1).unwrap();
        assert!(chunked_parts.len() > batch_parts.len());

        let read = |parts: &[SeriesPart]| {
            let mut all = BTreeMap::<SeriesLabels, Vec<(i64, f64)>>::new();
            for part in parts {
                let reader = SeriesPartReader::open(part.clone()).unwrap();
                for entry in reader.tenant_catalog(&test_tenant()).iter() {
                    all.entry(SeriesLabels::from_canonical(entry.labels.to_vec()))
                        .or_default()
                        .extend(reader.read_series(entry.chunk).unwrap());
                }
            }
            for samples in all.values_mut() {
                samples.sort_by_key(|(ts, _)| *ts);
            }
            all
        };
        assert_eq!(read(&chunked_parts), read(&batch_parts));
        std::fs::remove_dir_all(&batch_root).ok();
        std::fs::remove_dir_all(&chunked_root).ok();
    }

    #[test]
    fn the_bloom_prunes_absent_pairs_and_admits_present_ones() {
        let memtable = SeriesMemTable::new();
        memtable.insert(vec![sample(
            "test-tenant",
            &labels("queue_depth", "a"),
            1_772_000_000_000_000_000,
            1.0,
        )]);
        let snapshot = memtable.begin_flush();
        let root = temp_root("bloom");
        let parts = flush_series_snapshot(&snapshot, &root).unwrap();
        let reader = SeriesPartReader::open(parts.into_iter().next().unwrap()).unwrap();
        assert!(reader.may_match_pair("instance", "a"));
        assert!(reader.may_match_pair(METRIC_NAME_LABEL, "queue_depth"));
        assert!(
            !reader.may_match_pair("instance", "definitely-not-here-9f2c"),
            "a tiny bloom at 1% FPP must reject this"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn tenants_own_contiguous_catalog_ranges_and_reads_cannot_cross() {
        let memtable = SeriesMemTable::new();
        let series = labels("queue_depth", "shared-instance");
        memtable.insert(vec![
            sample("globex", &series, 1_772_000_000_000_000_000, 9.0),
            sample("acme", &series, 1_772_000_000_000_000_000, 1.0),
        ]);
        let snapshot = memtable.begin_flush();
        let root = temp_root("tenants");
        let parts = flush_series_snapshot(&snapshot, &root).unwrap();
        let reader = SeriesPartReader::open(parts.into_iter().next().unwrap()).unwrap();
        let acme = TenantId::parse("acme").unwrap();
        let globex = TenantId::parse("globex").unwrap();
        let outsider = TenantId::parse("initech").unwrap();
        assert_eq!(reader.tenant_catalog(&acme).len(), 1);
        assert_eq!(reader.tenant_catalog(&globex).len(), 1);
        assert!(reader.tenant_catalog(&outsider).is_empty());
        assert_eq!(
            reader
                .read_series(reader.tenant_catalog(&acme).get(0).unwrap().chunk)
                .unwrap()[0]
                .1,
            1.0
        );
        assert_eq!(
            reader
                .read_series(reader.tenant_catalog(&globex).get(0).unwrap().chunk)
                .unwrap()[0]
                .1,
            9.0
        );
        // The quota census: both tenants have non-empty, disjoint extents.
        let part = reader.part();
        assert_eq!(part.meta.tenants.len(), 2);
        assert!(
            part.meta
                .tenants
                .iter()
                .all(|segment| !segment.bytes.is_empty())
        );
        assert!(part.meta.tenants[0].bytes.end <= part.meta.tenants[1].bytes.start);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn samples_across_a_day_boundary_split_into_two_partitions() {
        let memtable = SeriesMemTable::new();
        let series = labels("queue_depth", "a");
        // 2026-02-25T23:59:50Z and ten seconds later on the 26th.
        let before_midnight = 1_772_063_990_000_000_000;
        let after_midnight = 1_772_064_000_000_000_000;
        memtable.insert(vec![
            sample("test-tenant", &series, before_midnight, 1.0),
            sample("test-tenant", &series, after_midnight, 2.0),
        ]);
        let snapshot = memtable.begin_flush();
        let root = temp_root("partition");
        let parts = flush_series_snapshot(&snapshot, &root).unwrap();
        assert_eq!(parts.len(), 2, "one part per partition");
        let partitions: Vec<&str> = parts
            .iter()
            .map(|part| part.meta.partition.as_str())
            .collect();
        assert_ne!(partitions[0], partitions[1]);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn discovery_reloads_what_flush_wrote_and_catalog_opens_without_the_body() {
        let memtable = SeriesMemTable::new();
        memtable.insert(vec![sample(
            "test-tenant",
            &labels("queue_depth", "a"),
            1_772_000_000_000_000_000,
            1.0,
        )]);
        let snapshot = memtable.begin_flush();
        let root = temp_root("discover");
        let written = flush_series_snapshot(&snapshot, &root).unwrap();
        let discovered = discover_series_parts(&root).unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].meta.id, written[0].meta.id);

        fs::remove_file(discovered[0].data_path()).unwrap();
        let reader =
            SeriesPartReader::open_cached(load_series_part(&discovered[0].dir).unwrap()).unwrap();
        assert!(reader.may_match_pair("instance", "a"));
        assert_eq!(reader.tenant_catalog(&test_tenant()).len(), 1);
        assert!(
            reader
                .read_series(reader.tenant_catalog(&test_tenant()).get(0).unwrap().chunk)
                .is_err(),
            "the body is gone; the catalog still answers selection"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn two_parts_holding_one_series_cost_no_heap_between_them() {
        let root = temp_root("catalog-label-pool");
        let labels = labels("queue_depth", "repeated");

        let first = SeriesMemTable::new();
        first.insert(vec![sample(
            "test-tenant",
            &labels,
            1_772_000_000_000_000_000,
            1.0,
        )]);
        let first_parts = flush_series_snapshot(&first.begin_flush(), &root).unwrap();

        let second = SeriesMemTable::new();
        second.insert(vec![sample(
            "test-tenant",
            &labels,
            1_772_000_001_000_000_000,
            2.0,
        )]);
        let second_parts = flush_series_snapshot(&second.begin_flush(), &root).unwrap();

        // The catalogs used to hold an `Arc` each and go to some trouble to
        // share it. Now each row points into its own part's mapping, so the
        // question of sharing does not arise: neither part allocated anything
        // for this identity, and both still answer with it.
        let first_reader = SeriesPartReader::open(first_parts[0].clone()).unwrap();
        let second_reader = SeriesPartReader::open(second_parts[0].clone()).unwrap();
        let first_row = first_reader.tenant_catalog(&test_tenant()).get(0).unwrap();
        let second_row = second_reader.tenant_catalog(&test_tenant()).get(0).unwrap();
        assert_eq!(first_row.labels, labels.as_bytes());
        assert_eq!(second_row.labels, labels.as_bytes());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_retired_series_is_still_answerable_from_the_catalog_that_owns_it() {
        let root = temp_root("catalog-outlives-index");
        let memtable = SeriesMemTable::new();
        let labels = labels("queue_depth", "retired");
        memtable.insert(vec![sample(
            "test-tenant",
            &labels,
            1_772_000_000_000_000_000,
            1.0,
        )]);
        let snapshot = memtable.begin_flush();
        let parts = flush_series_snapshot(&snapshot, &root).unwrap();
        // The order the flush commits in: the reader is opened while the
        // index still holds the identity, and only then does it retire.
        let reader = SeriesPartReader::open(parts[0].clone()).unwrap();
        memtable.commit_flush();
        assert_eq!(memtable.retire_flushed(&snapshot), 1);

        assert!(memtable.series_labels(&test_tenant()).is_empty());
        let catalog = reader.tenant_catalog(&test_tenant());
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog.get(0).unwrap().labels, labels.as_bytes());
        assert_eq!(
            reader.read_series(catalog.get(0).unwrap().chunk).unwrap(),
            vec![(1_772_000_000_000_000_000, 1.0)],
            "the identity the index dropped is the one the catalog answers with"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn the_index_is_a_fixed_stride_array_and_the_labels_live_beside_it() {
        let root = temp_root("fixed-stride");
        let memtable = SeriesMemTable::new();
        for index in 0..37 {
            memtable.insert(vec![sample(
                "test-tenant",
                &labels("queue_depth", &format!("row-{index}")),
                1_772_000_000_000_000_000,
                1.0,
            )]);
        }
        let parts = flush_series_snapshot(&memtable.begin_flush(), &root).unwrap();
        let dir = parts[0].dir.clone();

        let index_len = fs::metadata(dir.join(SERIES_INDEX_FILE)).unwrap().len() as usize;
        assert_eq!(
            index_len,
            SERIES_INDEX_HEADER_BYTES + 37 * SERIES_INDEX_ENTRY_BYTES,
            "a row that is not addressable by ordinal cannot be read from a mapping"
        );

        // The label payloads are the whole of the other file, so a selection
        // walk touches 28 bytes a row rather than stepping over them.
        let labels_len = fs::metadata(dir.join(SERIES_LABELS_FILE)).unwrap().len() as usize;
        let reader = SeriesPartReader::open(parts.into_iter().next().unwrap()).unwrap();
        let payload: usize = reader
            .tenant_catalog(&test_tenant())
            .iter()
            .map(|entry| entry.labels.len())
            .sum();
        assert_eq!(labels_len, SERIES_LABELS_MAGIC.len() + payload);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_millisecond_row_range_rounds_outward_and_still_prunes() {
        let root = temp_root("delta-window");
        let base = partition_base_ns(&partition_of(1_772_000_000_000_000_000)).unwrap();
        let memtable = SeriesMemTable::new();
        let series = labels("queue_depth", "sub-ms");
        // Half a millisecond past midnight: the stored range is [0, 1] ms, a
        // superset of the sample's instant rather than a truncation of it.
        memtable.insert(vec![sample("test-tenant", &series, base + 500_000, 1.0)]);
        let parts = flush_series_snapshot(&memtable.begin_flush(), &root).unwrap();
        let reader = SeriesPartReader::open(parts.into_iter().next().unwrap()).unwrap();
        let entry = reader.tenant_catalog(&test_tenant()).get(0).unwrap();

        assert!(
            entry.overlaps(reader.window(base + 400_000, base + 600_000)),
            "a window inside the sample's own millisecond must not prune it away"
        );
        assert!(entry.overlaps(reader.window(base, base + 1)));
        assert!(
            !entry.overlaps(reader.window(base + 10_000_000_000, base + 20_000_000_000)),
            "ten seconds later is a real miss and still prunes"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_reader_still_answers_after_its_part_directory_is_unlinked() {
        let root = temp_root("unlinked-mapping");
        let memtable = SeriesMemTable::new();
        let series = labels("queue_depth", "unlinked");
        memtable.insert(vec![sample(
            "test-tenant",
            &series,
            1_772_000_000_000_000_000,
            7.0,
        )]);
        let parts = flush_series_snapshot(&memtable.begin_flush(), &root).unwrap();
        let reader = SeriesPartReader::open(parts[0].clone()).unwrap();
        let chunk = reader.tenant_catalog(&test_tenant()).get(0).unwrap().chunk;
        let samples = reader.read_series(chunk).unwrap();

        // Compaction and retention unlink an input directory after the
        // registry drops its reader, but a query that took an `Arc` from an
        // earlier snapshot still holds one. The catalog is mapped, so its
        // inode outlives the directory entry and that query reads what it
        // opened rather than faulting on a deleted file.
        std::fs::remove_dir_all(&parts[0].dir).unwrap();
        let row = reader.tenant_catalog(&test_tenant()).get(0).unwrap();
        assert_eq!(row.labels, series.as_bytes());
        assert_eq!(samples, vec![(1_772_000_000_000_000_000, 7.0)]);
        std::fs::remove_dir_all(&root).ok();
    }

    fn histogram_sample(
        tenant: &str,
        series: &SeriesLabels,
        ts: i64,
        cumulative: &[u64],
        count: u64,
    ) -> MetricSample {
        MetricSample {
            tenant: TenantId::parse(tenant).unwrap(),
            labels: series.clone(),
            ts_ns: ts,
            value: MetricValue::Histogram(crate::series::HistogramPoint {
                bounds: vec![0.005, 0.01].into(),
                cumulative: cumulative.to_vec(),
                sum: Some(count as f64),
                count,
            }),
            kind: SampleKind::Cumulative,
            datapoint_index: 0,
        }
    }

    #[test]
    fn a_histogram_series_round_trips_through_a_part_as_one_row() {
        let root = temp_root("histogram-roundtrip");
        let memtable = SeriesMemTable::new();
        let series = labels("http_request_duration_seconds", "a");
        memtable.insert(vec![
            histogram_sample(
                "test-tenant",
                &series,
                1_772_000_000_000_000_000,
                &[1, 2],
                3,
            ),
            histogram_sample(
                "test-tenant",
                &series,
                1_772_000_010_000_000_000,
                &[4, 9],
                11,
            ),
        ]);
        let parts = flush_series_snapshot(&memtable.begin_flush(), &root).unwrap();
        let reader = SeriesPartReader::open(parts.into_iter().next().unwrap()).unwrap();

        let catalog = reader.tenant_catalog(&test_tenant());
        assert_eq!(
            catalog.len(),
            1,
            "an instrument that used to be five rows is one"
        );
        let row = catalog.get(0).unwrap();
        assert_eq!(row.kind, SeriesRowKind::Histogram);
        assert_eq!(row.labels, series.as_bytes());
        let points = reader.read_histogram_points(row.chunk).unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].1.cumulative, vec![1, 2]);
        assert_eq!(points[1].1.cumulative, vec![4, 9]);
        assert_eq!(points[1].1.count, 11);
        assert_eq!(&*points[1].1.bounds, &[0.005, 0.01]);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_part_keeps_scalar_and_histogram_rows_in_one_label_order() {
        let root = temp_root("histogram-mixed");
        let memtable = SeriesMemTable::new();
        let gauge = labels("aaa_queue_depth", "a");
        let hist = labels("zzz_request_duration", "a");
        memtable.insert(vec![
            sample("test-tenant", &gauge, 1_772_000_000_000_000_000, 1.0),
            histogram_sample("test-tenant", &hist, 1_772_000_000_000_000_000, &[1, 2], 3),
        ]);
        let parts = flush_series_snapshot(&memtable.begin_flush(), &root).unwrap();
        let reader = SeriesPartReader::open(parts.into_iter().next().unwrap()).unwrap();

        let catalog = reader.tenant_catalog(&test_tenant());
        assert_eq!(catalog.len(), 2);
        // The compactor's k-way merge is built on label order, so the two
        // shapes have to interleave rather than being appended in groups.
        assert_eq!(catalog.get(0).unwrap().kind, SeriesRowKind::Scalar);
        assert_eq!(catalog.get(1).unwrap().kind, SeriesRowKind::Histogram);
        assert!(catalog.get(0).unwrap().labels < catalog.get(1).unwrap().labels);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn compaction_merges_one_histogram_series_by_timestamp() {
        let root = temp_root("histogram-compaction");
        let series = labels("http_request_duration_seconds", "merged");

        let first = SeriesMemTable::new();
        first.insert(vec![histogram_sample(
            "test-tenant",
            &series,
            1_772_000_000_000_000_000,
            &[1, 2],
            3,
        )]);
        let first_parts = flush_series_snapshot(&first.begin_flush(), &root).unwrap();

        let second = SeriesMemTable::new();
        second.insert(vec![histogram_sample(
            "test-tenant",
            &series,
            1_772_000_010_000_000_000,
            &[4, 9],
            11,
        )]);
        let second_parts = flush_series_snapshot(&second.begin_flush(), &root).unwrap();

        let inputs: Vec<_> = first_parts
            .into_iter()
            .chain(second_parts)
            .map(|part| std::sync::Arc::new(SeriesPartReader::open(part).unwrap()))
            .collect();
        let merged = compact_series_parts(&inputs, &root).unwrap();
        assert_eq!(merged.len(), 1);

        let reader = SeriesPartReader::open(merged.into_iter().next().unwrap()).unwrap();
        let catalog = reader.tenant_catalog(&test_tenant());
        assert_eq!(catalog.len(), 1, "one identity, one row, after the merge");
        let row = catalog.get(0).unwrap();
        assert_eq!(row.kind, SeriesRowKind::Histogram);
        let points = reader.read_histogram_points(row.chunk).unwrap();
        assert_eq!(
            points.iter().map(|(ts, _)| *ts).collect::<Vec<_>>(),
            vec![1_772_000_000_000_000_000, 1_772_000_010_000_000_000]
        );
        assert_eq!(points[1].1.cumulative, vec![4, 9]);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_corrupt_label_region_refuses_to_load() {
        let memtable = SeriesMemTable::new();
        memtable.insert(vec![sample(
            "test-tenant",
            &labels("queue_depth", "corrupt-labels"),
            1_772_000_000_000_000_000,
            1.0,
        )]);
        let root = temp_root("corrupt-labels");
        let parts = flush_series_snapshot(&memtable.begin_flush(), &root).unwrap();
        let path = parts[0].dir.join(SERIES_LABELS_FILE);
        let mut bytes = fs::read(&path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        fs::write(&path, bytes).unwrap();
        assert!(
            load_series_part(&parts[0].dir)
                .unwrap_err()
                .contains("checksum")
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_corrupt_artifact_refuses_to_load() {
        let memtable = SeriesMemTable::new();
        memtable.insert(vec![sample(
            "test-tenant",
            &labels("queue_depth", "a"),
            1_772_000_000_000_000_000,
            1.0,
        )]);
        let snapshot = memtable.begin_flush();
        let root = temp_root("corrupt");
        let parts = flush_series_snapshot(&snapshot, &root).unwrap();
        let dir = parts[0].dir.clone();
        let index_path = dir.join(SERIES_INDEX_FILE);
        let mut bytes = fs::read(&index_path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        fs::write(&index_path, bytes).unwrap();
        assert!(load_series_part(&dir).unwrap_err().contains("checksum"));
        std::fs::remove_dir_all(&root).ok();
    }
}
