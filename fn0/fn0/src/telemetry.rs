use opentelemetry::{KeyValue, global};
use std::time::Duration;

pub fn wasmtime_error(func: &'static str, code_id: &str, error: &str) {
    let counter = global::meter("fn0").u64_counter("wasmtime_error").build();
    counter.add(
        1,
        &[
            KeyValue::new("func", func),
            KeyValue::new("code_id", code_id.to_string()),
            KeyValue::new("error", error.to_string()),
        ],
    );
}

pub fn oneshot_drop_before_response(code_id: &str) {
    let counter = global::meter("fn0")
        .u64_counter("oneshot_drop_before_response")
        .build();
    counter.add(1, &[KeyValue::new("code_id", code_id.to_string())]);
}

pub fn proxy_returns_error_code(code_id: &str, error_code: &str) {
    let counter = global::meter("fn0")
        .u64_counter("proxy_returns_error_code")
        .build();
    counter.add(
        1,
        &[
            KeyValue::new("code_id", code_id.to_string()),
            KeyValue::new("error_code", error_code.to_string()),
        ],
    );
}

pub fn request_task_join_error(code_id: &str, error: &str) {
    let counter = global::meter("fn0")
        .u64_counter("request_task_join_error")
        .build();
    counter.add(
        1,
        &[
            KeyValue::new("code_id", code_id.to_string()),
            KeyValue::new("error", error.to_string()),
        ],
    );
}

pub fn cpu_time(code_id: &str, cpu_time: Duration) {
    let histogram = global::meter("fn0")
        .f64_histogram("cpu_time_seconds")
        .build();
    histogram.record(
        cpu_time.as_secs_f64(),
        &[KeyValue::new("code_id", code_id.to_string())],
    );
}

pub fn cpu_timeout(code_id: &str, cpu_time: Duration) {
    let counter = global::meter("fn0").u64_counter("cpu_timeout").build();
    counter.add(1, &[KeyValue::new("code_id", code_id.to_string())]);

    // Also record the timeout duration
    let histogram = global::meter("fn0")
        .f64_histogram("cpu_timeout_seconds")
        .build();
    histogram.record(
        cpu_time.as_secs_f64(),
        &[KeyValue::new("code_id", code_id.to_string())],
    );
}

pub fn trapped(code_id: &str, trap: &str) {
    let counter = global::meter("fn0").u64_counter("trapped").build();
    counter.add(
        1,
        &[
            KeyValue::new("code_id", code_id.to_string()),
            KeyValue::new("trap", trap.to_string()),
        ],
    );
}

pub fn canceled_unexpectedly(code_id: &str, error: &str) {
    let counter = global::meter("fn0")
        .u64_counter("canceled_unexpectedly")
        .build();
    counter.add(
        1,
        &[
            KeyValue::new("code_id", code_id.to_string()),
            KeyValue::new("error", error.to_string()),
        ],
    );
}

pub fn create_instance(code_id: &str) {
    let counter = global::meter("fn0").u64_counter("create_instance").build();
    counter.add(1, &[KeyValue::new("code_id", code_id.to_string())]);
}

pub fn proxy_cache_error(code_id: &str, error: &str) {
    let counter = global::meter("fn0")
        .u64_counter("proxy_cache_error")
        .build();
    counter.add(
        1,
        &[
            KeyValue::new("code_id", code_id.to_string()),
            KeyValue::new("error", error.to_string()),
        ],
    );
}

pub fn code_id_parse_error() {
    let counter = global::meter("fn0")
        .u64_counter("code_id_parse_error")
        .build();
    counter.add(1, &[]);
}

pub fn function_invocation(code_id: &str) {
    let counter = global::meter("fn0")
        .u64_counter("function_invocation")
        .build();
    counter.add(1, &[KeyValue::new("code_id", code_id.to_string())]);
}

pub fn execution_time(code_id: &str, route: &str, duration: Duration) {
    let histogram = global::meter("fn0")
        .f64_histogram("execution_time_seconds")
        .build();
    histogram.record(
        duration.as_secs_f64(),
        &[
            KeyValue::new("code_id", code_id.to_string()),
            KeyValue::new("route", route.to_string()),
        ],
    );
}
