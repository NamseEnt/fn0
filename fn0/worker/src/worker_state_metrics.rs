//! The worker's own state, exported to the platform tenant so the operations
//! console can read it without reaching the loopback-only ops server.
//!
//! Every data point carries [`fn0::telemetry::SERVICE_INSTANCE_ID_ATTRIBUTE`]
//! and nothing that names a project, route or host.

use crate::worker_pool::DispatchError;
use fn0::telemetry::SERVICE_INSTANCE_ID_ATTRIBUTE;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Meter, ObservableGauge};
use opentelemetry_sdk::error::OTelSdkError;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

pub const MANIFEST_LOADED_METRIC: &str = "fn0.worker.manifest_loaded";
pub const DRAINING_METRIC: &str = "fn0.worker.draining";
pub const IN_FLIGHT_REQUESTS_METRIC: &str = "fn0.worker.requests.in_flight";
pub const WEBSOCKET_CONNECTIONS_METRIC: &str = "fn0.worker.websocket.connections";
pub const DISPATCH_REJECTIONS_METRIC: &str = "fn0.worker.dispatch.rejections";
const DISPATCH_REJECTION_REASONS: [DispatchError; 3] = [
    DispatchError::QueueFull,
    DispatchError::ProjectAdmissionFull,
    DispatchError::Closed,
];

pub fn generate_service_instance_id() -> String {
    format!("{:016x}", rand::random::<u64>())
}

pub struct WorkerStateSources {
    pub manifest_loaded: Arc<AtomicBool>,
    pub draining: Arc<AtomicBool>,
    pub in_flight_requests: Arc<AtomicU64>,
    pub websocket_connections: Arc<dyn Fn() -> u64 + Send + Sync>,
}

/// The gauges stop reporting when this is dropped.
pub struct WorkerStateGauges {
    _gauges: Vec<ObservableGauge<u64>>,
}

pub fn register(
    meter: &Meter,
    service_instance_id: &str,
    sources: WorkerStateSources,
) -> WorkerStateGauges {
    let WorkerStateSources {
        manifest_loaded,
        draining,
        in_flight_requests,
        websocket_connections,
    } = sources;
    let gauge =
        |name: &'static str, unit: &'static str, read: Box<dyn Fn() -> u64 + Send + Sync>| {
            let attributes = [service_instance_attribute(service_instance_id)];
            meter
                .u64_observable_gauge(name)
                .with_unit(unit)
                .with_callback(move |observer| observer.observe(read(), &attributes))
                .build()
        };
    WorkerStateGauges {
        _gauges: vec![
            gauge(
                MANIFEST_LOADED_METRIC,
                "1",
                Box::new(move || u64::from(manifest_loaded.load(Ordering::Acquire))),
            ),
            gauge(
                DRAINING_METRIC,
                "1",
                Box::new(move || u64::from(draining.load(Ordering::Relaxed))),
            ),
            gauge(
                IN_FLIGHT_REQUESTS_METRIC,
                "{request}",
                Box::new(move || in_flight_requests.load(Ordering::Relaxed)),
            ),
            gauge(
                WEBSOCKET_CONNECTIONS_METRIC,
                "{connection}",
                Box::new(move || websocket_connections()),
            ),
        ],
    }
}

/// Counts a user request the worker answered 503 or 500 without running it.
pub fn dispatch_rejection(error: &DispatchError) {
    if let Some(service_instance_id) = fn0::telemetry::service_instance_id() {
        record_dispatch_rejection(
            &opentelemetry::global::meter("fn0-worker"),
            service_instance_id,
            error,
        );
    }
}

fn record_dispatch_rejection(meter: &Meter, service_instance_id: &str, error: &DispatchError) {
    meter.u64_counter(DISPATCH_REJECTIONS_METRIC).build().add(
        1,
        &[
            KeyValue::new("reason", error.as_str()),
            service_instance_attribute(service_instance_id),
        ],
    );
}

#[derive(Debug)]
pub enum BaselineExportError {
    Export(OTelSdkError),
    DeadlinePassed,
    FlushThreadStopped,
}

