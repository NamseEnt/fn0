/// The first-party metric endpoints (`docs/QUERY_API.md`). Same contract as
/// the log and trace surfaces: NDJSON out, refusals that teach, the whole
/// answer decided before the first byte — and the response shapes are the
/// ones the comparison bed's parser froze before this file existed
/// (`digest_first_party_metric_response`): the engine grew to fit the ruler.
///
/// One operation per request. An optional per-series `func` (`rate`,
/// `increase` — the VictoriaMetrics definition: positive-delta sum over the
/// window, no extrapolation), then an optional one-level `agg` grouped `by`.
/// Ratios are two requests composed client-side, and the refusals say so.
const DEFAULT_METRIC_LOOKBACK_NS: i64 = 300 * 1_000_000_000;

pub async fn metrics_query(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_metric_params(raw.as_deref().unwrap_or(""), now_ns, METRIC_QUERY_PARAMS)
        .map_err(ApiError::bad_request)?;
    let metric = require_metric(&params)?;
    let step_ns = params.step_ns.ok_or_else(|| {
        ApiError::bad_request(
            "step is required: samples are aligned to start + k*step, like step=30s — \
see docs/QUERY_API.md"
                .to_string(),
        )
    })?;
    let grid = metric_grid(&state, &tenant, &params, step_ns, now_ns)?;
    let Some(grid) = grid else {
        return Ok(ndjson_response(String::new(), 0, 0));
    };

    let _slot = state.tenant_quota.begin_query(&tenant).map_err(|error| {
        ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
    })?;
    let metrics = state.metrics.clone();
    let started = std::time::Instant::now();
    let lookback_ns = params.lookback_ns.unwrap_or(DEFAULT_METRIC_LOOKBACK_NS);
    let request = MetricScanRequest {
        metric: Some(metric.clone()),
        filters: params.filters,
        start_ns: grid.start_ns,
        end_ns: grid.end_ns,
        steps: grid.steps,
        decode_margin_ns: params.range_ns.unwrap_or(0).max(lookback_ns),
    };
    let result = scan_metric_series(state, tenant, request).await;
    metrics.observe_query(crate::metrics::QueryEndpoint::MetricQuery, started.elapsed());
    let outcome = observe_metric_outcome(&metrics, result)?;

    let per_series = fold_series(&outcome, &grid, step_ns, lookback_ns, &params.func, params.range_ns);
    let rows = shape_output(per_series, &params.agg, &params.by, params.limit)?;
    let body = metric_samples_ndjson(&rows)?;
    Ok(ndjson_response(
        body,
        outcome.decoded_samples,
        outcome.estimated_bytes,
    ))
}

pub async fn metrics_instant(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_metric_params(raw.as_deref().unwrap_or(""), now_ns, METRIC_INSTANT_PARAMS)
        .map_err(ApiError::bad_request)?;
    let metric = require_metric(&params)?;
    let at_ns = params.at_ns.unwrap_or(now_ns);
    // The retention floor clamps a window; an instant either survives it or
    // answers empty, which keeps the alert path's semantics one sentence.
    if let Some(floor_ns) = state.tenant_policy.query_floor_ns(&tenant, crate::tenant_policy::Signal::Metrics)
        && at_ns < floor_ns
    {
        return Ok(ndjson_response(String::new(), 0, 0));
    }

    let _slot = state.tenant_quota.begin_query(&tenant).map_err(|error| {
        ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
    })?;
    let metrics = state.metrics.clone();
    let started = std::time::Instant::now();
    let lookback_ns = params.lookback_ns.unwrap_or(DEFAULT_METRIC_LOOKBACK_NS);
    let grid = EvalGrid {
        start_ns: at_ns,
        end_ns: at_ns,
        steps: 1,
    };
    let request = MetricScanRequest {
        metric: Some(metric.clone()),
        filters: params.filters,
        start_ns: at_ns,
        end_ns: at_ns,
        steps: 1,
        decode_margin_ns: params.range_ns.unwrap_or(0).max(lookback_ns),
    };
    let result = scan_metric_series(state, tenant, request).await;
    metrics.observe_query(
        crate::metrics::QueryEndpoint::MetricInstant,
        started.elapsed(),
    );
    let outcome = observe_metric_outcome(&metrics, result)?;

    let per_series = fold_series(
        &outcome,
        &grid,
        1,
        lookback_ns,
        &params.func,
        params.range_ns,
    );
    let rows = shape_output(per_series, &params.agg, &params.by, params.limit)?;
    let mut body = String::new();
    for (labels, samples) in &rows {
        let Some((ts, value)) = samples.first() else {
            continue;
        };
        body.push_str(
            &serde_json::to_string(&serde_json::json!({
                "labels": labels_object(labels)?,
                "timestamp": ts.to_string(),
                "value": value,
            }))
            .expect("an instant row serializes infallibly"),
        );
        body.push('\n');
    }
    Ok(ndjson_response(
        body,
        outcome.decoded_samples,
        outcome.estimated_bytes,
    ))
}

