/// The operator endpoints: readiness and the Prometheus text scrape.
pub async fn ready(
    State(state): State<Arc<AppState>>,
) -> Result<&'static str, (StatusCode, String)> {
    if state.shutdown.is_fenced() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "fenced by a newer writer; this instance no longer owns the object-store prefix"
                .to_string(),
        ));
    }
    if state.shutdown.is_draining() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "draining for shutdown: force_flush_complete={}, pending_flush_bytes={}",
                state.shutdown.is_flush_complete(),
                state.shutdown.pending_flush_bytes(),
            ),
        ));
    }
    let mut unavailable = Vec::new();
    if !state.journal.is_healthy() {
        unavailable.push("journal writer");
    }
    if !state.flush_healthy.load(Ordering::Acquire) {
        unavailable.push("flush worker");
    }
    if !state.merge_healthy.load(Ordering::Acquire) {
        unavailable.push("merge worker");
    }
    if !state.retention_healthy.load(Ordering::Acquire) {
        unavailable.push("retention worker");
    }
    if let Some(cache) = &state.remote_cache {
        if !cache.is_remote_healthy() {
            unavailable.push("object store");
        }
        if !cache.is_cache_healthy() {
            unavailable.push("local cache");
        }
    }

    if unavailable.is_empty() {
        Ok("ready")
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{} unavailable", unavailable.join(", ")),
        ))
    }
}


/// The allocator's own report, as text.
///
/// Deliberately not on `/metrics`: it is prose with a per-size-class table in
/// it, and the question it answers -- which size classes hold the slack -- is
/// asked once while looking at a problem, not every scrape.
pub async fn allocator_report() -> String {
    crate::allocator_stats::report()
}

/// Write a heap profile next to the data and answer with its path.
///
/// The directory is the engine's own, not the caller's: a route that took a
/// path would be a way to write a file anywhere the process can reach, and the
/// question this answers does not need one.
pub async fn allocator_profile(State(state): State<Arc<AppState>>) -> String {
    match crate::allocator_stats::dump_profile(&state.config.data_dir.join("heap-profiles")) {
        Ok(path) => format!("wrote {path}\n"),
        Err(error) => format!("no profile: {error}\n"),
    }
}

