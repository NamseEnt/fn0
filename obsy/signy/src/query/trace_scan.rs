/// The trace read path's scan orchestrator. Admission runs in the same order
/// as the log path's (`execution.rs`): pin (a restore is network wait and must
/// not hold a scan slot), then the memory account — priced from the catalogs
/// the pin just fixed, so a request that cannot be served never queues — then
/// a scan permit, then the blocking scan under an outer timeout. The permit
/// comes from the trace surface's own semaphore because a trace scan decodes
/// whole-span JSON payloads, a cost profile that would let either surface
/// starve the other if they shared slots.
const MAX_TRACE_SPANS: usize = 100_000;
const MAX_TRACE_SEARCH_LIMIT: usize = 1_000;

enum TraceScanTarget {
    ById(String),
    Window { start_ns: i64, end_ns: i64 },
}

struct TraceScanOutcome {
    /// Sorted by `(start_time_ns, span_id)`.
    spans: Vec<crate::trace::TraceSpan>,
    estimated_bytes: u64,
    /// Held with the spans it paid for; the handler drops both together after
    /// the NDJSON body is built (the `QueryExecution` precedent).
    _memory_charge: crate::memory_budget::MemoryCharge,
}

/// What a trace scan could hold at its ceiling, in bytes.
///
/// A trace part records a stored byte extent per tenant but nothing that says
/// what decoding it costs, so unlike the log path this cannot read a
/// materialized figure straight out of the catalog. It prices the stored
/// average and scales it — see `TraceRegistry::average_span_bytes` — and falls
/// back to that estimator's floor when the tenant has no stored spans at all,
/// which is the case where everything it could return is still in the
/// memtable.
fn estimated_trace_scan_bytes(
    state: &AppState,
    tenant: &TenantId,
    max_spans: usize,
    cap_bytes: u64,
) -> u64 {
    let per_span = state
        .trace_parts
        .average_span_bytes(tenant)
        .unwrap_or(crate::trace_registry::MIN_SPAN_BYTES);
    (max_spans as u64).saturating_mul(per_span).min(cap_bytes)
}

