use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Meter, MeterProvider, ObservableCounter, ObservableGauge};
use opentelemetry_http::{HttpClient, HttpError};
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use tokio::runtime::Handle;

use dodb_server::{
    REQUEST_LATENCY_BUCKETS_SECONDS, REQUEST_OPERATIONS, ServerMetrics, ServerMetricsSnapshot,
    StorageMetricsSnapshot,
};

const PLATFORM_TENANT: &str = "fn0";
const METRIC_EXPORT_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
struct TokioHttpClient {
    client: reqwest::Client,
    handle: Handle,
}

#[async_trait::async_trait]
impl HttpClient for TokioHttpClient {
    async fn send_bytes(
        &self,
        request: http::Request<Bytes>,
    ) -> Result<http::Response<Bytes>, HttpError> {
        let client = self.client.clone();
        self.handle
            .spawn(async move {
                let (parts, body) = request.into_parts();
                let request = http::Request::from_parts(parts, body).try_into()?;
                let mut response = client.execute(request).await?.error_for_status()?;
                let headers = std::mem::take(response.headers_mut());
                let status = response.status();
                let body = response.bytes().await?;
                let mut http_response = http::Response::builder().status(status).body(body)?;
                *http_response.headers_mut() = headers;
                Ok::<_, HttpError>(http_response)
            })
            .await?
    }
}

pub struct DodbTelemetry {
    provider: SdkMeterProvider,
    _counters: Vec<ObservableCounter<u64>>,
    _gauges: Vec<ObservableGauge<u64>>,
}

impl DodbTelemetry {
    pub fn start(
        metrics: ServerMetrics,
        storage: Arc<Mutex<StorageMetricsSnapshot>>,
        instance_id: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let exporter = MetricExporter::builder()
            .with_http()
            .with_http_client(TokioHttpClient {
                client: reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()?,
                handle: Handle::current(),
            })
            .with_endpoint("http://127.0.0.1:4318/v1/metrics")
            .with_protocol(Protocol::HttpBinary)
            .build()?;
        let reader = PeriodicReader::builder(exporter)
            .with_interval(METRIC_EXPORT_INTERVAL)
            .build();
        let resource = Resource::builder()
            .with_service_name("dodb-server")
            .with_attribute(KeyValue::new("tenant.id", PLATFORM_TENANT))
            .with_attribute(KeyValue::new("service.instance.id", instance_id.clone()))
            .with_attribute(KeyValue::new("host.name", instance_id))
            .build();
        let provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(reader)
            .build();
        let meter = provider.meter("dodb-server");
        let counters = register_counters(&meter, metrics.clone());
        let gauges = register_gauges(&meter, metrics, storage);
        Ok(Self {
            provider,
            _counters: counters,
            _gauges: gauges,
        })
    }

    pub fn shutdown(&self) -> Result<(), opentelemetry_sdk::error::OTelSdkError> {
        self.provider.shutdown()
    }
}