pub async fn metrics(State(state): State<Arc<AppState>>) -> String {
    let mem = state.memtable.global_stats();
    let disk = state.parts.global_stats();
    let remote_healthy = state
        .remote_cache
        .as_ref()
        .is_none_or(|cache| cache.is_remote_healthy());
    let cache_healthy = state
        .remote_cache
        .as_ref()
        .is_none_or(|cache| cache.is_cache_healthy());
    let wal_backlog_bytes = state.journal.wal_backlog_bytes();
    // Both of these are published by the workers that already hold the
    // snapshots they describe. A scrape must not be able to ask for a walk of
    // every part's tenant index.
    let merge_debt_parts = m_merge_debt(&state);
    // Read rather than computed: the registry maintains these as its set
    // changes, so they are current whether or not any worker has ticked.
    let layout = state.parts.layout_totals();
    let policy = tenant_policy_gauges(&state);
    let m = &state.metrics;
    let mut body = format!(
        "# TYPE signy_memtable_entries gauge\n\
signy_memtable_entries {}\n\
# TYPE signy_memtable_bytes gauge\n\
signy_memtable_bytes {}\n\
# TYPE signy_part_entries gauge\n\
signy_part_entries {}\n\
# TYPE signy_part_bytes gauge\n\
signy_part_bytes {}\n\
# TYPE signy_part_count gauge\n\
signy_part_count {}\n\
# TYPE signy_trace_part_count gauge\n\
signy_trace_part_count {}\n\
# HELP signy_metric_part_count Open metric parts. Each holds its whole series catalog resident, offloaded body or not, so this is the multiplier on a series' catalog residency.\n\
# TYPE signy_metric_part_count gauge\n\
signy_metric_part_count {}\n\
# HELP signy_capacity_probe Whether this process is running the dangerous raw-capacity probe (1) or normal guarded mode (0).\n\
# TYPE signy_capacity_probe gauge\n\
signy_capacity_probe {}\n\
# TYPE signy_remote_healthy gauge\n\
signy_remote_healthy {}\n\
# HELP signy_remote_consecutive_failures Object-store failures since the last success. The health flag hides these below its threshold, so this is where a degrading store shows before it is declared down.\n\
# TYPE signy_remote_consecutive_failures gauge\n\
signy_remote_consecutive_failures {}\n\
# TYPE signy_cache_healthy gauge\n\
signy_cache_healthy {}\n\
# TYPE signy_wal_backlog_bytes gauge\n\
signy_wal_backlog_bytes {}\n\
# HELP signy_inflight_push_bytes Request bodies admitted and not yet answered, bounded by max_inflight_push_bytes. Counted at admission because a body is already resident by the time a handler sees it.\n\
# TYPE signy_inflight_push_bytes gauge\n\
signy_inflight_push_bytes {}\n\
# HELP signy_data_dir_free_bytes Free space on the filesystem holding the data directory, as the unprivileged user sees it. Ingest is refused below SIGNY_MIN_FREE_DISK_BYTES.\n\
# TYPE signy_data_dir_free_bytes gauge\n\
signy_data_dir_free_bytes {}\n\
# HELP signy_data_dir_total_bytes Size of that filesystem, so the gauge above can be read as a fraction without knowing the volume.\n\
# TYPE signy_data_dir_total_bytes gauge\n\
signy_data_dir_total_bytes {}\n\
# TYPE signy_merge_debt_parts gauge\n\
signy_merge_debt_parts {}\n\
# HELP signy_part_tenant_segments (tenant, part) pairs. The shared-part layout spends a row group, two blooms and a metadata segment per pair.\n\
# TYPE signy_part_tenant_segments gauge\n\
signy_part_tenant_segments {}\n\
# HELP signy_part_sidecar_resident_bytes Bloom and stream-index bytes resident for open parts. The bloom half is bounded by sidecar_cache_max_bytes and evicted LRU; the stream-index half stays resident per part.\n\
# TYPE signy_part_sidecar_resident_bytes gauge\n\
signy_part_sidecar_resident_bytes {}\n\
# HELP signy_row_group_cache_bytes Decoded row groups held for reuse across scans, bounded by row_group_cache_max_bytes.\n\
# TYPE signy_row_group_cache_bytes gauge\n\
signy_row_group_cache_bytes {}\n\
# HELP signy_part_meta_bytes Total meta.json across parts, which startup parses before serving.\n\
# TYPE signy_part_meta_bytes gauge\n\
signy_part_meta_bytes {}\n\
# TYPE signy_ingest_requests_total counter\n\
signy_ingest_requests_total {}\n\
# TYPE signy_ingest_errors_total counter\n\
signy_ingest_errors_total {}\n\
# TYPE signy_ingest_throttled_total counter\n\
signy_ingest_throttled_total {}\n\
# HELP signy_collect_dropped_records_total Records a collected batch carried that will never be accepted, dropped rather than sent back for the collector to rediscover.\n\
# TYPE signy_collect_dropped_records_total counter\n\
signy_collect_dropped_records_total {}\n\
# HELP signy_collect_dropped_bytes_total Payload bytes behind signy_collect_dropped_records_total.\n\
# TYPE signy_collect_dropped_bytes_total counter\n\
signy_collect_dropped_bytes_total {}\n\
# HELP signy_collect_skipped_records_total Records a collecty sent again that this instance already had. A resend after a crash, skipped rather than stored twice.\n\
# TYPE signy_collect_skipped_records_total counter\n\
signy_collect_skipped_records_total {}\n\
# HELP signy_ingest_dropped_resources_total Resources dropped because they named no tenant this instance serves. An ingest answers whether the body arrived and nothing about who sent it, so this is the only place the loss shows.\n\
# TYPE signy_ingest_dropped_resources_total counter\n\
signy_ingest_dropped_resources_total{{reason=\"no_tenant\"}} {}\n\
signy_ingest_dropped_resources_total{{reason=\"invalid_tenant\"}} {}\n\
signy_ingest_dropped_resources_total{{reason=\"tenant_not_served\"}} {}\n\
signy_ingest_dropped_resources_total{{reason=\"tenant_not_served\",signal=\"logs\"}} {}\n\
signy_ingest_dropped_resources_total{{reason=\"tenant_not_served\",signal=\"traces\"}} {}\n\
signy_ingest_dropped_resources_total{{reason=\"tenant_not_served\",signal=\"metrics\"}} {}\n\
# HELP signy_query_quota_rejected_total Queries refused by the tenant's own concurrency limit, as opposed to queries this instance failed to answer.\n\
# TYPE signy_query_quota_rejected_total counter\n\
signy_query_quota_rejected_total {}\n\
# HELP signy_storage_limit_rejected_total Writes refused because the tenant already stores what its plan sells. Unlike the rate rejections this one clears only when retention retires parts.\n\
# TYPE signy_storage_limit_rejected_total counter\n\
signy_storage_limit_rejected_total {}\n\
# HELP signy_wal_replayed_records Records this process replayed from the WAL at startup. Non-zero means the previous run did not shut down cleanly.\n\
# TYPE signy_wal_replayed_records gauge\n\
signy_wal_replayed_records {}\n\
# HELP signy_wal_replayed_entries Log entries in those records — the upper bound on how many lines this restart may have duplicated.\n\
# TYPE signy_wal_replayed_entries gauge\n\
signy_wal_replayed_entries {}\n\
# TYPE signy_memtable_buffered_bytes gauge\n\
signy_memtable_buffered_bytes {}\n\
# TYPE signy_flush_success_total counter\n\
signy_flush_success_total {}\n\
# TYPE signy_flush_errors_total counter\n\
signy_flush_errors_total {}\n\
# TYPE signy_merge_success_total counter\n\
signy_merge_success_total {}\n\
# TYPE signy_merge_errors_total counter\n\
signy_merge_errors_total {}\n\
# TYPE signy_merge_inputs_changed_total counter\n\
signy_merge_inputs_changed_total {}\n\
# TYPE signy_retention_success_total counter\n\
signy_retention_success_total {}\n\
# TYPE signy_retention_errors_total counter\n\
signy_retention_errors_total {}\n\
# TYPE signy_retention_expired_rows_dropped_total counter\n\
signy_retention_expired_rows_dropped_total {}\n\
# TYPE signy_retention_parts_rewritten_total counter\n\
signy_retention_parts_rewritten_total {}\n\
# TYPE signy_retention_rewrite_skipped_total counter\n\
signy_retention_rewrite_skipped_total {}\n\
# TYPE signy_tenant_policy_push_accepted_total counter\n\
signy_tenant_policy_push_accepted_total {}\n\
# TYPE signy_tenant_policy_push_rejected_total counter\n\
signy_tenant_policy_push_rejected_total {}\n\
# TYPE signy_tenant_policy_push_persist_errors_total counter\n\
signy_tenant_policy_push_persist_errors_total {}\n\
# TYPE signy_tenant_policy_known_tenants gauge\n\
signy_tenant_policy_known_tenants {}\n\
# TYPE signy_tenant_policy_infinite_tenants gauge\n\
signy_tenant_policy_infinite_tenants {}\n\
# TYPE signy_tenant_policy_unknown_tenants gauge\n\
signy_tenant_policy_unknown_tenants {}\n\
# TYPE signy_tenant_policy_last_push_age_seconds gauge\n\
signy_tenant_policy_last_push_age_seconds {}\n\
# TYPE signy_query_success_total counter\n\
signy_query_success_total {}\n\
# TYPE signy_query_errors_total counter\n\
signy_query_errors_total {}\n\
# TYPE signy_query_scanned_rows_total counter\n\
signy_query_scanned_rows_total {}\n\
# TYPE signy_query_scanned_bytes_total counter\n\
signy_query_scanned_bytes_total {}\n\
# TYPE signy_query_latency_ns_total counter\n\
signy_query_latency_ns_total {}\n\
# HELP signy_query_scans_in_flight Scans holding a scheduler permit right now, out of max_concurrent_query_scans.\n\
# TYPE signy_query_scans_in_flight gauge\n\
signy_query_scans_in_flight {}\n\
# HELP signy_query_scans_in_flight_peak High-water mark of that since start. The memory budget's largest term is max_concurrent_query_scans x max_query_memory_bytes, and this is how far into it a run actually reached — a sampled gauge cannot see a burst that fills the scheduler and drains between two scrapes.\n\
# TYPE signy_query_scans_in_flight_peak gauge\n\
signy_query_scans_in_flight_peak {}\n\
# HELP signy_query_scans_queued_total Scans that found every slot taken and waited. Nonzero is proof the concurrency limit bound, which the peak alone only suggests.\n\
# TYPE signy_query_scans_queued_total counter\n\
signy_query_scans_queued_total {}\n\
# TYPE signy_query_scan_queue_wait_ns_total counter\n\
signy_query_scan_queue_wait_ns_total {}\n\
# TYPE signy_remote_restore_success_total counter\n\
signy_remote_restore_success_total {}\n\
# TYPE signy_remote_restore_errors_total counter\n\
signy_remote_restore_errors_total {}\n\
# TYPE signy_remote_restore_latency_ns_total counter\n\
signy_remote_restore_latency_ns_total {}\n\
# TYPE signy_cache_evictions_total counter\n\
signy_cache_evictions_total {}\n\
# TYPE signy_drain_in_progress gauge\n\
signy_drain_in_progress {}\n\
# TYPE signy_pending_flush_bytes gauge\n\
signy_pending_flush_bytes {}\n\
# TYPE signy_force_flush_complete gauge\n\
signy_force_flush_complete {}\n\
# HELP signy_build_info Build identity, always 1. Join on it to attribute a series to a revision.\n\
# TYPE signy_build_info gauge\n\
signy_build_info{{version=\"{}\",revision=\"{}\"}} 1\n\
# HELP signy_query_latency_ms Query latency by the endpoint the query arrived at. The cumulative _ns_total counters only ever yielded a mean; every target is written as p95/p99, so use histogram_quantile on this, and sum by (le) across endpoints for the whole read path.\n\
# TYPE signy_query_latency_ms histogram\n\
{}\
# HELP signy_remote_restore_latency_ms Object-store restore latency, the cost of a cache miss.\n\
# TYPE signy_remote_restore_latency_ms histogram\n\
{}",
        mem.entries,
        mem.bytes,
        disk.entries,
        disk.bytes,
        state.parts.part_count(),
        state.trace_parts.part_count(),
        state.series_parts.part_count(),
        state.config.capacity_probe as u8,
        remote_healthy as u8,
        state
            .remote_cache
            .as_ref()
            .map(|cache| cache.consecutive_remote_failures())
            .unwrap_or(0),
        cache_healthy as u8,
        wal_backlog_bytes,
        state.ingest_gate.inflight_body_bytes(),
        state.disk.free_bytes(),
        state.disk.total_bytes(),
        merge_debt_parts,
        layout.tenant_segments,
        layout
            .sidecar_resident_bytes
            .saturating_add(crate::part::bloom_cache_bytes()),
        crate::part::row_group_cache_bytes(),
        layout.meta_bytes,
        m.ingest_requests.load(Ordering::Relaxed),
        m.ingest_errors.load(Ordering::Relaxed),
        m.ingest_throttled.load(Ordering::Relaxed),
        m.collect_dropped_records.load(Ordering::Relaxed),
        m.collect_dropped_bytes.load(Ordering::Relaxed),
        m.collect_skipped_records.load(Ordering::Relaxed),
        m.ingest_dropped_no_tenant.load(Ordering::Relaxed),
        m.ingest_dropped_invalid_tenant.load(Ordering::Relaxed),
        m.ingest_dropped_tenant_not_served.load(Ordering::Relaxed),
        m.ingest_dropped_tenant_not_served_logs.load(Ordering::Relaxed),
        m.ingest_dropped_tenant_not_served_traces.load(Ordering::Relaxed),
        m.ingest_dropped_tenant_not_served_metrics.load(Ordering::Relaxed),
        m.query_quota_rejected.load(Ordering::Relaxed),
        m.storage_limit_rejected.load(Ordering::Relaxed),
        m.wal_replayed_records.load(Ordering::Relaxed),
        m.wal_replayed_entries.load(Ordering::Relaxed),
        state.ingest_gate.buffered_bytes(),
        m.flush_success.load(Ordering::Relaxed),
        m.flush_errors.load(Ordering::Relaxed),
        m.merge_success.load(Ordering::Relaxed),
        m.merge_errors.load(Ordering::Relaxed),
        m.merge_inputs_changed.load(Ordering::Relaxed),
        m.retention_success.load(Ordering::Relaxed),
        m.retention_errors.load(Ordering::Relaxed),
        m.retention_expired_rows_dropped.load(Ordering::Relaxed),
        m.retention_parts_rewritten.load(Ordering::Relaxed),
        m.retention_rewrite_skipped.load(Ordering::Relaxed),
        state
            .tenant_policy
            .metrics
            .push_accepted
            .load(Ordering::Relaxed),
        state
            .tenant_policy
            .metrics
            .push_rejected
            .load(Ordering::Relaxed),
        state
            .tenant_policy
            .metrics
            .push_persist_errors
            .load(Ordering::Relaxed),
        policy.known_tenants,
        policy.infinite_tenants,
        policy.unknown_tenants,
        policy.last_push_age_seconds,
        m.query_success.load(Ordering::Relaxed),
        m.query_errors.load(Ordering::Relaxed),
        m.query_scanned_rows.load(Ordering::Relaxed),
        m.query_scanned_bytes.load(Ordering::Relaxed),
        m.query_latency_ns.load(Ordering::Relaxed),
        m.query_scans_in_flight.load(Ordering::Relaxed),
        m.query_scans_in_flight_peak.load(Ordering::Relaxed),
        m.query_scans_queued.load(Ordering::Relaxed),
        m.query_scan_queue_wait_ns.load(Ordering::Relaxed),
        m.remote_restore_success.load(Ordering::Relaxed),
        m.remote_restore_errors.load(Ordering::Relaxed),
        m.remote_restore_latency_ns.load(Ordering::Relaxed),
        m.cache_evictions.load(Ordering::Relaxed),
        state.shutdown.is_draining() as u8,
        state.shutdown.pending_flush_bytes(),
        state.shutdown.is_flush_complete() as u8,
        env!("CARGO_PKG_VERSION"),
        build_revision(),
        crate::metrics::QueryEndpoint::ALL
            .iter()
            .map(|endpoint| {
                m.query_latency[*endpoint as usize].render_labeled(
                    "signy_query_latency_ms",
                    &format!("endpoint=\"{}\"", endpoint.label()),
                )
            })
            .collect::<String>(),
        m.remote_restore_latency
            .render("signy_remote_restore_latency_ms"),
    );
    body.push_str(&object_store_operation_metrics(&state));
    body.push_str(&object_store_gc_metrics(&state));
    body.push_str(&restore_economics_metrics());
    body.push_str(&delete_request_metrics(&state));
    body.push_str(&journal_writer_metrics(&state));
    body.push_str(&series_ladder_metrics(&state));
    body.push_str(&crate::memprof::render());
    body.push_str(&crate::allocator_stats::render());
    body
}

