use opentelemetry::metrics::Meter;
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
/// Both histograms carry [`SECONDS_BUCKETS`], and both are named apart from
/// the `fn0.request.duration` and `fn0.cpu_time` they replace. Those two were
/// exported as exponential histograms, whose bucket boundaries move with the
/// data: the store expands them into `le` series, so one series' boundaries
/// changed under it — 33 distinct `le` values for five request series in two
/// hours of production — and a quantile over a window holding both shapes
/// interpolates across brackets that never existed. New names keep the two
/// shapes in separate families; the old ones stop being written and leave with
/// the tenant's metric retention.
pub const REQUEST_DURATION_METRIC: &str = "fn0.http.server.request.duration";
pub const CPU_TIME_METRIC: &str = "fn0.guest.cpu.duration";
pub const MAX_ROUTES_PER_PROJECT: usize = 40;
/// Platform aggregates: every request and guest CPU sample on the worker,
/// with no project, route or hostname, so the operations console can read the
/// whole platform from the platform tenant alone. They carry only bounded
/// labels — `outcome` and [`SERVICE_INSTANCE_ID_ATTRIBUTE`].
pub const PLATFORM_REQUEST_DURATION_METRIC: &str = "fn0.platform.request.duration";
pub const PLATFORM_CPU_TIME_METRIC: &str = "fn0.platform.guest.cpu.duration";
pub const PLATFORM_CPU_TIMEOUTS_METRIC: &str = "fn0.platform.cpu_timeouts";
/// Set on platform metric data points, never on the resource: the resource
/// is copied into every project tenant, and a per-process label there would
/// split each project's series on every deploy. Without it, two worker
/// processes exporting the same cumulative series — a blue-green overlap, or
/// two hosts — would interleave into one series.
pub const SERVICE_INSTANCE_ID_ATTRIBUTE: &str = "service.instance.id";

const SECONDS_BUCKETS: [f64; 7] = [0.005, 0.025, 0.1, 0.5, 1.0, 5.0, 30.0];
const PLATFORM_REQUEST_SECONDS_BUCKETS: [f64; 13] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0,
];
const PLATFORM_CPU_SECONDS_BUCKETS: [f64; 11] = [
    0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5,
];
const UNKNOWN_ROUTE: &str = "unknown";

static PROJECT_ROUTES: OnceLock<Mutex<HashMap<String, HashSet<String>>>> = OnceLock::new();
static SERVICE_INSTANCE_ID: OnceLock<String> = OnceLock::new();

/// Turns on the platform metrics for this process. Until it is called — as
/// in `fn0 local` — only project metrics are recorded. The first id wins.
pub fn install_service_instance_id(service_instance_id: String) {
    let _ = SERVICE_INSTANCE_ID.set(service_instance_id);
}

pub fn service_instance_id() -> Option<&'static str> {
    SERVICE_INSTANCE_ID.get().map(String::as_str)
}

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
    record_cpu_time(
        &global::meter("fn0"),
        service_instance_id(),
        project_id,
        cpu_time,
    );
}

fn record_cpu_time(
    meter: &Meter,
    service_instance_id: Option<&str>,
    project_id: &str,
    cpu_time: Duration,
) {
    meter
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
    if let Some(service_instance_id) = service_instance_id {
        meter
            .f64_histogram(PLATFORM_CPU_TIME_METRIC)
            .with_unit("s")
            .with_boundaries(PLATFORM_CPU_SECONDS_BUCKETS.to_vec())
            .build()
            .record(
                cpu_time.as_secs_f64(),
                &[service_instance_attribute(service_instance_id)],
            );
    }
}

pub fn cpu_timeout(project_id: &str) {
    record_cpu_timeout(&global::meter("fn0"), service_instance_id(), project_id);
}

fn record_cpu_timeout(meter: &Meter, service_instance_id: Option<&str>, project_id: &str) {
    meter.u64_counter("fn0.cpu_timeouts").build().add(
        1,
        &[KeyValue::new(
            PROJECT_TENANT_ATTRIBUTE,
            project_id.to_string(),
        )],
    );
    if let Some(service_instance_id) = service_instance_id {
        meter
            .u64_counter(PLATFORM_CPU_TIMEOUTS_METRIC)
            .build()
            .add(1, &[service_instance_attribute(service_instance_id)]);
    }
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
    record_request_duration(
        &global::meter("fn0"),
        service_instance_id(),
        project_id,
        route,
        outcome,
        duration,
    );
}

fn record_request_duration(
    meter: &Meter,
    service_instance_id: Option<&str>,
    project_id: &str,
    route: &str,
    outcome: RequestOutcome,
    duration: Duration,
) {
    meter
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
    if let Some(service_instance_id) = service_instance_id {
        meter
            .f64_histogram(PLATFORM_REQUEST_DURATION_METRIC)
            .with_unit("s")
            .with_boundaries(PLATFORM_REQUEST_SECONDS_BUCKETS.to_vec())
            .build()
            .record(
                duration.as_secs_f64(),
                &[
                    KeyValue::new("outcome", outcome.as_str()),
                    service_instance_attribute(service_instance_id),
                ],
            );
    }
}

