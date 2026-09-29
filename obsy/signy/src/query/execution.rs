#[cfg(test)]
fn unified_query(
    state: &AppState,
    tenant: &TenantId,
    parsed: &logql::LogQuery,
    range: part::QueryTimeRange,
    limit: usize,
    forward: bool,
) -> Result<Vec<StreamResult>, String> {
    Ok(
        unified_query_with_stats(state, tenant, parsed, range, limit, forward, None)?.results,
    )
}

struct QueryExecution {
    results: Vec<StreamResult>,
    scanned_rows: u64,
    scanned_bytes: u64,
    /// What the pipeline materialized on its way to `results`. Reconciled
    /// against the charge below, whose estimate was made before any row was
    /// read and can undershoot.
    materialized_bytes: u64,
    /// The account charge backing `results`. Held for as long as the results
    /// are — releasing it while the rows it paid for are still being
    /// aggregated or serialized would let the account admit memory that is
    /// very much still resident.
    memory_charge: Option<crate::memory_budget::MemoryCharge>,
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn unified_query_with_stats(
    state: &AppState,
    tenant: &TenantId,
    parsed: &logql::LogQuery,
    range: part::QueryTimeRange,
    limit: usize,
    forward: bool,
    scan_budget: Option<usize>,
) -> Result<QueryExecution, String> {
    unified_query_with_stats_cancellable(
        state,
        tenant,
        parsed,
        range,
        limit,
        forward,
        scan_budget,
        None,
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn unified_query_with_stats_cancellable(
    state: &AppState,
    tenant: &TenantId,
    parsed: &logql::LogQuery,
    range: part::QueryTimeRange,
    limit: usize,
    forward: bool,
    scan_budget: Option<usize>,
    cancellation: Option<&AtomicBool>,
) -> Result<QueryExecution, String> {
    unified_query_with_stats_cancellable_with_memory(
        state,
        tenant,
        parsed,
        range,
        limit,
        forward,
        scan_budget,
        cancellation,
        None,
        None,
        part::ColumnSet::all(),
    )
}

/// The synchronous log scan. `run_unified_query_with_stats_cancellable_for_runtime`
/// wraps it with the scheduler permit and the blocking pool; the tests drive it
/// directly, without a runtime.
#[allow(clippy::too_many_arguments)]
fn unified_query_with_stats_cancellable_with_memory(
    state: &AppState,
    tenant: &TenantId,
    parsed: &logql::LogQuery,
    range: part::QueryTimeRange,
    limit: usize,
    forward: bool,
    scan_budget: Option<usize>,
    cancellation: Option<&AtomicBool>,
    max_memory_bytes: Option<u64>,
    max_scan_bytes: Option<u64>,
    columns: part::ColumnSet,
) -> Result<QueryExecution, String> {
    // The one place every read path meets its rows, which is why the deletion
    // mask is here and not at each handler. A second scan would be a second
    // place to forget it, and forgetting it means serving a line a tenant asked
    // to have deleted.
    let deleted = state.delete_requests.mask_for(tenant);
    let hidden_rows = &state.delete_requests.metrics.hidden_rows;
    let hidden = |_labels: &Labels, entry: &LogEntry| {
        if deleted.hides(entry) {
            hidden_rows.fetch_add(1, Ordering::Relaxed);
            return true;
        }
        false
    };
    let scan = crate::log_scan::LogScan::new(tenant, parsed, range, limit, forward)
        .columns(columns)
        .scan_budget(scan_budget)
        .max_scan_bytes(max_scan_bytes)
        .max_memory_bytes(max_memory_bytes)
        .cancellation(cancellation)
        .hidden(&hidden);
    let result = scan.run(&state.memtable, &state.parts)?;
    Ok(QueryExecution {
        results: result.results,
        scanned_rows: result.scanned_rows,
        scanned_bytes: result.scanned_bytes,
        materialized_bytes: result.materialized_bytes,
        memory_charge: None,
    })
}

#[cfg(test)]
async fn run_unified_query(
    state: Arc<AppState>,
    tenant: TenantId,
    parsed: logql::LogQuery,
    range: part::QueryTimeRange,
    limit: usize,
    forward: bool,
) -> Result<Vec<StreamResult>, String> {
    Ok(run_unified_query_with_stats(
        state,
        tenant,
        parsed,
        range,
        limit,
        forward,
        None,
        crate::metrics::QueryEndpoint::Query,
    )
    .await?
    .results)
}

#[allow(clippy::too_many_arguments)]
async fn run_unified_query_with_stats(
    state: Arc<AppState>,
    tenant: TenantId,
    parsed: logql::LogQuery,
    range: part::QueryTimeRange,
    limit: usize,
    forward: bool,
    scan_budget: Option<usize>,
    endpoint: crate::metrics::QueryEndpoint,
) -> Result<QueryExecution, String> {
    // Held for the whole scan. Every read path funnels through here — logs,
    // tail, volume, detected fields, restore probes — so the tenant's share of
    // this instance is bounded in one place rather than at each handler.
    let _slot = state
        .tenant_quota
        .begin_query(&tenant)
        .map_err(|error| format!("{TENANT_QUOTA_PREFIX}{}", error.message))?;
    let cancellation = Arc::new(AtomicBool::new(false));
    let metrics = state.metrics.clone();
    let started = std::time::Instant::now();
    let result = run_unified_query_with_stats_cancellable(
        state,
        tenant,
        parsed,
        range,
        limit,
        forward,
        scan_budget,
        cancellation,
    )
    .await;
    metrics.observe_query(endpoint, started.elapsed());
    match &result {
        Ok(execution) => {
            metrics
                .query_success
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            metrics
                .query_scanned_rows
                .fetch_add(execution.scanned_rows, std::sync::atomic::Ordering::Relaxed);
            metrics
                .query_scanned_bytes
                .fetch_add(execution.scanned_bytes, std::sync::atomic::Ordering::Relaxed);
        }
        Err(_) => {
            metrics
                .query_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    result
}

#[allow(clippy::too_many_arguments)]
async fn run_unified_query_with_stats_cancellable(
    state: Arc<AppState>,
    tenant: TenantId,
    parsed: logql::LogQuery,
    range: part::QueryTimeRange,
    limit: usize,
    forward: bool,
    scan_budget: Option<usize>,
    cancellation: Arc<AtomicBool>,
) -> Result<QueryExecution, String> {
    // The log path returns every pair a row stored, so it reads them all —
    // plus the `_pf:` columns when a `| json` stage can consume them
    // precomputed instead of parsing every surviving line.
    let columns = part::ColumnSet::for_log_query(&parsed);
    run_unified_query_with_stats_cancellable_for_runtime(
        state,
        tenant,
        parsed,
        range,
        limit,
        forward,
        scan_budget,
        cancellation,
        None,
        columns,
    )
    .await
}

/// The per-row figure used when no part has been written yet to measure one.
/// It covers the window between a fresh start and the first flush, and
/// nothing else.
const UNMEASURED_ROW_BYTES: u64 = 512;

/// What this log query will need, in bytes, decided before it reads a row.
///
/// A log query's answer is at most `limit` rows, and every part records the
/// memory its rows materialize into, so the two multiply into a real ceiling
/// rather than a guess: `min(limit, rows reachable) × this data's own average
/// row`. It reads part metadata only — see
/// [`crate::part_registry::PartRegistry::materialization_estimate`] for why it
/// must not reach for the blooms a candidate plan reads.
///
/// It can still undershoot — one part's rows can be far wider than its part's
/// average — which is why two things outlive it: `max_query_memory_bytes`
/// refuses the scan that runs away, and the charge is reconciled to the truth
/// when the scan returns.
fn estimated_log_query_bytes(
    state: &AppState,
    tenant: &TenantId,
    range: part::QueryTimeRange,
    limit: usize,
    cap_bytes: u64,
) -> u64 {
    let (part_rows, bytes_per_row) = state
        .parts
        .materialization_estimate(tenant, range)
        .unwrap_or((0, UNMEASURED_ROW_BYTES));
    // The memtable holds the same rows the parts will hold, so its bytes are
    // priced at the same average; what it cannot cheaply give is a row count,
    // so a non-empty memtable simply stops the stored row count from shrinking
    // the estimate below what `limit` allows.
    let reachable_rows = if state.memtable.approximate_size() > 0 {
        limit as u64
    } else {
        part_rows.min(limit as u64)
    };
    reachable_rows.saturating_mul(bytes_per_row).min(cap_bytes)
}

#[allow(clippy::too_many_arguments)]
async fn run_unified_query_with_stats_cancellable_for_runtime(
    state: Arc<AppState>,
    tenant: TenantId,
    parsed: logql::LogQuery,
    range: part::QueryTimeRange,
    limit: usize,
    forward: bool,
    scan_budget: Option<usize>,
    cancellation: Arc<AtomicBool>,
    runtime_override: Option<std::time::Duration>,
    columns: part::ColumnSet,
) -> Result<QueryExecution, String> {
    let max_runtime = runtime_override.unwrap_or(state.config.max_query_runtime);
    // Pin (and possibly restore) the parts *before* taking a scan slot. A
    // restore is object-store I/O; holding one of the few scan permits across
    // it stalled CPU-bound scans behind pure network wait. The pin is a
    // refcount, so nothing is scanned before the permit is held.
    let part_guard = tokio::time::timeout(
        max_runtime,
        pin_query_parts(&state, &tenant, &parsed, range),
    )
    .await
    .map_err(|_| "query timed out".to_string())??;
    let max_query_memory_bytes = state.config.max_query_memory_bytes;
    // Price the request against the account *before* it queues for a scan
    // slot. The parts are pinned, so this reads the catalogs of exactly what
    // the scan will read; refusing here costs one metadata pass, while
    // refusing after the slot would have made a query that can never run wait
    // behind queries that can.
    let memory_charge = state.memory_account.admit(estimated_log_query_bytes(
        &state,
        &tenant,
        range,
        limit,
        max_query_memory_bytes,
    ))?;
    let scan_permit = match state.query_scan_semaphore.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(tokio::sync::TryAcquireError::Closed) => {
            return Err("query scan scheduler is closed".to_string());
        }
        // Every slot was taken. Separated from the fast path only so the wait
        // can be counted: this branch is the evidence that
        // `max_concurrent_query_scans` actually bound, and the memory budget is
        // written against the state where it does. The pin above is deliberately
        // outside it: a restore is network wait, and counting it here would
        // report object-store latency as scheduler contention.
        Err(tokio::sync::TryAcquireError::NoPermits) => {
            let queued_at = std::time::Instant::now();
            let permit = tokio::time::timeout(
                max_runtime,
                state.query_scan_semaphore.clone().acquire_owned(),
            )
            .await
            .map_err(|_| "query timed out".to_string())?
            .map_err(|_| "query scan scheduler is closed".to_string())?;
            state.metrics.record_scan_queue_wait(queued_at.elapsed());
            permit
        }
    };
    let scan_occupancy = crate::metrics::ScanOccupancy::enter(state.metrics.clone());
    let task_cancellation = cancellation.clone();
    let max_query_runtime = max_runtime;
    let mut task = tokio::task::spawn_blocking(move || {
        // Keep the scheduler permit until the blocking task actually exits;
        // cancelling the request must not admit an unbounded second scan while
        // the first task is still consuming CPU and memory.
        let _scan_permit = scan_permit;
        let _scan_occupancy = scan_occupancy;
        let _part_guard = part_guard;
        let _arena = crate::memprof::enter(crate::memprof::Arena::Query);
        let mut execution = unified_query_with_stats_cancellable_with_memory(
            &state,
            &tenant,
            &parsed,
            range,
            limit,
            forward,
            scan_budget,
            Some(task_cancellation.as_ref()),
            Some(max_query_memory_bytes),
            Some(state.config.max_query_scan_bytes),
            columns,
        )?;
        // The estimate was made before a row was read. Correct the account to
        // what the scan really materialized, then leave with the charge: the
        // rows are resident until the response body is built.
        memory_charge.reconcile(execution.materialized_bytes);
        execution.memory_charge = Some(memory_charge);
        Ok::<_, String>(execution)
    });
    let execution = match tokio::time::timeout(max_query_runtime, &mut task).await {
        Ok(result) => result.map_err(|error| format!("query task failed: {error}"))?,
        Err(_) => {
            cancellation.store(true, Ordering::Release);
            let _ = task.await;
            Err("query timed out".to_string())
        }
    }?;
    let estimated_memory = estimated_query_memory_bytes(&execution.results);
    if estimated_memory > max_query_memory_bytes {
        return Err(format!(
            "query exceeds the maximum of {max_query_memory_bytes} materialized bytes"
        ));
    }
    Ok(execution)
}

/// The metric fast path's scan: same admission (part pin, then the scan
/// permit), same budgets, but a [`crate::log_scan::CountingSink`] instead of a
/// bounded row collector — nothing is materialized and nothing is returned but
/// the per-point totals.
#[allow(clippy::too_many_arguments)]
async fn run_metric_count_scan(
    state: Arc<AppState>,
    tenant: TenantId,
    parsed: logql::LogQuery,
    range: part::QueryTimeRange,
    scan_budget: Option<usize>,
    cancellation: Arc<AtomicBool>,
    runtime_override: Option<std::time::Duration>,
    columns: part::ColumnSet,
    times: Vec<i64>,
    range_ns: i64,
    bytes: bool,
) -> Result<(Vec<f64>, u64, u64, u64), String> {
    let max_runtime = runtime_override.unwrap_or(state.config.max_query_runtime);
    let part_guard = tokio::time::timeout(
        max_runtime,
        pin_query_parts(&state, &tenant, &parsed, range),
    )
    .await
    .map_err(|_| "query timed out".to_string())??;
    let scan_permit = tokio::time::timeout(
        max_runtime,
        state.query_scan_semaphore.clone().acquire_owned(),
    )
    .await
    .map_err(|_| "query timed out".to_string())?
    .map_err(|_| "query scan scheduler is closed".to_string())?;
    let task_cancellation = cancellation.clone();
    let mut task = tokio::task::spawn_blocking(move || {
        let _scan_permit = scan_permit;
        let _part_guard = part_guard;
        let _arena = crate::memprof::enter(crate::memprof::Arena::Query);
        let deleted = state.delete_requests.mask_for(&tenant);
        let hidden_rows = &state.delete_requests.metrics.hidden_rows;
        let hidden = |_labels: &Labels, entry: &LogEntry| {
            if deleted.hides(entry) {
                hidden_rows.fetch_add(1, Ordering::Relaxed);
                return true;
            }
            false
        };
        let hidden: Option<crate::log_scan::HiddenRow> =
            (!deleted.is_empty()).then_some(&hidden);
        let mut sink = crate::log_scan::CountingSink {
            query: &parsed,
            hidden,
            cancellation: Some(task_cancellation.as_ref()),
            times: &times,
            range_ns,
            bytes,
            diff: vec![0.0; times.len() + 1],
            rows: 0,
        };
        let scan = crate::log_scan::LogScan::new(&tenant, &parsed, range, usize::MAX, true)
            .columns(columns)
            .scan_budget(scan_budget)
            .max_scan_bytes(Some(state.config.max_query_scan_bytes))
            .cancellation(Some(task_cancellation.as_ref()));
        let (scanned_rows, scanned_bytes) = scan.run_into(&state.memtable, &state.parts, &mut sink)?;
        Ok::<_, String>((sink.diff, sink.rows, scanned_rows, scanned_bytes))
    });
    match tokio::time::timeout(max_runtime, &mut task).await {
        Ok(result) => result.map_err(|error| format!("query task failed: {error}"))?,
        Err(_) => {
            cancellation.store(true, Ordering::Release);
            let _ = task.await;
            Err("query timed out".to_string())
        }
    }
}