/// Where an accepted push's server-side time went.
///
/// Every push in the process is written by one task, so these four phases are
/// the whole of it and they are additive: queue, write, fsync, insert. The
/// question they exist to answer is which of them the push tail is made of —
/// a p50 of 12 ms beside a p95 that moves between 40 and 106 ms with nothing
/// but the client's connection count (`todo.md`, 2026-08-12) is a queue, and
/// until these there was no number in the process that could say so.
/// The M14 degradation ladder's observability: every rung moves one of these,
/// and the comparison bed's churn table is built from them.
fn object_store_gc_metrics(state: &AppState) -> String {
    let metrics = &state.metrics;
    let mut out = String::new();
    for (name, help, value) in [
        (
            "signy_object_store_gc_success_total",
            "Object-store collection passes that finished: orphaned part objects, and superseded catalog objects when SIGNY_CATALOG_PRUNE_MIN_AGE is set.",
            metrics.object_store_gc_success.load(Ordering::Relaxed),
        ),
        (
            "signy_object_store_gc_errors_total",
            "Object-store collection passes that failed or timed out. The next pass resumes from the stored orphan ledger.",
            metrics.object_store_gc_errors.load(Ordering::Relaxed),
        ),
        (
            "signy_orphan_collect_success_total",
            "Orphan collection passes that finished, whether or not their budget let them delete everything deletable.",
            metrics.orphan_collect_success.load(Ordering::Relaxed),
        ),
        (
            "signy_orphan_collect_errors_total",
            "Orphan collection passes that failed. The next pass resumes from the stored ledger and scan cursor.",
            metrics.orphan_collect_errors.load(Ordering::Relaxed),
        ),
        (
            "signy_orphan_objects_removed_total",
            "Part objects deleted because no manifest named them for longer than SIGNY_RETENTION_GRACE_PERIOD.",
            metrics.orphan_objects_removed.load(Ordering::Relaxed),
        ),
        (
            "signy_orphan_bytes_removed_total",
            "Bytes reclaimed by orphan collection.",
            metrics.orphan_bytes_removed.load(Ordering::Relaxed),
        ),
        (
            "signy_orphan_delete_errors_total",
            "Orphan deletions the object store refused. Their ledger entries survive, so a later pass retries them.",
            metrics.orphan_delete_errors.load(Ordering::Relaxed),
        ),
        (
            "signy_orphan_scan_cycles_total",
            "Completed walks of every part prefix.",
            metrics.orphan_scan_cycles.load(Ordering::Relaxed),
        ),
        (
            "signy_catalog_prune_success_total",
            "Catalog pruning passes that finished. Independent of orphan collection.",
            metrics.catalog_prune_success.load(Ordering::Relaxed),
        ),
        (
            "signy_catalog_prune_errors_total",
            "Catalog pruning passes that failed or timed out.",
            metrics.catalog_prune_errors.load(Ordering::Relaxed),
        ),
        (
            "signy_catalog_objects_pruned_total",
            "Catalog commit and snapshot objects deleted because no startup reads them any more.",
            metrics.catalog_objects_pruned.load(Ordering::Relaxed),
        ),
    ] {
        out.push_str(&format!(
            "# HELP {name} {help}\n# TYPE {name} counter\n{name} {value}\n"
        ));
    }
    for (name, help, value) in [
        (
            "signy_orphan_candidate_objects",
            "Objects the last pass found deletable, whether or not its budget let it delete them. With SIGNY_ORPHAN_GC_DRY_RUN this is what a real pass would delete.",
            metrics.orphan_candidate_objects.load(Ordering::Relaxed),
        ),
        (
            "signy_orphan_candidate_bytes",
            "Bytes the last pass found deletable.",
            metrics.orphan_candidate_bytes.load(Ordering::Relaxed),
        ),
        (
            "signy_orphan_ledger_entries",
            "Objects the collector is tracking outside the active set.",
            metrics.orphan_ledger_entries.load(Ordering::Relaxed),
        ),
    ] {
        out.push_str(&format!(
            "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value}\n"
        ));
    }
    out
}

