use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::RwLock;

use crate::logql::LineFilter;
use crate::memtable::{IndexStats, QueryResult, StreamResult};
use crate::object_storage::Manifest;
use crate::part::{
    ExactFieldPredicate, ExactFieldPruning, MetadataWindow, Part, PartReader, QueryTimeRange,
    discover_parts,
};
use crate::tenant::TenantId;

/// What a part holds for each of its tenants: the bytes its row groups occupy
/// in the shared object. The byte extent is already per tenant in `meta.json`,
/// which is where it has to come from: the local file's size is gone once the
/// cache evicts the body, and a quota that reads zero for evicted parts
/// charges for whatever happens to be resident.
fn reader_tenant_facts(reader: &PartReader) -> Vec<TenantFacts> {
    reader
        .meta()
        .tenants
        .iter()
        .map(|segment| TenantFacts {
            tenant: segment.tenant.clone(),
            stored_bytes: segment.bytes.len(),
        })
        .collect()
}

struct TenantFacts {
    tenant: TenantId,
    stored_bytes: u64,
}

/// Per-tenant stored bytes, maintained as the part set changes. Kept rather
/// than summed on demand because the storage quota is read on the ingest path,
/// and walking every part's tenant index per write is the cost this engine
/// bounds everywhere else.
#[derive(Default)]
struct TenantCensus {
    tenants: HashMap<TenantId, u64>,
}

impl TenantCensus {
    fn add(&mut self, facts: &TenantFacts) {
        let entry = self.tenants.entry(facts.tenant.clone()).or_default();
        *entry = entry.saturating_add(facts.stored_bytes);
    }

    fn remove(&mut self, facts: &TenantFacts) {
        let Some(entry) = self.tenants.get_mut(&facts.tenant) else {
            return;
        };
        *entry = entry.saturating_sub(facts.stored_bytes);
        if *entry == 0 {
            self.tenants.remove(&facts.tenant);
        }
    }

    fn stored_bytes(&self, tenant: &TenantId) -> u64 {
        self.tenants.get(tenant).copied().unwrap_or(0)
    }
}

/// What the shared-part layout costs across the current part set.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct LayoutTotals {
    /// `Σ parts.tenants.len()` — the number of (tenant, part) pairs, which is
    /// the unit this layout actually charges in.
    pub tenant_segments: u64,
    /// Sidecar bytes held resident by open parts, beyond what the bloom
    /// cache's own budget reports. With the stream index gone this is zero;
    /// the field stays so the gauge keeps its meaning if a resident sidecar
    /// returns.
    pub sidecar_resident_bytes: u64,
    /// Total `meta.json` across parts, which startup parses before serving.
    pub meta_bytes: u64,
}

impl LayoutTotals {
    fn add(&mut self, reader: &PartReader) {
        self.tenant_segments = self
            .tenant_segments
            .saturating_add(reader.meta().tenants.len() as u64);
        self.meta_bytes = self.meta_bytes.saturating_add(reader.meta().meta_bytes);
    }

    fn remove(&mut self, reader: &PartReader) {
        self.tenant_segments = self
            .tenant_segments
            .saturating_sub(reader.meta().tenants.len() as u64);
        self.meta_bytes = self.meta_bytes.saturating_sub(reader.meta().meta_bytes);
    }
}

pub struct PartRegistry {
    inner: RwLock<HashMap<String, Arc<PartReader>>>,
    /// Running totals for the layout gauges, maintained as the set changes.
    ///
    /// These were published from the merge tick, which meant they read zero in
    /// exactly the configuration that produces the part counts they exist to
    /// describe — measuring part accumulation requires turning merge off.
    /// Recomputing per scrape is the other trap, since it is O(parts × tenants)
    /// on an unauthenticated endpoint. Maintaining them here costs O(1) per
    /// registry change and cannot fall out of step with the set it describes.
    layout: RwLock<LayoutTotals>,
    /// What each tenant currently holds on disk: its streams, so ingest can
    /// tell a new one from one that already exists, and its bytes, so a
    /// storage quota can be answered without walking every part.
    census: RwLock<TenantCensus>,
    operation_lock: Arc<tokio::sync::RwLock<()>>,
    deletion_lock: Arc<tokio::sync::RwLock<()>>,
}

