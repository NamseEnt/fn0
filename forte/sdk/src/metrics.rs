//! OTLP metrics for forte components.
//!
//! A forte component only has CPU while serving a request, so there is no
//! background exporter. Metrics are aggregated with delta temporality (a
//! per-request component instance has no cross-request state to accumulate)
//! and flushed at request end through the host-provided HTTP client, mirroring
//! the trace path in [`crate::otel`]. `serve` calls [`flush`]; component code
//! records measurements through instruments created from [`meter`].

use crate::http::{Body, Client, Method, Request};
use opentelemetry::metrics::{Meter, MeterProvider};
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::reader::MetricReader;
use opentelemetry_sdk::metrics::{
    InstrumentKind, ManualReader, Pipeline, SdkMeterProvider, Temporality,
};
use prost::Message;
use std::sync::{Arc, OnceLock, Weak};
use std::time::Duration;

pub use opentelemetry::KeyValue;
pub use opentelemetry::metrics::{Counter, Gauge, Histogram, UpDownCounter};

const ENDPOINT: &str = "http://fn0-otel.fn0.dev/v1/metrics";

// `SdkMeterProvider` consumes the reader it is given, but `flush` has to call
// `collect` on that same reader after the provider owns it. Sharing one
// `ManualReader` behind an `Arc` keeps a flush handle: `register_pipeline` runs
// on the provider's clone and the shared inner reader sees the pipeline.
#[derive(Debug)]
struct ForteMetricReader(Arc<ManualReader>);

impl MetricReader for ForteMetricReader {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.0.register_pipeline(pipeline);
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult {
        self.0.collect(rm)
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.0.force_flush()
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.0.shutdown_with_timeout(timeout)
    }

    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.0.temporality(kind)
    }
}

struct MetricsState {
    _provider: SdkMeterProvider,
    reader: Arc<ManualReader>,
    meter: Meter,
}

static STATE: OnceLock<MetricsState> = OnceLock::new();

fn state() -> &'static MetricsState {
    STATE.get_or_init(|| {
        let service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "forte-app".to_string());
        let resource = Resource::builder()
            .with_attribute(KeyValue::new("service.name", service_name))
            .build();
        let reader = Arc::new(
            ManualReader::builder()
                .with_temporality(Temporality::Delta)
                .build(),
        );
        let provider = SdkMeterProvider::builder()
            .with_reader(ForteMetricReader(reader.clone()))
            .with_resource(resource)
            .build();
        let meter = provider.meter("forte-sdk");
        MetricsState {
            _provider: provider,
            reader,
            meter,
        }
    })
}

/// The process-wide [`Meter`]. Instruments created from it are aggregated and
/// exported when the current request finishes.
pub fn meter() -> Meter {
    state().meter.clone()
}

/// Collects the metrics recorded during the current request and exports them
/// over OTLP. A no-op if no instrument was ever created. Called by `serve`.
pub(crate) fn flush() {
    let Some(state) = STATE.get() else {
        return;
    };
    let mut resource_metrics = ResourceMetrics::default();
    if let Err(e) = state.reader.collect(&mut resource_metrics) {
        tracing::warn!(?e, "otlp metrics collect failed");
        return;
    }
    let request = ExportMetricsServiceRequest::from(&resource_metrics);
    if request
        .resource_metrics
        .iter()
        .all(|rm| rm.scope_metrics.is_empty())
    {
        return;
    }
    let mut buf = Vec::with_capacity(request.encoded_len());
    if let Err(e) = request.encode(&mut buf) {
        tracing::warn!(?e, "otlp metrics encode failed");
        return;
    }
    crate::runtime::spawn(async move {
        let request = match Request::builder()
            .method(Method::POST)
            .uri(ENDPOINT)
            .header("content-type", "application/x-protobuf")
            .body(Body::Bytes(buf))
        {
            Ok(request) => request,
            Err(e) => {
                tracing::warn!(?e, "otlp metrics request build failed");
                return;
            }
        };
        if let Err(e) = (Client {}).send(request).await {
            tracing::warn!(?e, "otlp metrics export failed");
        }
    });
}
