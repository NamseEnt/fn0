/// The metric read path's scan orchestrator. Admission runs in the fixed
/// order every read surface uses (`trace_scan.rs` states it): pin — a restore
/// is network wait and must not hold a scan slot — then a permit from the
/// metric surface's own semaphore (Gorilla decode plus per-step folds is a
/// third cost profile), then — uniquely on this surface — selection, and only
/// then admission to the memory account, because a metric query's cost is a
/// function of what the selector matched and selection is the cheap part. The
/// other two surfaces price themselves before the permit; this one cannot,
/// and does not need to: selection reads catalogs only. What follows is that
/// the cost is
/// **bounded before any chunk is decoded**: selection runs over the memtable
/// index and the part catalogs alone, and a selection past the caps is
/// refused with the matched count and the knob, never scanned.
///
/// Refused fixes deliberately do not offer `agg`: the scan decodes every
/// matched series whether or not the fold aggregates, so aggregation changes
/// the answer's size, not the scan's.
pub(crate) struct MetricScanRequest {
    pub metric: Option<String>,
    pub filters: Vec<MetricFilter>,
    /// The evaluation window; decoding reaches back a further
    /// `decode_margin_ns` so windowed folds have their bases.
    pub start_ns: i64,
    pub end_ns: i64,
    pub steps: u64,
    pub decode_margin_ns: i64,
}

pub(crate) struct MetricSeriesData {
    pub labels: SeriesLabels,
    /// Time-sorted, exact-timestamp duplicates collapsed (memtable and part
    /// may briefly both hold a sample around a flush; one survives).
    pub samples: Vec<(i64, f64)>,
}

pub(crate) struct MetricScanOutcome {
    pub series: Vec<MetricSeriesData>,
    pub decoded_samples: u64,
    pub estimated_bytes: u64,
    /// Held with the series it paid for; the handler drops both together
    /// after the NDJSON body is built (the `QueryExecution` precedent).
    _memory_charge: crate::memory_budget::MemoryCharge,
}

/// Whether a series identity passes the request's name and label filters.
/// Whether one series' canonical labels satisfy the selector.
///
/// Takes the bytes rather than a `SeriesLabels` because the caller usually
/// has no owned identity yet: this runs on every catalog row in the query's
/// window, and only a row that matches is worth materializing. A malformed
/// payload reads as no match rather than as an error — these bytes crossed a
/// checksum and were decoded once when the part opened, so corruption is
/// caught before a query can see it.
/// Whether a stored histogram could answer anything this request asked for.
///
/// A necessary condition, checked before its points are read: the selector's
/// metric must be one of the names this instrument answers as, and every
/// filter that a synthetic series inherits unchanged must already match. `le`
/// is the exception — it exists only on the series synthesis produces.
fn histogram_family_candidate(labels: &[u8], request: &MetricScanRequest) -> bool {
    if let Some(metric) = &request.metric {
        let Some(base) = crate::series::histogram_base_name(metric) else {
            return false;
        };
        let stored = crate::series::canonical_pairs(labels)
            .filter_map(Result::ok)
            .find(|(key, _)| *key == crate::series::METRIC_NAME_LABEL)
            .map(|(_, value)| value);
        if stored != Some(base) {
            return false;
        }
    }
    request
        .filters
        .iter()
        .filter(|filter| filter.key() != "le")
        .all(|filter| {
            filter.matches(&|key: &str| {
                crate::series::canonical_pairs(labels)
                    .filter_map(Result::ok)
                    .find(|(name, _)| *name == key)
                    .map(|(_, value)| value)
            })
        })
}

fn metric_labels_match(labels: &[u8], metric: &Option<String>, filters: &[MetricFilter]) -> bool {
    let lookup = |key: &str| {
        crate::series::canonical_pairs(labels)
            .filter_map(Result::ok)
            .find(|(name, _)| *name == key)
            .map(|(_, value)| value)
    };
    if let Some(metric) = metric
        && lookup(crate::series::METRIC_NAME_LABEL) != Some(metric.as_str())
    {
        return false;
    }
    filters.iter().all(|filter| filter.matches(&lookup))
}

