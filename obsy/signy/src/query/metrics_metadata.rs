/// Metric discovery: names, label keys, label values, series. Unlike the log
/// attribute endpoints these are **exact** — a series' identity lives in the
/// memtable index and the part catalogs, so no row sampling is involved and
/// `data.bin` is never read. Live series appear regardless of the window
/// (their index entry is the statement that they are current); part catalogs
/// are pruned to it.
fn metric_discovery_window(
    state: &AppState,
    tenant: &TenantId,
    params: &MetricParams,
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
    let start_ns = clamp_to_retention(start_ns, state.tenant_policy.query_floor_ns(tenant, crate::tenant_policy::Signal::Metrics));
    if start_ns > end_ns {
        return Ok(None);
    }
    Ok(Some((start_ns, end_ns)))
}

/// Every selected series identity: the memtable's live entries plus the
/// catalog entries overlapping the window, filtered by the optional
/// `metric`/`attr` narrowing.
fn discover_series(
    state: &AppState,
    tenant: &TenantId,
    params: &MetricParams,
    window: (i64, i64),
) -> Result<std::collections::BTreeSet<SeriesLabels>, ApiError> {
    let (start_ns, end_ns) = window;
    let mut series = std::collections::BTreeSet::new();
    // Windowed on both halves: the memtable answers from the bounds it
    // records per series, the parts from their catalogs, so neither offers a
    // key whose samples all sit outside the window the caller asked about.
    for labels in state
        .journal
        .series_memtable()
        .series_labels_in_range(tenant, start_ns, end_ns)
    {
        if metric_labels_match(labels.as_bytes(), &params.metric, &params.filters) {
            series.insert(labels);
            continue;
        }
        // A stored histogram is discoverable under the names it answers as,
        // not under the one it is stored as: a caller that asks what series
        // exist has to be told the same thing a query would answer from.
        let points = state
            .journal
            .series_memtable()
            .histogram_points_of(tenant, &labels);
        if !points.is_empty() {
            insert_synthetic(&mut series, &labels, &points, params)?;
        }
    }
    for reader in state.series_parts.snapshot() {
        if !reader.part().meta.overlaps_range(start_ns, end_ns) {
            continue;
        }
        let window = reader.window(start_ns, end_ns);
        for row in reader.tenant_catalog(tenant).iter() {
            if !row.overlaps(window) {
                continue;
            }
            match row.kind {
                crate::series_part::SeriesRowKind::Scalar => {
                    if metric_labels_match(row.labels, &params.metric, &params.filters) {
                        series.insert(SeriesLabels::from_canonical(row.labels.to_vec()));
                    }
                }
                crate::series_part::SeriesRowKind::Histogram => {
                    let labels = SeriesLabels::from_canonical(row.labels.to_vec());
                    let points = reader
                        .read_histogram_points(row.chunk)
                        .map_err(ApiError::from_engine)?;
                    insert_synthetic(&mut series, &labels, &points, params)?;
                }
            }
        }
    }
    Ok(series)
}

/// Expand one stored histogram and keep the identities the narrowing wanted.
fn insert_synthetic(
    series: &mut std::collections::BTreeSet<SeriesLabels>,
    labels: &SeriesLabels,
    points: &[(i64, crate::series::HistogramPoint)],
    params: &MetricParams,
) -> Result<(), ApiError> {
    for (synthetic, _) in crate::series::synthesize_histogram_series(labels, points)
        .map_err(ApiError::from_engine)?
    {
        if metric_labels_match(synthetic.as_bytes(), &params.metric, &params.filters) {
            series.insert(synthetic);
        }
    }
    Ok(())
}

struct DiscoveryAnswer {
    body: String,
    rows: u64,
}

fn discovery_lines<T: Ord + serde::Serialize>(
    field: &str,
    values: std::collections::BTreeSet<T>,
) -> DiscoveryAnswer {
    let rows = values.len() as u64;
    let mut body = String::new();
    for value in values {
        body.push_str(
            &serde_json::to_string(&serde_json::json!({ field: value }))
                .expect("a discovery row serializes infallibly"),
        );
        body.push('\n');
    }
    DiscoveryAnswer { body, rows }
}

