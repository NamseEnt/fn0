/// Decoded row groups, kept for the next scan.
///
/// The per-admitted-group constant the rare shapes pay is the wide reader's
/// build — dictionary pages and column-chunk setup for every projected
/// column, priced at ~1.5 ms per group even when the selection keeps four
/// rows. A part is immutable, so rows decoded once can serve every later
/// scan; the cache holds exactly what a completed decode already produced
/// (`RecordBatch` clones are `Arc` refcounts — filling costs no second
/// decode), and it dies with its `PartReader`, which is dropped on
/// unregister and replace — no invalidation code at all.
///
/// Entries are keyed by `(row group, selection)`, not by row group alone,
/// because the comparison bed's queries — and Grafana's — are sub-windows:
/// the time page selection almost never keeps a whole group, so a
/// whole-group-only cache never fills. A decode that ran its selection to
/// completion is cached under that selection; a later scan producing the
/// same selection (a repeated window, or a different predicate resolving to
/// the same rows — `metadata_rare` after `json_field_rare`) replays the
/// batches without touching Parquet. The full-group selection is one key
/// among these, and the only one general enough to also serve *other*
/// selections by slicing and the narrow pass by direct evaluation.
///
/// The budget is one process-wide byte counter shared by every reader
/// (`LayoutTotals`' shape). A reader that pushes the total past the budget
/// evicts its own least-recently-used entries first; strict global LRU
/// across readers is deliberately not built — eviction here only decides
/// which *speedup* to keep.
static GLOBAL_BYTES: std::sync::OnceLock<Arc<std::sync::atomic::AtomicU64>> =
    std::sync::OnceLock::new();
/// 0 = off. Plain atomic so startup sets it before any reader opens and
/// tests can flip it without a set-once fight.
static GLOBAL_BUDGET: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn global_bytes() -> Arc<std::sync::atomic::AtomicU64> {
    GLOBAL_BYTES
        .get_or_init(|| Arc::new(std::sync::atomic::AtomicU64::new(0)))
        .clone()
}

/// Startup wiring: the budget every subsequently-opened reader's cache runs
/// under. Called before the registries load their parts.
pub fn configure_row_group_cache(budget: Option<u64>) {
    GLOBAL_BUDGET.store(budget.unwrap_or(0), std::sync::atomic::Ordering::Release);
}

/// The process-wide resident total, for the metrics endpoint.
pub fn row_group_cache_bytes() -> u64 {
    global_bytes().load(std::sync::atomic::Ordering::Acquire)
}

/// A selection normalized to alternating-run form: `(skip, rows)` with
/// adjacent same-kind runs merged and empty runs dropped, so every
/// `RowSelection` describing the same row set — and `None`, meaning all
/// rows — produces byte-identical keys.
pub(crate) type SelectionKey = Box<[(bool, u32)]>;

pub(crate) fn selection_key(
    selection: Option<&RowSelection>,
    group_rows: usize,
) -> SelectionKey {
    let Some(selection) = selection else {
        return Box::new([(false, group_rows as u32)]);
    };
    let mut runs: Vec<(bool, u32)> = Vec::new();
    for selector in selection.iter() {
        if selector.row_count == 0 {
            continue;
        }
        match runs.last_mut() {
            Some((skip, rows)) if *skip == selector.skip => {
                *rows += selector.row_count as u32;
            }
            _ => runs.push((selector.skip, selector.row_count as u32)),
        }
    }
    // A trailing skip is not part of the row set's identity: a selection
    // that skips the tail explicitly and one padded short of it select the
    // same rows, and the key is the row set. Dropping it is the one
    // canonical form (the full group becomes `[(false, group_rows)]`).
    if matches!(runs.last(), Some((true, _))) {
        runs.pop();
    }
    runs.into_boxed_slice()
}

