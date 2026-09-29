/// The HTTP surface of the deletion requests, first-party form: the selector
/// is the same flat filter grammar every read endpoint speaks. What a request
/// means, and why it both hides now and removes later, is in
/// [`crate::delete_requests`].
///
/// `POST /logs/delete` — hide the selected lines now, remove them at the next
/// rewrite. At least one `attr` filter is required, and `parse=` is refused by
/// the parameter grammar: a deletion must name rows that exist, not values a
/// parser derives.
pub async fn submit_delete_request(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<StatusCode, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let now_ns = state.clock.now_ns();
    let params = parse_filter_params(raw.as_deref().unwrap_or(""), now_ns, DELETE_PARAMS)
        .map_err(ApiError::bad_request)?;
    let Some(start_ns) = params.start_ns else {
        return Err(ApiError::bad_request(
            "a delete request needs an explicit start: deleting an unbounded past is a mistake \
this endpoint refuses to guess at"
                .to_string(),
        ));
    };
    // Absent `end` means "up to the moment the request was made". Leaving it
    // open-ended instead would make the request cover lines that had not been
    // written when it was submitted.
    let end_ns = params.end_ns.unwrap_or(now_ns);
    if params.query.matchers.is_empty() {
        return Err(ApiError::bad_request(
            "a delete request must name at least one attr filter, like attr=app=api".to_string(),
        ));
    }
    let query = canonical_filter_query(&params.query);
    state
        .delete_requests
        .submit(&tenant, &query, start_ns, end_ns, now_ns)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(delete_error_response)
}

/// `GET /logs/delete` — the tenant's requests and what has happened to each,
/// one request per NDJSON line. The `query` each carries is the persisted
/// canonical form — resubmittable as-is.
pub async fn list_delete_requests(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<axum::response::Response, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let mut body = String::new();
    for request in state.delete_requests.list(&tenant) {
        body.push_str(&request.to_json().to_string());
        body.push('\n');
    }
    Ok(ndjson_response(body, 0, 0))
}

/// `DELETE /logs/delete?request_id=` — withdraw a request whose rows are
/// still only hidden.
pub async fn cancel_delete_request(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<StatusCode, ApiError> {
    let tenant = crate::tenant::from_headers(&headers, &state.config, &state.tenant_policy)
        .map_err(ApiError::from_tenant)?;
    let mut request_id = None;
    for (key, value) in url::form_urlencoded::parse(raw.as_deref().unwrap_or("").as_bytes()) {
        match key.as_ref() {
            "request_id" => {
                set_once("request_id", &mut request_id, value.into_owned())
                    .map_err(ApiError::bad_request)?;
            }
            other => {
                return Err(ApiError::bad_request(format!(
                    "unknown parameter '{other}': cancelling takes request_id only — see \
docs/QUERY_API.md"
                )));
            }
        }
    }
    let Some(request_id) = request_id else {
        return Err(ApiError::bad_request(
            "cancelling needs request_id=<id>, from the GET listing".to_string(),
        ));
    };
    state
        .delete_requests
        .cancel(&tenant, &request_id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(delete_error_response)
}

fn delete_error_response(error: crate::delete_requests::DeleteRequestError) -> ApiError {
    use crate::delete_requests::{DeleteRequestError, MAX_DELETE_REQUESTS_PER_TENANT};
    match error {
        DeleteRequestError::Invalid(message) => ApiError(StatusCode::BAD_REQUEST, message),
        DeleteRequestError::TooMany => ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            format!(
                "a tenant may hold {MAX_DELETE_REQUESTS_PER_TENANT} deletion requests at once; \
cancel a processed one before submitting another"
            ),
        ),
        DeleteRequestError::NotFound => ApiError(
            StatusCode::NOT_FOUND,
            "no such deletion request for this tenant".to_string(),
        ),
        // The request is not durable, so it must not be reported as accepted.
        DeleteRequestError::Storage(message) => ApiError(StatusCode::SERVICE_UNAVAILABLE, message),
    }
}
