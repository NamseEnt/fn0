use std::io::Error as IoError;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use prost014::Message as Prost014Message;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot};

use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;

mod marks;
pub use marks::{CollectMark, CollectMarks, CollectSignal, Position, SenderId};
use marks::{MARK_RECORD_BYTES, decode_mark, frame_mark};

use crate::config::Config;
use crate::memtable::{LogEntry, MemTable, MemTableSnapshot};
use crate::metrics::LatencyHistogram;
use crate::series::{MetricSample, SeriesMemTable};
use crate::tenant::TenantId;
use crate::trace::{ExportTraceServiceRequest, TraceMemTable, TraceSpan, normalize_request};

const RECORD_HEADER_SIZE: usize = 8;
const MAX_RECORD_BYTES: usize = 256 * 1024 * 1024;
const WAL_FILE: &str = "journal.wal";
const CKPT_FILE: &str = "journal.ckpt";
const COMPACTION_STATE_FILE: &str = "journal.wal.compact.state";
/// `LGY3 | kind | tenant_len:u8 | tenant | payload`.
const TENANT_RECORD_MAGIC: &[u8; 4] = b"LGY3";
const TENANT_RECORD_KIND_LOGS: u8 = 0;
const TENANT_RECORD_KIND_TRACES: u8 = 1;
/// The payload is the OTLP `ExportLogsServiceRequest` as it arrived, the same
/// pattern traces have used from the start: replay re-normalizes instead of
/// the ingest path materializing a second message just so the WAL can hold it.
const TENANT_RECORD_KIND_OTLP_LOGS: u8 = 2;
/// The third signal (M14, issue #8): the raw `ExportMetricsServiceRequest`,
/// replayed through the same pure decomposition live ingest uses.
const TENANT_RECORD_KIND_OTLP_METRICS: u8 = 3;
const TENANT_RECORD_PREFIX_SIZE: usize = TENANT_RECORD_MAGIC.len() + 2;

/// Where a compaction crash is simulated.
///
/// Keyed by WAL path rather than held in a plain flag: tests run in parallel,
/// and a process-wide flag is consumed by whichever compaction reaches it
/// first, which is rarely the one that armed it.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum CompactionFault {
    AfterRename,
    BeforeStateRemoval,
}

#[cfg(test)]
static COMPACTION_FAULTS: std::sync::Mutex<Vec<(PathBuf, CompactionFault)>> =
    std::sync::Mutex::new(Vec::new());

#[cfg(test)]
fn inject_compaction_fault(wal_path: &Path, fault: CompactionFault) {
    COMPACTION_FAULTS
        .lock()
        .unwrap()
        .push((wal_path.to_path_buf(), fault));
}

#[cfg(test)]
fn take_compaction_fault(wal_path: &Path, fault: CompactionFault) -> bool {
    let mut armed = COMPACTION_FAULTS.lock().unwrap();
    let Some(index) = armed
        .iter()
        .position(|(path, pending)| path == wal_path && *pending == fault)
    else {
        return false;
    };
    armed.swap_remove(index);
    true
}

/// An append the writer has taken and not yet made durable.
///
/// Handed back so a caller with more than one record to write can keep them
/// moving instead of paying a whole fsync round trip each. The commands reach
/// the writer in the order they were sent, and a batch is written or not at
/// all, so awaiting these in the same order walks the durable prefix.
pub struct PendingAppend(oneshot::Receiver<Result<(), IoError>>);

impl PendingAppend {
    pub async fn settle(self) -> Result<(), IoError> {
        match self.0.await {
            Ok(result) => result,
            Err(_) => Err(IoError::new(
                std::io::ErrorKind::BrokenPipe,
                "journal writer dropped",
            )),
        }
    }
}

/// Bytes reserved by a metric export while its WAL append is queued or being
/// inserted.  The reservation is owned by the journal append item, so a
/// failed send/write releases it even though the HTTP task has already
/// returned an error.
pub(crate) struct MetricMemoryPermit {
    counter: Arc<AtomicU64>,
    bytes: u64,
}

impl MetricMemoryPermit {
    pub(crate) fn split(&mut self, bytes: u64) -> Self {
        let bytes = bytes.min(self.bytes);
        self.bytes -= bytes;
        Self {
            counter: self.counter.clone(),
            bytes,
        }
    }
}

impl Drop for MetricMemoryPermit {
    fn drop(&mut self) {
        if self.bytes == 0 {
            return;
        }
        let _ = self
            .counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                Some(current.saturating_sub(self.bytes))
            });
    }
}

/// One tenant's portion of a metric export. The journal accepts a vector of
/// these in a single channel command so splitting an export by tenant cannot
/// turn a later enqueue failure into an earlier partial acceptance.
pub(crate) struct ReservedMetricAppend {
    pub(crate) tenant: TenantId,
    pub(crate) data: Vec<u8>,
    pub(crate) samples: Vec<MetricSample>,
    pub(crate) mark: Option<CollectMark>,
    pub(crate) metric_memory_permit: MetricMemoryPermit,
    pub(crate) metric_series_admission: crate::series::SeriesAdmission,
}