async fn scan_trace_spans(
    state: Arc<AppState>,
    tenant: TenantId,
    target: TraceScanTarget,
) -> Result<TraceScanOutcome, ApiError> {
    let max_runtime = state.config.max_trace_query_runtime;
    let guard = match &target {
        TraceScanTarget::ById(trace_id) => pin_trace_parts(&state, &tenant, trace_id).await?,
        TraceScanTarget::Window { start_ns, end_ns } => {
            pin_all_trace_parts(&state, &tenant, Some((*start_ns, *end_ns))).await?
        }
    };
    let max_spans = state.config.max_trace_spans.min(MAX_TRACE_SPANS);
    let max_query_memory_bytes = state.config.max_query_memory_bytes;
    // What this scan could hold at its worst: the span ceiling it already
    // enforces, priced at what one of this tenant's spans costs. Pessimistic
    // on purpose — a by-id lookup that finds twenty spans is admitted against
    // the hundred thousand it was allowed to find — and settled back down to
    // the truth by `reconcile` the moment the scan returns.
    let memory_charge = state
        .memory_account
        .admit(estimated_trace_scan_bytes(
            &state,
            &tenant,
            max_spans,
            max_query_memory_bytes,
        ))
        .map_err(ApiError::from_engine)?;
    let scan_permit = tokio::time::timeout(
        max_runtime,
        state.trace_scan_semaphore.clone().acquire_owned(),
    )
    .await
    .map_err(|_| ApiError::from_engine("trace query timed out".to_string()))?
    .map_err(|error| ApiError::from_engine(format!("trace scan scheduler is closed: {error}")))?;

    let cancellation = Arc::new(AtomicBool::new(false));
    let task_cancellation = cancellation.clone();
    let task_state = state.clone();
    let mut task = tokio::task::spawn_blocking(move || {
        // Keep the permit until the blocking task actually exits; cancelling
        // the request must not admit a second scan while the first is still
        // consuming CPU and memory.
        let _scan_permit = scan_permit;
        let _guard = guard;
        let _arena = crate::memprof::enter(crate::memprof::Arena::Query);
        let memtable = task_state.journal.trace_memtable();
        let mut spans = match &target {
            TraceScanTarget::ById(trace_id) => {
                memtable.query_trace_id_limited(&tenant, trace_id, max_spans)?
            }
            TraceScanTarget::Window { start_ns, end_ns } => {
                memtable.snapshot_range_limited(&tenant, *start_ns, *end_ns, max_spans)?
            }
        };
        let mut estimated_bytes: u64 = spans.iter().map(crate::trace::span_query_bytes).sum();
        let remaining = max_spans.saturating_sub(spans.len());
        let part_spans = match &target {
            TraceScanTarget::ById(trace_id) => task_state.trace_parts.query_trace_id(
                &tenant,
                trace_id,
                Some(remaining),
                Some(&task_cancellation),
            )?,
            TraceScanTarget::Window { start_ns, end_ns } => task_state.trace_parts.query_range(
                &tenant,
                Some((*start_ns, *end_ns)),
                Some(remaining),
                Some(&task_cancellation),
            )?,
        };
        estimated_bytes += part_spans
            .iter()
            .map(crate::trace::span_query_bytes)
            .sum::<u64>();
        spans.extend(part_spans);
        // The admission bought the worst case; this is what it turned out to
        // be, and the difference goes back to the account before the response
        // is even serialized.
        memory_charge.reconcile(estimated_bytes);
        spans.sort_by(|left, right| {
            left.start_time_ns
                .cmp(&right.start_time_ns)
                .then_with(|| left.span_id.cmp(&right.span_id))
        });
        Ok::<_, String>(TraceScanOutcome {
            spans,
            estimated_bytes,
            _memory_charge: memory_charge,
        })
    });

    let outcome = match tokio::time::timeout(max_runtime, &mut task).await {
        Ok(result) => result
            .map_err(|error| ApiError::from_engine(format!("trace query task failed: {error}")))?
            .map_err(ApiError::from_engine)?,
        Err(_) => {
            cancellation.store(true, Ordering::Release);
            let _ = task.await;
            return Err(ApiError::from_engine("trace query timed out".to_string()));
        }
    };
    if outcome.estimated_bytes > max_query_memory_bytes {
        return Err(ApiError::from_engine(format!(
            "trace query exceeds the maximum of {max_query_memory_bytes} materialized bytes"
        )));
    }
    Ok(outcome)
}

fn pin_error(error: crate::runtime_error::RuntimeError) -> ApiError {
    let (status, message) = error.into_http();
    ApiError(status, message)
}

async fn pin_trace_parts(
    state: &AppState,
    tenant: &TenantId,
    trace_id: &str,
) -> Result<tokio::sync::OwnedRwLockReadGuard<()>, ApiError> {
    crate::remote_lifecycle::pin_remote_parts(
        state.parts.operation_lock(),
        state.remote_cache.clone(),
        || state.trace_parts.candidate_part_ids(tenant, trace_id),
        |required| state.trace_parts.missing_data_ids(required),
        crate::remote_lifecycle::RemoteDomain::Traces,
        state.config.max_trace_restore_runtime,
        || Ok(()),
        Some(state.metrics.clone()),
    )
    .await
    .map_err(pin_error)
}

/// Pin the tenant's trace parts, narrowed to a time range when the caller has
/// one. Pinning is what downloads a part body, so the range has to reach this
/// far or the scan-side pruning saves nothing.
async fn pin_all_trace_parts(
    state: &AppState,
    tenant: &TenantId,
    range: Option<(i64, i64)>,
) -> Result<tokio::sync::OwnedRwLockReadGuard<()>, ApiError> {
    crate::remote_lifecycle::pin_remote_parts(
        state.parts.operation_lock(),
        state.remote_cache.clone(),
        || match range {
            Some((start_ns, end_ns)) => state
                .trace_parts
                .tenant_part_ids_in_range(tenant, start_ns, end_ns),
            None => state.trace_parts.tenant_part_ids(tenant),
        },
        |required| state.trace_parts.missing_data_ids(required),
        crate::remote_lifecycle::RemoteDomain::Traces,
        state.config.max_trace_restore_runtime,
        || Ok(()),
        Some(state.metrics.clone()),
    )
    .await
    .map_err(pin_error)
}

