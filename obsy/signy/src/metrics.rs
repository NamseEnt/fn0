use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Low-cardinality process metrics shared by handlers and background workers.
/// Counters are deliberately monotonic; `/metrics` renders the current
/// snapshot in Prometheus text format.
pub struct RuntimeMetrics {
    pub ingest_requests: AtomicU64,
    pub ingest_errors: AtomicU64,
    /// Requests refused because the durable path was already behind. Distinct
    /// from `ingest_errors`: this one says the server is healthy and saying no,
    /// which is the signal an operator scales or tunes flush on.
    pub ingest_throttled: AtomicU64,
    /// Queries refused by the tenant's own concurrency limit. Separate from
    /// `query_errors`, which counts queries this instance failed to answer:
    /// this one says the instance was willing and the tenant was over what it
    /// was sold.
    pub query_quota_rejected: AtomicU64,
    /// Records a collected batch carried that this instance will never accept —
    /// a body it cannot decode, a tenant at its storage limit, a payload past a
    /// limit. The collect route drops them instead of answering an error,
    /// because collecty would only halve the batch to find them and drop them
    /// itself; these two counters are the only place that loss is visible.
    pub collect_dropped_records: AtomicU64,
    pub collect_dropped_bytes: AtomicU64,
    /// Records a collecty sent again that this instance already had. Not a
    /// loss and not an error: it is what a resend after a crash looks like,
    /// and a number that stays flat while a collector restarts is the sign
    /// that the numbering is not doing its job.
    pub collect_skipped_records: AtomicU64,
    /// Resources thrown away because nothing said whose they were.
    ///
    /// The tenant rides in a resource attribute, and an export naming none
    /// this instance serves is dropped rather than refused: the status code an
    /// ingest answers says whether the body arrived, and nothing about who
    /// sent it. These three counters are the only place that loss is visible,
    /// and they are apart because they send an operator to three different
    /// places — an exporter never configured, an exporter configured with a
    /// value this engine cannot store, and a tenant the control plane has not
    /// onboarded.
    pub ingest_dropped_no_tenant: AtomicU64,
    pub ingest_dropped_invalid_tenant: AtomicU64,
    pub ingest_dropped_tenant_not_served: AtomicU64,
    pub ingest_dropped_tenant_not_served_logs: AtomicU64,
    pub ingest_dropped_tenant_not_served_traces: AtomicU64,
    pub ingest_dropped_tenant_not_served_metrics: AtomicU64,
    /// Writes refused because the tenant is already storing everything its plan
    /// sells. It clears only when retention retires parts, which is why the
    /// refusal carries a long Retry-After.
    pub storage_limit_rejected: AtomicU64,
    /// Records and entries the last startup replayed out of the WAL.
    ///
    /// Delivery is at-least-once: the checkpoint advances after a flush, so a
    /// crash in between leaves records that are already durable in parts and
    /// replay writes them again. The trade is deliberate, but until these
    /// existed nothing said it had happened — an operator could not tell a
    /// restart that duplicated nothing from one that duplicated a minute of
    /// logs, and neither could anyone reading a query result. They are set once
    /// at startup and never move, so a non-zero value describes this process's
    /// own recovery rather than a rate.
    pub wal_replayed_records: AtomicU64,
    pub wal_replayed_entries: AtomicU64,
    /// Point-in-time backlog gauges, published by the workers that already walk
    /// the structures they describe. Computing them per scrape instead was
    /// O(parts × tenants) of work on an unauthenticated endpoint, and the
    /// numbers are only as fresh as a worker tick either way.
    pub merge_debt_parts: AtomicU64,
    pub unknown_tenants: AtomicU64,
    pub flush_success: AtomicU64,
    pub flush_errors: AtomicU64,
    pub merge_success: AtomicU64,
    pub merge_errors: AtomicU64,
    /// Merge groups abandoned because another writer replaced their inputs
    /// first. Benign on its own — retention retiring an expired part races with
    /// merge by design, nothing was written, and the next tick sees the new
    /// state. It is counted rather than only logged because a number that keeps
    /// rising while `merge_success` makes no progress is the one way to see the
    /// registry failing to converge on the manifest.
    pub merge_inputs_changed: AtomicU64,
    pub retention_success: AtomicU64,
    pub retention_errors: AtomicU64,
    pub retention_expired_rows_dropped: AtomicU64,
    pub retention_parts_rewritten: AtomicU64,
    /// Retention-only merge groups whose inputs could not be read, so the
    /// expired rows in them stay on disk. A number that keeps rising means a
    /// part is permanently too large for `merge_max_memory_bytes`.
    pub retention_rewrite_skipped: AtomicU64,
    pub object_store_gc_success: AtomicU64,
    pub object_store_gc_errors: AtomicU64,
    pub orphan_collect_success: AtomicU64,
    pub orphan_collect_errors: AtomicU64,
    pub orphan_objects_removed: AtomicU64,
    pub orphan_bytes_removed: AtomicU64,
    /// Deletions the store refused. The objects keep their ledger entries, so
    /// a later pass retries them.
    pub orphan_delete_errors: AtomicU64,
    /// Completed walks of every part prefix. Until the first one finishes, an
    /// entry for an object that no longer exists is still in the ledger.
    pub orphan_scan_cycles: AtomicU64,
    /// What the last pass found deletable, whether or not its budget let it
    /// delete that much. This is the dry run's answer.
    pub orphan_candidate_objects: AtomicU64,
    pub orphan_candidate_bytes: AtomicU64,
    pub orphan_ledger_entries: AtomicU64,
    pub catalog_prune_success: AtomicU64,
    pub catalog_prune_errors: AtomicU64,
    pub catalog_objects_pruned: AtomicU64,
    pub query_success: AtomicU64,
    pub query_errors: AtomicU64,
    pub query_scanned_rows: AtomicU64,
    pub query_scanned_bytes: AtomicU64,
    pub query_latency_ns: AtomicU64,
    /// Bucketed query latency, so an operator can compute a real quantile.
    ///
    /// The cumulative `*_latency_ns_total` counters only ever yielded a mean,
    /// and every target in the plan documents is written as p95/p99 — numbers
    /// that were literally not derivable from what this endpoint exposed.
    /// One per endpoint. The single histogram this replaced could not say
    /// which endpoint was slow, and did not cover the metric path at all.
    pub query_latency: [LatencyHistogram; QueryEndpoint::ALL.len()],
    /// Scans holding a scheduler permit right now.
    pub query_scans_in_flight: AtomicU64,
    /// High-water mark of that, since start.
    ///
    /// `peak_materialized_bytes` is dominated by
    /// `max_concurrent_query_scans × max_query_memory_bytes` — four of the five
    /// configured gigabytes at the defaults — and no load run has ever
    /// exercised it, because every run so far has been ingest-dominated. A run
    /// that claims to have reached the limit has to show it, and a gauge
    /// scraped every few hundred milliseconds cannot: a burst that fills the
    /// scheduler and drains again between two scrapes is invisible to sampling.
    /// The mark is therefore recorded as it happens.
    pub query_scans_in_flight_peak: AtomicU64,
    /// Scans that found every slot taken and had to wait for one.
    ///
    /// This is the stronger claim of the two. A peak equal to the limit says
    /// the scheduler was full at an instant; a nonzero queued count says it was
    /// full and someone was behind it, which is the state the memory budget is
    /// written against.
    pub query_scans_queued: AtomicU64,
    pub query_scan_queue_wait_ns: AtomicU64,
    pub remote_restore_success: AtomicU64,
    pub remote_restore_errors: AtomicU64,
    pub remote_restore_latency_ns: AtomicU64,
    pub remote_restore_latency: LatencyHistogram,
    pub cache_evictions: AtomicU64,
    /// Where a flush pass's time goes, phase by phase.
    ///
    /// The capacity of this engine at a 2 GiB budget is set here and not in the
    /// WAL: the rate ladder of 2026-08-13 pinned `memtable_buffered` at its
    /// limit while the WAL backlog sat at 3% of its own, so what an operator
    /// can offer is decided by how fast a memtable becomes parts. These are the
    /// same shape as the journal writer's, and for the same reason — the push
    /// tail was argued about for a week from the client's side because nothing
    /// inside the process could say which phase was spending the time.
    pub flush: FlushMetrics,
}

