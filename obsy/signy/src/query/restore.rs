async fn pin_query_parts(
    state: &AppState,
    tenant: &TenantId,
    parsed: &logql::LogQuery,
    range: part::QueryTimeRange,
) -> Result<tokio::sync::OwnedRwLockReadGuard<()>, String> {
    pin_query_parts_with_gap_hook(state, tenant, parsed, range, || Ok(())).await
}

async fn pin_query_parts_with_gap_hook<F>(
    state: &AppState,
    tenant: &TenantId,
    parsed: &logql::LogQuery,
    range: part::QueryTimeRange,
    after_read_guard: F,
) -> Result<tokio::sync::OwnedRwLockReadGuard<()>, String>
where
    F: FnOnce() -> Result<(), String>,
{
    let exact_fields = parsed.exact_field_predicates();
    let guard = crate::remote_lifecycle::pin_remote_parts(
        state.parts.operation_lock(),
        state.remote_cache.clone(),
        || {
            state.parts.candidate_part_ids_with_exact_fields(
                tenant,
                &parsed.line_filters,
                &exact_fields,
                range,
            )
        },
        |required| state.parts.missing_data_ids(required),
        crate::remote_lifecycle::RemoteDomain::Logs,
        state.config.max_restore_runtime,
        after_read_guard,
        Some(state.metrics.clone()),
    )
    .await
    .map_err(|error| error.into_http().1)?;
    Ok(guard)
}