pub async fn metrics_quantile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_metric_params(raw.as_deref().unwrap_or(""), now_ns, METRIC_QUANTILE_PARAMS)
        .map_err(ApiError::bad_request)?;
    let metric = require_metric(&params)?;
    let q = params.q.ok_or_else(|| {
        ApiError::bad_request(
            "q is required: the quantile to interpolate, like q=0.99 — see docs/QUERY_API.md"
                .to_string(),
        )
    })?;
    let range_ns = params.range_ns.ok_or_else(|| {
        ApiError::bad_request(
            "range is required: a bucket count without a window is a lifetime total — \
write range=60s, see docs/QUERY_API.md"
                .to_string(),
        )
    })?;
    let step_ns = params.step_ns.ok_or_else(|| {
        ApiError::bad_request(
            "step is required: samples are aligned to start + k*step, like step=30s — \
see docs/QUERY_API.md"
                .to_string(),
        )
    })?;
    let grid = metric_grid(&state, &tenant, &params, step_ns, now_ns)?;
    let Some(grid) = grid else {
        return Ok(ndjson_response(String::new(), 0, 0));
    };

    let _slot = state.tenant_quota.begin_query(&tenant).map_err(|error| {
        ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
    })?;
    let metrics = state.metrics.clone();
    let started = std::time::Instant::now();
    let bucket_metric = format!("{metric}_bucket");
    let request = MetricScanRequest {
        metric: Some(bucket_metric),
        filters: params.filters,
        start_ns: grid.start_ns,
        end_ns: grid.end_ns,
        steps: grid.steps,
        decode_margin_ns: range_ns,
    };
    let scanned_state = state.clone();
    let result = scan_metric_series(state, tenant.clone(), request).await;
    metrics.observe_query(
        crate::metrics::QueryEndpoint::MetricQuantile,
        started.elapsed(),
    );
    let outcome = observe_metric_outcome(&metrics, result)?;
    if outcome.series.is_empty() && summary_backed(&scanned_state, &tenant, &metric) {
        return Err(ApiError::bad_request(format!(
            "{metric} is summary-backed: its quantiles were computed by the client and cannot \
be re-aggregated — query the {metric}{{quantile=\"0.99\"}} series with /metrics/query \
instead, see docs/QUERY_API.md"
        )));
    }

    let rows = fold_quantile(&outcome, &grid, step_ns, range_ns, q, &params.by, params.limit)?;
    let body = metric_samples_ndjson(&rows)?;
    Ok(ndjson_response(
        body,
        outcome.decoded_samples,
        outcome.estimated_bytes,
    ))
}

fn require_metric(params: &MetricParams) -> Result<String, ApiError> {
    params.metric.clone().ok_or_else(|| {
        ApiError::bad_request(
            "metric is required: the exact __name__ to read, like metric=http_requests_total — \
see docs/QUERY_API.md"
                .to_string(),
        )
    })
}

/// Whether `<metric>{quantile=...}` series exist for the tenant — the check
/// behind the summary-backed refusal. Catalog and index reads only.
fn summary_backed(state: &AppState, tenant: &TenantId, metric: &str) -> bool {
    let has_quantile = |labels: &[u8]| {
        let mut named = false;
        let mut quantile = false;
        for (key, value) in crate::series::canonical_pairs(labels).filter_map(Result::ok) {
            named |= key == crate::series::METRIC_NAME_LABEL && value == metric;
            quantile |= key == "quantile";
        }
        named && quantile
    };
    state
        .journal
        .series_memtable()
        .series_labels(tenant)
        .iter()
        .any(|labels| has_quantile(labels.as_bytes()))
        || state.series_parts.snapshot().iter().any(|reader| {
            reader
                .tenant_catalog(tenant)
                .iter()
                .any(|row| has_quantile(row.labels))
        })
}

