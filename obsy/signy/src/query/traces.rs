/// The first-party trace endpoints (`docs/QUERY_API.md`). Same contract as
/// the log surface: NDJSON out, refusals that teach, and the whole answer is
/// collected before the first byte is written.
pub async fn trace_by_id(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(trace_id): Path<String>,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    if raw.as_deref().is_some_and(|raw| !raw.is_empty()) {
        return Err(ApiError::bad_request(
            "the trace-by-id endpoint takes no parameters: \
GET /signy/api/v1/traces/{trace_id} returns every span retention still holds — \
see docs/QUERY_API.md"
                .to_string(),
        ));
    }
    let trace_id = crate::trace::canonical_trace_id(&trace_id).map_err(ApiError::bad_request)?;

    let _slot = state.tenant_quota.begin_query(&tenant).map_err(|error| {
        ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
    })?;
    let metrics = state.metrics.clone();
    let retention_floor_ns = state.tenant_policy.query_floor_ns(&tenant, crate::tenant_policy::Signal::Traces);
    let started = std::time::Instant::now();
    let result = scan_trace_spans(state, tenant, TraceScanTarget::ById(trace_id.clone())).await;
    metrics.observe_query(crate::metrics::QueryEndpoint::TraceById, started.elapsed());
    let mut outcome = match result {
        Ok(outcome) => {
            metrics
                .query_success
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            outcome
        }
        Err(error) => {
            metrics
                .query_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Err(error);
        }
    };
    metrics
        .query_scanned_rows
        .fetch_add(outcome.spans.len() as u64, std::sync::atomic::Ordering::Relaxed);
    metrics
        .query_scanned_bytes
        .fetch_add(outcome.estimated_bytes, std::sync::atomic::Ordering::Relaxed);

    // Retention holds span by span here rather than clamping a window: the
    // route has no window, and a trace straddling the floor should show the
    // part of itself the tenant is still entitled to see.
    if let Some(floor_ns) = retention_floor_ns {
        outcome.spans.retain(|span| span.start_time_ns >= floor_ns);
    }
    if outcome.spans.is_empty() {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            format!(
                "trace {trace_id} was not found: no spans remain above the tenant's retention \
floor — it may have expired or never existed here"
            ),
        ));
    }
    let body = trace_span_rows_ndjson(&outcome.spans);
    Ok(ndjson_response(
        body,
        outcome.spans.len() as u64,
        outcome.estimated_bytes,
    ))
}

pub async fn traces_search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params =
        parse_trace_filter_params(raw.as_deref().unwrap_or(""), now_ns, TRACE_SEARCH_PARAMS)
            .map_err(ApiError::bad_request)?;

    let limit = parse_limit(
        params.limit,
        state.config.max_trace_search_limit.min(MAX_TRACE_SEARCH_LIMIT),
    )
    .map_err(ApiError::bad_request)?;
    let Some((start_ns, end_ns)) = trace_window(&state, &tenant, &params, now_ns)? else {
        return Ok(ndjson_response(String::new(), 0, 0));
    };

    let _slot = state.tenant_quota.begin_query(&tenant).map_err(|error| {
        ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
    })?;
    let metrics = state.metrics.clone();
    let started = std::time::Instant::now();
    let result = scan_trace_spans(state, tenant, TraceScanTarget::Window { start_ns, end_ns }).await;
    metrics.observe_query(crate::metrics::QueryEndpoint::Traces, started.elapsed());
    let outcome = match result {
        Ok(outcome) => {
            metrics
                .query_success
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            outcome
        }
        Err(error) => {
            metrics
                .query_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Err(error);
        }
    };
    metrics
        .query_scanned_rows
        .fetch_add(outcome.spans.len() as u64, std::sync::atomic::Ordering::Relaxed);
    metrics
        .query_scanned_bytes
        .fetch_add(outcome.estimated_bytes, std::sync::atomic::Ordering::Relaxed);

    let body = trace_summaries_ndjson(&outcome.spans, &params.filters, limit);
    Ok(ndjson_response(
        body,
        outcome.spans.len() as u64,
        outcome.estimated_bytes,
    ))
}

/// The shared window arithmetic of every windowed trace endpoint: the same
/// defaults as the log endpoints, the range validated, and the retention
/// floor applied logically before any scan. `None` means the floor swallowed
/// the whole window — the empty answer, not an error, exactly as on ranges
/// it merely shortens.
fn trace_window(
    state: &AppState,
    tenant: &TenantId,
    params: &TraceFilterParams,
    now_ns: i64,
) -> Result<Option<(i64, i64)>, ApiError> {
    let end_ns = params.end_ns.unwrap_or(now_ns);
    let start_ns = params.start_ns.unwrap_or_else(|| {
        state
            .config
            .max_query_range
            .map(duration_to_i64_ns)
            .map(|range| end_ns.saturating_sub(range))
            .unwrap_or(i64::MIN)
    });
    validate_query_range(&state.config, start_ns, end_ns).map_err(ApiError::bad_request)?;
    let start_ns = clamp_to_retention(start_ns, state.tenant_policy.query_floor_ns(tenant, crate::tenant_policy::Signal::Traces));
    if start_ns > end_ns {
        return Ok(None);
    }
    Ok(Some((start_ns, end_ns)))
}