fn series_ladder_metrics(state: &AppState) -> String {
    use std::sync::atomic::Ordering;
    let series = state.journal.series_memtable();
    let counters = series.counters();
    let mut body = format!(
        "# HELP signy_active_series Live metric series index entries across tenants. \
The optional SIGNY_MAX_ACTIVE_SERIES emergency guard is process-wide; normal \
admission is charged against the shared byte budget.\n\
# TYPE signy_active_series gauge\n\
signy_active_series {}\n\
# TYPE signy_series_created_total counter\n\
signy_series_created_total {}\n\
# HELP signy_series_evicted_idle_total Series whose index state left at the idle \
horizon (SIGNY_METRIC_SERIES_IDLE_TIMEOUT); their history stays in parts.\n\
# TYPE signy_series_evicted_idle_total counter\n\
signy_series_evicted_idle_total {}\n\
# HELP signy_series_retired_flushed_total Series whose index state left as soon \
as their samples became durable, rather than at the idle horizon. A gauge or \
cumulative series needs no index entry once a part holds it; the labels stay \
resident in that part's catalog.\n\
# TYPE signy_series_retired_flushed_total counter\n\
signy_series_retired_flushed_total {}\n\
# HELP signy_series_rejected_total New series refused at the optional \
process-wide SIGNY_MAX_ACTIVE_SERIES emergency boundary.\n\
# TYPE signy_series_rejected_total counter\n\
signy_series_rejected_total {}\n\
# TYPE signy_metric_datapoints_rejected_total counter\n\
signy_metric_datapoints_rejected_total {}\n\
# TYPE signy_metric_samples_rejected_total counter\n\
signy_metric_samples_rejected_total {}\n\
# HELP signy_metric_cardinality_rejected_total Metric exports refused whole by \
the optional process-wide active-series emergency guard.\n\
# TYPE signy_metric_cardinality_rejected_total counter\n\
signy_metric_cardinality_rejected_total {}\n\
# HELP signy_metric_memory_rejected_total Metric exports refused whole because \
the shared process-wide memtable byte budget had no room.\n\
# TYPE signy_metric_memory_rejected_total counter\n\
signy_metric_memory_rejected_total {}\n\
# TYPE signy_series_memtable_bytes gauge\n\
signy_series_memtable_bytes {}\n\
# HELP signy_metric_memtable_reserved_bytes Projected metric sample bytes held \
by queued journal appends.\n\
# TYPE signy_metric_memtable_reserved_bytes gauge\n\
signy_metric_memtable_reserved_bytes {}\n",
        counters.active_series.load(Ordering::Relaxed),
        counters.series_created_total.load(Ordering::Relaxed),
        counters.series_evicted_idle_total.load(Ordering::Relaxed),
        counters.series_retired_flushed_total.load(Ordering::Relaxed),
        counters.series_rejected_total.load(Ordering::Relaxed),
        counters
            .metric_datapoints_rejected_total
            .load(Ordering::Relaxed),
        counters
            .metric_samples_rejected_total
            .load(Ordering::Relaxed),
        counters
            .metric_cardinality_rejected_total
            .load(Ordering::Relaxed),
        counters
            .metric_memory_rejected_total
            .load(Ordering::Relaxed),
        series.approximate_size(),
        state.journal.metric_reserved_bytes(),
    );
    // These gauges intentionally belong only to the disposable raw-capacity
    // probe.  Summing millions of HashMap lengths/capacities and walking every
    // arena entry on each production scrape would turn operator telemetry into
    // measurable ingest contention; normal mode already has the cheap byte and
    // active-series gauges above.  The probe is explicitly asking for this
    // structural attribution, so it opts into the walk via its existing switch.
    if state.config.capacity_probe {
        let stats = series.memory_stats();
        body.push_str(&format!(
            "# HELP signy_series_states_len Live metric index entries across all tenants.\n\
# TYPE signy_series_states_len gauge\n\
signy_series_states_len {}\n\
# HELP signy_series_states_capacity Sum of the backing capacities of tenant state maps.\n\
# TYPE signy_series_states_capacity gauge\n\
signy_series_states_capacity {}\n\
# HELP signy_series_buffers_len Live sample-arena entries.\n\
# TYPE signy_series_buffers_len gauge\n\
signy_series_buffers_len {}\n\
# HELP signy_series_buffers_capacity Sum of the backing capacities of tenant sample arenas.\n\
# TYPE signy_series_buffers_capacity gauge\n\
signy_series_buffers_capacity {}\n\
# TYPE signy_series_buffers_empty gauge\n\
signy_series_buffers_empty {}\n\
# HELP signy_series_buffers_inline One-sample sample-arena entries that have not been promoted to Gorilla.\n\
# TYPE signy_series_buffers_inline gauge\n\
signy_series_buffers_inline {}\n\
# HELP signy_series_buffers_stream Sample-arena entries using the boxed Gorilla stream.\n\
# TYPE signy_series_buffers_stream gauge\n\
signy_series_buffers_stream {}\n\
# HELP signy_series_flushing_series Series entries held by the in-flight flush snapshot.\n\
# TYPE signy_series_flushing_series gauge\n\
signy_series_flushing_series {}\n\
# TYPE signy_series_flushing_tenants gauge\n\
signy_series_flushing_tenants {}\n\
",
            stats.states_len,
            stats.states_capacity,
            stats.buffers_len,
            stats.buffers_capacity,
            stats.empty_buffers,
            stats.inline_buffers,
            stats.stream_buffers,
            stats.flushing_series,
            stats.flushing_tenants,
        ));
    }
    body
}

