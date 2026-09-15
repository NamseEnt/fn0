use opentelemetry::{KeyValue, global};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

pub const REQUEST_SPAN_NAME: &str = "fn0.request";
/// Marks a span or data point as the named project's own telemetry. The
/// worker's exporter moves everything carrying it into that project's tenant
/// and removes the attribute; everything without it stays in the platform
/// tenant. The request span sets it under this literal name, so the two must
/// not drift apart.
pub const PROJECT_TENANT_ATTRIBUTE: &str = "fn0.project_tenant";
pub const SLOW_REQUEST: Duration = Duration::from_secs(1);
pub const REQUEST_DURATION_METRIC: &str = "fn0.request.duration";
pub const CPU_TIME_METRIC: &str = "fn0.cpu_time";
pub const MAX_ROUTES_PER_PROJECT: usize = 40;

const SECONDS_BUCKETS: [f64; 7] = [0.005, 0.025, 0.1, 0.5, 1.0, 5.0, 30.0];
const UNKNOWN_ROUTE: &str = "unknown";

static PROJECT_ROUTES: OnceLock<Mutex<HashMap<String, HashSet<String>>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureComponent {
    Instantiate,
    RunConcurrent,
    ResponseIntoHttp,
    GuestHandler,
    Executor,
    ResponseChannel,
}

impl FailureComponent {
    fn as_str(self) -> &'static str {
        match self {
            Self::Instantiate => "instantiate",
            Self::RunConcurrent => "run_concurrent",
            Self::ResponseIntoHttp => "response_into_http",
            Self::GuestHandler => "guest_handler",
            Self::Executor => "executor",
            Self::ResponseChannel => "response_channel",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestOutcome {
    Ok,
    ClientError,
    ServerError,
    Failed,
}

impl RequestOutcome {
    pub fn from_status(status_code: u16) -> Self {
        match status_code {
            400..=499 => Self::ClientError,
            500..=599 => Self::ServerError,
            _ => Self::Ok,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::ClientError => "client_error",
            Self::ServerError => "server_error",
            Self::Failed => "failed",
        }
    }

    pub fn is_error(self) -> bool {
        matches!(self, Self::ServerError | Self::Failed)
    }
}

/// `error_type` is `&'static str` so that no caller can put a formatted error
/// message into a metric attribute; the message belongs in a log.
pub fn failure(component: FailureComponent, error_type: &'static str) {
    global::meter("fn0")
        .u64_counter("fn0.failures")
        .build()
        .add(
            1,
            &[
                KeyValue::new("component", component.as_str()),
                KeyValue::new("error.type", error_type),
            ],
        );
}

pub fn trap_error_type(trap: &wasmtime::Trap) -> &'static str {
    match trap {
        wasmtime::Trap::StackOverflow => "stack_overflow",
        wasmtime::Trap::MemoryOutOfBounds => "memory_out_of_bounds",
        wasmtime::Trap::TableOutOfBounds => "table_out_of_bounds",
        wasmtime::Trap::IndirectCallToNull => "indirect_call_to_null",
        wasmtime::Trap::BadSignature => "bad_signature",
        wasmtime::Trap::IntegerOverflow => "integer_overflow",
        wasmtime::Trap::IntegerDivisionByZero => "integer_division_by_zero",
        wasmtime::Trap::BadConversionToInteger => "bad_conversion_to_integer",
        wasmtime::Trap::UnreachableCodeReached => "unreachable_code_reached",
        wasmtime::Trap::Interrupt => "interrupt",
        wasmtime::Trap::OutOfFuel => "out_of_fuel",
        wasmtime::Trap::AllocationTooLarge => "allocation_too_large",
        _ => "other_trap",
    }
}

pub fn error_code_error_type(error_code: &ErrorCode) -> &'static str {
    match error_code {
        ErrorCode::DnsTimeout => "dns_timeout",
        ErrorCode::DnsError(_) => "dns_error",
        ErrorCode::DestinationNotFound => "destination_not_found",
        ErrorCode::DestinationUnavailable => "destination_unavailable",
        ErrorCode::DestinationIpProhibited => "destination_ip_prohibited",
        ErrorCode::DestinationIpUnroutable => "destination_ip_unroutable",
        ErrorCode::ConnectionRefused => "connection_refused",
        ErrorCode::ConnectionTerminated => "connection_terminated",
        ErrorCode::ConnectionTimeout => "connection_timeout",
        ErrorCode::ConnectionReadTimeout => "connection_read_timeout",
        ErrorCode::ConnectionWriteTimeout => "connection_write_timeout",
        ErrorCode::ConnectionLimitReached => "connection_limit_reached",
        ErrorCode::TlsProtocolError => "tls_protocol_error",
        ErrorCode::TlsCertificateError => "tls_certificate_error",
        ErrorCode::TlsAlertReceived(_) => "tls_alert_received",
        ErrorCode::HttpRequestDenied => "http_request_denied",
        ErrorCode::HttpRequestLengthRequired => "http_request_length_required",
        ErrorCode::HttpRequestBodySize(_) => "http_request_body_size",
        ErrorCode::HttpRequestMethodInvalid => "http_request_method_invalid",
        ErrorCode::HttpRequestUriInvalid => "http_request_uri_invalid",
        ErrorCode::HttpRequestUriTooLong => "http_request_uri_too_long",
        ErrorCode::HttpRequestHeaderSectionSize(_) => "http_request_header_section_size",
        ErrorCode::HttpRequestHeaderSize(_) => "http_request_header_size",
        ErrorCode::HttpRequestTrailerSectionSize(_) => "http_request_trailer_section_size",
        ErrorCode::HttpRequestTrailerSize(_) => "http_request_trailer_size",
        ErrorCode::HttpResponseIncomplete => "http_response_incomplete",
        ErrorCode::HttpResponseHeaderSectionSize(_) => "http_response_header_section_size",
        ErrorCode::HttpResponseHeaderSize(_) => "http_response_header_size",
        ErrorCode::HttpResponseBodySize(_) => "http_response_body_size",
        ErrorCode::HttpResponseTrailerSectionSize(_) => "http_response_trailer_section_size",
        ErrorCode::HttpResponseTrailerSize(_) => "http_response_trailer_size",
        ErrorCode::HttpResponseTransferCoding(_) => "http_response_transfer_coding",
        ErrorCode::HttpResponseContentCoding(_) => "http_response_content_coding",
        ErrorCode::HttpResponseTimeout => "http_response_timeout",
        ErrorCode::HttpUpgradeFailed => "http_upgrade_failed",
        ErrorCode::HttpProtocolError => "http_protocol_error",
        ErrorCode::LoopDetected => "loop_detected",
        ErrorCode::ConfigurationError => "configuration_error",
        ErrorCode::InternalError(_) => "internal_error",
    }
}