/// Hold the parts discovery is about to read, for as long as it reads them.
///
/// A metric part's body is evictable — `SeriesPartReader::open_cached` exists
/// for exactly that, and `evict_metric_cache` acts on it — and this route
/// reaches the body: a stored histogram is enumerated under the names it
/// answers as, which means decoding its chunk. The scan path pins for the same
/// reason. Discovery did not, so a body the disk-cache budget had taken back
/// reached the caller as the reader's bare `No such file or directory`, a 500
/// on a request the store could have answered.
///
/// The guard is the other half: it is the lock metric compaction takes to
/// swap a tier and unlink its inputs, so holding it also keeps the directory
/// under a reader that has already planned its read.
///
/// The required set is narrower than the scan's on purpose. Discovery needs a
/// body only where a histogram row falls in the window, so asking a store of
/// scalars what series it holds still restores nothing.
async fn pin_discovery_parts(
    state: &AppState,
    tenant: &TenantId,
    window: (i64, i64),
) -> Result<tokio::sync::OwnedRwLockReadGuard<()>, ApiError> {
    let (start_ns, end_ns) = window;
    let series_parts = state.series_parts.clone();
    let owner = tenant.clone();
    crate::remote_lifecycle::pin_remote_parts(
        state.parts.operation_lock(),
        state.remote_cache.clone(),
        move || {
            series_parts
                .snapshot()
                .into_iter()
                .filter(|reader| {
                    let meta = &reader.part().meta;
                    meta.tenant_segment(&owner).is_some()
                        && meta.overlaps_range(start_ns, end_ns)
                })
                .filter(|reader| {
                    let catalog_window = reader.window(start_ns, end_ns);
                    reader.tenant_catalog(&owner).iter().any(|row| {
                        row.overlaps(catalog_window)
                            && matches!(row.kind, crate::series_part::SeriesRowKind::Histogram)
                    })
                })
                .map(|reader| reader.part().meta.id.clone())
                .collect()
        },
        |required| state.series_parts.missing_data_ids(required),
        crate::remote_lifecycle::RemoteDomain::Metrics,
        state.config.max_metric_restore_runtime,
        || Ok(()),
        Some(state.metrics.clone()),
    )
    .await
    .map_err(pin_error)
}

async fn metric_discovery(
    state: Arc<AppState>,
    headers: HeaderMap,
    raw: Option<String>,
    allowed: &'static [&'static str],
    endpoint: crate::metrics::QueryEndpoint,
    answer: impl FnOnce(
        &AppState,
        &MetricParams,
        std::collections::BTreeSet<SeriesLabels>,
    ) -> Result<DiscoveryAnswer, ApiError>,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_metric_params(raw.as_deref().unwrap_or(""), now_ns, allowed)
        .map_err(ApiError::bad_request)?;
    let Some(window) = metric_discovery_window(&state, &tenant, &params, now_ns)? else {
        return Ok(ndjson_response(String::new(), 0, 0));
    };
    // Pinned before the slot, the order every read surface uses: a restore is
    // network wait and must not hold a tenant's query slot while it waits.
    let _pinned = pin_discovery_parts(&state, &tenant, window).await?;
    let _slot = state.tenant_quota.begin_query(&tenant).map_err(|error| {
        ApiError::from_engine(format!("{TENANT_QUOTA_PREFIX}{}", error.message))
    })?;
    let metrics = state.metrics.clone();
    let started = std::time::Instant::now();
    let result =
        discover_series(&state, &tenant, &params, window).and_then(|series| {
            answer(&state, &params, series)
        });
    metrics.observe_query(endpoint, started.elapsed());
    match result {
        Ok(answer) => {
            metrics
                .query_success
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(ndjson_response(answer.body, answer.rows, 0))
        }
        Err(error) => {
            metrics
                .query_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Err(error)
        }
    }
}

pub async fn metrics_names(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    metric_discovery(
        state,
        headers,
        raw,
        METRIC_NAMES_PARAMS,
        crate::metrics::QueryEndpoint::MetricNames,
        |_, _, series| {
            let mut names = std::collections::BTreeSet::new();
            for labels in &series {
                if let Some(name) = labels.metric_name() {
                    names.insert(name);
                }
            }
            Ok(discovery_lines("name", names))
        },
    )
    .await
}

pub async fn metrics_labels(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    metric_discovery(
        state,
        headers,
        raw,
        METRIC_LABELS_PARAMS,
        crate::metrics::QueryEndpoint::MetricLabels,
        |_, _, series| {
            let mut keys = std::collections::BTreeSet::new();
            for labels in &series {
                for (key, _) in labels.pairs().map_err(ApiError::from_engine)? {
                    if key != crate::series::METRIC_NAME_LABEL {
                        keys.insert(key);
                    }
                }
            }
            Ok(discovery_lines("key", keys))
        },
    )
    .await
}

pub async fn metrics_label_values(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(key): Path<String>,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    metric_discovery(
        state,
        headers,
        raw,
        METRIC_LABEL_VALUES_PARAMS,
        crate::metrics::QueryEndpoint::MetricLabelValues,
        move |_, _, series| {
            let mut values = std::collections::BTreeSet::new();
            for labels in &series {
                for (name, value) in labels.pairs().map_err(ApiError::from_engine)? {
                    if name == key {
                        values.insert(value);
                    }
                }
            }
            Ok(discovery_lines("value", values))
        },
    )
    .await
}

pub async fn metrics_series(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    metric_discovery(
        state,
        headers,
        raw,
        METRIC_SERIES_PARAMS,
        crate::metrics::QueryEndpoint::MetricSeries,
        |_, params, series| {
            // This route enumerates identities, so — alone on the metric
            // surface — its labels objects keep `__name__`: without it two
            // metrics' series would be indistinguishable.
            let mut body = String::new();
            let mut rows = 0u64;
            for labels in series.iter().take(params.limit.unwrap_or(usize::MAX)) {
                body.push_str(
                    &serde_json::to_string(&serde_json::json!({
                        "labels": labels_object(labels)?,
                    }))
                    .expect("a series row serializes infallibly"),
                );
                body.push('\n');
                rows += 1;
            }
            Ok(DiscoveryAnswer { body, rows })
        },
    )
    .await
}