pub struct CheckpointSnapshot {
    pub offset: u64,
    pub snapshot: Arc<MemTableSnapshot>,
    pub trace_snapshot: Arc<Vec<TraceSpan>>,
    pub series_snapshot: Arc<crate::series::SeriesSnapshot>,
}

/// A decoded tenant-framed WAL record: who wrote it, what kind it is, and the
/// original payload.
type TenantRecord<'a> = (TenantId, u8, &'a [u8]);

/// Frame a payload so replay can recover which tenant produced it.
///
/// The tenant is written into the WAL rather than derived at replay because
/// the header it came from is gone by then, and mis-attributing a replayed
/// record would breach isolation after a crash.
fn framed_record_len(tenant: &TenantId, payload: &[u8]) -> usize {
    TENANT_RECORD_PREFIX_SIZE + tenant.as_str().len() + payload.len()
}

/// The WAL stores the export zstd-compressed. Uncompressed it was the
/// dominant write-amplification term — ~418 bytes per entry of protobuf the
/// client's own compression had already been stripped from — and every one of
/// those bytes was written, fsynced, and rewritten again by each local
/// compaction. Level 1: the ingest tasks pay the compression in parallel, and
/// at this level zstd moves data far faster than the disk the WAL is
/// protecting.
///
/// Runs on the caller's task, not the writer loop, so compression scales with
/// connections instead of serializing behind the single writer.
fn compress_payload(data: &[u8]) -> Result<Vec<u8>, IoError> {
    let _arena = crate::memprof::enter(crate::memprof::Arena::Wal);
    zstd::encode_all(data, 1)
}

/// The inverse, bounded: a frame header is attacker-writable bytes on disk,
/// so the decompressed size is capped at [`MAX_RECORD_BYTES`] rather than
/// trusted.
fn decompress_payload(payload: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let decoder = zstd::Decoder::new(payload)
        .map_err(|e| format!("journal payload zstd header invalid: {e}"))?;
    let mut out = Vec::new();
    decoder
        .take(MAX_RECORD_BYTES as u64 + 1)
        .read_to_end(&mut out)
        .map_err(|e| format!("journal payload zstd decode failed: {e}"))?;
    if out.len() > MAX_RECORD_BYTES {
        return Err(format!(
            "journal payload decompresses past the {MAX_RECORD_BYTES}-byte record limit"
        ));
    }
    Ok(out)
}

/// The framed form as one buffer. Production writes the frame straight into
/// the writer loop's batch buffer; this remains as the reference encoding the
/// journal tests build WAL bytes with.
#[cfg(test)]
fn frame_tenant_record(tenant: &TenantId, kind: u8, payload: &[u8]) -> Vec<u8> {
    let tenant_bytes = tenant.as_str().as_bytes();
    let mut framed =
        Vec::with_capacity(TENANT_RECORD_PREFIX_SIZE + tenant_bytes.len() + payload.len());
    framed.extend_from_slice(TENANT_RECORD_MAGIC);
    framed.push(kind);
    framed.push(tenant_bytes.len() as u8);
    framed.extend_from_slice(tenant_bytes);
    framed.extend_from_slice(payload);
    framed
}

fn decode_tenant_record(data: &[u8]) -> Result<Option<TenantRecord<'_>>, String> {
    if !data.starts_with(TENANT_RECORD_MAGIC) {
        return Ok(None);
    }
    if data.len() < TENANT_RECORD_PREFIX_SIZE {
        return Err("tenant journal record is truncated".to_string());
    }
    let kind = data[TENANT_RECORD_MAGIC.len()];
    let tenant_len = data[TENANT_RECORD_MAGIC.len() + 1] as usize;
    let tenant_end = TENANT_RECORD_PREFIX_SIZE + tenant_len;
    if data.len() < tenant_end {
        return Err("tenant journal record is truncated".to_string());
    }
    let tenant = std::str::from_utf8(&data[TENANT_RECORD_PREFIX_SIZE..tenant_end])
        .map_err(|error| format!("tenant journal record has a non-UTF-8 tenant: {error}"))?;
    let tenant = TenantId::parse(tenant)?;
    Ok(Some((tenant, kind, &data[tenant_end..])))
}

#[allow(clippy::large_enum_variant)]
enum JournalCmd {
    Append(AppendItem),
    AppendBatch(Vec<AppendItem>),
    Checkpoint {
        done: oneshot::Sender<Result<CheckpointSnapshot, IoError>>,
    },
    Compact {
        offset: u64,
        done: oneshot::Sender<Result<(), IoError>>,
    },
}