/// Trace autocomplete answers from a bounded window scan — unlike the log
/// autocomplete there is no catalog to read instead, because a span's
/// attributes live inside its stored payload. The scan pays the same
/// admission as any trace scan (its semaphore, the memory account, the span
/// budget), and QUERY_API.md says so.
pub async fn traces_attributes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_trace_filter_params(
        raw.as_deref().unwrap_or(""),
        now_ns,
        TRACE_ATTRIBUTE_KEYS_PARAMS,
    )
    .map_err(ApiError::bad_request)?;
    let Some((start_ns, end_ns)) = trace_window(&state, &tenant, &params, now_ns)? else {
        return Ok(ndjson_response(String::new(), 0, 0));
    };
    let _slot = state.tenant_quota.begin_query(&tenant).map_err(|error| {
        ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
    })?;
    let outcome =
        scan_trace_spans(state, tenant, TraceScanTarget::Window { start_ns, end_ns }).await?;

    let mut keys = std::collections::BTreeSet::new();
    if !outcome.spans.is_empty() {
        // The intrinsics every span answers, in the same list the grammar
        // filters on.
        keys.extend(["duration".to_string(), "name".to_string(), "status".to_string()]);
    }
    for span in &outcome.spans {
        if span.service_name().is_some() {
            keys.insert("service.name".to_string());
        }
        for attribute in &span.span.attributes {
            keys.insert(attribute.key.clone());
        }
        if let Some(resource) = &span.resource {
            for attribute in &resource.attributes {
                keys.insert(attribute.key.clone());
            }
        }
    }
    let mut body = String::new();
    for key in keys {
        body.push_str(
            &serde_json::to_string(&serde_json::json!({ "key": key }))
                .expect("a key serializes infallibly"),
        );
        body.push('\n');
    }
    Ok(ndjson_response(
        body,
        outcome.spans.len() as u64,
        outcome.estimated_bytes,
    ))
}

pub async fn traces_attribute_values(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(key): Path<String>,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_trace_filter_params(
        raw.as_deref().unwrap_or(""),
        now_ns,
        TRACE_ATTRIBUTE_VALUES_PARAMS,
    )
    .map_err(ApiError::bad_request)?;
    let Some((start_ns, end_ns)) = trace_window(&state, &tenant, &params, now_ns)? else {
        return Ok(ndjson_response(String::new(), 0, 0));
    };
    let _slot = state.tenant_quota.begin_query(&tenant).map_err(|error| {
        ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
    })?;
    let outcome =
        scan_trace_spans(state, tenant, TraceScanTarget::Window { start_ns, end_ns }).await?;

    // Values come from the traces the already-placed filters match — search
    // semantics, per trace — so a dropdown offers only values whose click
    // still returns something.
    let mut traces: std::collections::BTreeMap<&str, Vec<&crate::trace::TraceSpan>> =
        std::collections::BTreeMap::new();
    for span in &outcome.spans {
        traces.entry(&span.trace_id).or_default().push(span);
    }
    let mut values = std::collections::BTreeSet::new();
    for trace_spans in traces.values() {
        if !params
            .filters
            .iter()
            .all(|filter| trace_spans.iter().any(|span| filter.matches(span)))
        {
            continue;
        }
        for span in trace_spans {
            if let Some(value) = span.tag_value(&key) {
                values.insert(value);
            }
        }
    }
    let mut body = String::new();
    for value in values {
        body.push_str(
            &serde_json::to_string(&serde_json::json!({ "value": value }))
                .expect("a value serializes infallibly"),
        );
        body.push('\n');
    }
    Ok(ndjson_response(
        body,
        outcome.spans.len() as u64,
        outcome.estimated_bytes,
    ))
}

#[derive(Serialize)]
struct TraceSummaryRow<'a> {
    trace_id: &'a str,
    root_service: String,
    root_name: &'a str,
    start: String,
    end: String,
    duration: String,
    span_count: usize,
}

