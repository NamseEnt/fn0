/// The first-party log search endpoint (`docs/QUERY_API.md`).
///
/// Refusals are `{"error": "..."}` with a message that teaches; data is
/// NDJSON, one row per line. The whole top-K is collected before the first
/// byte is written, so this endpoint has no mid-stream failure mode — an
/// error is always a plain HTTP status with a JSON body.
#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub String);

impl ApiError {
    fn bad_request(message: String) -> Self {
        Self(StatusCode::BAD_REQUEST, message)
    }

    fn from_engine(error: String) -> Self {
        Self(metric_error_status(&error), error)
    }

    fn from_tenant(error: crate::tenant::TenantError) -> Self {
        let (status, message) = error.into_http();
        Self(status, message)
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}

/// Every unmatched path answers with the surface, so one wrong `curl` teaches
/// the whole API.
pub async fn api_fallback(uri: axum::http::Uri) -> ApiError {
    ApiError(
        StatusCode::NOT_FOUND,
        format!(
            "no route '{}': the first-party API routes are {}, plus /metrics, /ready, and the \
admin routes under /signy/api/v1/admin — see docs/QUERY_API.md",
            uri.path(),
            ROUTES.join(", ")
        ),
    )
}

const NDJSON_CONTENT_TYPE: &str = "application/x-ndjson";
pub(crate) const SCANNED_ROWS_HEADER: &str = "x-signy-scanned-rows";
pub(crate) const SCANNED_BYTES_HEADER: &str = "x-signy-scanned-bytes";

pub async fn logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_filter_params(raw.as_deref().unwrap_or(""), now_ns, LOGS_PARAMS)
        .map_err(ApiError::bad_request)?;

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
    let limit = parse_limit(params.limit, state.config.max_log_limit.min(MAX_LOG_LIMIT))
        .map_err(ApiError::bad_request)?;

    // Retention is enforced logically before any scan; a floor above `end`
    // yields the empty answer rather than an error, exactly as on the ranges
    // it merely shortens.
    let retention_floor_ns = state.tenant_policy.query_floor_ns(&tenant, crate::tenant_policy::Signal::Logs);
    let start_ns = clamp_to_retention(start_ns, retention_floor_ns);
    if start_ns > end_ns {
        return Ok(ndjson_response(String::new(), 0, 0));
    }

    let max_scan_rows = state.config.max_query_scan_rows.min(MAX_LOG_SCAN_ROWS);
    let execution = run_unified_query_with_stats(
        state,
        tenant,
        params.query,
        part::QueryTimeRange::half_open(start_ns, end_ns),
        limit,
        params.forward,
        Some(max_scan_rows),
        crate::metrics::QueryEndpoint::Logs,
    )
    .await
    .map_err(ApiError::from_engine)?;

    let body = log_rows_ndjson(execution.results, params.forward);
    Ok(ndjson_response(
        body,
        execution.scanned_rows,
        execution.scanned_bytes,
    ))
}

/// The bucket ladder the auto-width walks: the smallest width that keeps the
/// bucket count at or under 100, so a chart stays a chart whatever the range.
const BUCKET_LADDER_NS: [i64; 6] = [
    1_000_000_000,
    10_000_000_000,
    60_000_000_000,
    600_000_000_000,
    3_600_000_000_000,
    86_400_000_000_000,
];
const AUTO_BUCKET_TARGET: i128 = 100;

