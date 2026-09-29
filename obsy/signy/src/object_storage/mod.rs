use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use bytes::Bytes;
use object_store::path::Path as ObjectPath;
use object_store::{
    GetOptions, GetRange, MultipartUpload, ObjectStore, PutMode, PutOptions, PutResult,
    UpdateVersion,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use crate::part::{self, DATA_FILE, INDEX_FILE, META_FILE, Part};
use crate::series_part::{
    SERIES_BLOOM_FILE, SERIES_DATA_FILE, SERIES_INDEX_FILE, SERIES_LABELS_FILE, SERIES_META_FILE,
    SeriesPart, SeriesPartReader, discover_series_parts,
};
use crate::trace_part::{
    TRACE_BLOOM_FILE, TRACE_DATA_FILE, TRACE_META_FILE, TracePart, TracePartReader,
    discover_trace_parts,
};

const MANIFEST_FILE: &str = "manifest.json";
const UPLOAD_MARKER_FILE: &str = ".object-store-uploading";
const PART_FILES: [&str; 3] = [DATA_FILE, INDEX_FILE, META_FILE];
const CATALOG_FILES: [&str; 2] = [INDEX_FILE, META_FILE];
/// Part downloads in flight while restoring a catalog or a set of bodies.
///
/// Restore was sequential, so its cost was `parts × round trip` — a startup on
/// a 10,000-part manifest spent tens of thousands of round trips one at a time,
/// against a store whose latency is the dominant term and whose throughput is
/// not. The bound exists because the opposite mistake is just as easy: an
/// unbounded fan-out opens a connection per part and turns a restore into a
/// self-inflicted outage of the store it is reading.
const RESTORE_CONCURRENCY: usize = 16;
const TRACE_MANIFEST_FILE: &str = "trace-manifest.json";
/// Per-tenant retention policies, one object per tenant. Deliberately outside
/// the `parts`/`trace_parts` prefixes that `garbage_collect_orphans` sweeps.
pub const TENANT_POLICY_PREFIX: &str = "tenant_policies";
/// Deletion requests, one object per request. Outside the `parts` prefixes for
/// the same reason as the policies: `garbage_collect_orphans` sweeps those.
pub const DELETE_REQUEST_PREFIX: &str = "delete_requests";
const TRACE_PART_FILES: [&str; 3] = [TRACE_DATA_FILE, TRACE_BLOOM_FILE, TRACE_META_FILE];
const TRACE_CATALOG_FILES: [&str; 2] = [TRACE_BLOOM_FILE, TRACE_META_FILE];
const METRIC_MANIFEST_FILE: &str = "metric-manifest.json";
const METRIC_PART_FILES: [&str; 5] = [
    SERIES_DATA_FILE,
    SERIES_INDEX_FILE,
    SERIES_LABELS_FILE,
    SERIES_BLOOM_FILE,
    SERIES_META_FILE,
];
/// Everything metric selection needs without the body: the catalog rows, the
/// labels they point at, the label-pair bloom, and the metadata the registry
/// census reads.
const METRIC_CATALOG_FILES: [&str; 4] = [
    SERIES_INDEX_FILE,
    SERIES_LABELS_FILE,
    SERIES_BLOOM_FILE,
    SERIES_META_FILE,
];
/// Distinguishes one restore attempt's staging directory from another's.
///
/// The staging directory used to be named for the part alone, which is correct
/// only while one restore runs at a time. Restores hold a *shared* lifecycle
/// guard on purpose — network latency must not block writers exclusively — so
/// concurrent readers that miss the same part each enter this path at once, and
/// they were tearing down each other's staging area mid-download: `remove_dir_all`
/// then `create_dir` on a name both of them owned. The losers failed with
/// `File exists` or `No such file or directory`, which the restore path counts
/// as an object-store failure, degrading `remote_healthy` and pushing `/ready`
/// toward 503 over a race the store had no part in. Measured under 24 concurrent
/// readers with fault injection *disabled*, 266 of 358 restores failed this way.
///
/// A process-local counter suffices: `cleanup_tmp` clears `.tmp` at startup, and
/// two processes cannot share a cache root — the writer fence sees to that.
static RESTORE_STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A staging area that removes itself unless a rename carried its contents away.
///
/// The attempt number sits *above* the `<partition>/<id>` pair rather than
/// inside either name: `load_part` validates a staged part against the directory
/// it is read from, and it checks both — the leaf must be the part id and its
/// parent must be the partition. Only a component above those is free.
///
/// Removing the whole attempt directory on drop also replaces what the shared
/// name used to do for free — every failed download used to leave its partial
/// bytes under `.tmp` until the next `cleanup_tmp` at startup.
struct StagingDir {
    /// The attempt-scoped parent, removed on drop.
    root: PathBuf,
    /// Where the downloaded files go.
    dir: PathBuf,
}

impl StagingDir {
    fn create(cache_root: &Path, partition: &str, id: &str) -> Result<Self, String> {
        let attempt = RESTORE_STAGING_SEQUENCE
            .fetch_add(1, Ordering::Relaxed)
            .to_string();
        let dir = ensure_safe_directory_chain(
            cache_root,
            &[".tmp", "remote", &attempt, partition, id],
            true,
        )?;
        let root = dir
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| format!("staging path has no attempt root: {}", dir.display()))?
            .to_path_buf();
        Ok(Self { root, dir })
    }

    fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for StagingDir {
    fn drop(&mut self) {
        // After a successful commit the part directory has already been renamed
        // out and only the empty attempt directory is left, so nothing here is
        // worth reporting a failure over.
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

const MAX_CAS_ATTEMPTS: usize = 16;
const FLUSH_TRANSACTION_FILE: &str = "flush.txn";
/// Where the orphan collector remembers when it first saw an object leave the
/// active set. Nothing but the collector reads it, so retirement time never
/// becomes a concept every manifest reader has to carry.
const GC_ORPHANS_FILE: &str = "gc-orphans.json";
/// The prefixes one orphan-collection cycle walks, in order.
const ORPHAN_PREFIXES: [&str; 3] = ["parts", "trace_parts", "metric_parts"];
const ORPHAN_LEDGER_VERSION: u32 = 2;
/// Objects a scan may list before it saves what it has learned. A pass that
/// saved only at the end lost every first sighting it had made when its budget
/// ran out mid-scan, which is what kept a large store's orphans from ever
/// reaching their second sighting.
const ORPHAN_LEDGER_SAVE_INTERVAL: usize = 20_000;
/// Objects per delete request, which is the S3 bulk-delete maximum.
const ORPHAN_DELETE_CHUNK: usize = 1_000;
/// Failed deletions one pass tolerates before it stops and reports. Deletion
/// is idempotent and the ledger keeps what was not deleted, so stopping costs
/// the rest of this pass and nothing else.
const ORPHAN_DELETE_ERROR_LIMIT: usize = 100;

/// Prefix of the error `publish` returns when another writer already replaced
/// the inputs of a replacement. The store is healthy and nothing was written:
/// the CAS refused precisely so two outputs could not both survive. A caller
/// must treat it as work skipped and retried on the next tick, never as a store
/// failure — reporting it as one takes `/ready` to 503 over a benign race.
pub const INPUTS_CHANGED_ERROR: &str = "manifest replacement conflict";

pub fn is_inputs_changed_error(error: &str) -> bool {
    error.starts_with(INPUTS_CHANGED_ERROR)
}

/// Another process claimed the writer role. Unlike every other manifest
/// failure this one is terminal: retrying cannot succeed, because the epoch
/// this instance holds will never come back.
pub const FENCED_ERROR: &str = "fenced by a newer writer";

pub fn is_fenced_error(error: &str) -> bool {
    error.starts_with(FENCED_ERROR)
}

fn normalized_object_store_options<I>(variables: I) -> Vec<(String, String)>
where
    I: IntoIterator<Item = (String, String)>,
{
    let variables: Vec<_> = variables.into_iter().collect();
    let mut options = BTreeMap::new();

    // object_store's config-key parser expects lowercase names even though
    // process environment variables conventionally use uppercase names.
    for (key, value) in &variables {
        if key.starts_with("AWS_") {
            options.insert(key.to_ascii_lowercase(), value.clone());
        }
    }
    // Explicit OBJECT_STORE_* values override the corresponding AWS value.
    // Both OBJECT_STORE_ENDPOINT and OBJECT_STORE_AWS_ENDPOINT are accepted.
    for (key, value) in &variables {
        if let Some(key) = key.strip_prefix("OBJECT_STORE_") {
            options.insert(key.to_ascii_lowercase(), value.clone());
        }
    }
    options.into_iter().collect()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManifestPart {
    pub id: String,
    pub partition: String,
}

impl From<&Part> for ManifestPart {
    fn from(part: &Part) -> Self {
        Self {
            id: part.meta.id.clone(),
            partition: part.meta.partition.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub generation: u64,
    /// Which writer owns this prefix. Carried in the manifest rather than in
    /// an object of its own so that checking it costs nothing: every write
    /// already loads the manifest it is about to replace.
    ///
    /// Zero means unclaimed, which is what a manifest written before fencing
    /// existed reads as and what a claim then bumps.
    #[serde(default)]
    pub writer_epoch: u64,
    pub parts: Vec<ManifestPart>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceManifestPart {
    pub id: String,
    pub partition: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceManifest {
    pub generation: u64,
    #[serde(default)]
    pub writer_epoch: u64,
    pub parts: Vec<TraceManifestPart>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetricManifestPart {
    pub id: String,
    pub partition: String,
}

impl From<&SeriesPart> for MetricManifestPart {
    fn from(part: &SeriesPart) -> Self {
        Self {
            id: part.meta.id.clone(),
            partition: part.meta.partition.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetricManifest {
    pub generation: u64,
    #[serde(default)]
    pub writer_epoch: u64,
    pub parts: Vec<MetricManifestPart>,
}

/// Durable intent for the cross-domain flush boundary. The journal
/// checkpoint is the commit record: before it advances, startup removes all
/// catalog additions; after it advances, startup only clears this intent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FlushTransaction {
    pub offset: u64,
    pub log_parts: Vec<ManifestPart>,
    pub trace_parts: Vec<TraceManifestPart>,
    /// `default` so an intent written before the metrics signal existed still
    /// parses — an empty list is exactly what that transaction meant.
    #[serde(default)]
    pub metric_parts: Vec<MetricManifestPart>,
}

struct LocalMergeGroup {
    old_ids: Vec<String>,
    added: Vec<Part>,
}

mod counting_store;
mod fault_store;

pub use counting_store::{CountingStore, ObjectStoreOpCounts, ObjectStoreOps, PathByteCounts};

include!("paths.rs");
include!("journal.rs");
include!("catalog.rs");
include!("object_io.rs");
include!("cache.rs");
include!("recovery.rs");

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
