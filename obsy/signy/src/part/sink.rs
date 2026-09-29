/// Where a scan puts the rows it reads.
///
/// The read path used to hand `Vec<StreamResult>` up three hops — reader,
/// registry, executor — and every hop materialized the whole match set and
/// sorted it (`docs/VISION.md` II, "three materialize-and-sort hops"). A limit
/// could not reach the scan through that shape, because the only hop that knew
/// whether a row survived the pipeline was the last one, so
/// `normal_scan_limit` was `usize::MAX` for every query with a `| json`.
///
/// So the rows travel the other way. The scan offers them one at a time, the
/// sink holds the bounded result, and the scan asks the sink where its
/// [`RowSink::frontier_ns`] is and skips whatever it can prove is on the far
/// side of it. That is how a `limit` reaches a `| json | field=` query: not by
/// truncating raw rows before the pipeline ran, but by scanning only until
/// `limit` rows have *survived* it.
pub trait RowSink {
    /// `accept`, with the row's `| json` extraction already reconstructed
    /// from the part's `_pf:` columns when the scan was asked to project
    /// them. The default discards it; sinks that run a pipeline override
    /// this to spare the per-row parse.
    fn accept_extracted(
        &mut self,
        labels: &SharedLabels,
        entry: LogEntry,
        _extracted_json: Option<std::collections::BTreeMap<String, String>>,
    ) -> Result<(), String> {
        self.accept(labels, entry)
    }

    /// Offer one row. Ownership is taken: a row the sink does not want is
    /// dropped here rather than carried up a hop first.
    fn accept(&mut self, labels: &SharedLabels, entry: LogEntry) -> Result<(), String>;

    /// The timestamp a row must beat to enter the result, or `None` while the
    /// sink still has room for anything at all.
    ///
    /// Equality is beyond it, not on it: a row offered later cannot displace one
    /// already held at the same timestamp, because ties go to whichever arrived
    /// first. That is what makes a bounded sink return exactly what a stable
    /// sort followed by a truncation returned.
    fn frontier_ns(&self) -> Option<i64> {
        None
    }

    /// Rows the sink can still take, when it is bounded.
    ///
    /// A hint for how much a scan should decode at once, not a limit: with a
    /// pipeline stage the scan has to read more rows than this to fill it.
    fn remaining(&self) -> Option<usize> {
        None
    }

    /// Whether the sink can never accept anything, so there is nothing to read.
    /// True only for `limit = 0`, which a client may ask for and which used to
    /// scan the whole window before discarding it.
    fn is_closed(&self) -> bool {
        false
    }
}

/// Whether a row at `timestamp_ns` is on the far side of `frontier` and
/// therefore cannot enter the result.
pub fn beyond_frontier(frontier: Option<i64>, forward: bool, timestamp_ns: i64) -> bool {
    match frontier {
        None => false,
        Some(frontier) if forward => timestamp_ns >= frontier,
        Some(frontier) => timestamp_ns <= frontier,
    }
}

/// The same lifted to a span: true only when even the span's *best* timestamp is
/// beyond the frontier, so nothing inside it can enter the result. What lets a
/// scan reject a whole row group from `meta.json` without opening the Parquet
/// body, and a whole part from its tenant segment without opening the part.
pub fn span_beyond_frontier(
    frontier: Option<i64>,
    forward: bool,
    min_ts_ns: i64,
    max_ts_ns: i64,
) -> bool {
    beyond_frontier(
        frontier,
        forward,
        if forward { min_ts_ns } else { max_ts_ns },
    )
}

/// What a scan read, whatever the sink did with it.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScanStats {
    pub scanned_rows: usize,
    pub scanned_bytes: u64,
}

/// The `limit` rows nearest the query's direction, and nothing else.
///
/// Replaces "collect every match, sort it, throw all but `limit` away" — which
/// the read path did three times over. Ties are broken by arrival, so the rows
/// this returns are the rows a stable sort by timestamp followed by
/// `truncate(limit)` returned, and a row arriving later at the frontier's own
/// timestamp is rejected rather than swapped in.
pub struct TopKRows {
    limit: usize,
    forward: bool,
    held: Held,
    /// Rows offered so far, which is the tie-break: among equal timestamps the
    /// row that arrived first wins, exactly as a stable sort would have had it.
    arrived: u64,
}

/// A sink that has not yet filled has nothing to evict, so it has nothing to
/// rank. That matters because `limit` is `usize::MAX` on the paths that mean
/// "every row" and a million on the metric path: ranking every row there would
/// charge 24 bytes and a heap position per row to bound a set that is never
/// bounded.
enum Held {
    Growing(Vec<(SharedLabels, LogEntry)>),
    /// `limit` rows, worst first, so the row an arrival has to beat is the root
    /// and replacing it is O(log limit).
    Bounded(BinaryHeap<RankedRow>),
}

struct RankedRow {
    timestamp_ns: i64,
    arrived: u64,
    forward: bool,
    labels: SharedLabels,
    entry: LogEntry,
}

