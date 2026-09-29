use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use parking_lot::RwLock;

use crate::logql::LineFilter;
use crate::part::{MetadataWindow, QueryTimeRange};
use crate::tenant::TenantId;

/// A string→string map. Once the identity of a stream; now only the shape the
/// LogQL pipeline uses for its per-entry field map and the query layer for
/// attribute sets.
pub type Labels = BTreeMap<String, String>;

/// A shared, usually empty, label map.
///
/// The stream concept is gone from storage — a row carries its attributes in
/// its own `structured_metadata` — but the sink and response plumbing still
/// pass a shared map alongside each entry. Production code passes an empty
/// one; the alias survives for that plumbing and for the pipeline's field
/// maps.
pub type SharedLabels = Arc<Labels>;

/// One tenant's buffered entries, flat. There is no grouping by attributes:
/// an entry's attributes live in its own `structured_metadata`.
pub type MemTableSnapshot = HashMap<TenantId, Vec<LogEntry>>;

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp_ns: i64,
    pub line: String,
    pub structured_metadata: Vec<(String, String)>,
}

pub struct StreamResult {
    pub labels: SharedLabels,
    pub entries: Vec<LogEntry>,
}

pub struct QueryResult {
    pub results: Vec<StreamResult>,
    pub scanned_rows: usize,
    pub scanned_bytes: u64,
}

pub struct MemTable {
    inner: RwLock<MemTableSnapshot>,
    /// The snapshot a flush is currently writing, shared with the flush rather
    /// than copied to it.
    ///
    /// It was cloned, which doubled the memtable's memory at exactly the moment
    /// the memtable is largest — and the failure mode this guards against is a
    /// flush that cannot keep up, where the buffer is larger still.
    flushing: RwLock<Option<Arc<MemTableSnapshot>>>,
    /// Live byte totals for the two buffers, maintained by every mutation.
    ///
    /// Walking the entries instead would be O(rows) under a read lock, and the
    /// callers are the flush loop's twice-a-second size check and the ingest
    /// backpressure gate — both of which get slower exactly when the memtable
    /// is growing, which is when they most need to be cheap.
    inner_bytes: AtomicU64,
    flushing_bytes: AtomicU64,
}