pub fn cpu_time(project_id: &str, cpu_time: Duration) {
    global::meter("fn0")
        .f64_histogram(CPU_TIME_METRIC)
        .with_unit("s")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build()
        .record(
            cpu_time.as_secs_f64(),
            &[KeyValue::new(
                PROJECT_TENANT_ATTRIBUTE,
                project_id.to_string(),
            )],
        );
}

pub fn cpu_timeout(project_id: &str) {
    global::meter("fn0")
        .u64_counter("fn0.cpu_timeouts")
        .build()
        .add(
            1,
            &[KeyValue::new(
                PROJECT_TENANT_ATTRIBUTE,
                project_id.to_string(),
            )],
        );
}

pub fn create_instance() {
    global::meter("fn0")
        .u64_counter("fn0.instances_created")
        .build()
        .add(1, &[]);
}

pub fn request_duration(
    project_id: &str,
    route: &str,
    outcome: RequestOutcome,
    duration: Duration,
) {
    global::meter("fn0")
        .f64_histogram(REQUEST_DURATION_METRIC)
        .with_unit("s")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build()
        .record(
            duration.as_secs_f64(),
            &[
                KeyValue::new(PROJECT_TENANT_ATTRIBUTE, project_id.to_string()),
                KeyValue::new("route", bounded_route(project_id, route)),
                KeyValue::new("outcome", outcome.as_str()),
            ],
        );
}

pub fn stage_duration(stage: &'static str, duration: Duration) {
    global::meter("fn0")
        .f64_histogram("fn0.stage.duration")
        .with_unit("s")
        .with_boundaries(SECONDS_BUCKETS.to_vec())
        .build()
        .record(duration.as_secs_f64(), &[KeyValue::new("stage", stage)]);
}

pub fn bounded_route(project_id: &str, route: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::{MAX_ROUTES_PER_PROJECT, RequestOutcome, bounded_route};

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
    fn classifies_request_outcomes_by_status() {
        assert_eq!(RequestOutcome::from_status(200), RequestOutcome::Ok);
        assert_eq!(RequestOutcome::from_status(399), RequestOutcome::Ok);
        assert_eq!(
            RequestOutcome::from_status(400),
            RequestOutcome::ClientError
        );
        assert_eq!(
            RequestOutcome::from_status(499),
            RequestOutcome::ClientError
        );
        assert_eq!(
            RequestOutcome::from_status(500),
            RequestOutcome::ServerError
        );
        assert_eq!(
            RequestOutcome::from_status(599),
            RequestOutcome::ServerError
        );
        assert!(!RequestOutcome::ClientError.is_error());
        assert!(RequestOutcome::ServerError.is_error());
        assert!(RequestOutcome::Failed.is_error());
    }
}