impl Ord for RankedRow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Greatest is worst, because `BinaryHeap` is a max-heap and the root has
        // to be the row the next arrival competes with. Worst is furthest from
        // the direction the query reads in, and among equal timestamps it is
        // whichever arrived later.
        let by_time = if self.forward {
            self.timestamp_ns.cmp(&other.timestamp_ns)
        } else {
            other.timestamp_ns.cmp(&self.timestamp_ns)
        };
        by_time.then_with(|| self.arrived.cmp(&other.arrived))
    }
}

impl PartialOrd for RankedRow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RankedRow {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for RankedRow {}

impl TopKRows {
    pub fn new(limit: usize, forward: bool) -> Self {
        Self {
            limit,
            forward,
            // Unreserved on purpose: a query whose row groups are all pruned
            // must not pay for a heap it never uses.
            held: Held::Growing(Vec::new()),
            arrived: 0,
        }
    }

    pub fn offer(&mut self, labels: &SharedLabels, entry: LogEntry) {
        if self.limit == 0 {
            return;
        }
        let forward = self.forward;
        let arrived = self.arrived;
        self.arrived = self.arrived.saturating_add(1);
        match &mut self.held {
            Held::Growing(rows) => {
                rows.push((labels.clone(), entry));
                if rows.len() == self.limit {
                    // Full: from here on every arrival displaces one, so the
                    // ranking starts to matter. Arrival order is the row's index,
                    // which is what a stable sort would have used.
                    let ranked = std::mem::take(rows)
                        .into_iter()
                        .enumerate()
                        .map(|(arrived, (labels, entry))| RankedRow {
                            timestamp_ns: entry.timestamp_ns,
                            arrived: arrived as u64,
                            forward,
                            labels,
                            entry,
                        })
                        .collect::<Vec<_>>();
                    self.held = Held::Bounded(BinaryHeap::from(ranked));
                }
            }
            Held::Bounded(heap) => {
                let row = RankedRow {
                    timestamp_ns: entry.timestamp_ns,
                    arrived,
                    forward,
                    labels: labels.clone(),
                    entry,
                };
                if heap.peek().is_some_and(|worst| &row < worst) {
                    heap.pop();
                    heap.push(row);
                }
            }
        }
    }

    pub fn frontier_ns(&self) -> Option<i64> {
        match &self.held {
            Held::Growing(_) => None,
            Held::Bounded(heap) => heap.peek().map(|worst| worst.timestamp_ns),
        }
    }

    pub fn remaining(&self) -> Option<usize> {
        (self.limit != usize::MAX).then(|| self.limit.saturating_sub(self.len()))
    }

    pub fn len(&self) -> usize {
        match &self.held {
            Held::Growing(rows) => rows.len(),
            Held::Bounded(heap) => heap.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_closed(&self) -> bool {
        self.limit == 0
    }

    /// The rows in the query's direction. The one sort the read path performs,
    /// over `limit` rows rather than over everything the window matched.
    pub fn into_rows(self) -> Vec<(SharedLabels, LogEntry)> {
        match self.held {
            Held::Growing(mut rows) => {
                // Stable, so rows sharing a timestamp keep the order the scan
                // produced them in — the same tie-break the ranking encodes.
                if self.forward {
                    rows.sort_by_key(|(_, entry)| entry.timestamp_ns);
                } else {
                    rows.sort_by_key(|(_, entry)| std::cmp::Reverse(entry.timestamp_ns));
                }
                rows
            }
            Held::Bounded(heap) => {
                let mut rows = heap.into_vec();
                // Ascending in the ranking is best first, and the ranking is
                // total because `arrived` is unique — deterministic without
                // being stable.
                rows.sort_unstable();
                rows.into_iter()
                    .map(|row| (row.labels, row.entry))
                    .collect()
            }
        }
    }

    pub fn into_stream_results(self) -> Vec<StreamResult> {
        group_by_labels(self.into_rows())
    }
}

impl RowSink for TopKRows {
    fn accept(&mut self, labels: &SharedLabels, entry: LogEntry) -> Result<(), String> {
        self.offer(labels, entry);
        Ok(())
    }

    fn frontier_ns(&self) -> Option<i64> {
        self.frontier_ns()
    }

    fn remaining(&self) -> Option<usize> {
        self.remaining()
    }

    fn is_closed(&self) -> bool {
        TopKRows::is_closed(self)
    }
}

/// Every row a scan produced, as `Row`s that carry their own tenant.
///
/// Merge's sink: it rewrites a part and needs all of it, so there is no limit
/// and no frontier. It exists so the merge path stops going through
/// `StreamResult`s — which grouped rows by label set only for the caller to
/// immediately flatten them again.
pub struct RowCollector<'a> {
    tenant: &'a TenantId,
    rows: Vec<Row>,
}

impl<'a> RowCollector<'a> {
    pub fn new(tenant: &'a TenantId) -> Self {
        Self {
            tenant,
            rows: Vec::new(),
        }
    }

    pub fn into_rows(self) -> Vec<Row> {
        self.rows
    }
}

impl RowSink for RowCollector<'_> {
    fn accept(&mut self, _labels: &SharedLabels, entry: LogEntry) -> Result<(), String> {
        self.rows.push(Row {
            tenant: self.tenant.clone(),
            timestamp_ns: entry.timestamp_ns,
            line: entry.line,
            structured_metadata: entry.structured_metadata,
        });
        Ok(())
    }
}
