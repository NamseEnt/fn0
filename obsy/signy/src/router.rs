use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post, put};

use crate::{AppState, admin, collect, query};

pub fn build_router(state: Arc<AppState>) -> Router {
    // The collect route carries its own body limit — none at all — so it is
    // merged as a separate router rather than layered onto this one.
    // `Router::layer` applies to every route registered before it, which would
    // leave the effective limit of a route depending on where in this chain it
    // sits.
    // The read surface is the first-party API alone: the Loki and Tempo
    // compatibility routes went with the read-path decision (issue #3), and
    // the trace routes below are their first-party replacement.
    let router = Router::new()
        .merge(collect_router())
        .route("/metrics", get(query::metrics))
        // The allocator's full report, pulled deliberately rather than
        // scraped: it is a page of text whose per-size-class table is the
        // evidence for where retained memory sits, and no dashboard wants it
        // every fifteen seconds.
        .route("/debug/allocator", get(query::allocator_report))
        .route("/debug/allocator/profile", get(query::allocator_profile))
        .route("/signy/api/v1/logs", get(query::logs))
        .route("/signy/api/v1/logs/histogram", get(query::logs_histogram))
        .route("/signy/api/v1/logs/attributes", get(query::logs_attributes))
        .route(
            "/signy/api/v1/logs/attributes/{key}/values",
            get(query::logs_attribute_values),
        )
        .route("/signy/api/v1/logs/tail", get(query::logs_tail))
        // Moved from /loki/api/v1/delete with the selector grammar: the old
        // route and the new one could not share one parser, and one parser is
        // the point.
        .route(
            "/signy/api/v1/logs/delete",
            post(query::submit_delete_request)
                .get(query::list_delete_requests)
                .delete(query::cancel_delete_request),
        )
        .route("/signy/api/v1/traces", get(query::traces_search))
        .route("/signy/api/v1/traces/{trace_id}", get(query::trace_by_id))
        .route(
            "/signy/api/v1/traces/attributes",
            get(query::traces_attributes),
        )
        .route(
            "/signy/api/v1/traces/attributes/{key}/values",
            get(query::traces_attribute_values),
        )
        .route("/signy/api/v1/metrics/query", get(query::metrics_query))
        .route("/signy/api/v1/metrics/instant", get(query::metrics_instant))
        .route(
            "/signy/api/v1/metrics/quantile",
            get(query::metrics_quantile),
        )
        .route("/signy/api/v1/metrics/names", get(query::metrics_names))
        .route("/signy/api/v1/metrics/labels", get(query::metrics_labels))
        .route(
            "/signy/api/v1/metrics/labels/{key}/values",
            get(query::metrics_label_values),
        )
        .route("/signy/api/v1/metrics/series", get(query::metrics_series))
        .route("/ready", get(query::ready))
        .fallback(query::api_fallback);
    // The admin routes carry no authentication of their own: signy is
    // not built to be reachable from the outside network, and assumes every
    // request arrives through a secured channel.
    router.merge(admin_router()).with_state(state)
}

/// The whole write surface: one route, with no body limit at all.
///
/// The handler reads the body as it arrives and holds one record, so there is
/// nothing here for a limit to protect: what a batch may contain is bounded
/// per record, inside the handler, and how large a batch may be is collecty's
/// decision. A `DefaultBodyLimit` would put that decision back here, and put
/// it back as a number nothing on the collector's side can see.
///
/// It is also where the in-flight body ceiling is charged. The OTLP push
/// routes needed middleware for that, because a handler taking `body: Bytes`
/// has already had the whole thing put in the heap for it by the time its own
/// code runs. This handler is handed the stream instead, and charges each
/// record as it decodes it.
fn collect_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/signy/api/v1/collect", post(collect::collect))
        .layer(DefaultBodyLimit::disable())
}

fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/signy/api/v1/admin/tenants", get(admin::list_tenants))
        .route(
            "/signy/api/v1/admin/tenants/{tenant}/retention",
            put(admin::put_retention)
                .get(admin::get_retention)
                .delete(admin::delete_retention),
        )
        .route(
            "/signy/api/v1/admin/tenants/{tenant}/usage",
            get(admin::get_usage),
        )
        .route(
            "/signy/api/v1/admin/project-tenants/{project_id}/retention",
            put(admin::put_project_retention),
        )
        .route(
            "/signy/api/v1/admin/project-tenants/{project_id}/access/revoke",
            post(admin::revoke_project_access),
        )
        .layer(DefaultBodyLimit::max(admin::MAX_ADMIN_BODY_BYTES))
}