struct AppendItem {
    /// Record kind byte, framed by the writer loop. The payload travels
    /// unframed: framing it here built a prefix+tenant+payload copy of every
    /// export just to copy it again into the batch buffer — invariant II's
    /// "WAL write buffer" copy paid twice.
    kind: u8,
    payload: Vec<u8>,
    /// Absent when the append carries nothing but a mark: a record signy will
    /// never accept still has to move the sender's number past it, and there
    /// is no tenant frame to write for one.
    tenant: Option<TenantId>,
    entries: Vec<LogEntry>,
    traces: Vec<TraceSpan>,
    metric_samples: Vec<MetricSample>,
    metric_memory_permit: Option<MetricMemoryPermit>,
    metric_series_admission: Option<crate::series::SeriesAdmission>,
    /// Where this record sat in the queue of the collecty that sent it, when
    /// one did. The writer takes the highest per sender in a batch and writes
    /// it as a record of its own, so the mark and the records it covers share
    /// an fsync.
    mark: Option<CollectMark>,
    /// When the pushing task handed this to the channel. Every push in the
    /// process funnels through one writer task, so the interval between this
    /// and the batch that carries it is the queue in front of that task — the
    /// term nothing measured while the push tail was being argued about from
    /// the client's side (`todo.md`, 2026-08-12).
    queued_at: Instant,
    done: oneshot::Sender<Result<(), IoError>>,
}

/// Where a push's time goes between the handler and its acknowledgement.
///
/// The whole of it is spent in one place: a single writer task that frames a
/// batch, writes it, `sync_all`s once for the batch, and inserts the batch's
/// entries into the memtable before answering any of it. A 12 ms median against
/// a 40–106 ms p95 at 200 pushes/s is what a queue in front of one server looks
/// like, and until these existed the process published no number that could tell
/// the queue from the service — the flush log line carries no duration, and the
/// append path carried no timing at all. The client-side percentiles could not
/// settle it either: they move with the connection count in both directions
/// (`todo.md`, "the connection count moved the queue instead of removing it").
///
/// Four histograms because there are four things a push can be waiting for, and
/// naming which one it was is the entire point. The counters beside them give
/// the batch size, which is what decides how many pushes share each `sync_all`.
#[derive(Default)]
pub struct JournalMetrics {
    /// Channel time: handed to the writer, not yet part of a batch being
    /// written. This is the queue itself.
    pub append_queue_wait: LatencyHistogram,
    /// `write_all` + `flush` for one batch.
    pub batch_write: LatencyHistogram,
    /// `sync_all` for one batch — the durability cost every push in the batch
    /// shares, and the reason batching exists.
    pub batch_fsync: LatencyHistogram,
    /// Memtable and trace-memtable inserts for one batch, which happen in the
    /// writer task after the fsync and before any of its pushes are answered,
    /// and which take a lock the flush also wants.
    pub batch_insert: LatencyHistogram,
    /// A checkpoint runs in the same task, so its duration is time no push can
    /// be written in. Flush asks for one about once a second.
    pub checkpoint: LatencyHistogram,
    pub batches: AtomicU64,
    pub batched_records: AtomicU64,
}

/// A batch slower than this gets a line naming which phase it was.
///
/// Aggregates answer "where does the median go"; a p99 of 128 ms beside a max
/// of 4.7 s is a question about individual events, and a histogram cannot say
/// which phase any one of them was in. At the soak's 200 pushes/s a threshold
/// this far above the median costs a line only when there is something to look
/// at.
const SLOW_BATCH: Duration = Duration::from_millis(250);

pub struct Journal {
    tx: mpsc::Sender<JournalCmd>,
    collect_marks: Arc<CollectMarks>,
    wal_path: PathBuf,
    ckpt_path: PathBuf,
    healthy: Arc<AtomicBool>,
    metrics: Arc<JournalMetrics>,
    memtable: Arc<MemTable>,
    trace_memtable: Arc<TraceMemTable>,
    series_memtable: Arc<SeriesMemTable>,
    backlog: Arc<WalBacklog>,
    metric_reserved_bytes: Arc<AtomicU64>,
}

/// Durable WAL bytes the flush loop has not yet retired.
///
/// Both numbers are already known to the code that changes them, so the ingest
/// gate and `/metrics` read them instead of paying a `stat` plus a checkpoint
/// file read per request.
#[derive(Default)]
pub struct WalBacklog {
    wal_bytes: AtomicU64,
    checkpoint_bytes: AtomicU64,
}

impl WalBacklog {
    fn set_wal_bytes(&self, bytes: u64) {
        self.wal_bytes.store(bytes, Ordering::Release);
    }

    fn set_checkpoint_bytes(&self, bytes: u64) {
        self.checkpoint_bytes.store(bytes, Ordering::Release);
    }

    /// Saturating because the two stores are independent: a checkpoint that
    /// lands between them can briefly exceed the WAL length it refers to.
    pub fn bytes(&self) -> u64 {
        self.wal_bytes
            .load(Ordering::Acquire)
            .saturating_sub(self.checkpoint_bytes.load(Ordering::Acquire))
    }

    /// The WAL file's durable length — dead prefix and live suffix together.
    /// The compaction policy reads this: the backlog alone cannot see a file
    /// whose checkpoint keeps up while its prefix grows without bound.
    pub fn wal_bytes(&self) -> u64 {
        self.wal_bytes.load(Ordering::Acquire)
    }
}

include!("writer.rs");
include!("checkpoint.rs");
include!("replay.rs");
include!("compaction.rs");

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