/// Sorted by key, duplicate keys collapsed to their first occurrence.
pub fn canonicalize_structured_metadata(pairs: &mut Vec<(String, String)>) {
    if pairs.windows(2).all(|pair| pair[0].0 < pair[1].0) {
        return;
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs.dedup_by(|later, earlier| later.0 == earlier.0);
}

fn entries_bytes(entries: &[LogEntry]) -> u64 {
    entries
        .iter()
        .map(|entry| {
            let metadata_bytes: usize = entry
                .structured_metadata
                .iter()
                .map(|(name, value)| name.len() + value.len())
                .sum();
            (entry.line.len() + metadata_bytes) as u64
        })
        .sum()
}

/// The O(rows) walk the counters replaced, kept as the thing tests compare
/// them against.
#[cfg(test)]
fn snapshot_bytes(snapshot: &MemTableSnapshot) -> u64 {
    snapshot
        .values()
        .map(|entries| entries_bytes(entries))
        .sum()
}

/// Take ownership of a shared snapshot, copying only if someone else still
/// holds it. On the paths that call this the other reference has already been
/// dropped, so the copy is a fallback that keeps a stray share from being
/// silently discarded rather than an expected cost.
fn unwrap_snapshot(snapshot: Arc<MemTableSnapshot>) -> MemTableSnapshot {
    Arc::try_unwrap(snapshot).unwrap_or_else(|shared| (*shared).clone())
}

/// Merge `source` into `target`. Entries are appended per tenant; with no
/// per-stream identity there is no overhead to reconcile.
fn merge_snapshot(target: &mut MemTableSnapshot, source: MemTableSnapshot) {
    for (tenant, entries) in source {
        target.entry(tenant).or_default().extend(entries);
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            flushing: RwLock::new(None),
            inner_bytes: AtomicU64::new(0),
            flushing_bytes: AtomicU64::new(0),
        }
    }

    pub fn insert(&self, tenant: TenantId, mut entries: Vec<LogEntry>) {
        // Every ingest path — OTLP over either transport, and journal replay —
        // converges here, so this is where structured metadata takes its
        // canonical form: sorted by key, one value per key, first occurrence
        // winning. First-wins is the visibility the pipeline already has
        // (`fields.entry(name).or_insert`), so canonicalizing does not change
        // what a query can see; OTLP genuinely produces duplicates when a
        // record attribute shares a name with a resource attribute.
        // Sorted-unique is what lets the part format store the pairs as
        // columns and rebuild them with a merge instead of a sort, and it
        // makes `Row::sort_key`'s dedup order-insensitive — strictly stronger,
        // which is the safe direction for at-least-once replay.
        for entry in &mut entries {
            canonicalize_structured_metadata(&mut entry.structured_metadata);
        }
        let delta = entries_bytes(&entries);
        let mut inner = self.inner.write();
        match inner.entry(tenant) {
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                occupied.get_mut().extend(entries);
            }
            // Moved, not extended into an empty vector: a tenant's first push
            // used to copy every `LogEntry` it carried.
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(entries);
            }
        }
        // Published under the same write lock as the mutation, so a reader
        // that sees the entries also sees them counted.
        self.inner_bytes.fetch_add(delta, Ordering::Relaxed);
    }

    /// Move the buffer into the flushing slot and hand the flush a share of it.
    ///
    /// The `Arc` is what makes this cheap: reads consult the flushing snapshot
    /// as well as the live buffer, so it has to stay reachable here, but there
    /// is no reason for the flush to hold a second copy of it.
    pub fn begin_flush(&self) -> Arc<MemTableSnapshot> {
        let mut inner = self.inner.write();
        let mut flushing = self.flushing.write();
        let mut snapshot = std::mem::take(&mut *inner);
        let moved = self.inner_bytes.swap(0, Ordering::Relaxed);
        if let Some(previous_snapshot) = flushing.take() {
            merge_snapshot(&mut snapshot, unwrap_snapshot(previous_snapshot));
        }
        let snapshot = Arc::new(snapshot);
        *flushing = Some(snapshot.clone());
        // `flushing_bytes` already covers the snapshot that was merged in.
        self.flushing_bytes.fetch_add(moved, Ordering::Relaxed);
        snapshot
    }

    pub fn commit_flush(&self) {
        let mut flushing = self.flushing.write();
        *flushing = None;
        self.flushing_bytes.store(0, Ordering::Relaxed);
    }

    pub fn abort_flush(&self, snapshot: Arc<MemTableSnapshot>) {
        let mut inner = self.inner.write();
        // Clearing the slot first drops this snapshot's other reference, so the
        // unwrap below takes ownership instead of copying. The lock order is
        // the one every other path uses — inner, then flushing — because
        // reversing it here would be the deadlock the ordering exists to avoid.
        let mut flushing = self.flushing.write();
        *flushing = None;
        merge_snapshot(&mut inner, unwrap_snapshot(snapshot));
        // The aborted snapshot is the one `flushing_bytes` was counting, so it
        // moves back wholesale rather than being recomputed.
        let returned = self.flushing_bytes.swap(0, Ordering::Relaxed);
        self.inner_bytes.fetch_add(returned, Ordering::Relaxed);
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.inner.read();
        if inner.values().any(|entries| !entries.is_empty()) {
            return false;
        }
        let flushing = self.flushing.read();
        flushing
            .as_ref()
            .map(|snapshot| snapshot.values().all(|entries| entries.is_empty()))
            .unwrap_or(true)
    }

    /// Every tenant with entries that have not been flushed yet, including the
    /// ones in a flush that is still in flight. A tenant that has only ever
    /// pushed is invisible in `meta.json` until its first flush, so the
    /// unknown-tenant gauge reads this too.
    pub fn tenants(&self) -> BTreeSet<TenantId> {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        inner
            .keys()
            .chain(flushing.iter().flat_map(|snapshot| snapshot.keys()))
            .cloned()
            .collect()
    }

    /// Bytes held across both buffers, in O(1).
    ///
    /// Takes no lock: the two counters are maintained under the same write
    /// locks as the buffers, and a reader that lands between the pair sees a
    /// value that is one in-flight mutation stale. Callers use it for
    /// thresholds, and no threshold is meaningful at that resolution.
    pub fn approximate_size(&self) -> usize {
        self.inner_bytes
            .load(Ordering::Relaxed)
            .saturating_add(self.flushing_bytes.load(Ordering::Relaxed)) as usize
    }

    pub fn query(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
    ) -> Vec<StreamResult> {
        self.query_with_scan_limit(tenant, line_filters, range, limit, forward, None, None)
            .results
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_with_scan_limit(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
        scan_limit: Option<usize>,
        cancellation: Option<&AtomicBool>,
    ) -> QueryResult {
        let mut rows = crate::part::TopKRows::new(limit, forward);
        // A `TopKRows` never refuses a row, so the scan cannot fail here; the
        // fallible signature is for the executor's sink, which runs the pipeline.
        let scanned_rows = self
            .scan_into(
                tenant,
                line_filters,
                range,
                forward,
                scan_limit,
                cancellation,
                &mut rows,
            )
            .unwrap_or_default();
        QueryResult {
            results: rows.into_stream_results(),
            scanned_rows,
            scanned_bytes: 0,
        }
    }

    /// Both buffers' rows for one tenant, offered to `sink` in the query's
    /// direction.
    ///
    /// Attribute selection is not done here: matchers became pipeline field
    /// filters, evaluated by the sink against each entry's own metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn scan_into(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        range: QueryTimeRange,
        forward: bool,
        scan_limit: Option<usize>,
        cancellation: Option<&AtomicBool>,
        sink: &mut dyn crate::part::RowSink,
    ) -> Result<usize, String> {
        let mut scanned_rows = 0usize;
        if sink.is_closed() {
            return Ok(scanned_rows);
        }

        // begin_flush and abort_flush acquire inner before flushing. Holding
        // both read guards in that order prevents observing the same entry in
        // both buffers if a flush starts between the two reads.
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        // The live buffer first, so rows pushed earlier arrive earlier on a
        // timestamp tie; the sort below is stable, so arrival order survives
        // it and a limited query returns the same rows on every run.
        let mut ordered: Vec<&LogEntry> = Vec::new();
        for buffer in std::iter::once(&*inner).chain(flushing.as_deref()) {
            if let Some(entries) = buffer.get(tenant) {
                ordered.extend(entries.iter());
            }
        }
        ordered.sort_by_key(|entry| entry.timestamp_ns);
        if !forward {
            ordered.reverse();
        }

        let empty_labels: SharedLabels = SharedLabels::default();
        let mut result = Ok(());
        for entry in ordered {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                break;
            }
            if crate::part::beyond_frontier(sink.frontier_ns(), forward, entry.timestamp_ns) {
                break;
            }
            if scan_limit.is_some_and(|limit| scanned_rows >= limit) {
                break;
            }
            scanned_rows = scanned_rows.saturating_add(1);
            if range.contains(entry.timestamp_ns)
                && line_filters
                    .iter()
                    .all(|filter| filter.matches(&entry.line))
            {
                result = sink.accept(&empty_labels, entry.clone());
                if result.is_err() {
                    break;
                }
            }
        }
        drop(flushing);
        drop(inner);
        result.map(|()| scanned_rows)
    }

    /// Run `visit` over both buffers of one tenant while holding the read
    /// guards in the order `begin_flush`/`abort_flush` acquire them, so an
    /// entry can never be observed in both buffers or in neither. Restricted
    /// to entries the window still covers.
    fn for_each_tenant_entry(
        &self,
        tenant: &TenantId,
        window: MetadataWindow,
        mut visit: impl FnMut(&LogEntry),
    ) {
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        for buffer in std::iter::once(&*inner).chain(flushing.as_deref()) {
            if let Some(entries) = buffer.get(tenant) {
                for entry in entries {
                    if window.contains(entry.timestamp_ns) {
                        visit(entry);
                    }
                }
            }
        }
    }

    /// Attribute names present in the tenant's buffered metadata.
    pub fn label_names(&self, tenant: &TenantId, window: MetadataWindow) -> Vec<String> {
        let mut names = BTreeSet::new();
        self.for_each_tenant_entry(tenant, window, |entry| {
            for (name, _) in &entry.structured_metadata {
                if !names.contains(name) {
                    names.insert(name.clone());
                }
            }
        });
        names.into_iter().collect()
    }

    /// Values the tenant's buffered metadata holds for one attribute name.
    pub fn label_values(
        &self,
        tenant: &TenantId,
        name: &str,
        window: MetadataWindow,
    ) -> Vec<String> {
        let mut values = BTreeSet::new();
        self.for_each_tenant_entry(tenant, window, |entry| {
            for (key, value) in &entry.structured_metadata {
                if key == name && !values.contains(value) {
                    values.insert(value.clone());
                }
            }
        });
        values.into_iter().collect()
    }

    /// Distinct attribute sets in the tenant's buffer, filtered by `matchers`
    /// evaluated against each entry's metadata.
    pub fn series(
        &self,
        tenant: &TenantId,
        matchers: &[crate::logql::LabelMatcher],
        window: MetadataWindow,
    ) -> Vec<Labels> {
        let mut result: BTreeSet<Labels> = BTreeSet::new();
        self.for_each_tenant_entry(tenant, window, |entry| {
            let set: Labels = entry.structured_metadata.iter().cloned().collect();
            if matchers.iter().all(|m| m.matches(&set)) {
                result.insert(set);
            }
        });
        result.into_iter().collect()
    }

    /// Process-wide totals for the operator metrics endpoint. Not a query
    /// path: nothing here is returned to a tenant.
    pub fn global_stats(&self) -> IndexStats {
        let mut entries = 0usize;
        let mut bytes = 0u64;
        let inner = self.inner.read();
        let flushing = self.flushing.read();
        for snapshot in std::iter::once(&*inner).chain(flushing.as_deref()) {
            for tenant_entries in snapshot.values() {
                entries += tenant_entries.len();
                for entry in tenant_entries {
                    bytes += entry.line.len() as u64;
                }
            }
        }
        IndexStats { entries, bytes }
    }

    pub fn stats(&self, tenant: &TenantId, window: MetadataWindow) -> IndexStats {
        let mut entries = 0usize;
        let mut bytes = 0u64;
        self.for_each_tenant_entry(tenant, window, |entry| {
            entries += 1;
            bytes += entry.line.len() as u64;
        });
        IndexStats { entries, bytes }
    }
}