fn register_counters(meter: &Meter, metrics: ServerMetrics) -> Vec<ObservableCounter<u64>> {
    let mut counters = Vec::new();
    let request_metrics = metrics.clone();
    counters.push(
        meter
            .u64_observable_counter("dodb.server.requests")
            .with_unit("{request}")
            .with_callback(move |observer| {
                let snapshot = request_metrics.snapshot();
                for (operation, operation_index) in REQUEST_OPERATIONS {
                    observer.observe(
                        snapshot.operations[operation_index],
                        &[KeyValue::new("operation", operation)],
                    );
                }
            })
            .build(),
    );
    counters.push(observable_counter(
        meter,
        "dodb.server.connections",
        metrics.clone(),
        |snapshot| snapshot.connections_total,
    ));
    counters.push(observable_counter(
        meter,
        "dodb.server.request.bytes",
        metrics.clone(),
        |snapshot| snapshot.request_bytes,
    ));
    counters.push(observable_counter(
        meter,
        "dodb.server.response.bytes",
        metrics.clone(),
        |snapshot| snapshot.response_bytes,
    ));
    counters.push(observable_counter(
        meter,
        "dodb.server.protocol.errors",
        metrics.clone(),
        |snapshot| snapshot.protocol_errors,
    ));
    counters.push(observable_counter(
        meter,
        "dodb.server.transport.errors",
        metrics.clone(),
        |snapshot| snapshot.transport_errors,
    ));
    counters.push(observable_counter(
        meter,
        "dodb.server.application.errors",
        metrics.clone(),
        |snapshot| snapshot.application_errors,
    ));
    counters.push(observable_counter(
        meter,
        "dodb.server.overloaded.responses",
        metrics.clone(),
        |snapshot| snapshot.overloaded_responses,
    ));
    let histogram_metrics = metrics;
    counters.push(
        meter
            .u64_observable_counter("dodb.server.request.duration_bucket")
            .with_unit("1")
            .with_callback(move |observer| {
                let snapshot = histogram_metrics.snapshot();
                for (operation, operation_index) in REQUEST_OPERATIONS {
                    for (bucket_index, upper_bound) in
                        REQUEST_LATENCY_BUCKETS_SECONDS.iter().enumerate()
                    {
                        observer.observe(
                            snapshot.request_latency_buckets[operation_index][bucket_index],
                            &[
                                KeyValue::new("operation", operation),
                                KeyValue::new("le", format!("{upper_bound:.6}")),
                            ],
                        );
                    }
                    observer.observe(
                        snapshot.request_latency_buckets[operation_index]
                            [REQUEST_LATENCY_BUCKETS_SECONDS.len()],
                        &[
                            KeyValue::new("operation", operation),
                            KeyValue::new("le", "+Inf"),
                        ],
                    );
                }
            })
            .build(),
    );
    counters
}

fn observable_counter(
    meter: &Meter,
    name: &'static str,
    metrics: ServerMetrics,
    read: impl Fn(&ServerMetricsSnapshot) -> u64 + Send + Sync + 'static,
) -> ObservableCounter<u64> {
    meter
        .u64_observable_counter(name)
        .with_callback(move |observer| observer.observe(read(&metrics.snapshot()), &[]))
        .build()
}

fn register_gauges(
    meter: &Meter,
    metrics: ServerMetrics,
    storage: Arc<Mutex<StorageMetricsSnapshot>>,
) -> Vec<ObservableGauge<u64>> {
    let mut gauges = Vec::new();
    gauges.push(observable_gauge(
        meter,
        "dodb.server.connections.active",
        metrics.clone(),
        |snapshot| snapshot.active_connections,
    ));
    gauges.push(observable_gauge(
        meter,
        "dodb.server.streams.active",
        metrics,
        |snapshot| snapshot.active_streams,
    ));
    for (name, read) in [
        (
            "dodb.storage.database.file.bytes",
            (|snapshot: &StorageMetricsSnapshot| snapshot.database_file_bytes)
                as fn(&StorageMetricsSnapshot) -> u64,
        ),
        (
            "dodb.storage.wal.file.bytes",
            |snapshot: &StorageMetricsSnapshot| snapshot.wal_file_bytes,
        ),
        (
            "dodb.storage.shards.persisted",
            |snapshot: &StorageMetricsSnapshot| snapshot.persisted_shards,
        ),
        (
            "dodb.storage.shards.open",
            |snapshot: &StorageMetricsSnapshot| snapshot.open_shards,
        ),
    ] {
        let storage = Arc::clone(&storage);
        gauges.push(
            meter
                .u64_observable_gauge(name)
                .with_callback(move |observer| {
                    if let Ok(snapshot) = storage.lock() {
                        observer.observe(read(&snapshot), &[]);
                    }
                })
                .build(),
        );
    }
    gauges
}