/// Rebuild the `RowSelection` a key describes, padded back out to the
/// group. Round-tripping through `selection_key` is lossless on the row
/// set, which is what makes a cached narrow outcome equal to running the
/// pass.
pub(crate) fn selection_of(key: &SelectionKey, group_rows: usize) -> RowSelection {
    use parquet::arrow::arrow_reader::RowSelector;
    let mut selectors: Vec<RowSelector> = Vec::with_capacity(key.len() + 1);
    let mut described = 0usize;
    for &(skip, rows) in key.iter() {
        described += rows as usize;
        selectors.push(if skip {
            RowSelector::skip(rows as usize)
        } else {
            RowSelector::select(rows as usize)
        });
    }
    if described < group_rows {
        selectors.push(RowSelector::skip(group_rows - described));
    }
    RowSelection::from(selectors)
}

pub(crate) fn selected_rows_of(key: &SelectionKey) -> usize {
    key.iter()
        .filter(|(skip, _)| !skip)
        .map(|(_, rows)| *rows as usize)
        .sum()
}

/// The narrow pass's inputs, all deterministic per immutable part: the
/// group, the query window, the base selection it examined, and the
/// definitive predicates as (sm column, pf column, extracted, value) — the
/// column indices are schema positions, stable for the reader's lifetime.
/// The cached outcome is the selection the pass produced, or `None` for a
/// group it rejected — rejections are half the value, since a repeated
/// rare query pays the narrow pass mostly on groups holding nothing.
/// One definitive predicate's identity: (sm column, pf column, extracted,
/// value).
pub(crate) type NarrowPredicate = (Option<usize>, Option<usize>, bool, String);

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct NarrowQuery {
    pub rgu: u32,
    pub time: (i64, i64, bool),
    pub base: SelectionKey,
    pub defs: Box<[NarrowPredicate]>,
}

struct NarrowEntry {
    outcome: Option<SelectionKey>,
    bytes: u64,
    last_touch: u64,
}

fn narrow_entry_bytes(query: &NarrowQuery, outcome: &Option<SelectionKey>) -> u64 {
    let defs: usize = query
        .defs
        .iter()
        .map(|(_, _, _, value)| value.len() + 24)
        .sum();
    let runs = query.base.len() + outcome.as_ref().map_or(0, |key| key.len());
    (64 + defs + runs * 8) as u64
}

pub(crate) struct GroupCache {
    inner: std::sync::Mutex<GroupCacheInner>,
    shared_bytes: Arc<std::sync::atomic::AtomicU64>,
    budget_bytes: Option<u64>,
    /// Test-only proof that a serve came from subset slicing rather than a
    /// decode fallback, which answers identically and would pass equality.
    #[cfg(test)]
    pub(crate) subset_serves: std::sync::atomic::AtomicU64,
}

#[derive(Default)]
struct GroupCacheInner {
    groups: HashMap<(u32, SelectionKey), CachedGroup>,
    narrows: HashMap<NarrowQuery, NarrowEntry>,
    /// Monotonic touch counter; smallest = least recently used.
    clock: u64,
}

struct CachedGroup {
    batches: Arc<Vec<RecordBatch>>,
    bytes: u64,
    last_touch: u64,
}

/// A cached decode read out for serving: the batches and each batch's
/// starting row within the entry, so a row index maps straight to
/// `(batch, batch-relative row)`. For a full-group entry the indices are
/// group-absolute.
pub(crate) struct CachedGroupRead {
    pub batches: Arc<Vec<RecordBatch>>,
    pub offsets: Vec<usize>,
    pub total_rows: usize,
    pub bytes: u64,
}