pub struct IndexStats {
    pub entries: usize,
    pub bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Insert is the choke point every ingest path converges on, so metadata
    /// leaving the memtable is canonical whatever order or duplication it
    /// arrived with — OTLP genuinely duplicates a key when a record attribute
    /// shares a name with a resource attribute, and first-wins is the
    /// visibility the pipeline already gives the earlier value.
    #[test]
    fn inserted_metadata_is_sorted_and_first_wins_on_duplicate_keys() {
        let memtable = MemTable::new();
        let tenant = crate::tenant::test_tenant();
        memtable.insert(
            tenant.clone(),
            vec![LogEntry {
                timestamp_ns: 1,
                line: "line".to_string(),
                structured_metadata: vec![
                    ("zeta".to_string(), "z".to_string()),
                    ("alpha".to_string(), "record".to_string()),
                    ("alpha".to_string(), "resource".to_string()),
                ],
            }],
        );
        let snapshot = memtable.begin_flush();
        let entry = &snapshot[&tenant][0];
        assert_eq!(
            entry.structured_metadata,
            vec![
                ("alpha".to_string(), "record".to_string()),
                ("zeta".to_string(), "z".to_string()),
            ]
        );
        memtable.abort_flush(snapshot);
    }

    /// The counters replace an O(rows) walk, so what has to hold is that they
    /// still say what the walk would. Checked after each transition rather
    /// than only at the end: a counter that drifts and later recovers is a
    /// counter that reported a wrong threshold in between.
    #[test]
    fn the_size_counters_track_the_walked_total_across_a_flush_cycle() {
        let memtable = MemTable::new();
        let walked = |table: &MemTable| {
            let inner = table.inner.read();
            let flushing = table.flushing.read();
            snapshot_bytes(&inner) + flushing.as_deref().map(snapshot_bytes).unwrap_or(0)
        };
        let tenant = crate::tenant::test_tenant();

        memtable.insert(tenant.clone(), vec![sample_entry("first", 1)]);
        assert_eq!(memtable.approximate_size() as u64, walked(&memtable));

        memtable.insert(tenant.clone(), vec![sample_entry("second", 2)]);
        assert_eq!(memtable.approximate_size() as u64, walked(&memtable));

        let snapshot = memtable.begin_flush();
        assert_eq!(memtable.approximate_size() as u64, walked(&memtable));

        // An insert while a flush is in flight lands in the other buffer.
        memtable.insert(tenant, vec![sample_entry("third", 3)]);
        assert_eq!(memtable.approximate_size() as u64, walked(&memtable));

        memtable.abort_flush(snapshot);
        assert_eq!(memtable.approximate_size() as u64, walked(&memtable));

        let snapshot = memtable.begin_flush();
        memtable.commit_flush();
        drop(snapshot);
        assert_eq!(memtable.approximate_size(), 0);
        assert_eq!(walked(&memtable), 0);
    }