impl std::fmt::Display for BaselineExportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Export(error) => write!(formatter, "export failed: {error}"),
            Self::DeadlinePassed => write!(formatter, "export did not finish before its deadline"),
            Self::FlushThreadStopped => {
                write!(formatter, "the flush thread stopped without a result")
            }
        }
    }
}

/// Exports every dispatch rejection reason at zero, and waits for the export.
///
/// Signy's `increase` counts a counter's growth from its previous sample, so
/// the first sample of a series is never counted. Without a zero exported
/// first, the rejections a worker counts before its first periodic export
/// would vanish from every window. Call this before the worker accepts
/// traffic.
///
/// The SDK's flush has no deadline of its own and neither does the exporter's
/// HTTP client, so it runs on its own thread and is abandoned after
/// `deadline`: telemetry must never keep the worker from serving.
pub fn export_dispatch_rejection_baseline(
    meter_provider: &SdkMeterProvider,
    meter: &Meter,
    service_instance_id: &str,
    deadline: Duration,
) -> Result<(), BaselineExportError> {
    let counter = meter.u64_counter(DISPATCH_REJECTIONS_METRIC).build();
    for reason in &DISPATCH_REJECTION_REASONS {
        counter.add(
            0,
            &[
                KeyValue::new("reason", reason.as_str()),
                service_instance_attribute(service_instance_id),
            ],
        );
    }
    let (result_sender, result_receiver) = std::sync::mpsc::channel();
    let flushing_provider = meter_provider.clone();
    std::thread::spawn(move || {
        let _ = result_sender.send(flushing_provider.force_flush());
    });
    match result_receiver.recv_timeout(deadline) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(BaselineExportError::Export(error)),
        Err(RecvTimeoutError::Timeout) => Err(BaselineExportError::DeadlinePassed),
        Err(RecvTimeoutError::Disconnected) => Err(BaselineExportError::FlushThreadStopped),
    }
}