pub async fn logs_histogram(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_filter_params(raw.as_deref().unwrap_or(""), now_ns, HISTOGRAM_PARAMS)
        .map_err(ApiError::bad_request)?;

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
    let retention_floor_ns = state.tenant_policy.query_floor_ns(&tenant, crate::tenant_policy::Signal::Logs);
    let start_ns = clamp_to_retention(start_ns, retention_floor_ns);
    if start_ns >= end_ns {
        return Ok(ndjson_response(String::new(), 0, 0));
    }

    let span = i128::from(end_ns) - i128::from(start_ns);
    let bucket_ns = match params.bucket_ns {
        Some(bucket_ns) => bucket_ns,
        None => auto_bucket_ns(span),
    };
    // Half-open buckets, epoch-aligned to the width, clipped to the range:
    // the first bucket contains `start`, the last contains `end - 1`.
    let first_bucket_start = start_ns.div_euclid(bucket_ns) * bucket_ns;
    let last_bucket_start = (end_ns - 1).div_euclid(bucket_ns) * bucket_ns;
    let bucket_count = (i128::from(last_bucket_start) - i128::from(first_bucket_start))
        / i128::from(bucket_ns)
        + 1;
    let max_buckets = state.config.max_histogram_buckets.min(MAX_HISTOGRAM_BUCKETS);
    if bucket_count > max_buckets as i128 {
        return Err(ApiError::bad_request(format!(
            "histogram would have {bucket_count} buckets over this range, more than the maximum \
of {max_buckets}: widen bucket= or narrow the range"
        )));
    }
    let bucket_count = bucket_count as usize;
    // The counting sink totals `ts ∈ (t - bucket, t]`; evaluating at
    // `bucket_end - 1` makes that exactly `[bucket_start, bucket_end)`.
    let times: Vec<i64> = (0..bucket_count)
        .map(|at| first_bucket_start + (at as i64) * bucket_ns + (bucket_ns - 1))
        .collect();

    let _slot = state
        .tenant_quota
        .begin_query(&tenant)
        .map_err(|error| {
            ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
        })?;
    let metrics = state.metrics.clone();
    let started = std::time::Instant::now();
    let columns = part::ColumnSet::for_count_query(&params.query);
    let max_scan_rows = state.config.max_query_scan_rows.min(MAX_LOG_SCAN_ROWS);
    let result = run_metric_count_scan(
        state,
        tenant,
        params.query,
        // Closed on `end - 1`: the histogram's own range is `[start, end)`,
        // and the partial first and last buckets must count only rows inside
        // it.
        part::QueryTimeRange::closed(start_ns, end_ns - 1),
        Some(max_scan_rows),
        Arc::new(AtomicBool::new(false)),
        None,
        columns,
        times,
        bucket_ns,
        false,
    )
    .await;
    metrics.observe_query(crate::metrics::QueryEndpoint::Histogram, started.elapsed());
    let (diff, _accepted, scanned_rows, scanned_bytes) = match result {
        Ok(result) => {
            metrics
                .query_success
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            result
        }
        Err(error) => {
            metrics
                .query_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Err(ApiError::from_engine(error));
        }
    };
    metrics
        .query_scanned_rows
        .fetch_add(scanned_rows, std::sync::atomic::Ordering::Relaxed);
    metrics
        .query_scanned_bytes
        .fetch_add(scanned_bytes, std::sync::atomic::Ordering::Relaxed);

    let mut body = String::new();
    let mut running = 0.0;
    for (at, delta) in diff.iter().take(bucket_count).enumerate() {
        running += delta;
        let bucket_start = first_bucket_start + (at as i64) * bucket_ns;
        body.push_str(&format!(
            "{{\"bucket_start\":\"{}\",\"bucket_end\":\"{}\",\"count\":{}}}\n",
            bucket_start,
            bucket_start + bucket_ns,
            running as u64,
        ));
    }
    Ok(ndjson_response(body, scanned_rows, scanned_bytes))
}

fn auto_bucket_ns(span_ns: i128) -> i64 {
    for bucket_ns in BUCKET_LADDER_NS {
        if span_ns <= i128::from(bucket_ns) * AUTO_BUCKET_TARGET {
            return bucket_ns;
        }
    }
    let day_ns = *BUCKET_LADDER_NS.last().unwrap();
    let per_bucket = (span_ns + AUTO_BUCKET_TARGET - 1) / AUTO_BUCKET_TARGET;
    let days = (per_bucket + i128::from(day_ns) - 1) / i128::from(day_ns);
    i64::try_from(days).unwrap_or(i64::MAX / day_ns) * day_ns
}

#[derive(Serialize)]
struct LogRow<'a> {
    /// Nanoseconds as a string: the values exceed 2^53, past which a JSON
    /// number silently loses precision in every JavaScript consumer.
    timestamp: String,
    line: &'a str,
    attributes: &'a StreamKey,
}

fn log_rows_ndjson(results: Vec<StreamResult>, forward: bool) -> String {
    let mut rows: Vec<(i64, String, StreamKey)> = results
        .into_iter()
        .flat_map(|result| {
            let labels = result.labels;
            result.entries.into_iter().map(move |entry| {
                (
                    entry.timestamp_ns,
                    entry.line,
                    StreamKey::new(labels.clone(), entry.structured_metadata),
                )
            })
        })
        .collect();
    // Stable, so rows sharing a timestamp keep scan order.
    if forward {
        rows.sort_by_key(|(timestamp_ns, _, _)| *timestamp_ns);
    } else {
        rows.sort_by_key(|(timestamp_ns, _, _)| std::cmp::Reverse(*timestamp_ns));
    }
    let mut body = String::new();
    for (timestamp_ns, line, attributes) in &rows {
        let row = LogRow {
            timestamp: timestamp_ns.to_string(),
            line,
            attributes,
        };
        body.push_str(&serde_json::to_string(&row).expect("a log row serializes infallibly"));
        body.push('\n');
    }
    body
}

fn ndjson_response(
    body: String,
    scanned_rows: u64,
    scanned_bytes: u64,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    (
        [
            (axum::http::header::CONTENT_TYPE, NDJSON_CONTENT_TYPE.to_string()),
            (
                axum::http::HeaderName::from_static(SCANNED_ROWS_HEADER),
                scanned_rows.to_string(),
            ),
            (
                axum::http::HeaderName::from_static(SCANNED_BYTES_HEADER),
                scanned_bytes.to_string(),
            ),
        ],
        body,
    )
        .into_response()
}