    fn sample_entry(line: &str, ts_ns: i64) -> LogEntry {
        LogEntry {
            timestamp_ns: ts_ns,
            line: line.to_string(),
            structured_metadata: vec![],
        }
    }

    fn tenant(name: &str) -> TenantId {
        TenantId::parse(name).unwrap()
    }

    fn sample_tenant() -> TenantId {
        tenant("acme")
    }

    fn total_entries(results: &[StreamResult]) -> usize {
        results.iter().map(|stream| stream.entries.len()).sum()
    }

    #[test]
    fn a_tenant_never_sees_another_tenants_entries() {
        let memtable = MemTable::new();
        memtable.insert(tenant("acme"), vec![sample_entry("acme line", 100)]);
        memtable.insert(tenant("globex"), vec![sample_entry("globex line", 100)]);

        let acme = memtable.query(
            &tenant("acme"),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            100,
            true,
        );
        let lines: Vec<_> = acme
            .iter()
            .flat_map(|stream| stream.entries.iter().map(|entry| entry.line.as_str()))
            .collect();
        assert_eq!(lines, vec!["acme line"]);

        assert_eq!(
            memtable
                .stats(&tenant("acme"), MetadataWindow::unbounded())
                .entries,
            1
        );
        assert_eq!(
            memtable
                .stats(&tenant("globex"), MetadataWindow::unbounded())
                .entries,
            1
        );
        assert!(
            memtable
                .query(
                    &tenant("initech"),
                    &[],
                    crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                    100,
                    true
                )
                .is_empty(),
            "an unknown tenant must see nothing"
        );
        assert!(
            memtable
                .label_names(&tenant("initech"), MetadataWindow::unbounded())
                .is_empty()
        );
        assert!(
            memtable
                .series(&tenant("initech"), &[], MetadataWindow::unbounded())
                .is_empty()
        );
    }

