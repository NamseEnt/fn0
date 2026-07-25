use opentelemetry::{KeyValue, global};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const SECONDS_BUCKETS: [f64; 7] = [0.005, 0.025, 0.1, 0.5, 1.0, 5.0, 30.0];
const UNKNOWN_ROUTE: &str = "unknown";
pub const MAX_ROUTES_PER_PROJECT: usize = 40;

static PROJECT_ROUTES: OnceLock<Mutex<HashMap<String, HashSet<String>>>> = OnceLock::new();

pub fn wasmtime_error(func: &'static str, error: &str) {
    let counter = global::meter("fn0").u64_counter("wasmtime_error").build();
    counter.add(
        1,
        &[
            KeyValue::new("func", func),
            KeyValue::new("error", error.to_string()),
        ],
    );
}

pub fn oneshot_drop_before_response() {
    let counter = global::meter("fn0")
        .u64_counter("oneshot_drop_before_response")
        .build();
    counter.add(1, &[]);
}

pub fn proxy_returns_error_code(error_code: &str) {
    let counter = global::meter("fn0")
        .u64_counter("proxy_returns_error_code")
        .build();
    counter.add(1, &[KeyValue::new("error_code", error_code.to_string())]);
}

pub fn request_task_join_error(error: &str) {
    let counter = global::meter("fn0")
        .u64_counter("request_task_join_error")
        .build();
    counter.add(1, &[KeyValue::new("error", error.to_string())]);
}

pub fn cpu_time(project_id: &str, cpu_time: Duration) {
    let histogram = global::meter("fn0")
        .f64_histogram("cpu_time_seconds")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build();
    histogram.record(
        cpu_time.as_secs_f64(),
        &[KeyValue::new("fn0.project_id", project_id.to_string())],
    );
}

pub fn cpu_timeout(project_id: &str, cpu_time: Duration) {
    let counter = global::meter("fn0").u64_counter("cpu_timeout").build();
    counter.add(
        1,
        &[KeyValue::new("fn0.project_id", project_id.to_string())],
    );

    let histogram = global::meter("fn0")
        .f64_histogram("cpu_timeout_seconds")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build();
    histogram.record(
        cpu_time.as_secs_f64(),
        &[KeyValue::new("fn0.project_id", project_id.to_string())],
    );
}

pub fn trapped(trap: &str) {
    let counter = global::meter("fn0").u64_counter("trapped").build();
    counter.add(1, &[KeyValue::new("trap", trap.to_string())]);
}

pub fn canceled_unexpectedly(error: &str) {
    let counter = global::meter("fn0")
        .u64_counter("canceled_unexpectedly")
        .build();
    counter.add(1, &[KeyValue::new("error", error.to_string())]);
}

pub fn create_instance() {
    let counter = global::meter("fn0").u64_counter("create_instance").build();
    counter.add(1, &[]);
}

pub fn proxy_cache_error(error: &str) {
    let counter = global::meter("fn0")
        .u64_counter("proxy_cache_error")
        .build();
    counter.add(1, &[KeyValue::new("error", error.to_string())]);
}

pub fn project_id_parse_error() {
    let counter = global::meter("fn0")
        .u64_counter("project_id_parse_error")
        .build();
    counter.add(1, &[]);
}

pub fn function_invocation() {
    let counter = global::meter("fn0")
        .u64_counter("function_invocation")
        .build();
    counter.add(1, &[]);
}

pub fn execution_time(project_id: &str, route: &str, duration: Duration, status_code: u16) {
    let route = bounded_route(project_id, route);
    let histogram = global::meter("fn0")
        .f64_histogram("execution_time_seconds")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build();
    histogram.record(
        duration.as_secs_f64(),
        &[
            KeyValue::new("fn0.project_id", project_id.to_string()),
            KeyValue::new("route", route.clone()),
        ],
    );

    if let Some(status_class) = error_status_class(status_code) {
        let counter = global::meter("fn0").u64_counter("request_errors").build();
        counter.add(
            1,
            &[
                KeyValue::new("fn0.project_id", project_id.to_string()),
                KeyValue::new("route", route),
                KeyValue::new("status_class", status_class),
            ],
        );
    }
}

pub fn panicked() {
    let counter = global::meter("fn0").u64_counter("panicked").build();
    counter.add(1, &[]);
}

pub fn request_deadline_exceeded() {
    let counter = global::meter("fn0")
        .u64_counter("request_deadline_exceeded")
        .build();
    counter.add(1, &[]);
}

pub fn stage_duration(stage: &'static str, duration: Duration) {
    let histogram = global::meter("fn0")
        .f64_histogram("stage_duration_seconds")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build();
    histogram.record(duration.as_secs_f64(), &[KeyValue::new("stage", stage)]);
}

fn bounded_route(project_id: &str, route: &str) -> String {
    if route == UNKNOWN_ROUTE {
        return UNKNOWN_ROUTE.to_string();
    }

    let routes = PROJECT_ROUTES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut routes = routes.lock().unwrap_or_else(|error| error.into_inner());
    let project_routes = routes.entry(project_id.to_string()).or_default();

    if project_routes.contains(route) {
        return route.to_string();
    }

    if project_routes.len() < MAX_ROUTES_PER_PROJECT - 1 {
        project_routes.insert(route.to_string());
        return route.to_string();
    }

    UNKNOWN_ROUTE.to_string()
}

fn error_status_class(status_code: u16) -> Option<&'static str> {
    match status_code {
        400..=499 => Some("4xx"),
        500..=599 => Some("5xx"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_ROUTES_PER_PROJECT, bounded_route, error_status_class};

    #[test]
    fn bounds_routes_per_project() {
        let project_id = "telemetry-route-cap-test";

        for route_number in 0..(MAX_ROUTES_PER_PROJECT - 1) {
            let route = format!("/route-{route_number}");
            assert_eq!(bounded_route(project_id, &route), route);
        }

        assert_eq!(bounded_route(project_id, "/overflow"), "unknown");
        assert_eq!(bounded_route(project_id, "/route-0"), "/route-0");
    }

    #[test]
    fn classifies_error_statuses() {
        assert_eq!(error_status_class(399), None);
        assert_eq!(error_status_class(400), Some("4xx"));
        assert_eq!(error_status_class(499), Some("4xx"));
        assert_eq!(error_status_class(500), Some("5xx"));
        assert_eq!(error_status_class(599), Some("5xx"));
        assert_eq!(error_status_class(600), None);
    }
}
