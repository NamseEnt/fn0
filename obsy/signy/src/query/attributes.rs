/// Rows the sampled metadata answers read at most. The value index went with
/// the stream concept, so values come from the newest rows in the window
/// rather than from a catalog.
pub(crate) const METADATA_SAMPLE_ROWS: usize = 1000;

/// What every metadata endpoint has to acquire before it touches a registry.
///
/// These four used to have none of it: no concurrency bound, no deadline, and
/// no time range, so each call read every part of every tenant it was allowed
/// to see. They share the log scan semaphore rather than getting their own,
/// because they compete for the same thing a log query does — the part readers.
struct MetadataGuard {
    _permit: tokio::sync::OwnedSemaphorePermit,
    _occupancy: crate::metrics::ScanOccupancy,
    window: crate::part::MetadataWindow,
    deadline: std::time::Instant,
}

impl MetadataGuard {
    async fn acquire_window(
        state: &Arc<AppState>,
        tenant: &crate::tenant::TenantId,
        window: crate::part::MetadataWindow,
    ) -> Result<Option<Self>, (StatusCode, String)> {
        let window = window.clamped_to(state.tenant_policy.query_floor_ns(tenant, crate::tenant_policy::Signal::Logs));
        // An empty window is a valid question with an empty answer, not an
        // error: a tenant whose retention already passed the requested range
        // asks this on every dashboard refresh.
        if window.is_empty() {
            return Ok(None);
        }
        let permit = match state.query_scan_semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(tokio::sync::TryAcquireError::Closed) => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "query semaphore closed".to_string(),
                ));
            }
            Err(tokio::sync::TryAcquireError::NoPermits) => {
                let queued_at = std::time::Instant::now();
                let permit = state
                    .query_scan_semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .map_err(|_| {
                        (
                            StatusCode::SERVICE_UNAVAILABLE,
                            "query semaphore closed".to_string(),
                        )
                    })?;
                state.metrics.record_scan_queue_wait(queued_at.elapsed());
                permit
            }
        };
        Ok(Some(Self {
            _permit: permit,
            _occupancy: crate::metrics::ScanOccupancy::enter(state.metrics.clone()),
            window,
            deadline: std::time::Instant::now() + state.config.max_query_runtime,
        }))
    }

    /// Checked between units of work rather than enforced by a timer: these
    /// lookups are synchronous walks, so the only place a deadline can act is
    /// where the walk yields control back.
    fn check_deadline(&self) -> Result<(), (StatusCode, String)> {
        if std::time::Instant::now() > self.deadline {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata query exceeded its time budget".to_string(),
            ));
        }
        Ok(())
    }
}

/// The first-party autocomplete endpoints (`docs/QUERY_API.md`): attribute
/// keys in a window, and a key's values, both bounded — keys come from the
/// memtable and the part metadata census, values from the newest
/// `METADATA_SAMPLE_ROWS` rows in the window rather than from a catalog.
/// Rare or old values may not appear over a long range; the doc says so.
pub async fn logs_attributes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let params = attribute_window_params(&state, raw.as_deref().unwrap_or(""), ATTRIBUTE_KEYS_PARAMS)?;
    let Some(guard) = MetadataGuard::acquire_window(&state, &tenant, params.window)
        .await
        .map_err(|(status, message)| ApiError(status, message))?
    else {
        return Ok(ndjson_response(String::new(), 0, 0));
    };

    let mut names = std::collections::BTreeSet::new();
    for name in state.memtable.label_names(&tenant, guard.window) {
        names.insert(name);
    }
    guard
        .check_deadline()
        .map_err(|(status, message)| ApiError(status, message))?;
    for name in state.parts.metadata_key_names(&tenant, guard.window) {
        names.insert(name);
    }

    let mut body = String::new();
    for name in names {
        body.push_str(
            &serde_json::to_string(&serde_json::json!({ "key": name }))
                .expect("a key serializes infallibly"),
        );
        body.push('\n');
    }
    Ok(ndjson_response(body, 0, 0))
}

pub async fn logs_attribute_values(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(key): Path<String>,
    RawQuery(raw): RawQuery,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let params =
        attribute_window_params(&state, raw.as_deref().unwrap_or(""), ATTRIBUTE_VALUES_PARAMS)?;
    let Some(guard) = MetadataGuard::acquire_window(&state, &tenant, params.window)
        .await
        .map_err(|(status, message)| ApiError(status, message))?
    else {
        return Ok(ndjson_response(String::new(), 0, 0));
    };

    let mut values = std::collections::BTreeSet::new();
    if params.matchers.is_empty() {
        for value in state.memtable.label_values(&tenant, &key, guard.window) {
            values.insert(value);
        }
        guard
            .check_deadline()
            .map_err(|(status, message)| ApiError(status, message))?;
        for metadata in state
            .parts
            .sample_metadata(&tenant, guard.window, METADATA_SAMPLE_ROWS)
            .map_err(|error| ApiError(StatusCode::INTERNAL_SERVER_ERROR, error))?
        {
            for (name, value) in metadata {
                if name == key {
                    values.insert(value);
                }
            }
        }
    } else {
        // A dropdown built without the other filters offers values that
        // belong to other rows and return nothing when clicked, so the
        // matchers are evaluated against each sampled row's own metadata.
        for labels in state
            .memtable
            .series(&tenant, &params.matchers, guard.window)
        {
            if let Some(value) = labels.get(&key) {
                values.insert(value.clone());
            }
        }
        guard
            .check_deadline()
            .map_err(|(status, message)| ApiError(status, message))?;
        for metadata in state
            .parts
            .sample_metadata(&tenant, guard.window, METADATA_SAMPLE_ROWS)
            .map_err(|error| ApiError(StatusCode::INTERNAL_SERVER_ERROR, error))?
        {
            let set: crate::memtable::Labels = metadata.into_iter().collect();
            if params.matchers.iter().all(|matcher| matcher.matches(&set))
                && let Some(value) = set.get(&key)
            {
                values.insert(value.clone());
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
    Ok(ndjson_response(body, 0, 0))
}

struct AttributeParams {
    window: part::MetadataWindow,
    matchers: Vec<logql::LabelMatcher>,
}

fn attribute_window_params(
    state: &Arc<AppState>,
    raw: &str,
    allowed: &'static [&'static str],
) -> Result<AttributeParams, ApiError> {
    let now_ns = state.clock.now_ns();
    let params = parse_filter_params(raw, now_ns, allowed).map_err(ApiError::bad_request)?;
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
    Ok(AttributeParams {
        // Inclusive `end`, like every metadata window: these endpoints answer
        // "did this attribute exist in the window", not "which rows are in
        // the window".
        window: part::MetadataWindow { start_ns, end_ns },
        matchers: params.query.matchers,
    })
}