impl Default for PartRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PartRegistry {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            layout: RwLock::new(LayoutTotals::default()),
            census: RwLock::new(TenantCensus::default()),
            operation_lock: Arc::new(tokio::sync::RwLock::new(())),
            deletion_lock: Arc::new(tokio::sync::RwLock::new(())),
        }
    }

    pub fn operation_lock(&self) -> Arc<tokio::sync::RwLock<()>> {
        self.operation_lock.clone()
    }

    /// Guards part *files* against deletion, and nothing else.
    ///
    /// A merge rewrite reads its input directories for as long as the group
    /// takes — 13 seconds was measured for one 56-part group — and it used to
    /// hold the read half of `operation_lock` for all of it. That lock is
    /// fair, so the moment a flush queued its write, every query arriving
    /// after queued too: the whole 13 seconds became query tail latency, at
    /// every merge tick. But the rewrite never cared about visibility — only
    /// that nobody deletes the files under it. That is this lock. Long readers
    /// of part files that do not need the visibility lock take only this one.
    ///
    /// **Deleters take both, and this one first.** Retention retirement and
    /// cache eviction used to take `operation_lock` first, and the soak measured
    /// what that costs: a deleter's wait for this lock is a wait for a whole
    /// merge rewrite, and holding the visibility lock through it stops every
    /// query and every flush for the duration. All 39 freezes in four one-hour
    /// runs began on a retention tick, the longest 52 s, one of them to delete a
    /// single part — and because merge's commit wants `operation_lock` while
    /// holding this one's read half, the old order also had the two spinning
    /// against each other until the deleter's `try_write` won a gap. This order
    /// is merge's order too, so there is no cycle left to survive.
    pub fn deletion_lock(&self) -> Arc<tokio::sync::RwLock<()>> {
        self.deletion_lock.clone()
    }

    /// A write acquisition that never convoys the readers.
    ///
    /// The merge story above repeated itself with the roles recast: queries
    /// hold `operation_lock`'s read half for their whole scan, and the lock
    /// is fair — so the moment retention (or a flush or merge visibility
    /// transition) *queued* a parked write, every query arriving after
    /// queued behind it, for as long as the slowest in-flight scan. The
    /// 24-hour soak read that as ~20 s query-counter freezes and a 9.4 s
    /// query p99 whose maximum sat just under `max_query_runtime` (todo.md).
    ///
    /// So writers poll `try_write` instead of parking: between attempts no
    /// writer is queued and readers flow freely. The price is honest and
    /// paid by the writer — a commit's visibility or a retention tick waits
    /// for a moment with no scan in flight, bounded by the slowest query —
    /// and the deadline caps starvation under a saturated read side by
    /// falling back to one parked (convoying) acquisition rather than
    /// waiting forever.
    pub async fn write_without_convoy(
        lock: Arc<tokio::sync::RwLock<()>>,
    ) -> tokio::sync::OwnedRwLockWriteGuard<()> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(120);
        loop {
            match lock.clone().try_write_owned() {
                Ok(guard) => return guard,
                Err(_) => {
                    if tokio::time::Instant::now() >= deadline {
                        tracing::warn!(
                            "a lifecycle write waited two minutes for a reader-free moment; \
falling back to a parked acquisition, which briefly convoys new readers"
                        );
                        return lock.write_owned().await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
            }
        }
    }

    /// Every tenant that owns a segment in some part. Visits under the read
    /// guard instead of going through `snapshot`, which would clone one `Arc`
    /// per part; `/metrics` asks for this on every scrape.
    pub fn visit_tenants(&self, mut visit: impl FnMut(&TenantId)) {
        for reader in self.inner.read().values() {
            for segment in &reader.meta().tenants {
                visit(&segment.tenant);
            }
        }
    }

    pub fn load_from_disk(parts_root: &Path) -> Result<Self, String> {
        let registry = Self::new();
        registry.reload_from_disk(parts_root)?;
        Ok(registry)
    }

    pub fn reload_from_disk(&self, parts_root: &Path) -> Result<(), String> {
        let parts = discover_parts(parts_root)?;
        self.replace_all(parts, false)
    }

    pub fn load_from_manifest(parts_root: &Path, manifest: &Manifest) -> Result<Self, String> {
        let registry = Self::new();
        registry.reload_from_manifest(parts_root, manifest)?;
        Ok(registry)
    }

    pub fn reload_from_manifest(
        &self,
        parts_root: &Path,
        manifest: &Manifest,
    ) -> Result<(), String> {
        let mut parts = Vec::with_capacity(manifest.parts.len());
        for descriptor in &manifest.parts {
            let dir = parts_root.join(&descriptor.partition).join(&descriptor.id);
            let part = crate::part::load_part(&dir).map_err(|error| {
                format!("failed to load manifest part {}: {error}", descriptor.id)
            })?;
            if part.meta.id != descriptor.id || part.meta.partition != descriptor.partition {
                return Err(format!(
                    "cached part metadata does not match manifest descriptor {}/{}",
                    descriptor.partition, descriptor.id
                ));
            }
            parts.push(part);
        }
        self.replace_all(parts, true)
    }

    fn replace_all(&self, parts: Vec<Part>, allow_missing_data: bool) -> Result<(), String> {
        let mut opened = Vec::with_capacity(parts.len());
        for part in parts {
            let id = part.meta.id.clone();
            let reader = if allow_missing_data {
                PartReader::open_cached(part)
            } else {
                PartReader::open(part)
            }
            .map_err(|e| format!("failed to open part {id} during startup: {e}"))?;
            opened.push((id, Arc::new(reader)));
        }
        let count = opened.len();
        let readers: HashMap<String, Arc<PartReader>> = opened.into_iter().collect();
        self.reset_layout(&readers);
        *self.inner.write() = readers;
        if count > 0 {
            tracing::info!(parts = count, "loaded parts from disk");
        }
        Ok(())
    }

    pub fn has_missing_cache_files(&self) -> bool {
        self.inner
            .read()
            .values()
            .any(|reader| !reader.part().data_path().exists())
    }

    pub fn candidate_part_ids(
        &self,
        tenant: &TenantId,
        range: QueryTimeRange,
    ) -> std::collections::HashSet<String> {
        self.candidate_part_ids_with_exact_fields(tenant, &[], &[], range)
    }

    /// Plans against catalog-resident indexes only, including the optional
    /// BTF2/BTF3 exact-field blooms. BTF1 readers conservatively return
    /// candidates; BTF2 also conservatively scans typed canonical predicates.
    pub fn candidate_part_ids_with_exact_fields(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        exact_fields: &[ExactFieldPredicate],
        range: QueryTimeRange,
    ) -> std::collections::HashSet<String> {
        self.inner
            .read()
            .iter()
            .filter(|(_, reader)| {
                reader.may_match_exact_fields(tenant, line_filters, exact_fields, range)
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

    /// Open readers for freshly written parts, without touching the registry.
    ///
    /// Opening validates sidecar checksums and parses the Parquet footer —
    /// file I/O proportional to the flush — so it belongs on a blocking
    /// thread *before* the exclusive lifecycle lock is taken, not under it.
    /// A chunked flush registers many parts at once, which is exactly when
    /// paying that I/O inside the lock stalls every queued query.
    pub fn open_parts(parts: Vec<Part>) -> Result<Vec<(String, Arc<PartReader>)>, String> {
        let mut opened = Vec::with_capacity(parts.len());
        for part in parts {
            let id = part.meta.id.clone();
            let reader = PartReader::open(part)
                .map_err(|e| format!("failed to open freshly written part {id}: {e}"))?;
            opened.push((id, Arc::new(reader)));
        }
        Ok(opened)
    }

    pub fn register(&self, parts: Vec<Part>) -> Result<Vec<String>, String> {
        Ok(self.register_opened(Self::open_parts(parts)?))
    }

    /// Install already-opened readers: map inserts and derived-index updates
    /// only, so the caller can hold the write half of the lifecycle lock for
    /// exactly the visibility transition and nothing else.
    pub fn register_opened(&self, opened: Vec<(String, Arc<PartReader>)>) -> Vec<String> {
        let ids = opened.iter().map(|(id, _)| id.clone()).collect();
        let mut inner = self.inner.write();
        let mut layout = self.layout.write();
        let mut census = self.census.write();
        for (id, reader) in opened {
            // Registering an id that is already present replaces it, so its
            // predecessor has to leave the derived indexes with it.
            if let Some(previous) = inner.insert(id, reader.clone()) {
                layout.remove(&previous);
                for facts in reader_tenant_facts(&previous) {
                    census.remove(&facts);
                }
            }
            layout.add(&reader);
            for facts in reader_tenant_facts(&reader) {
                census.add(&facts);
            }
        }
        ids
    }

    pub fn unregister(&self, ids: &[String]) {
        let mut inner = self.inner.write();
        let mut layout = self.layout.write();
        let mut census = self.census.write();
        for id in ids {
            if let Some(removed) = inner.remove(id) {
                layout.remove(&removed);
                for facts in reader_tenant_facts(&removed) {
                    census.remove(&facts);
                }
            }
        }
    }

    pub fn replace(&self, old_ids: &[String], new_parts: Vec<Part>) -> Result<Vec<String>, String> {
        if new_parts.is_empty() {
            return Err("cannot replace parts with an empty part set".to_string());
        }

        // Open every replacement before taking the registry mutation. If any
        // one is corrupt, the old set remains queryable and removable only by
        // a later successful merge.
        let opened = Self::open_parts(new_parts)
            .map_err(|e| format!("failed to open fresh merged part: {e}"))?;
        Ok(self.replace_opened(old_ids, opened))
    }

    /// The mutation half of [`replace`](Self::replace): map updates only, so
    /// the caller can open the replacements — checksum validation over a
    /// whole merged part — before taking the exclusive lifecycle lock rather
    /// than under it.
    pub fn replace_opened(
        &self,
        old_ids: &[String],
        opened: Vec<(String, Arc<PartReader>)>,
    ) -> Vec<String> {
        let new_ids: Vec<String> = opened.iter().map(|(id, _)| id.clone()).collect();
        let mut inner = self.inner.write();
        let mut layout = self.layout.write();
        let mut census = self.census.write();
        for (id, reader) in opened {
            if let Some(previous) = inner.insert(id, reader.clone()) {
                layout.remove(&previous);
                for facts in reader_tenant_facts(&previous) {
                    census.remove(&facts);
                }
            }
            layout.add(&reader);
            for facts in reader_tenant_facts(&reader) {
                census.add(&facts);
            }
        }
        for id in old_ids {
            if let Some(removed) = inner.remove(id) {
                layout.remove(&removed);
                for facts in reader_tenant_facts(&removed) {
                    census.remove(&facts);
                }
            }
        }
        new_ids
    }

    pub fn snapshot(&self) -> Vec<Arc<PartReader>> {
        self.inner.read().values().cloned().collect()
    }

    pub fn layout_totals(&self) -> LayoutTotals {
        *self.layout.read()
    }

    /// Rebuild the derived indexes from a whole set, for the paths that replace
    /// it outright rather than adding and removing.
    fn reset_layout(&self, readers: &HashMap<String, Arc<PartReader>>) {
        let mut totals = LayoutTotals::default();
        let mut census = TenantCensus::default();
        for reader in readers.values() {
            totals.add(reader);
            for facts in reader_tenant_facts(reader) {
                census.add(&facts);
            }
        }
        *self.layout.write() = totals;
        *self.census.write() = census;
    }

    /// Parts holding at least one row for the tenant. Its share of the
    /// (tenant, part) pairs the layout charges in.
    pub fn tenant_part_count(&self, tenant: &TenantId) -> usize {
        self.inner
            .read()
            .values()
            .filter(|reader| reader.meta().tenant_segment(tenant).is_some())
            .count()
    }

    /// Bytes the tenant's row groups occupy across every registered part.
    ///
    /// The tenant's share of the shared objects, not of the local disk: what a
    /// plan sells is storage in the object store, and the local copy is a cache
    /// whose contents say nothing about what is being kept. Excludes the
    /// Parquet footer and the sidecars, which belong to no single tenant.
    pub fn tenant_stored_bytes(&self, tenant: &TenantId) -> u64 {
        self.census.read().stored_bytes(tenant)
    }

    pub fn part_count(&self) -> usize {
        self.inner.read().len()
    }

    /// What a scan over `range` could materialize, as (rows, bytes per row).
    ///
    /// The per-row figure is measured, not guessed: every part records the
    /// memory a full read of it materializes (`PartMeta::materialized_bytes`,
    /// written at flush time), and dividing by the rows that produced it gives
    /// this data's own average row. The tenant's share is taken by row
    /// proportion, since `materialized_bytes` covers the whole part.
    ///
    /// It reads `meta.json` alone — the tenant's segment and its timestamps —
    /// and deliberately does **not** run the bloom pruning a candidate plan
    /// runs. That pruning decodes every candidate part's blooms, which a heap
    /// profile of the 2 GiB soak measured at 227 MiB of live bytes and the
    /// single largest allocator in the process; pricing a request is not worth
    /// a second pass of it. It costs nothing to skip: blooms decide *which*
    /// parts hold a matching line, and this needs the average row and a row
    /// ceiling, neither of which a line filter moves. Skipping it counts more
    /// parts, so the estimate is looser in the safe direction, and `limit`
    /// dominates the row term in every case but a nearly empty tenant.
    ///
    /// Returns `None` when the tenant owns no rows in the window, which is the
    /// caller's cue to price the memtable's rows alone.
    pub fn materialization_estimate(
        &self,
        tenant: &TenantId,
        range: QueryTimeRange,
    ) -> Option<(u64, u64)> {
        let mut rows = 0u64;
        let mut bytes = 0u64;
        for reader in self.inner.read().values() {
            let meta = reader.meta();
            let Some(segment) = meta.tenant_segment(tenant) else {
                continue;
            };
            if meta.row_count == 0 || !range.overlaps(segment.min_ts_ns, segment.max_ts_ns) {
                continue;
            }
            rows = rows.saturating_add(segment.row_count);
            // u128 so a large part's `materialized_bytes × row_count` cannot
            // wrap before the division brings it back into range.
            let share = (u128::from(meta.materialized_bytes) * u128::from(segment.row_count)
                / u128::from(meta.row_count)) as u64;
            bytes = bytes.saturating_add(share);
        }
        (rows > 0).then(|| (rows, bytes.div_ceil(rows)))
    }

    /// Every registered part's directory. What eviction walks instead of the
    /// filesystem.
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

    pub fn query(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
    ) -> Result<Vec<StreamResult>, String> {
        Ok(self
            .query_with_exact_field_pruning_and_scan_limit(
                tenant,
                ExactFieldPruning::new(line_filters, &[]),
                range,
                limit,
                forward,
                None,
                None,
            )?
            .results)
    }

    pub fn query_with_exact_field_pruning(
        &self,
        tenant: &TenantId,
        pruning: ExactFieldPruning<'_>,
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
    ) -> Result<Vec<StreamResult>, String> {
        Ok(self
            .query_with_exact_field_pruning_and_scan_limit(
                tenant, pruning, range, limit, forward, None, None,
            )?
            .results)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_with_exact_field_pruning_and_scan_limit(
        &self,
        tenant: &TenantId,
        pruning: ExactFieldPruning<'_>,
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
        scan_limit: Option<usize>,
        cancellation: Option<&AtomicBool>,
    ) -> Result<QueryResult, String> {
        self.query_with_exact_field_pruning_and_scan_limits(
            tenant,
            pruning,
            range,
            limit,
            forward,
            scan_limit,
            None,
            cancellation,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_with_exact_field_pruning_and_scan_limits(
        &self,
        tenant: &TenantId,
        pruning: ExactFieldPruning<'_>,
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
        scan_limit: Option<usize>,
        scan_bytes_limit: Option<u64>,
        cancellation: Option<&AtomicBool>,
    ) -> Result<QueryResult, String> {
        let mut rows = crate::part::TopKRows::new(limit, forward);
        let stats = self.scan_into(
            tenant,
            pruning,
            range,
            forward,
            scan_limit,
            scan_bytes_limit,
            cancellation,
            &crate::part::ColumnSet::all(),
            &mut rows,
        )?;
        Ok(QueryResult {
            results: rows.into_stream_results(),
            scanned_rows: stats.scanned_rows,
            scanned_bytes: stats.scanned_bytes,
        })
    }

    /// Every candidate part's rows, offered to `sink` in the query's direction.
    ///
    /// The registry used to materialize each part's result, flatten it into one
    /// `Vec`, sort that and truncate it — the middle of the three
    /// materialize-and-sort hops (`docs/VISION.md` II). It now passes the
    /// caller's sink straight down, so the rows are never assembled here at all,
    /// and it uses the sink's frontier to skip a whole part from its tenant
    /// segment's timestamp span: a part that cannot hold a row good enough to
    /// enter the result is never opened, and its `.access` marker is not written.
    #[allow(clippy::too_many_arguments)]
    pub fn scan_into(
        &self,
        tenant: &TenantId,
        pruning: ExactFieldPruning<'_>,
        range: QueryTimeRange,
        forward: bool,
        scan_limit: Option<usize>,
        scan_bytes_limit: Option<u64>,
        cancellation: Option<&AtomicBool>,
        columns: &crate::part::ColumnSet,
        sink: &mut dyn crate::part::RowSink,
    ) -> Result<crate::part::ScanStats, String> {
        let mut stats = crate::part::ScanStats::default();
        let readers = self.snapshot();
        if readers.is_empty() || sink.is_closed() {
            return Ok(stats);
        }

        let mut candidates: Vec<Arc<PartReader>> = readers
            .into_iter()
            .filter(|r| {
                // Part-level time pruning is per tenant: a shared part spans
                // every tenant's range, so the part-wide min/max would keep
                // parts that hold nothing for this tenant.
                let Some(segment) = r.meta().tenant_segment(tenant) else {
                    return false;
                };
                range.overlaps(segment.min_ts_ns, segment.max_ts_ns)
            })
            .collect();
        candidates.sort_by_key(|r| {
            r.meta()
                .tenant_segment(tenant)
                .map(|segment| segment.min_ts_ns)
                .unwrap_or(i64::MAX)
        });
        if !forward {
            candidates.reverse();
        }

        for reader in &candidates {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                break;
            }
            // Parts can hold overlapping and out-of-order timestamp ranges, so a
            // full earlier part is not a reason to skip a later one. What *is* a
            // reason is the frontier: once the sink holds `limit` rows, a part
            // whose whole segment is behind the worst of them cannot contribute,
            // whatever its overlap with the others.
            let Some(segment) = reader.meta().tenant_segment(tenant) else {
                continue;
            };
            if crate::part::span_beyond_frontier(
                sink.frontier_ns(),
                forward,
                segment.min_ts_ns,
                segment.max_ts_ns,
            ) {
                continue;
            }
            let part_scan_limit =
                scan_limit.map(|budget| budget.saturating_sub(stats.scanned_rows));
            let part_scan_bytes_limit =
                scan_bytes_limit.map(|budget| budget.saturating_sub(stats.scanned_bytes));
            if part_scan_limit == Some(0) {
                break;
            }
            if part_scan_bytes_limit == Some(0) {
                break;
            }
            let access_marker = reader.part().dir.join(".access");
            match std::fs::symlink_metadata(&access_marker) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    tracing::warn!(
                        path = %access_marker.display(),
                        "refusing to follow symlinked cache access marker"
                    );
                }
                Ok(_) | Err(_) => {
                    let _ = std::fs::write(&access_marker, []);
                }
            }
            let part_stats = reader
                .scan_into(
                    tenant,
                    pruning.line_filters,
                    pruning.exact_fields,
                    range,
                    forward,
                    part_scan_limit,
                    part_scan_bytes_limit,
                    cancellation,
                    None,
                    columns,
                    sink,
                )
                .map_err(|error| {
                    format!("failed to query part {}: {error}", reader.part().meta.id)
                })?;
            stats.scanned_rows = stats.scanned_rows.saturating_add(part_stats.scanned_rows);
            stats.scanned_bytes = stats.scanned_bytes.saturating_add(part_stats.scanned_bytes);
            if part_scan_limit.is_some_and(|limit| part_stats.scanned_rows >= limit) {
                break;
            }
            if part_scan_bytes_limit.is_some_and(|limit| part_stats.scanned_bytes >= limit) {
                break;
            }
        }

        Ok(stats)
    }

    /// Whether the part still holds anything for the tenant that its
    /// retention covers, at part granularity.
    fn within_window(reader: &PartReader, tenant: &TenantId, window: MetadataWindow) -> bool {
        reader
            .meta()
            .tenant_segment(tenant)
            .is_some_and(|segment| window.overlaps(segment.min_ts_ns, segment.max_ts_ns))
    }

    /// Attribute names stored as `_sm:` columns in parts overlapping the
    /// window.
    ///
    /// The stream index that used to answer the metadata endpoints went with
    /// the stream concept, so names come from the per-part key census in
    /// `meta.json`. The census is part-wide rather than per-tenant, and keys
    /// past the column cap live only in the residual blob — both err toward
    /// listing a name over hiding one.
    pub fn metadata_key_names(&self, tenant: &TenantId, window: MetadataWindow) -> Vec<String> {
        let mut names = BTreeSet::new();
        for reader in self.snapshot() {
            if !Self::within_window(&reader, tenant, window) {
                continue;
            }
            for (key, _) in &reader.meta().metadata_columns {
                if !names.contains(key) {
                    names.insert(key.clone());
                }
            }
        }
        names.into_iter().collect()
    }

    /// The newest `limit` rows' metadata in the window, for the sampled
    /// answers the metadata endpoints give now that no value index exists.
    pub fn sample_metadata(
        &self,
        tenant: &TenantId,
        window: MetadataWindow,
        limit: usize,
    ) -> Result<Vec<Vec<(String, String)>>, String> {
        let mut rows = crate::part::TopKRows::new(limit, false);
        self.scan_into(
            tenant,
            ExactFieldPruning::new(&[], &[]),
            QueryTimeRange::closed(window.start_ns, window.end_ns),
            false,
            None,
            None,
            None,
            &crate::part::ColumnSet::all(),
            &mut rows,
        )?;
        Ok(rows
            .into_rows()
            .into_iter()
            .map(|(_, entry)| entry.structured_metadata)
            .collect())
    }

    /// Process-wide totals for the operator metrics endpoint.
    pub fn global_stats(&self) -> IndexStats {
        let mut entries = 0usize;
        let mut bytes = 0u64;
        for reader in self.snapshot() {
            entries += reader.meta().row_count as usize;
            if let Ok(metadata) = std::fs::metadata(reader.part().data_path()) {
                bytes += metadata.len();
            }
        }
        IndexStats { entries, bytes }
    }

    pub fn stats(&self, tenant: &TenantId, window: MetadataWindow) -> IndexStats {
        let mut entries = 0usize;
        let mut bytes = 0u64;
        for reader in self.snapshot() {
            if !Self::within_window(&reader, tenant, window) {
                continue;
            }
            let Some(segment) = reader.meta().tenant_segment(tenant) else {
                continue;
            };
            entries += segment.row_count as usize;
            // The extent `meta.json` records, not the local file's size. The
            // previous arithmetic prorated `fs::metadata` by row share, which
            // read nothing at all once the cache evicted the body — so the
            // number a plan is billed on fell as parts went cold, and a tenant
            // whose data had all aged out of the cache looked free.
            bytes = bytes.saturating_add(segment.bytes.len());
        }
        IndexStats { entries, bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memtable::Labels;
    use crate::part::{self, INDEX_FILE, Row};
    use crate::tenant::test_tenant;
    use std::collections::BTreeMap;

    fn temp_dir() -> std::path::PathBuf {
        crate::test_support::temp_dir("registry")
    }

    fn row(line: &str, timestamp_ns: i64) -> Row {
        let mut labels: Labels = BTreeMap::new();
        labels.insert("app".to_string(), "test".to_string());
        Row {
            tenant: test_tenant(),
            timestamp_ns,
            line: line.to_string(),
            structured_metadata: vec![],
        }
    }

    fn row_for(tenant: &str, line: &str, timestamp_ns: i64) -> Row {
        let mut labels: Labels = BTreeMap::new();
        labels.insert("app".to_string(), "test".to_string());
        Row {
            tenant: TenantId::parse(tenant).unwrap(),
            timestamp_ns,
            line: line.to_string(),
            structured_metadata: vec![],
        }
    }

    /// The number a plan is billed on must not depend on what the cache still
    /// holds. Eviction removes the Parquet body and leaves the catalog, which
    /// is exactly the state the previous `fs::metadata` arithmetic read as zero.
    #[test]
    fn stored_bytes_survive_the_body_being_evicted() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        let parts = part::flush_rows(
            vec![
                row_for("acme", "one", 1_000),
                row_for("acme", "two", 2_000),
                row_for("globex", "three", 3_000),
            ],
            &parts_root,
            1,
        )
        .unwrap();
        let data_paths: Vec<_> = parts.iter().map(|part| part.data_path()).collect();
        registry.register(parts).unwrap();

        let acme = TenantId::parse("acme").unwrap();
        let globex = TenantId::parse("globex").unwrap();
        let before = registry.tenant_stored_bytes(&acme);
        assert!(before > 0, "a registered part must charge the tenant bytes");
        assert!(registry.tenant_stored_bytes(&globex) > 0);

        for path in &data_paths {
            std::fs::remove_file(path).unwrap();
        }
        assert_eq!(registry.tenant_stored_bytes(&acme), before);
        assert_eq!(
            registry.stats(&acme, MetadataWindow::unbounded()).bytes,
            before
        );
    }

    /// One tenant's bytes are its own: unregistering the part that held them
    /// takes them back, and a neighbour in the same object is untouched.
    #[test]
    fn stored_bytes_follow_the_part_set() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        let parts = part::flush_rows(
            vec![
                row_for("acme", "one", 1_000),
                row_for("globex", "two", 2_000),
            ],
            &parts_root,
            1,
        )
        .unwrap();
        let ids: Vec<String> = parts.iter().map(|part| part.meta.id.clone()).collect();
        registry.register(parts).unwrap();

        let acme = TenantId::parse("acme").unwrap();
        let globex = TenantId::parse("globex").unwrap();
        let globex_before = registry.tenant_stored_bytes(&globex);
        assert!(registry.tenant_stored_bytes(&acme) > 0);

        registry.unregister(&ids);
        assert_eq!(registry.tenant_stored_bytes(&acme), 0);
        assert_eq!(registry.tenant_stored_bytes(&globex), 0);
        assert!(globex_before > 0);
    }

    /// The reason these locks are not `std::sync`.
    ///
    /// A panic under a read guard is a real possibility on the query path — a
    /// Parquet decode over a file this process did not write, an arithmetic
    /// edge — and axum catches it per request rather than ending the process.
    /// With a poisoning lock the *next* reader panics too, and so does every
    /// one after it: one bad query turns into an instance that is up, passing
    /// its liveness probe, and answering nothing. Nothing here writes under a
    /// read guard, so there is no half-written state for the poison flag to be
    /// warning about.
    #[test]
    fn a_panic_under_a_read_guard_does_not_wedge_the_registry() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        registry
            .register(part::flush_rows(vec![row("one", 1_000)], &parts_root, 100).unwrap())
            .unwrap();

        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = registry.inner.read();
            panic!("deliberate: a reader panicking while holding the guard");
        }));
        assert!(panicked.is_err());

        assert_eq!(registry.part_count(), 1, "the registry still answers");
        assert!(
            registry
                .query(
                    &test_tenant(),
                    &[],
                    crate::part::QueryTimeRange::closed(0, 2_000),
                    10,
                    true
                )
                .is_ok(),
            "and still serves queries"
        );
    }

    #[test]
    fn query_considers_later_parts_with_overlapping_out_of_order_timestamps() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        let first =
            part::flush_rows(vec![row("first", 0), row("late", 1_000)], &parts_root, 100).unwrap();
        let second = part::flush_rows(vec![row("out-of-order", 1)], &parts_root, 100).unwrap();
        registry.register(first).unwrap();
        registry.register(second).unwrap();

        let result = registry
            .query(
                &test_tenant(),
                &[],
                crate::part::QueryTimeRange::closed(0, 1_000),
                2,
                true,
            )
            .unwrap();
        let entries: Vec<_> = result
            .into_iter()
            .flat_map(|stream| stream.entries)
            .map(|entry| (entry.timestamp_ns, entry.line))
            .collect();
        assert_eq!(
            entries,
            vec![(0, "first".to_string()), (1, "out-of-order".to_string())]
        );
    }

    /// The same, backwards, which is the direction the sink's frontier prunes
    /// parts in.
    ///
    /// The part whose segment starts latest is scanned first and holds only an
    /// old row, so a bound that took "this part is full" for "the answer is
    /// full" would return `out-of-order` and miss `late`. The frontier is what
    /// makes skipping a part sound: it skips one whose whole segment is behind
    /// the worst row already held, and this part's segment is not.
    #[test]
    fn a_backward_limit_still_finds_the_newest_row_in_an_overlapping_part() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        registry
            .register(
                part::flush_rows(vec![row("first", 0), row("late", 1_000)], &parts_root, 100)
                    .unwrap(),
            )
            .unwrap();
        registry
            .register(part::flush_rows(vec![row("out-of-order", 1)], &parts_root, 100).unwrap())
            .unwrap();

        let newest = |limit| {
            registry
                .query(
                    &test_tenant(),
                    &[],
                    crate::part::QueryTimeRange::closed(0, 1_000),
                    limit,
                    false,
                )
                .unwrap()
                .into_iter()
                .flat_map(|stream| stream.entries)
                .map(|entry| (entry.timestamp_ns, entry.line))
                .collect::<Vec<_>>()
        };
        assert_eq!(newest(1), vec![(1_000, "late".to_string())]);
        assert_eq!(
            newest(2),
            vec![(1_000, "late".to_string()), (1, "out-of-order".to_string())]
        );
        assert_eq!(
            newest(3),
            vec![
                (1_000, "late".to_string()),
                (1, "out-of-order".to_string()),
                (0, "first".to_string())
            ]
        );
    }

    #[test]
    fn replace_keeps_old_set_when_any_new_part_fails_to_open() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        let old = part::flush_rows(
            vec![row("old", 1_700_000_000_000_000_000)],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        let old_id = old.meta.id.clone();
        registry.register(vec![old.clone()]).unwrap();

        let new = part::flush_rows(
            vec![row("new", 1_700_000_000_000_000_001)],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        std::fs::write(new.dir.join(INDEX_FILE), b"corrupt").unwrap();

        let result = registry.replace(&[old_id], vec![part::load_part(&new.dir).unwrap()]);
        assert!(result.is_err());
        assert_eq!(registry.part_count(), 1);
        let results = registry
            .query(
                &test_tenant(),
                &[],
                crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
            )
            .expect("part query");
        assert_eq!(results.iter().map(|r| r.entries.len()).sum::<usize>(), 1);
        assert!(old.dir.exists());
        assert!(new.dir.exists());
    }

    #[test]
    fn register_is_atomic_when_any_part_fails_to_open() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        let valid = part::flush_rows(
            vec![row("valid", 1_700_000_000_000_000_000)],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        let corrupt = part::flush_rows(
            vec![row("corrupt", 1_700_086_400_000_000_000)],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        std::fs::write(corrupt.dir.join(INDEX_FILE), b"corrupt").unwrap();

        let result = registry.register(vec![valid, corrupt]);

        assert!(result.is_err());
        assert_eq!(registry.part_count(), 0);
    }

    #[test]
    fn load_from_disk_fails_when_a_part_cannot_be_opened() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let part = part::flush_rows(
            vec![row("corrupt", 1_700_000_000_000_000_000)],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        std::fs::write(part.dir.join(INDEX_FILE), b"corrupt").unwrap();

        let result = PartRegistry::load_from_disk(&parts_root);

        assert!(result.is_err());
    }

    #[test]
    fn query_propagates_part_read_failure() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        let part = part::flush_rows(
            vec![row("will become unreadable", 1_700_000_000_000_000_000)],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        let data_path = part.data_path();
        registry.register(vec![part]).unwrap();

        std::fs::write(data_path, b"corrupt").unwrap();

        let result = registry.query(
            &test_tenant(),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            100,
            true,
        );
        assert!(result.is_err());
    }

    #[test]
    fn query_includes_i64_max_timestamp() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let registry = PartRegistry::new();
        let parts =
            part::flush_rows(vec![row("maximum timestamp", i64::MAX)], &parts_root, 100).unwrap();
        registry.register(parts).unwrap();

        let results = registry
            .query(
                &test_tenant(),
                &[],
                crate::part::QueryTimeRange::closed(i64::MAX, i64::MAX),
                100,
                true,
            )
            .expect("part query");
        assert_eq!(results.iter().map(|r| r.entries.len()).sum::<usize>(), 1);
    }

    #[test]
    fn exact_field_candidates_preserve_row_group_time_correlation() {
        let dir = temp_dir();
        let parts_root = dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let first_ts = 1_700_000_000_000_000_000;
        let second_ts = first_ts + 1_000_000_000;
        let mut first = row("first", first_ts);
        first.structured_metadata = vec![("trace_id".to_string(), "first".to_string())];
        let mut second = row("second", second_ts);
        second.structured_metadata = vec![("trace_id".to_string(), "second".to_string())];
        let parts = part::flush_rows(vec![first, second], &parts_root, 1).unwrap();
        let part_id = parts[0].meta.id.clone();
        let registry = PartRegistry::new();
        registry.register(parts).unwrap();

        let predicate = ExactFieldPredicate::new("trace_id", "second");
        assert!(
            registry
                .candidate_part_ids_with_exact_fields(
                    &test_tenant(),
                    &[],
                    std::slice::from_ref(&predicate),
                    crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                )
                .contains(&part_id)
        );
        assert!(
            registry
                .candidate_part_ids_with_exact_fields(
                    &test_tenant(),
                    &[],
                    &[predicate],
                    crate::part::QueryTimeRange::closed(first_ts, first_ts),
                )
                .is_empty(),
            "a field value in a later row group must not force restoration"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_polite_writer_does_not_convoy_new_readers() {
        let lock = Arc::new(tokio::sync::RwLock::new(()));

        // The scenario the 24-hour soak measured: a slow query holds the
        // read half while a lifecycle writer wants the write half.
        let long_scan = lock.clone().read_owned().await;

        let writer = tokio::spawn(PartRegistry::write_without_convoy(lock.clone()));
        // Let the writer reach its polling loop.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // A new query arriving now must not queue behind the waiting writer —
        // this is the assertion the parked acquisition fails.
        let new_scan = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            lock.clone().read_owned(),
        )
        .await
        .expect("a polite writer must leave new readers unblocked");

        drop(long_scan);
        drop(new_scan);
        drop(writer.await.expect("writer task"));

        // The contrast that makes the helper worth existing: a parked
        // writer convoys the same new reader, because the lock is fair.
        let long_scan = lock.clone().read_owned().await;
        let parked = tokio::spawn({
            let lock = lock.clone();
            async move { lock.write_owned().await }
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let blocked = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            lock.clone().read_owned(),
        )
        .await;
        assert!(
            blocked.is_err(),
            "a parked writer convoys new readers; if this ever passes, the \
             fairness premise changed and write_without_convoy can be retired"
        );
        drop(long_scan);
        drop(parked.await.expect("parked writer task"));
    }
}
