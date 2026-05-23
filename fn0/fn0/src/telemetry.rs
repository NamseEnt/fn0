use opentelemetry::{KeyValue, global};
use std::time::Duration;

const SECONDS_BUCKETS: [f64; 7] = [0.005, 0.025, 0.1, 0.5, 1.0, 5.0, 30.0];

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

pub fn cpu_time(cpu_time: Duration) {
    let histogram = global::meter("fn0")
        .f64_histogram("cpu_time_seconds")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build();
    histogram.record(cpu_time.as_secs_f64(), &[]);
}

pub fn cpu_timeout(cpu_time: Duration) {
    let counter = global::meter("fn0").u64_counter("cpu_timeout").build();
    counter.add(1, &[]);

    let histogram = global::meter("fn0")
        .f64_histogram("cpu_timeout_seconds")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build();
    histogram.record(cpu_time.as_secs_f64(), &[]);
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

pub fn execution_time(route: &str, duration: Duration) {
    let histogram = global::meter("fn0")
        .f64_histogram("execution_time_seconds")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build();
    histogram.record(
        duration.as_secs_f64(),
        &[KeyValue::new("route", route.to_string())],
    );
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