fn journal_writer_metrics(state: &AppState) -> String {
    let metrics = state.journal.metrics();
    let mut out = String::new();
    out.push_str(
        "# HELP signy_journal_batches_total Batches the writer task wrote, one fsync each.\n\
# TYPE signy_journal_batches_total counter\n",
    );
    out.push_str(&format!(
        "signy_journal_batches_total {}\n",
        metrics.batches.load(Ordering::Relaxed)
    ));
    out.push_str(
        "# HELP signy_journal_batched_records_total Appends carried by those batches. Divided by the batches, the number of pushes sharing each fsync.\n\
# TYPE signy_journal_batched_records_total counter\n",
    );
    out.push_str(&format!(
        "signy_journal_batched_records_total {}\n",
        metrics.batched_records.load(Ordering::Relaxed)
    ));
    out.push_str(
        &metrics
            .append_queue_wait
            .render("signy_journal_append_queue_wait_ms"),
    );
    out.push_str(&metrics.batch_write.render("signy_journal_write_ms"));
    out.push_str(&metrics.batch_fsync.render("signy_journal_fsync_ms"));
    out.push_str(&metrics.batch_insert.render("signy_journal_insert_ms"));
    out.push_str(
        &metrics
            .checkpoint
            .render("signy_journal_checkpoint_ms"),
    );
    out.push_str(&flush_phase_metrics(state));
    out.push_str(
        "# HELP signy_query_memory_exhausted_total Requests refused because this instance's memory account had no room. Distinct from the tenant read quota, which says a tenant asked for more than it was sold, from a scan-limit refusal, which says the query was too broad, and from the over-budget 400, which says the request is larger than this instance could ever serve: this one says the instance ran out of room for work it was willing to do, and is the read side's counterpart to ingest_throttled.\n\
# TYPE signy_query_memory_exhausted_total counter\n",
    );
    out.push_str(&format!(
        "signy_query_memory_exhausted_total {}\n",
        state.memory_account.exhausted()
    ));
    let (bloom_hits, bloom_misses, bloom_read_bytes) = crate::part::bloom_cache_counters();
    out.push_str(
        "# HELP signy_part_sidecar_hits_total Pruning queries that found a part's blooms already resident.\n\
# TYPE signy_part_sidecar_hits_total counter\n",
    );
    out.push_str(&format!("signy_part_sidecar_hits_total {bloom_hits}\n"));
    out.push_str(
        "# HELP signy_part_sidecar_misses_total Pruning queries that had to re-read index.bin because the bloom cache had evicted it. Read against hits, this is what sidecar_cache_max_bytes is costing: the resident gauge shows the cache sitting at its ceiling whether it is serving every query or re-reading on every query, and only this pair tells the two apart.\n\
# TYPE signy_part_sidecar_misses_total counter\n",
    );
    out.push_str(&format!("signy_part_sidecar_misses_total {bloom_misses}\n"));
    out.push_str(
        "# HELP signy_part_sidecar_read_bytes_total Bytes of index.bin re-read on those misses. Each miss reads the whole file into an owned buffer and drops it after decoding, so the rate of this counter is a rate of large allocate-and-free -- the shape that fills an allocator's dirty page cache without live memory growing.\n\
# TYPE signy_part_sidecar_read_bytes_total counter\n",
    );
    out.push_str(&format!(
        "signy_part_sidecar_read_bytes_total {bloom_read_bytes}\n"
    ));
    let (installs, install_bytes, evictions, resident_ns, redecodes, gap_ns) =
        crate::part::bloom_cache_lifecycle();
    for (name, help, kind, value) in [
        ("signy_part_sidecar_installs_total", "Decoded bloom sets installed into the cache. One per miss that finished decoding.", "counter", installs),
        ("signy_part_sidecar_installed_bytes_total", "Decoded bytes those installs added, against sidecar_read_bytes_total's raw file size: what a miss puts in memory as against what it reads.", "counter", install_bytes),
        ("signy_part_sidecar_evictions_total", "Entries the cache dropped to stay under its budget, or that left with their reader.", "counter", evictions),
        ("signy_part_sidecar_resident_nanos_total", "Summed time those evicted entries spent resident. Divided by evictions it is how long an install survives -- the number a hit rate cannot give, because a cache that installs, evicts and re-decodes the same part in a loop still serves hits to the queries that land inside each residency.", "counter", resident_ns),
        ("signy_part_sidecar_redecodes_total", "Installs into a slot that had been evicted before: the same part's blooms decoded again.", "counter", redecodes),
        ("signy_part_sidecar_redecode_gap_nanos_total", "Summed time between those evictions and the re-decode that followed. Divided by redecodes it is what the eviction bought; a short gap beside a short residency is thrashing, and says the working set does not fit rather than that the cache is misbehaving.", "counter", gap_ns),
    ] {
        out.push_str(&format!("# HELP {name} {help}\n# TYPE {name} {kind}\n{name} {value}\n"));
    }
    out.push_str(
        "# HELP signy_memory_account_in_use_bytes Bytes charged to the shared memory account and not yet released: what admitted queries, flushes and compactions have declared they hold right now. Read against signy_memory_account_budget_bytes, this is what an arriving request is admitted or refused on. It is an accounted total and not a heap measurement, so it will sit below process_rss_bytes, which also carries the caches, the memtables and whatever the allocator has not returned.\n\
# TYPE signy_memory_account_in_use_bytes gauge\n",
    );
    out.push_str(&format!(
        "signy_memory_account_in_use_bytes {}\n",
        state.memory_account.in_use_bytes()
    ));
    out.push_str(
        "# HELP signy_memory_account_budget_bytes The ceiling the above is admitted against (SIGNY_MEMORY_ACCOUNT_BYTES).\n\
# TYPE signy_memory_account_budget_bytes gauge\n",
    );
    out.push_str(&format!(
        "signy_memory_account_budget_bytes {}\n",
        state.memory_account.budget_bytes()
    ));
    out.push_str(
        "# HELP signy_memory_account_deferred_total Background passes -- metric flush and metric compaction -- that found no room in the account and left their input for the next tick. They have no client to refuse, so this is their form of the counter above. Climbing without settling is compaction debt that never drains.\n\
# TYPE signy_memory_account_deferred_total counter\n",
    );
    out.push_str(&format!(
        "signy_memory_account_deferred_total {}\n",
        state.memory_account.deferred()
    ));
    out
}