fn observable_gauge(
    meter: &Meter,
    name: &'static str,
    metrics: ServerMetrics,
    read: impl Fn(&ServerMetricsSnapshot) -> u64 + Send + Sync + 'static,
) -> ObservableGauge<u64> {
    meter
        .u64_observable_gauge(name)
        .with_callback(move |observer| observer.observe(read(&metrics.snapshot()), &[]))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::metrics::MeterProvider;
    use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData, ResourceMetrics};
    use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider, Temporality};
    use std::collections::BTreeMap;
    use std::time::Duration;

    #[derive(Clone, Default)]
    struct SchemaExporter {
        points: Arc<Mutex<BTreeMap<String, Vec<Vec<(String, String)>>>>>,
    }

    impl opentelemetry_sdk::metrics::exporter::PushMetricExporter for SchemaExporter {
        async fn export(
            &self,
            metrics: &ResourceMetrics,
        ) -> opentelemetry_sdk::error::OTelSdkResult {
            let mut points = BTreeMap::new();
            for metric in metrics.scope_metrics().flat_map(|scope| scope.metrics()) {
                let mut metric_points = Vec::new();
                match metric.data() {
                    AggregatedMetrics::U64(MetricData::Gauge(gauge)) => {
                        metric_points.extend(gauge.data_points().map(|point| {
                            point
                                .attributes()
                                .map(|attribute| {
                                    (
                                        attribute.key.as_str().to_owned(),
                                        attribute.value.as_str().into_owned(),
                                    )
                                })
                                .collect::<Vec<_>>()
                        }));
                    }
                    AggregatedMetrics::U64(MetricData::Sum(sum)) => {
                        metric_points.extend(sum.data_points().map(|point| {
                            point
                                .attributes()
                                .map(|attribute| {
                                    (
                                        attribute.key.as_str().to_owned(),
                                        attribute.value.as_str().into_owned(),
                                    )
                                })
                                .collect::<Vec<_>>()
                        }));
                    }
                    other => panic!("unexpected metric aggregation: {other:?}"),
                }
                for attributes in &mut metric_points {
                    attributes.sort();
                }
                points.insert(metric.name().to_owned(), metric_points);
            }
            *self.points.lock().expect("schema exporter mutex") = points;
            Ok(())
        }

        fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
            Ok(())
        }

        fn shutdown_with_timeout(
            &self,
            _timeout: Duration,
        ) -> opentelemetry_sdk::error::OTelSdkResult {
            Ok(())
        }

        fn temporality(&self) -> Temporality {
            Temporality::Cumulative
        }
    }

    #[test]
    fn metric_schema_uses_bounded_operations_and_cumulative_latency_buckets() {
        let exporter = SchemaExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(exporter.clone()).build())
            .build();
        let meter = provider.meter("dodb-server-test");
        let metrics = ServerMetrics::default();
        let _counters = register_counters(&meter, metrics.clone());
        let _gauges = register_gauges(
            &meter,
            metrics,
            Arc::new(Mutex::new(StorageMetricsSnapshot::default())),
        );

        provider.force_flush().expect("flush metrics");
        let points = exporter.points.lock().expect("exported metric schema");

        let operations = points.get("dodb.server.requests").expect("request counter");
        assert_eq!(operations.len(), REQUEST_OPERATIONS.len());
        assert!(operations.iter().all(|attributes| {
            attributes.len() == 1
                && attributes[0].0 == "operation"
                && REQUEST_OPERATIONS
                    .iter()
                    .any(|(operation, _)| operation == &attributes[0].1)
        }));

        let buckets = points
            .get("dodb.server.request.duration_bucket")
            .expect("latency buckets");
        assert_eq!(
            buckets.len(),
            REQUEST_OPERATIONS.len() * (REQUEST_LATENCY_BUCKETS_SECONDS.len() + 1)
        );
        assert!(buckets.iter().all(|attributes| {
            attributes.len() == 2
                && attributes.iter().any(|(key, _)| key == "operation")
                && attributes.iter().any(|(key, _)| key == "le")
        }));
        assert!(points.contains_key("dodb.storage.shards.persisted"));
        assert!(points.contains_key("dodb.storage.shards.open"));
    }
}