/// One summary per matching trace, newest first. A trace matches when every
/// filter is matched by at least one of its spans in the window — including
/// the duration comparisons, which are per-span, not per-trace. The summary
/// is built from the windowed spans only: reading the rest of the trace would
/// mean restoring the parts the window was pruned to avoid, and the full
/// extent is what the by-id fetch shows.
fn trace_summaries_ndjson(
    spans: &[crate::trace::TraceSpan],
    filters: &[TraceFilter],
    limit: usize,
) -> String {
    // The scan's `(start_time_ns, span_id)` order survives the grouping, so
    // each trace's spans arrive sorted by start.
    let mut traces: std::collections::BTreeMap<&str, Vec<&crate::trace::TraceSpan>> =
        std::collections::BTreeMap::new();
    for span in spans {
        traces.entry(&span.trace_id).or_default().push(span);
    }

    let mut summaries: Vec<(i64, TraceSummaryRow)> = Vec::new();
    for (trace_id, trace_spans) in &traces {
        if !filters
            .iter()
            .all(|filter| trace_spans.iter().any(|span| filter.matches(span)))
        {
            continue;
        }
        let root = trace_spans
            .iter()
            .find(|span| span.span.parent_span_id.is_empty())
            .unwrap_or(&trace_spans[0]);
        let start = trace_spans[0].start_time_ns;
        let end = trace_spans
            .iter()
            .map(|span| span.end_time_ns)
            .max()
            .unwrap_or(root.end_time_ns);
        summaries.push((
            start,
            TraceSummaryRow {
                trace_id,
                root_service: root.service_name().unwrap_or_default().to_string(),
                root_name: &root.span.name,
                start: start.to_string(),
                end: end.to_string(),
                duration: end.saturating_sub(start).to_string(),
                span_count: trace_spans.len(),
            },
        ));
    }
    // Newest first; the BTreeMap already ordered equal starts by trace id.
    summaries.sort_by_key(|(start, _)| std::cmp::Reverse(*start));
    let mut body = String::new();
    for (_, row) in summaries.into_iter().take(limit) {
        body.push_str(&serde_json::to_string(&row).expect("a summary row serializes infallibly"));
        body.push('\n');
    }
    body
}

#[derive(Serialize)]
struct TraceSpanRow {
    trace_id: String,
    span_id: String,
    parent_span_id: String,
    name: String,
    kind: &'static str,
    service: String,
    status: String,
    /// Nanoseconds as strings, like every timestamp this API emits: the
    /// values exceed 2^53, past which a JSON number silently loses precision
    /// in every JavaScript consumer.
    start: String,
    end: String,
    duration: String,
    attributes: std::collections::BTreeMap<String, String>,
    events: Vec<TraceEventRow>,
}

#[derive(Serialize)]
struct TraceEventRow {
    timestamp: String,
    name: String,
    attributes: std::collections::BTreeMap<String, String>,
}

fn trace_span_rows_ndjson(spans: &[crate::trace::TraceSpan]) -> String {
    let mut body = String::new();
    for span in spans {
        let row = trace_span_row(span);
        body.push_str(&serde_json::to_string(&row).expect("a span row serializes infallibly"));
        body.push('\n');
    }
    body
}

fn trace_span_row(span: &crate::trace::TraceSpan) -> TraceSpanRow {
    TraceSpanRow {
        trace_id: span.trace_id.clone(),
        span_id: span.span_id.clone(),
        parent_span_id: hex_id(&span.span.parent_span_id),
        name: span.span.name.clone(),
        kind: span_kind_label(span.span.kind),
        service: span.service_name().unwrap_or_default().to_string(),
        status: span
            .tag_value("status")
            .expect("status is an intrinsic and always answers"),
        start: span.start_time_ns.to_string(),
        end: span.end_time_ns.to_string(),
        duration: span.duration_ns().to_string(),
        attributes: merged_attributes(span),
        events: span
            .span
            .events
            .iter()
            .map(|event| TraceEventRow {
                timestamp: event.time_unix_nano.to_string(),
                name: event.name.clone(),
                attributes: attribute_map(&event.attributes),
            })
            .collect(),
    }
}

fn hex_id(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn span_kind_label(kind: i32) -> &'static str {
    use opentelemetry_proto::tonic::trace::v1::span::SpanKind;
    match SpanKind::try_from(kind) {
        Ok(SpanKind::Internal) => "internal",
        Ok(SpanKind::Server) => "server",
        Ok(SpanKind::Client) => "client",
        Ok(SpanKind::Producer) => "producer",
        Ok(SpanKind::Consumer) => "consumer",
        _ => "unspecified",
    }
}

/// Resource attributes first, span attributes overwriting same-named keys —
/// the shadowing order `tag_value` already prefers, stated in QUERY_API.md.
fn merged_attributes(
    span: &crate::trace::TraceSpan,
) -> std::collections::BTreeMap<String, String> {
    let mut attributes = span
        .resource
        .as_ref()
        .map(|resource| attribute_map(&resource.attributes))
        .unwrap_or_default();
    for attribute in &span.span.attributes {
        if let Some(value) = &attribute.value {
            attributes.insert(
                attribute.key.clone(),
                crate::trace::attr_display_string(value),
            );
        }
    }
    attributes
}

fn attribute_map(
    attributes: &[opentelemetry_proto::tonic::common::v1::KeyValue],
) -> std::collections::BTreeMap<String, String> {
    attributes
        .iter()
        .filter_map(|attribute| {
            attribute.value.as_ref().map(|value| {
                (
                    attribute.key.clone(),
                    crate::trace::attr_display_string(value),
                )
            })
        })
        .collect()
}