/// Where a flush pass's time goes.
///
/// The companion to the journal writer's phases, and the more consequential of
/// the two: the rate ladder of 2026-08-13 put this engine's capacity ceiling
/// here rather than in the WAL. A pass that takes longer than the memtable
/// takes to refill *is* the ceiling, and before these the only evidence of it
/// reaching one was a `429` arriving at a client.
fn flush_phase_metrics(state: &AppState) -> String {
    let flush = &state.metrics.flush;
    let mut out = String::new();
    out.push_str(
        "# HELP signy_flush_rows_total Rows written into parts by the flush loop.\n\
# TYPE signy_flush_rows_total counter\n",
    );
    out.push_str(&format!(
        "signy_flush_rows_total {}\n",
        flush.rows.load(Ordering::Relaxed)
    ));
    out.push_str(
        "# HELP signy_flush_parts_total Parts those passes produced. Divided by the rows, the part size the chunker is choosing.\n\
# TYPE signy_flush_parts_total counter\n",
    );
    out.push_str(&format!(
        "signy_flush_parts_total {}\n",
        flush.parts.load(Ordering::Relaxed)
    ));
    out.push_str(
        &flush
            .checkpoint_wait
            .render("signy_flush_checkpoint_wait_ms"),
    );
    out.push_str(&flush.build.render("signy_flush_build_ms"));
    out.push_str(&flush.open.render("signy_flush_open_ms"));
    out.push_str(&flush.visibility.render("signy_flush_visibility_ms"));
    out.push_str(&flush.advance_checkpoint.render("signy_flush_advance_ms"));
    // The build phase's own four, counted per part rather than per pass because
    // a flush cuts its snapshot into chunks. Only the flush path observes them;
    // a merge rewrite runs the same code and is deliberately excluded.
    let build = &crate::part::FLUSH_BUILD;
    out.push_str(&build.sort.render("signy_flush_build_sort_ms"));
    out.push_str(&build.parse.render("signy_flush_build_parse_ms"));
    out.push_str(&build.write.render("signy_flush_build_write_ms"));
    out.push_str(&build.parquet.render("signy_flush_build_parquet_ms"));
    out.push_str(&build.index.render("signy_flush_build_index_ms"));
    out.push_str(&build.meta.render("signy_flush_build_meta_ms"));
    out.push_str(&build.commit.render("signy_flush_build_commit_ms"));
    out
}