struct EvalGrid {
    start_ns: i64,
    end_ns: i64,
    steps: u64,
}

impl EvalGrid {
    fn timestamps(&self, step_ns: i64) -> impl Iterator<Item = i64> + '_ {
        (0..self.steps).map(move |index| self.start_ns + index as i64 * step_ns)
    }
}

/// The shared window arithmetic of the range routes. `start` is required —
/// a step grid needs its origin — while `end` defaults to now; the range is
/// validated and the retention floor applied before any scan, `None` meaning
/// the floor swallowed the whole window.
fn metric_grid(
    state: &AppState,
    tenant: &TenantId,
    params: &MetricParams,
    step_ns: i64,
    now_ns: i64,
) -> Result<Option<EvalGrid>, ApiError> {
    let start_ns = params.start_ns.ok_or_else(|| {
        ApiError::bad_request(
            "start is required: the step grid is aligned to it, like start=-1h — \
see docs/QUERY_API.md"
                .to_string(),
        )
    })?;
    let end_ns = params.end_ns.unwrap_or(now_ns);
    validate_query_range(&state.config, start_ns, end_ns).map_err(ApiError::bad_request)?;
    let start_ns = clamp_to_retention(start_ns, state.tenant_policy.query_floor_ns(tenant, crate::tenant_policy::Signal::Metrics));
    if start_ns > end_ns {
        return Ok(None);
    }
    let steps = ((end_ns - start_ns) / step_ns) as u64 + 1;
    Ok(Some(EvalGrid {
        start_ns,
        end_ns,
        steps,
    }))
}

/// One folded series: its labels (still carrying `__name__` until the output
/// shaping strips it) and its points on the grid, missing steps omitted.
type FoldedSeries = (SeriesLabels, Vec<(i64, f64)>);

fn fold_series(
    outcome: &MetricScanOutcome,
    grid: &EvalGrid,
    step_ns: i64,
    lookback_ns: i64,
    func: &Option<MetricFunc>,
    range_ns: Option<i64>,
) -> Vec<FoldedSeries> {
    outcome
        .series
        .iter()
        .map(|series| {
            let points: Vec<(i64, f64)> = grid
                .timestamps(step_ns)
                .filter_map(|t| {
                    let value = match func {
                        None => raw_at(&series.samples, t, lookback_ns),
                        Some(MetricFunc::Increase) => {
                            increase_over(&series.samples, t, range_ns.unwrap_or(0))
                        }
                        Some(MetricFunc::Rate) => {
                            let range_ns = range_ns.unwrap_or(0);
                            increase_over(&series.samples, t, range_ns)
                                .map(|increase| increase / (range_ns as f64 / 1e9))
                        }
                    };
                    value.map(|value| (t, value))
                })
                .collect();
            (series.labels.clone(), points)
        })
        .collect()
}

/// The raw fold: the newest sample in `(t - lookback, t]`, or no point.
fn raw_at(samples: &[(i64, f64)], t: i64, lookback_ns: i64) -> Option<f64> {
    let at = samples.partition_point(|(ts, _)| *ts <= t);
    if at == 0 {
        return None;
    }
    let (ts, value) = samples[at - 1];
    (ts > t.saturating_sub(lookback_ns)).then_some(value)
}

/// The increase fold, the VictoriaMetrics definition (the M14 decision
/// record): the positive deltas over `(t - range, t]`, walking from the last
/// sample at or before the window's start when one exists — a counter reset
/// contributes the post-reset value, absorbed rather than extrapolated. Fewer
/// than two samples in the walk is no point, not zero.
fn increase_over(samples: &[(i64, f64)], t: i64, range_ns: i64) -> Option<f64> {
    let window_start = t.saturating_sub(range_ns);
    let end = samples.partition_point(|(ts, _)| *ts <= t);
    let first_inside = samples[..end].partition_point(|(ts, _)| *ts <= window_start);
    let base = first_inside.checked_sub(1);
    let walk_start = base.unwrap_or(first_inside);
    let walk = &samples[walk_start..end];
    if walk.len() < 2 {
        return None;
    }
    let mut increase = 0.0;
    for pair in walk.windows(2) {
        let (_, previous) = pair[0];
        let (_, next) = pair[1];
        increase += if next >= previous { next - previous } else { next };
    }
    Some(increase)
}