fn service_instance_attribute(service_instance_id: &str) -> KeyValue {
    KeyValue::new(
        SERVICE_INSTANCE_ID_ATTRIBUTE,
        service_instance_id.to_string(),
    )
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
    use super::{
        CPU_TIME_METRIC, MAX_ROUTES_PER_PROJECT, PLATFORM_CPU_TIME_METRIC,
        PLATFORM_CPU_TIMEOUTS_METRIC, PLATFORM_REQUEST_DURATION_METRIC, PROJECT_TENANT_ATTRIBUTE,
        REQUEST_DURATION_METRIC, RequestOutcome, SERVICE_INSTANCE_ID_ATTRIBUTE, bounded_route,
        record_cpu_time, record_cpu_timeout, record_request_duration,
    };
    use opentelemetry::KeyValue;
    use opentelemetry::metrics::MeterProvider;
    use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData, ResourceMetrics};
    use opentelemetry_sdk::metrics::{InMemoryMetricExporter, SdkMeterProvider};
    use std::collections::{BTreeMap, BTreeSet};
    use std::time::Duration;

    type AttributeKeysByMetric = BTreeMap<String, Vec<BTreeSet<String>>>;

    fn attribute_keys<'a>(attributes: impl Iterator<Item = &'a KeyValue>) -> BTreeSet<String> {
        attributes
            .map(|attribute| attribute.key.as_str().to_string())
            .collect()
    }

    fn attribute_keys_by_metric(resource_metrics: &[ResourceMetrics]) -> AttributeKeysByMetric {
        let mut keys_by_metric = AttributeKeysByMetric::new();
        for metric in resource_metrics
            .iter()
            .flat_map(|resource| resource.scope_metrics())
            .flat_map(|scope| scope.metrics())
        {
            let point_keys: Vec<BTreeSet<String>> = match metric.data() {
                AggregatedMetrics::F64(MetricData::Histogram(histogram)) => histogram
                    .data_points()
                    .map(|point| attribute_keys(point.attributes()))
                    .collect(),
                AggregatedMetrics::U64(MetricData::Sum(sum)) => sum
                    .data_points()
                    .map(|point| attribute_keys(point.attributes()))
                    .collect(),
                other => panic!("unexpected aggregation for {}: {other:?}", metric.name()),
            };
            keys_by_metric
                .entry(metric.name().to_string())
                .or_default()
                .extend(point_keys);
        }
        keys_by_metric
    }

    fn record_one_of_each(service_instance_id: Option<&str>) -> AttributeKeysByMetric {
        let exporter = InMemoryMetricExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();
        let meter = provider.meter("fn0");
        let project_id = "telemetry-platform-metric-test";
        record_request_duration(
            &meter,
            service_instance_id,
            project_id,
            "/items/[id]",
            RequestOutcome::ServerError,
            Duration::from_millis(40),
        );
        record_cpu_time(
            &meter,
            service_instance_id,
            project_id,
            Duration::from_millis(3),
        );
        record_cpu_timeout(&meter, service_instance_id, project_id);
        provider.force_flush().expect("flush");
        attribute_keys_by_metric(&exporter.get_finished_metrics().expect("metrics"))
    }

    fn keys(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn platform_metrics_carry_only_outcome_and_instance() {
        let keys_by_metric = record_one_of_each(Some("instance-a"));

        assert_eq!(
            keys_by_metric[PLATFORM_REQUEST_DURATION_METRIC],
            vec![keys(&["outcome", SERVICE_INSTANCE_ID_ATTRIBUTE])]
        );
        assert_eq!(
            keys_by_metric[PLATFORM_CPU_TIME_METRIC],
            vec![keys(&[SERVICE_INSTANCE_ID_ATTRIBUTE])]
        );
        assert_eq!(
            keys_by_metric[PLATFORM_CPU_TIMEOUTS_METRIC],
            vec![keys(&[SERVICE_INSTANCE_ID_ATTRIBUTE])]
        );
    }

    #[test]
    fn project_metrics_keep_their_labels_and_gain_no_instance() {
        let keys_by_metric = record_one_of_each(Some("instance-a"));

        assert_eq!(
            keys_by_metric[REQUEST_DURATION_METRIC],
            vec![keys(&[PROJECT_TENANT_ATTRIBUTE, "route", "outcome"])]
        );
        assert_eq!(
            keys_by_metric[CPU_TIME_METRIC],
            vec![keys(&[PROJECT_TENANT_ATTRIBUTE])]
        );
        assert_eq!(
            keys_by_metric["fn0.cpu_timeouts"],
            vec![keys(&[PROJECT_TENANT_ATTRIBUTE])]
        );
    }

    #[test]
    fn platform_metrics_are_off_without_an_instance_id() {
        let keys_by_metric = record_one_of_each(None);

        assert_eq!(
            keys_by_metric
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["fn0.cpu_timeouts", CPU_TIME_METRIC, REQUEST_DURATION_METRIC]
        );
    }

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