/// One flush pass, split where it hands work to something else.
#[derive(Default)]
pub struct FlushMetrics {
    /// `journal.checkpoint()` from the flush loop's side. The checkpoint runs
    /// **in the writer task**, so this includes queueing behind whatever pushes
    /// are being fsynced — the flush's own wait on the ingest path.
    pub checkpoint_wait: LatencyHistogram,
    /// Materializing rows and writing parts: sort, dedup, Arrow build, Parquet
    /// encode with zstd, blooms, `index.bin`, fsync, fadvise.
    pub build: LatencyHistogram,
    /// Re-opening what was just written, which validates checksums over all of
    /// it. Deliberately outside the visibility lock, and therefore a candidate
    /// for the largest phase nobody has ever measured.
    pub open: LatencyHistogram,
    /// The write-locked visibility transition: register the parts, commit the
    /// memtable. Every queued query waits out this one.
    pub visibility: LatencyHistogram,
    /// Advancing the journal checkpoint, and WAL compaction when it fires.
    pub advance_checkpoint: LatencyHistogram,
    pub rows: AtomicU64,
    pub parts: AtomicU64,
}

impl RuntimeMetrics {
    pub fn new() -> Self {
        Self {
            ingest_requests: AtomicU64::new(0),
            ingest_errors: AtomicU64::new(0),
            ingest_throttled: AtomicU64::new(0),
            collect_dropped_records: AtomicU64::new(0),
            collect_dropped_bytes: AtomicU64::new(0),
            collect_skipped_records: AtomicU64::new(0),
            ingest_dropped_no_tenant: AtomicU64::new(0),
            ingest_dropped_invalid_tenant: AtomicU64::new(0),
            ingest_dropped_tenant_not_served: AtomicU64::new(0),
            ingest_dropped_tenant_not_served_logs: AtomicU64::new(0),
            ingest_dropped_tenant_not_served_traces: AtomicU64::new(0),
            ingest_dropped_tenant_not_served_metrics: AtomicU64::new(0),
            query_quota_rejected: AtomicU64::new(0),
            storage_limit_rejected: AtomicU64::new(0),
            wal_replayed_records: AtomicU64::new(0),
            wal_replayed_entries: AtomicU64::new(0),
            merge_debt_parts: AtomicU64::new(0),
            unknown_tenants: AtomicU64::new(0),
            flush_success: AtomicU64::new(0),
            flush_errors: AtomicU64::new(0),
            merge_success: AtomicU64::new(0),
            merge_errors: AtomicU64::new(0),
            merge_inputs_changed: AtomicU64::new(0),
            retention_success: AtomicU64::new(0),
            retention_errors: AtomicU64::new(0),
            retention_expired_rows_dropped: AtomicU64::new(0),
            retention_parts_rewritten: AtomicU64::new(0),
            retention_rewrite_skipped: AtomicU64::new(0),
            object_store_gc_success: AtomicU64::new(0),
            object_store_gc_errors: AtomicU64::new(0),
            orphan_collect_success: AtomicU64::new(0),
            orphan_collect_errors: AtomicU64::new(0),
            orphan_objects_removed: AtomicU64::new(0),
            orphan_bytes_removed: AtomicU64::new(0),
            orphan_delete_errors: AtomicU64::new(0),
            orphan_scan_cycles: AtomicU64::new(0),
            orphan_candidate_objects: AtomicU64::new(0),
            orphan_candidate_bytes: AtomicU64::new(0),
            orphan_ledger_entries: AtomicU64::new(0),
            catalog_prune_success: AtomicU64::new(0),
            catalog_prune_errors: AtomicU64::new(0),
            catalog_objects_pruned: AtomicU64::new(0),
            query_success: AtomicU64::new(0),
            query_errors: AtomicU64::new(0),
            query_scanned_rows: AtomicU64::new(0),
            query_scanned_bytes: AtomicU64::new(0),
            query_latency_ns: AtomicU64::new(0),
            query_latency: std::array::from_fn(|_| LatencyHistogram::default()),
            query_scans_in_flight: AtomicU64::new(0),
            query_scans_in_flight_peak: AtomicU64::new(0),
            query_scans_queued: AtomicU64::new(0),
            query_scan_queue_wait_ns: AtomicU64::new(0),
            remote_restore_success: AtomicU64::new(0),
            remote_restore_errors: AtomicU64::new(0),
            remote_restore_latency_ns: AtomicU64::new(0),
            remote_restore_latency: LatencyHistogram::default(),
            cache_evictions: AtomicU64::new(0),
            flush: FlushMetrics::default(),
        }
    }