/// Aggregate the folded series (or pass them through), strip `__name__`, and
/// apply the output `limit` — smallest labels first, so the cut is
/// deterministic.
fn shape_output(
    per_series: Vec<FoldedSeries>,
    agg: &Option<MetricAgg>,
    by: &[String],
    limit: Option<usize>,
) -> Result<Vec<FoldedSeries>, ApiError> {
    let mut rows: Vec<FoldedSeries> = match agg {
        None => per_series
            .into_iter()
            .filter(|(_, points)| !points.is_empty())
            .map(|(labels, points)| Ok((strip_name(&labels)?, points)))
            .collect::<Result<_, ApiError>>()?,
        Some(agg) => {
            let mut groups: std::collections::BTreeMap<
                SeriesLabels,
                std::collections::BTreeMap<i64, Vec<f64>>,
            > = std::collections::BTreeMap::new();
            for (labels, points) in &per_series {
                let key = project_by(labels, by)?;
                let group = groups.entry(key).or_default();
                for (t, value) in points {
                    group.entry(*t).or_default().push(*value);
                }
            }
            groups
                .into_iter()
                .map(|(labels, by_step)| {
                    let points: Vec<(i64, f64)> = by_step
                        .into_iter()
                        .map(|(t, values)| {
                            let folded = match agg {
                                MetricAgg::Sum => values.iter().sum(),
                                MetricAgg::Avg => {
                                    values.iter().sum::<f64>() / values.len() as f64
                                }
                                MetricAgg::Min => {
                                    values.iter().copied().fold(f64::INFINITY, f64::min)
                                }
                                MetricAgg::Max => {
                                    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                                }
                                MetricAgg::Count => values.len() as f64,
                            };
                            (t, folded)
                        })
                        .collect();
                    (labels, points)
                })
                .filter(|(_, points)| !points.is_empty())
                .collect()
        }
    };
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    if let Some(limit) = limit {
        rows.truncate(limit);
    }
    Ok(rows)
}

/// The output identity never carries `__name__`: the query names the metric.
fn strip_name(labels: &SeriesLabels) -> Result<SeriesLabels, ApiError> {
    let pairs = labels
        .pairs()
        .map_err(ApiError::from_engine)?
        .into_iter()
        .filter(|(key, _)| key != crate::series::METRIC_NAME_LABEL)
        .collect();
    Ok(SeriesLabels::from_pairs(pairs))
}

/// The `by` projection: the named keys that are present on the series, absent
/// keys omitted rather than materialized as empty. `agg` without `by` folds
/// everything into the one empty-labeled group.
fn project_by(labels: &SeriesLabels, by: &[String]) -> Result<SeriesLabels, ApiError> {
    let pairs = labels
        .pairs()
        .map_err(ApiError::from_engine)?
        .into_iter()
        .filter(|(key, _)| by.contains(key))
        .collect();
    Ok(SeriesLabels::from_pairs(pairs))
}

fn labels_object(labels: &SeriesLabels) -> Result<serde_json::Value, ApiError> {
    let mut object = serde_json::Map::new();
    for (key, value) in labels.pairs().map_err(ApiError::from_engine)? {
        object.insert(key, serde_json::Value::String(value));
    }
    Ok(serde_json::Value::Object(object))
}

/// The range response: one line per output series,
/// `{"labels":{...},"samples":[["<ns>",value],...]}`, samples ascending —
/// exactly the shape the comparison bed's parser pinned in Phase 2.
fn metric_samples_ndjson(rows: &[FoldedSeries]) -> Result<String, ApiError> {
    let mut body = String::new();
    for (labels, points) in rows {
        let samples: Vec<serde_json::Value> = points
            .iter()
            .map(|(t, value)| serde_json::json!([t.to_string(), value]))
            .collect();
        body.push_str(
            &serde_json::to_string(&serde_json::json!({
                "labels": labels_object(labels)?,
                "samples": samples,
            }))
            .expect("a series row serializes infallibly"),
        );
        body.push('\n');
    }
    Ok(body)
}