/// Deletion is the one operation here that destroys data on request, so how
/// many were accepted, how many were refused, and how many rows are being
/// hidden are all things an operator has to be able to see without asking a
/// tenant.
fn delete_request_metrics(state: &AppState) -> String {
    let metrics = &state.delete_requests.metrics;
    format!(
        "# TYPE signy_delete_requests_accepted_total counter\n\
signy_delete_requests_accepted_total {}\n\
# HELP signy_delete_requests_rejected_total Submissions refused for exceeding the per-tenant limit. Each outstanding request is a predicate every scan for that tenant evaluates per row.\n\
# TYPE signy_delete_requests_rejected_total counter\n\
signy_delete_requests_rejected_total {}\n\
# TYPE signy_delete_requests_cancelled_total counter\n\
signy_delete_requests_cancelled_total {}\n\
# HELP signy_delete_hidden_rows_total Rows a scan dropped because a deletion request covered them. Stops growing for a request once a rewrite has removed its bytes.\n\
# TYPE signy_delete_hidden_rows_total counter\n\
signy_delete_hidden_rows_total {}\n",
        metrics.accepted.load(Ordering::Relaxed),
        metrics.rejected.load(Ordering::Relaxed),
        metrics.cancelled.load(Ordering::Relaxed),
        metrics.hidden_rows.load(Ordering::Relaxed),
    )
}

/// The two numbers that decide the sign of "add Parquet range reads": what a
/// selective download would cost in requests, and what the whole-object
/// download earns by leaving a reusable copy behind. See
/// [`crate::restore_meter`] for why those two and not the byte total.
fn restore_economics_metrics() -> String {
    let meter = crate::restore_meter::global().snapshot();
    format!(
        "# HELP signy_query_part_scans_total Query scans that read a part body. A rewrite is excluded: it reads what it was told to.\n\
# TYPE signy_query_part_scans_total counter\n\
signy_query_part_scans_total {}\n\
# HELP signy_query_row_groups_total Row groups in the parts those scans read, by how far selection narrowed them. `present` is the whole part a restore downloads, `tenant` is the querying tenant's segment, `selected` is what the scan read.\n\
# TYPE signy_query_row_groups_total counter\n\
signy_query_row_groups_total{{stage=\"present\"}} {}\n\
signy_query_row_groups_total{{stage=\"tenant\"}} {}\n\
signy_query_row_groups_total{{stage=\"selected\"}} {}\n\
# HELP signy_query_selected_runs_total Contiguous runs among the selected row groups. Column chunks of a row group are contiguous and the log path projects every column, so a run is one byte range: this plus one footer read is what a selective download would issue where a whole restore issues one GET.\n\
# TYPE signy_query_selected_runs_total counter\n\
signy_query_selected_runs_total {}\n\
# HELP signy_restore_first_scan_total The same three numbers over the first scan of each restored body alone. That scan is the query the download was issued for, so its selection is the one a selective download would have applied; the aggregates above mix it with scans of bodies that were never downloaded.\n\
# TYPE signy_restore_first_scan_total counter\n\
signy_restore_first_scan_total{{stage=\"parts\"}} {}\n\
signy_restore_first_scan_total{{stage=\"present\"}} {}\n\
signy_restore_first_scan_total{{stage=\"selected\"}} {}\n\
signy_restore_first_scan_total{{stage=\"runs\"}} {}\n\
# HELP signy_restored_body_scans_total Query scans served by a body that was downloaded whole after eviction and is still on disk. Divided by the restore count, this is how much later work one over-fetch prepaid.\n\
# TYPE signy_restored_body_scans_total counter\n\
signy_restored_body_scans_total {}\n\
# HELP signy_restored_bodies_total Bodies restored, and how many of them eviction has since taken. A restore still resident has not finished earning.\n\
# TYPE signy_restored_bodies_total counter\n\
signy_restored_bodies_total{{state=\"restored\"}} {}\n\
signy_restored_bodies_total{{state=\"retired\"}} {}\n\
# HELP signy_restored_tenant_slices_total Distinct (restored body, querying tenant) pairs. A whole restore costs one GET however many tenants read it; a selective download serves one slice, so this is how many it would have taken.\n\
# TYPE signy_restored_tenant_slices_total counter\n\
signy_restored_tenant_slices_total {}\n",
        meter.part_scans,
        meter.row_groups_present,
        meter.row_groups_tenant,
        meter.row_groups_selected,
        meter.selected_runs,
        meter.first_scan_parts,
        meter.first_scan_row_groups_present,
        meter.first_scan_row_groups_selected,
        meter.first_scan_runs,
        meter.restored_scans,
        meter.restores,
        meter.restored_retired,
        meter.restored_tenant_slices,
    )
}