async fn scan_metric_series(
    state: Arc<AppState>,
    tenant: TenantId,
    request: MetricScanRequest,
) -> Result<MetricScanOutcome, ApiError> {
    let max_runtime = state.config.max_metric_query_runtime;
    let decode_start_ns = request.start_ns.saturating_sub(request.decode_margin_ns);
    let guard =
        pin_metric_parts(&state, &tenant, &request, decode_start_ns, request.end_ns).await?;
    let scan_permit = tokio::time::timeout(
        max_runtime,
        state.metric_scan_semaphore.clone().acquire_owned(),
    )
    .await
    .map_err(|_| ApiError::from_engine("metric query timed out".to_string()))?
    .map_err(|error| ApiError::from_engine(format!("metric scan scheduler is closed: {error}")))?;

    let cancellation = Arc::new(AtomicBool::new(false));
    let task_cancellation = cancellation.clone();
    let max_series = state.config.max_metric_series_per_query;
    let max_points = state.config.max_metric_points_per_query;
    let memory_account = state.memory_account.clone();
    let task_state = state.clone();
    let mut task = tokio::task::spawn_blocking(move || {
        // Keep the permit until the blocking task actually exits; a cancelled
        // request must not admit a second scan while this one still burns CPU.
        let _scan_permit = scan_permit;
        let _guard = guard;
        let _arena = crate::memprof::enter(crate::memprof::Arena::Query);

        // Selection: the memtable index and the part catalogs, no chunk read.
        let memtable = task_state.journal.series_memtable();
        struct Sources {
            in_memtable: bool,
            parts: Vec<(
                Arc<crate::series_part::SeriesPartReader>,
                crate::series_part::ChunkRef,
            )>,
        }
        let mut selected: std::collections::BTreeMap<SeriesLabels, Sources> =
            std::collections::BTreeMap::new();
        // A stored histogram is one identity that answers as many. It is
        // selected by its own name and gathered by its points; which of the
        // series it can answer as the selector actually wanted is decided
        // after they exist.
        let mut histograms: std::collections::BTreeMap<SeriesLabels, Sources> =
            std::collections::BTreeMap::new();
        for labels in memtable.series_labels(&tenant) {
            if metric_labels_match(labels.as_bytes(), &request.metric, &request.filters) {
                selected.entry(labels).or_insert(Sources {
                    in_memtable: true,
                    parts: Vec::new(),
                });
            } else if histogram_family_candidate(labels.as_bytes(), &request)
                && !memtable.histogram_points_of(&tenant, &labels).is_empty()
            {
                histograms.entry(labels).or_insert(Sources {
                    in_memtable: true,
                    parts: Vec::new(),
                });
            }
        }
        for reader in task_state.series_parts.snapshot() {
            if task_cancellation.load(Ordering::Acquire) {
                return Err("metric query timed out".to_string());
            }
            if !reader
                .part()
                .meta
                .overlaps_range(decode_start_ns, request.end_ns)
            {
                continue;
            }
            let window = reader.window(decode_start_ns, request.end_ns);
            for row in reader.tenant_catalog(&tenant).iter() {
                if !row.overlaps(window) {
                    continue;
                }
                let into = match row.kind {
                    crate::series_part::SeriesRowKind::Scalar => {
                        if !metric_labels_match(row.labels, &request.metric, &request.filters) {
                            continue;
                        }
                        &mut selected
                    }
                    crate::series_part::SeriesRowKind::Histogram => {
                        if !histogram_family_candidate(row.labels, &request) {
                            continue;
                        }
                        &mut histograms
                    }
                };
                // Only a row that matched becomes an owned identity. The walk
                // above touched the fixed-stride row array and nothing else.
                let labels = SeriesLabels::from_canonical(row.labels.to_vec());
                let sources = into.entry(labels).or_insert(Sources {
                    in_memtable: false,
                    parts: Vec::new(),
                });
                sources.parts.push((reader.clone(), row.chunk));
            }
        }

        // The bound, enforced while nothing has been decoded.
        let matched = selected.len();
        if matched > max_series {
            return Err(format!(
                "metric selection exceeds the maximum of {max_series} series: the selector \
matched {matched} (SIGNY_MAX_METRIC_SERIES_PER_QUERY) — narrow the selector with more \
attr filters"
            ));
        }
        let points = (matched as u64).saturating_mul(request.steps);
        if points > max_points as u64 {
            return Err(format!(
                "metric selection exceeds the maximum of {max_points} output points: {matched} \
series over {} steps is {points} (SIGNY_MAX_METRIC_POINTS_PER_QUERY) — narrow the \
selector, shorten the window, or coarsen step",
                request.steps
            ));
        }
        let label_bytes: u64 = selected
            .keys()
            .map(|labels| labels.byte_len() as u64)
            .sum();
        // Selection is over, nothing is decoded, and the answer's size is now
        // known exactly: `series × steps` timestamped doubles plus the labels
        // that name them. That is the whole admission — a metric query never
        // asks the account twice.
        let mut estimated_bytes = points.saturating_mul(16).saturating_add(label_bytes);
        let memory_charge = memory_account.admit(estimated_bytes)?;

        let mut series = Vec::with_capacity(selected.len());
        let mut decoded_samples = 0u64;
        for (labels, sources) in selected {
            if task_cancellation.load(Ordering::Acquire) {
                return Err("metric query timed out".to_string());
            }
            let mut samples = if sources.in_memtable {
                memtable.sorted_samples_of(&tenant, &labels)?
            } else {
                Vec::new()
            };
            for (reader, chunk) in &sources.parts {
                samples.extend(reader.read_series(*chunk)?);
            }
            samples.retain(|(ts, _)| *ts >= decode_start_ns && *ts <= request.end_ns);
            samples.sort_by_key(|(ts, _)| *ts);
            samples.dedup_by_key(|(ts, _)| *ts);
            decoded_samples += samples.len() as u64;
            estimated_bytes = estimated_bytes.saturating_add(samples.len() as u64 * 16);
            series.push(MetricSeriesData { labels, samples });
        }

        // Each stored histogram is expanded once and then asked, per series it
        // can answer as, whether the selector wanted that one. The expansion
        // happens after the scalar walk so the caps below see both.
        for (labels, sources) in histograms {
            if task_cancellation.load(Ordering::Acquire) {
                return Err("metric query timed out".to_string());
            }
            let mut points = if sources.in_memtable {
                memtable.histogram_points_of(&tenant, &labels)
            } else {
                Vec::new()
            };
            for (reader, chunk) in &sources.parts {
                points.extend(reader.read_histogram_points(*chunk)?);
            }
            points.retain(|(ts, _)| *ts >= decode_start_ns && *ts <= request.end_ns);
            points.sort_by_key(|(ts, _)| *ts);
            points.dedup_by_key(|(ts, _)| *ts);
            for (synthetic, mut samples) in
                crate::series::synthesize_histogram_series(&labels, &points)?
            {
                if !metric_labels_match(synthetic.as_bytes(), &request.metric, &request.filters) {
                    continue;
                }
                samples.sort_by_key(|(ts, _)| *ts);
                if series.len() >= max_series {
                    return Err(format!(
                        "metric selection exceeds the maximum of {max_series} series \
(SIGNY_MAX_METRIC_SERIES_PER_QUERY) — narrow the selector with more attr filters"
                    ));
                }
                decoded_samples += samples.len() as u64;
                estimated_bytes = estimated_bytes
                    .saturating_add(samples.len() as u64 * 16)
                    .saturating_add(synthetic.byte_len() as u64);
                series.push(MetricSeriesData {
                    labels: synthetic,
                    samples,
                });
            }
        }
        // Decoding reaches back past the window for windowed folds, so the
        // samples held can exceed the output points the admission was priced
        // on. Reconcile before the fold runs on them.
        memory_charge.reconcile(estimated_bytes);
        Ok::<_, String>(MetricScanOutcome {
            series,
            decoded_samples,
            estimated_bytes,
            _memory_charge: memory_charge,
        })
    });

    let outcome = match tokio::time::timeout(max_runtime, &mut task).await {
        Ok(result) => result
            .map_err(|error| ApiError::from_engine(format!("metric query task failed: {error}")))?
            .map_err(ApiError::from_engine)?,
        Err(_) => {
            cancellation.store(true, Ordering::Release);
            let _ = task.await;
            return Err(ApiError::from_engine("metric query timed out".to_string()));
        }
    };
    Ok(outcome)
}