    #[test]
    fn flushing_buffer_visible_during_flush() {
        // begin_flush clears inner and moves the data to the flushing buffer.
        // unified_query also scans the flushing buffer, so data is not lost during flushing (#2 regression).
        let mt = MemTable::new();
        mt.insert(sample_tenant(), vec![sample_entry("hello", 100)]);

        let snapshot = mt.begin_flush();
        assert_eq!(snapshot.len(), 1);

        let results = mt.query(
            &sample_tenant(),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            100,
            true,
        );
        assert_eq!(
            total_entries(&results),
            1,
            "flushing buffer should remain visible during flush"
        );

        mt.commit_flush();
        let results2 = mt.query(
            &sample_tenant(),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            100,
            true,
        );
        assert_eq!(
            total_entries(&results2),
            0,
            "after commit_flush, no data should remain visible"
        );
    }

    #[test]
    fn abort_flush_restores_to_inner() {
        let mt = MemTable::new();
        mt.insert(sample_tenant(), vec![sample_entry("hello", 100)]);

        let snapshot = mt.begin_flush();
        mt.abort_flush(snapshot);

        let results = mt.query(
            &sample_tenant(),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            100,
            true,
        );
        assert_eq!(
            total_entries(&results),
            1,
            "abort_flush should restore data to inner"
        );
        assert!(!mt.is_empty());
    }