/// The cost model of this design is operation counts, not bytes. R2 bills per
/// request, and the whole shared-part layout exists because per-tenant objects
/// multiplied that count. These are the numbers to divide by flush, merge and
/// retention cycles to get the per-cycle cost, and they measure the same
/// locally as they do against a paid backend.
fn object_store_operation_metrics(state: &AppState) -> String {
    let Some(counts) = state
        .remote_cache
        .as_ref()
        .map(|cache| cache.storage.operation_counts())
    else {
        return String::new();
    };
    format!(
        "# HELP signy_object_store_operations_total Object-store requests issued, by kind. Which kinds are billed how is the backend's policy; how many of each this engine issues is not.\n\
# TYPE signy_object_store_operations_total counter\n\
signy_object_store_operations_total{{kind=\"put\"}} {}\n\
signy_object_store_operations_total{{kind=\"put_multipart\"}} {}\n\
signy_object_store_operations_total{{kind=\"get\"}} {}\n\
signy_object_store_operations_total{{kind=\"delete\"}} {}\n\
signy_object_store_operations_total{{kind=\"list\"}} {}\n\
signy_object_store_operations_total{{kind=\"copy\"}} {}\n\
# HELP signy_object_store_listed_objects_total Objects the listings returned. A backend pages a listing, so its request count follows from this and the page size rather than from the list count.\n\
# TYPE signy_object_store_listed_objects_total counter\n\
signy_object_store_listed_objects_total {}\n\
# HELP signy_object_store_ranged_gets_total GETs that asked for a byte range rather than a whole object. Zero means every restore moves the whole part, including the rows belonging to other tenants of a shared part.\n\
# TYPE signy_object_store_ranged_gets_total counter\n\
signy_object_store_ranged_gets_total {}\n\
# HELP signy_object_store_bytes_total Bytes moved to and from the object store. Read bytes are what the responses agreed to return, not what a caller consumed.\n\
# TYPE signy_object_store_bytes_total counter\n\
signy_object_store_bytes_total{{direction=\"get\"}} {}\n\
signy_object_store_bytes_total{{direction=\"put\"}} {}\n\
# HELP signy_object_store_bytes_by_kind_total The same bytes split by what was read or written. A part restore and a manifest rewrite are both bytes and only one of them is a part; the totals above cannot tell them apart.\n\
# TYPE signy_object_store_bytes_by_kind_total counter\n\
signy_object_store_bytes_by_kind_total{{direction=\"get\",kind=\"manifest\"}} {}\n\
signy_object_store_bytes_by_kind_total{{direction=\"get\",kind=\"part\"}} {}\n\
signy_object_store_bytes_by_kind_total{{direction=\"get\",kind=\"trace_part\"}} {}\n\
signy_object_store_bytes_by_kind_total{{direction=\"get\",kind=\"other\"}} {}\n\
signy_object_store_bytes_by_kind_total{{direction=\"put\",kind=\"manifest\"}} {}\n\
signy_object_store_bytes_by_kind_total{{direction=\"put\",kind=\"part\"}} {}\n\
signy_object_store_bytes_by_kind_total{{direction=\"put\",kind=\"trace_part\"}} {}\n\
signy_object_store_bytes_by_kind_total{{direction=\"put\",kind=\"other\"}} {}\n",
        counts.puts,
        counts.multipart_puts,
        counts.gets,
        counts.deletes,
        counts.lists,
        counts.copies,
        counts.listed_objects,
        counts.ranged_gets,
        counts.get_bytes,
        counts.put_bytes,
        counts.get_bytes_by_kind.manifest,
        counts.get_bytes_by_kind.part,
        counts.get_bytes_by_kind.trace_part,
        counts.get_bytes_by_kind.other,
        counts.put_bytes_by_kind.manifest,
        counts.put_bytes_by_kind.part,
        counts.put_bytes_by_kind.trace_part,
        counts.put_bytes_by_kind.other,
    )
}

/// The revision this binary was built from, or `unknown` when the build did
/// not supply one. Without it a scraped series cannot be attributed to code,
/// which is the first question asked when two deployments behave differently.
pub fn build_revision() -> &'static str {
    option_env!("SIGNY_BUILD_REVISION").unwrap_or("unknown")
}

fn m_merge_debt(state: &AppState) -> u64 {
    state
        .metrics
        .merge_debt_parts
        .load(std::sync::atomic::Ordering::Relaxed)
}

struct TenantPolicyGauges {
    known_tenants: usize,
    infinite_tenants: usize,
    unknown_tenants: u64,
    last_push_age_seconds: u64,
}

/// The policy map is small and in memory, so its two counts are computed here.
/// The unknown-tenant count is not: it walks every part's tenant index, so the
/// retention worker publishes it and this reads what that worker last saw.
fn tenant_policy_gauges(state: &AppState) -> TenantPolicyGauges {
    let Some(snapshot) = state.tenant_policy.snapshot() else {
        return TenantPolicyGauges {
            known_tenants: 0,
            infinite_tenants: 0,
            unknown_tenants: 0,
            last_push_age_seconds: 0,
        };
    };
    TenantPolicyGauges {
        known_tenants: snapshot.tenant_count(),
        infinite_tenants: snapshot.infinite_tenant_count(),
        unknown_tenants: state
            .metrics
            .unknown_tenants
            .load(std::sync::atomic::Ordering::Relaxed),
        last_push_age_seconds: snapshot
            .newest_push_age(state.clock.now())
            .as_secs(),
    }
}