fn observe_metric_outcome(
    metrics: &crate::metrics::RuntimeMetrics,
    result: Result<MetricScanOutcome, ApiError>,
) -> Result<MetricScanOutcome, ApiError> {
    match result {
        Ok(outcome) => {
            metrics
                .query_success
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            metrics
                .query_scanned_rows
                .fetch_add(outcome.decoded_samples, std::sync::atomic::Ordering::Relaxed);
            metrics
                .query_scanned_bytes
                .fetch_add(outcome.estimated_bytes, std::sync::atomic::Ordering::Relaxed);
            Ok(outcome)
        }
        Err(error) => {
            metrics
                .query_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Err(error)
        }
    }
}

/// The quantile fold: per group (labels minus `le`, or the `by` projection),
/// per step, the per-bucket increase over `(t - range, t]`, boundaries sorted
/// with a running-max monotone fix, then linear interpolation within the
/// bracketing bucket — `histogram_quantile`'s convention, the `+Inf` bracket
/// answering the highest finite bound.
fn fold_quantile(
    outcome: &MetricScanOutcome,
    grid: &EvalGrid,
    step_ns: i64,
    range_ns: i64,
    q: f64,
    by: &[String],
    limit: Option<usize>,
) -> Result<Vec<FoldedSeries>, ApiError> {
    struct Bucket<'a> {
        bound: f64,
        samples: &'a [(i64, f64)],
    }
    let mut groups: std::collections::BTreeMap<SeriesLabels, Vec<Bucket>> =
        std::collections::BTreeMap::new();
    for series in &outcome.series {
        let pairs = series.labels.pairs().map_err(ApiError::from_engine)?;
        let Some((_, le)) = pairs.iter().find(|(key, _)| key == "le") else {
            // A `_bucket` series without `le` is not part of any family.
            continue;
        };
        let bound = if le == "+Inf" {
            f64::INFINITY
        } else {
            le.parse::<f64>().map_err(|_| {
                ApiError::from_engine(format!("invalid le boundary '{le}' in a bucket series"))
            })?
        };
        let family: Vec<(String, String)> = pairs
            .iter()
            .filter(|(key, _)| key != "le" && key != crate::series::METRIC_NAME_LABEL)
            .cloned()
            .collect();
        let key = if by.is_empty() {
            SeriesLabels::from_pairs(family)
        } else {
            project_by(&SeriesLabels::from_pairs(family), by)?
        };
        groups.entry(key).or_default().push(Bucket {
            bound,
            samples: &series.samples,
        });
    }

    let mut rows: Vec<FoldedSeries> = Vec::new();
    for (labels, mut buckets) in groups {
        buckets.sort_by(|left, right| left.bound.total_cmp(&right.bound));
        let mut points: Vec<(i64, f64)> = Vec::new();
        for t in grid.timestamps(step_ns) {
            let mut cumulative: Vec<(f64, f64)> = Vec::with_capacity(buckets.len());
            let mut running = 0f64;
            for bucket in &buckets {
                let increase = increase_over(bucket.samples, t, range_ns).unwrap_or(0.0);
                // The monotone fix: cumulative le counts cannot decrease.
                running = running.max(increase);
                cumulative.push((bucket.bound, running));
            }
            let Some(&(_, total)) = cumulative.last() else {
                continue;
            };
            if total <= 0.0 {
                continue;
            }
            let rank = q * total;
            let mut previous_bound = 0.0;
            let mut previous_count = 0.0;
            let mut answer = None;
            for (bound, count) in &cumulative {
                if *count >= rank {
                    answer = Some(if bound.is_infinite() {
                        // The +Inf bracket has no width to interpolate in.
                        previous_bound
                    } else if *count > previous_count {
                        previous_bound
                            + (bound - previous_bound) * ((rank - previous_count)
                                / (count - previous_count))
                    } else {
                        *bound
                    });
                    break;
                }
                previous_bound = *bound;
                previous_count = *count;
            }
            if let Some(value) = answer {
                points.push((t, value));
            }
        }
        if !points.is_empty() {
            rows.push((labels, points));
        }
    }
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    if let Some(limit) = limit {
        rows.truncate(limit);
    }
    Ok(rows)
}