    /// The snapshot is shared with the flush, not copied to it. The copy
    /// doubled memtable memory at exactly the moment the memtable is largest,
    /// and the failure this guards against — a flush that cannot keep up — is
    /// where the buffer is larger still.
    #[test]
    fn a_flush_shares_the_snapshot_instead_of_duplicating_it() {
        let memtable = MemTable::new();
        memtable.insert(sample_tenant(), vec![sample_entry("shared", 100)]);

        let snapshot = memtable.begin_flush();
        assert_eq!(
            Arc::strong_count(&snapshot),
            2,
            "the flush and the memtable hold the same snapshot, not two of them"
        );

        // Committing releases the memtable's share, leaving the flush's own.
        memtable.commit_flush();
        assert_eq!(Arc::strong_count(&snapshot), 1);
    }

    /// Aborting has to put the entries back, which needs ownership. The
    /// memtable's share is dropped first so that ownership is taken rather than
    /// the snapshot being copied on the way back in.
    #[test]
    fn an_aborted_flush_returns_its_entries_without_copying_them() {
        let memtable = MemTable::new();
        memtable.insert(sample_tenant(), vec![sample_entry("returned", 100)]);
        let before = memtable.approximate_size();

        let snapshot = memtable.begin_flush();
        assert_eq!(
            total_entries(&memtable.query(
                &sample_tenant(),
                &[],
                crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                10,
                true
            )),
            1
        );
        memtable.abort_flush(snapshot);

        let results = memtable.query(
            &sample_tenant(),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            10,
            true,
        );
        assert_eq!(
            total_entries(&results),
            1,
            "the aborted entries are back in the live buffer"
        );
        assert_eq!(
            memtable.approximate_size(),
            before,
            "and the accounting returns with them"
        );
    }

    #[test]
    fn begin_flush_keeps_query_consistent_with_concurrent_insert() {
        // Data inserted while flushing is in progress must also be visible.
        let mt = MemTable::new();
        mt.insert(sample_tenant(), vec![sample_entry("first", 100)]);

        let _snapshot = mt.begin_flush();
        // Receive new data while flushing is in progress.
        mt.insert(sample_tenant(), vec![sample_entry("second", 200)]);

        let results = mt.query(
            &sample_tenant(),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            100,
            true,
        );
        assert_eq!(
            total_entries(&results),
            2,
            "both flushing buffer and inner should be visible concurrently"
        );
    }

    #[test]
    fn begin_flush_preserves_previous_uncommitted_snapshot() {
        let memtable = MemTable::new();
        memtable.insert(sample_tenant(), vec![sample_entry("first", 100)]);

        let _first_snapshot = memtable.begin_flush();
        memtable.insert(sample_tenant(), vec![sample_entry("second", 200)]);

        let second_snapshot = memtable.begin_flush();
        let snapshot_entries: usize = second_snapshot.values().map(Vec::len).sum();
        assert_eq!(snapshot_entries, 2);

        memtable.abort_flush(second_snapshot);
        let results = memtable.query(
            &sample_tenant(),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            100,
            true,
        );
        assert_eq!(total_entries(&results), 2);
    }

    #[test]
    fn the_range_decides_whether_a_row_on_end_is_returned() {
        let memtable = MemTable::new();
        memtable.insert(sample_tenant(), vec![sample_entry("on the boundary", 200)]);
        let rows = |range| {
            memtable
                .query(&sample_tenant(), &[], range, 100, true)
                .iter()
                .map(|stream| stream.entries.len())
                .sum::<usize>()
        };

        // This assertion used to be the whole test, under a name that made the
        // scan's inclusive `end` sound like the contract. It is not: it is what
        // a closed window means, and a log query does not ask for one.
        assert_eq!(rows(QueryTimeRange::closed(100, 200)), 1);
        assert_eq!(
            rows(QueryTimeRange::half_open(100, 200)),
            0,
            "a row at exactly `end` is outside a Loki log window"
        );
        assert_eq!(
            rows(QueryTimeRange::half_open(200, 300)),
            1,
            "`start` is included on both contracts"
        );
        assert_eq!(
            rows(QueryTimeRange::half_open(200, 200)),
            0,
            "an empty window returns nothing, not the row on its boundary"
        );
        assert_eq!(
            rows(QueryTimeRange::half_open(300, 100)),
            0,
            "an inverted window is empty rather than inverted"
        );
    }
}