/// Pin the metric parts the scan will read: the tenant's parts overlapping
/// the decode window, pruned by the bloom for every equality the selector
/// states — pinning is what downloads a body, so the pruning has to reach
/// this far or the catalog-side pruning saves nothing.
async fn pin_metric_parts(
    state: &AppState,
    tenant: &TenantId,
    request: &MetricScanRequest,
    decode_start_ns: i64,
    end_ns: i64,
) -> Result<tokio::sync::OwnedRwLockReadGuard<()>, ApiError> {
    let metric = request.metric.clone();
    let equalities: Vec<(String, String)> = request
        .filters
        .iter()
        .filter_map(|filter| match filter {
            MetricFilter::Eq { key, value } => Some((key.clone(), value.clone())),
            _ => None,
        })
        .collect();
    let series_parts = state.series_parts.clone();
    let tenant = tenant.clone();
    crate::remote_lifecycle::pin_remote_parts(
        state.parts.operation_lock(),
        state.remote_cache.clone(),
        move || {
            series_parts
                .snapshot()
                .into_iter()
                .filter(|reader| {
                    let meta = &reader.part().meta;
                    meta.tenant_segment(&tenant).is_some()
                        && meta.overlaps_range(decode_start_ns, end_ns)
                        && metric
                            .as_ref()
                            .is_none_or(|name| {
                                reader.may_match_pair(crate::series::METRIC_NAME_LABEL, name)
                            })
                        && equalities
                            .iter()
                            .all(|(key, value)| reader.may_match_pair(key, value))
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