    pub fn add_duration(target: &AtomicU64, duration: Duration) {
        target.fetch_add(
            duration.as_nanos().min(u64::MAX as u128) as u64,
            Ordering::Relaxed,
        );
    }

    pub fn observe(histogram: &LatencyHistogram, counter: &AtomicU64, duration: Duration) {
        Self::add_duration(counter, duration);
        histogram.observe(duration);
    }

    /// Time one query against the endpoint it arrived at.
    ///
    /// The aggregate `query_latency_ns` counter stays fed from here so the two
    /// cannot disagree about how many queries ran, and so the endpoint split
    /// is an addition rather than a replacement for anything already scraped.
    pub fn observe_query(&self, endpoint: QueryEndpoint, duration: Duration) {
        Self::add_duration(&self.query_latency_ns, duration);
        self.query_latency[endpoint as usize].observe(duration);
    }

    pub fn load(target: &AtomicU64) -> u64 {
        target.load(Ordering::Relaxed)
    }

    /// Records that a scan queued behind a full scheduler, and for how long.
    pub fn record_scan_queue_wait(&self, waited: Duration) {
        self.query_scans_queued.fetch_add(1, Ordering::Relaxed);
        Self::add_duration(&self.query_scan_queue_wait_ns, waited);
    }
}

/// Holds the scan-occupancy gauges up for as long as a scan holds its permit.
///
/// A guard rather than a pair of calls because the permit outlives every early
/// return on the query path — a cancelled request, a timeout, a scan that
/// exceeds its memory budget — and an occupancy that only decrements on the
/// success path would climb to the limit and stay there, reporting saturation
/// that had long since drained.
pub struct ScanOccupancy {
    metrics: std::sync::Arc<RuntimeMetrics>,
}