impl GroupCache {
    pub(crate) fn new(
        shared_bytes: Arc<std::sync::atomic::AtomicU64>,
        budget_bytes: Option<u64>,
    ) -> Self {
        Self {
            inner: std::sync::Mutex::new(GroupCacheInner::default()),
            shared_bytes,
            budget_bytes,
            #[cfg(test)]
            subset_serves: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// The cache a production reader gets: the global counter and whatever
    /// budget startup configured (0 = disabled).
    pub(crate) fn from_global() -> Self {
        let budget = GLOBAL_BUDGET.load(std::sync::atomic::Ordering::Acquire);
        Self::new(global_bytes(), (budget > 0).then_some(budget))
    }

    pub(crate) fn enabled(&self) -> bool {
        self.budget_bytes.is_some()
    }

    #[cfg(test)]
    pub(crate) fn resident_bytes_for_test(&self) -> u64 {
        let inner = self.inner.lock().expect("group cache poisoned");
        inner.groups.values().map(|group| group.bytes).sum::<u64>()
            + inner.narrows.values().map(|entry| entry.bytes).sum::<u64>()
    }

    /// The whole-group entry, the one kind that can serve *any* selection
    /// (by slicing) and the narrow pass (by direct evaluation).
    pub(crate) fn get_full(&self, rgu: u32, group_rows: usize) -> Option<CachedGroupRead> {
        self.get(rgu, &selection_key(None, group_rows))
    }

    pub(crate) fn get(&self, rgu: u32, key: &SelectionKey) -> Option<CachedGroupRead> {
        let mut inner = self.inner.lock().expect("group cache poisoned");
        inner.clock += 1;
        let clock = inner.clock;
        let group = inner.groups.get_mut(&(rgu, key.clone()))?;
        group.last_touch = clock;
        Some(read_of(group))
    }

    /// A remembered narrow-pass run. `Some(outcome)` is a hit; the inner
    /// `None` means the pass rejected the group outright.
    pub(crate) fn get_narrow(&self, query: &NarrowQuery) -> Option<Option<SelectionKey>> {
        let mut inner = self.inner.lock().expect("group cache poisoned");
        inner.clock += 1;
        let clock = inner.clock;
        let entry = inner.narrows.get_mut(query)?;
        entry.last_touch = clock;
        Some(entry.outcome.clone())
    }

    pub(crate) fn insert_narrow(&self, query: NarrowQuery, outcome: Option<SelectionKey>) {
        let Some(budget) = self.budget_bytes else {
            return;
        };
        let mut inner = self.inner.lock().expect("group cache poisoned");
        if inner.narrows.contains_key(&query) {
            return;
        }
        inner.clock += 1;
        let clock = inner.clock;
        let bytes = narrow_entry_bytes(&query, &outcome);
        inner.narrows.insert(
            query.clone(),
            NarrowEntry {
                outcome,
                bytes,
                last_touch: clock,
            },
        );
        let mut total = self
            .shared_bytes
            .fetch_add(bytes, std::sync::atomic::Ordering::AcqRel)
            .saturating_add(bytes);
        // Entries are ~100 bytes, so pressure here almost always means the
        // batch entries hold the budget; still, narrow results evict their
        // own kind oldest-first rather than growing without bound.
        while total > budget && inner.narrows.len() > 1 {
            let Some(victim) = inner
                .narrows
                .iter()
                .filter(|(key, _)| **key != query)
                .min_by_key(|(_, entry)| entry.last_touch)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            let removed = inner.narrows.remove(&victim).expect("chosen from the map");
            self.shared_bytes
                .fetch_sub(removed.bytes, std::sync::atomic::Ordering::AcqRel);
            total = total.saturating_sub(removed.bytes);
        }
        // Same fence as the batch map: never leave the shared total over
        // budget on this reader's account.
        if total > budget {
            let removed = inner.narrows.remove(&query).expect("inserted above");
            self.shared_bytes
                .fetch_sub(removed.bytes, std::sync::atomic::Ordering::AcqRel);
        }
    }

    /// Insert a completed decode. The batches must hold exactly the key's
    /// selected rows, in order — the caller only inserts when the decode
    /// ran its selection to the group's end.
    pub(crate) fn insert(&self, rgu: u32, key: SelectionKey, batches: Vec<RecordBatch>) {
        let Some(budget) = self.budget_bytes else {
            return;
        };
        let _arena = crate::memprof::enter(crate::memprof::Arena::RowGroupCache);
        let batches = shrink_batches(batches);
        let bytes: u64 = batches
            .iter()
            .map(|batch| batch.get_array_memory_size() as u64)
            .sum();
        if bytes > budget {
            return;
        }
        let mut inner = self.inner.lock().expect("group cache poisoned");
        let entry = (rgu, key);
        if inner.groups.contains_key(&entry) {
            return;
        }
        inner.clock += 1;
        let clock = inner.clock;
        inner.groups.insert(
            entry.clone(),
            CachedGroup {
                batches: Arc::new(batches),
                bytes,
                last_touch: clock,
            },
        );
        let mut total = self
            .shared_bytes
            .fetch_add(bytes, std::sync::atomic::Ordering::AcqRel)
            .saturating_add(bytes);
        // Over budget: this reader gives back its own least-recently-used
        // entries, newest-inserted excluded until it is the only one left.
        while total > budget && inner.groups.len() > 1 {
            let Some(victim) = inner
                .groups
                .iter()
                .filter(|(key, _)| **key != entry)
                .min_by_key(|(_, group)| group.last_touch)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            let removed = inner.groups.remove(&victim).expect("chosen from the map");
            self.shared_bytes
                .fetch_sub(removed.bytes, std::sync::atomic::Ordering::AcqRel);
            total = total.saturating_sub(removed.bytes);
        }
        // Still over: the bulk is held by other readers, whose entries this
        // one cannot touch. Retract the insert rather than hold the total
        // over its budget — measured live at 284.7 MiB against a 256 MiB
        // budget before this fence, an 11% overshoot under concurrent
        // readers. What remains possible is the transient of inserts racing
        // the counter, bounded by the concurrent-scan cap times one entry.
        if total > budget {
            let removed = inner.groups.remove(&entry).expect("inserted above");
            self.shared_bytes
                .fetch_sub(removed.bytes, std::sync::atomic::Ordering::AcqRel);
        }
    }
}

impl Drop for GroupCache {
    fn drop(&mut self) {
        let held: u64 = self
            .inner
            .get_mut()
            .map(|inner| {
                inner.groups.values().map(|group| group.bytes).sum::<u64>()
                    + inner.narrows.values().map(|entry| entry.bytes).sum::<u64>()
            })
            .unwrap_or(0);
        if held > 0 {
            self.shared_bytes
                .fetch_sub(held, std::sync::atomic::Ordering::AcqRel);
        }
    }
}

fn read_of(group: &CachedGroup) -> CachedGroupRead {
    let mut offsets = Vec::with_capacity(group.batches.len());
    let mut at = 0usize;
    for batch in group.batches.iter() {
        offsets.push(at);
        at += batch.num_rows();
    }
    CachedGroupRead {
        batches: group.batches.clone(),
        offsets,
        total_rows: at,
        bytes: group.bytes,
    }
}

/// Give back the slack Arrow's doubling left in a decoded batch.
///
/// Parquet's byte-array decoders grow their value buffer by doubling — a heap
/// profile names the frame `OffsetBuffer::extend_from_dictionary` — so a batch
/// arrives holding up to twice the bytes its rows need, and
/// `get_array_memory_size`, the unit this cache's budget is counted in, charges
/// the capacity. One `realloc` per column at insert time stops the budget being
/// spent on the overshoot; the entry is then measured at what it really holds.
///
/// Arrow shrinks only an exclusively-owned buffer and makes a shared one a
/// no-op, which is why the backwards scan shrinks before it clones.
pub(crate) fn shrink_batches(batches: Vec<RecordBatch>) -> Vec<RecordBatch> {
    batches
        .into_iter()
        .map(|batch| {
            let (schema, mut columns, row_count) = batch.into_parts();
            for column in &mut columns {
                column.shrink_to_fit();
            }
            // No length changed, so the batch that went in is the batch that
            // comes out; the row count is carried explicitly because a
            // column-less batch would otherwise lose it.
            RecordBatch::try_new_with_options(
                schema,
                columns,
                &RecordBatchOptions::new().with_row_count(Some(row_count)),
            )
            .expect("shrinking a batch changes no length")
        })
        .collect()
}