fn service_instance_attribute(service_instance_id: &str) -> KeyValue {
    KeyValue::new(
        SERVICE_INSTANCE_ID_ATTRIBUTE,
        service_instance_id.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::metrics::MeterProvider;
    use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData, ResourceMetrics};
    use opentelemetry_sdk::metrics::{InMemoryMetricExporter, SdkMeterProvider};
    use std::collections::BTreeMap;

    type PointsByMetric = BTreeMap<String, Vec<(Vec<(String, String)>, u64)>>;

    fn attributes<'a>(attributes: impl Iterator<Item = &'a KeyValue>) -> Vec<(String, String)> {
        let mut attributes: Vec<(String, String)> = attributes
            .map(|attribute| {
                (
                    attribute.key.as_str().to_string(),
                    attribute.value.as_str().into_owned(),
                )
            })
            .collect();
        attributes.sort();
        attributes
    }

    fn points_by_metric(resource_metrics: &[ResourceMetrics]) -> PointsByMetric {
        let mut points = PointsByMetric::new();
        for metric in resource_metrics
            .iter()
            .flat_map(|resource| resource.scope_metrics())
            .flat_map(|scope| scope.metrics())
        {
            let metric_points: Vec<(Vec<(String, String)>, u64)> = match metric.data() {
                AggregatedMetrics::U64(MetricData::Gauge(gauge)) => gauge
                    .data_points()
                    .map(|point| (attributes(point.attributes()), point.value()))
                    .collect(),
                AggregatedMetrics::U64(MetricData::Sum(sum)) => sum
                    .data_points()
                    .map(|point| (attributes(point.attributes()), point.value()))
                    .collect(),
                other => panic!("unexpected aggregation for {}: {other:?}", metric.name()),
            };
            points
                .entry(metric.name().to_string())
                .or_default()
                .extend(metric_points);
        }
        for metric_points in points.values_mut() {
            metric_points.sort();
        }
        points
    }

    struct NeverFinishingExporter;

    impl opentelemetry_sdk::metrics::exporter::PushMetricExporter for NeverFinishingExporter {
        async fn export(
            &self,
            _metrics: &ResourceMetrics,
        ) -> opentelemetry_sdk::error::OTelSdkResult {
            std::future::pending().await
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

        fn temporality(&self) -> opentelemetry_sdk::metrics::Temporality {
            opentelemetry_sdk::metrics::Temporality::Cumulative
        }
    }

    #[test]
    fn the_baseline_exports_every_rejection_reason_at_zero() {
        let exporter = InMemoryMetricExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();
        let meter = provider.meter("fn0-worker");

        export_dispatch_rejection_baseline(&provider, &meter, "instance-a", Duration::from_secs(5))
            .expect("baseline export");

        let points = points_by_metric(&exporter.get_finished_metrics().expect("metrics"));
        let with_reason = |reason: &str| {
            let mut attributes = instance_only();
            attributes.push(("reason".to_string(), reason.to_string()));
            attributes.sort();
            attributes
        };
        assert_eq!(
            points[DISPATCH_REJECTIONS_METRIC],
            vec![
                (with_reason("closed"), 0),
                (with_reason("project_admission_full"), 0),
                (with_reason("queue_full"), 0),
            ]
        );
    }

    #[test]
    fn a_hanging_export_gives_up_at_the_deadline_instead_of_blocking_startup() {
        let provider = SdkMeterProvider::builder()
            .with_periodic_exporter(NeverFinishingExporter)
            .build();
        let meter = provider.meter("fn0-worker");
        let started = std::time::Instant::now();

        let result = export_dispatch_rejection_baseline(
            &provider,
            &meter,
            "instance-a",
            Duration::from_millis(200),
        );

        assert!(matches!(result, Err(BaselineExportError::DeadlinePassed)));
        assert!(started.elapsed() < Duration::from_secs(2));
        // Dropping the provider would wait on the reader thread the hanging
        // export holds.
        std::mem::forget(provider);
    }

    fn instance_only() -> Vec<(String, String)> {
        vec![(
            SERVICE_INSTANCE_ID_ATTRIBUTE.to_string(),
            "instance-a".to_string(),
        )]
    }

    #[test]
    fn gauges_report_the_live_worker_state_under_the_instance_alone() {
        let exporter = InMemoryMetricExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();
        let meter = provider.meter("fn0-worker");
        let manifest_loaded = Arc::new(AtomicBool::new(false));
        let in_flight_requests = Arc::new(AtomicU64::new(0));
        let _gauges = register(
            &meter,
            "instance-a",
            WorkerStateSources {
                manifest_loaded: manifest_loaded.clone(),
                draining: Arc::new(AtomicBool::new(true)),
                in_flight_requests: in_flight_requests.clone(),
                websocket_connections: Arc::new(|| 7),
            },
        );
        manifest_loaded.store(true, Ordering::Release);
        in_flight_requests.store(3, Ordering::Relaxed);

        provider.force_flush().expect("flush");
        let points = points_by_metric(&exporter.get_finished_metrics().expect("metrics"));

        assert_eq!(points[MANIFEST_LOADED_METRIC], vec![(instance_only(), 1)]);
        assert_eq!(points[DRAINING_METRIC], vec![(instance_only(), 1)]);
        assert_eq!(
            points[IN_FLIGHT_REQUESTS_METRIC],
            vec![(instance_only(), 3)]
        );
        assert_eq!(
            points[WEBSOCKET_CONNECTIONS_METRIC],
            vec![(instance_only(), 7)]
        );
    }

    #[test]
    fn dispatch_rejections_keep_capacity_apart_from_project_admission() {
        let exporter = InMemoryMetricExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();
        let meter = provider.meter("fn0-worker");
        for error in [
            DispatchError::QueueFull,
            DispatchError::ProjectAdmissionFull,
            DispatchError::ProjectAdmissionFull,
            DispatchError::Closed,
        ] {
            record_dispatch_rejection(&meter, "instance-a", &error);
        }

        provider.force_flush().expect("flush");
        let points = points_by_metric(&exporter.get_finished_metrics().expect("metrics"));

        let with_reason = |reason: &str| {
            let mut attributes = instance_only();
            attributes.push(("reason".to_string(), reason.to_string()));
            attributes.sort();
            attributes
        };
        assert_eq!(
            points[DISPATCH_REJECTIONS_METRIC],
            vec![
                (with_reason("closed"), 1),
                (with_reason("project_admission_full"), 2),
                (with_reason("queue_full"), 1),
            ]
        );
    }
}