impl ScanOccupancy {
    pub fn enter(metrics: std::sync::Arc<RuntimeMetrics>) -> Self {
        let in_flight = metrics
            .query_scans_in_flight
            .fetch_add(1, Ordering::Relaxed)
            + 1;
        metrics
            .query_scans_in_flight_peak
            .fetch_max(in_flight, Ordering::Relaxed);
        Self { metrics }
    }
}

impl Drop for ScanOccupancy {
    fn drop(&mut self) {
        self.metrics
            .query_scans_in_flight
            .fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for RuntimeMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Prometheus-shaped latency buckets.
///
/// Cumulative (`le`) counts, which is the only shape `histogram_quantile` can
/// read. The bounds span 1 ms to 30 s: below that a query is instant and above
/// it the resource limits have already refused the work, so finer resolution at
/// either end would only cost cardinality.
pub const LATENCY_BUCKET_BOUNDS_MS: [f64; 12] = [
    1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1_000.0, 2_500.0, 5_000.0, 30_000.0,
];

/// Which endpoint a query arrived at, as the one dimension
/// `signy_query_latency_ms` carries.
///
/// A fixed set, not a string: the label's cardinality has to be a property of
/// this binary rather than of what a client sends, which is the same reason
/// tenants are not a metric label.
///
/// `Volume` is here because `index/volume` reduces to `bytes_over_time` and is
/// answered by the metric evaluator — it is a different endpoint arriving at
/// the same machinery, and a Grafana dashboard slow because volume is slow
/// cannot be told from a slow `query_range` without this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryEndpoint {
    Query,
    Tail,
    Logs,
    Histogram,
    Traces,
    TraceById,
    MetricQuery,
    MetricInstant,
    MetricQuantile,
    MetricNames,
    MetricLabels,
    MetricLabelValues,
    MetricSeries,
}

impl QueryEndpoint {
    pub const ALL: [Self; 13] = [
        Self::Query,
        Self::Tail,
        Self::Logs,
        Self::Histogram,
        Self::Traces,
        Self::TraceById,
        Self::MetricQuery,
        Self::MetricInstant,
        Self::MetricQuantile,
        Self::MetricNames,
        Self::MetricLabels,
        Self::MetricLabelValues,
        Self::MetricSeries,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            // The internal unified-query path (restore probes and the like).
            Self::Query => "query",
            Self::Tail => "tail",
            Self::Logs => "logs",
            Self::Histogram => "logs_histogram",
            Self::Traces => "traces",
            Self::TraceById => "trace_by_id",
            Self::MetricQuery => "metrics_query",
            Self::MetricInstant => "metrics_instant",
            Self::MetricQuantile => "metrics_quantile",
            Self::MetricNames => "metrics_names",
            Self::MetricLabels => "metrics_labels",
            Self::MetricLabelValues => "metrics_label_values",
            Self::MetricSeries => "metrics_series",
        }
    }
}

#[derive(Default)]
pub struct LatencyHistogram {
    buckets: [AtomicU64; LATENCY_BUCKET_BOUNDS_MS.len()],
    count: AtomicU64,
    sum_ms: AtomicU64,
}

impl LatencyHistogram {
    /// `const` so a histogram can live in a `static`, which is how the flush
    /// build's sub-phases are measured: they sit inside `part::format`, which
    /// takes no metrics handle and is called from both the flush and the merge.
    pub const fn new() -> Self {
        Self {
            buckets: [const { AtomicU64::new(0) }; LATENCY_BUCKET_BOUNDS_MS.len()],
            count: AtomicU64::new(0),
            sum_ms: AtomicU64::new(0),
        }
    }

    pub fn observe(&self, duration: Duration) {
        let millis = duration.as_secs_f64() * 1_000.0;
        for (index, bound) in LATENCY_BUCKET_BOUNDS_MS.iter().enumerate() {
            if millis <= *bound {
                // Cumulative: an observation belongs to its own bucket and to
                // every wider one, which is what makes the series monotonic in
                // `le` and therefore readable by `histogram_quantile`.
                self.buckets[index].fetch_add(1, Ordering::Relaxed);
            }
        }
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum_ms
            .fetch_add(millis.round().max(0.0) as u64, Ordering::Relaxed);
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// `(le bound, cumulative count)` pairs plus the `+Inf` total and sum.
    pub fn render(&self, name: &str) -> String {
        self.render_labeled(name, "")
    }

    /// The same, with a label set every series carries. `labels` is a rendered
    /// `k="v"` list without braces; `le` is appended to it on the buckets,
    /// which is the order Prometheus's own exposition uses and the order
    /// `histogram_quantile` needs to see.
    pub fn render_labeled(&self, name: &str, labels: &str) -> String {
        let separator = if labels.is_empty() { "" } else { "," };
        let scalar = if labels.is_empty() {
            String::new()
        } else {
            format!("{{{labels}}}")
        };
        let mut out = String::new();
        for (index, bound) in LATENCY_BUCKET_BOUNDS_MS.iter().enumerate() {
            out.push_str(&format!(
                "{name}_bucket{{{labels}{separator}le=\"{bound}\"}} {}\n",
                self.buckets[index].load(Ordering::Relaxed)
            ));
        }
        let count = self.count.load(Ordering::Relaxed);
        out.push_str(&format!(
            "{name}_bucket{{{labels}{separator}le=\"+Inf\"}} {count}\n"
        ));
        out.push_str(&format!(
            "{name}_sum{scalar} {}\n",
            self.sum_ms.load(Ordering::Relaxed)
        ));
        out.push_str(&format!("{name}_count{scalar} {count}\n"));
        out
    }
}
