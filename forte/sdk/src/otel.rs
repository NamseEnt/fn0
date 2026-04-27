use crate::http::{Body, Client, Method, Request};
use opentelemetry::KeyValue;
use opentelemetry::trace::SpanId;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::trace::{SdkTracerProvider, Span, SpanData, SpanExporter, SpanProcessor};
use prost::Message;
use std::sync::{Arc, Mutex, OnceLock};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const ENDPOINT: &str = "http://fn0-otel.fn0.dev/v1/traces";

#[derive(Debug, Clone)]
struct ForteOtlpExporter {
    endpoint: String,
    resource: Resource,
}

impl SpanExporter for ForteOtlpExporter {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        let resource_spans =
            opentelemetry_proto::transform::trace::tonic::group_spans_by_resource_and_scope(
                batch,
                &(&self.resource).into(),
            );
        let payload = ExportTraceServiceRequest { resource_spans };
        let mut buf = Vec::with_capacity(payload.encoded_len());
        payload
            .encode(&mut buf)
            .map_err(|e| OTelSdkError::InternalFailure(e.to_string()))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(&self.endpoint)
            .header("content-type", "application/x-protobuf")
            .body(Body::Bytes(buf))
            .map_err(|e| OTelSdkError::InternalFailure(e.to_string()))?;

        Client {}
            .send(req)
            .await
            .map_err(|e| OTelSdkError::InternalFailure(e.to_string()))?;
        Ok(())
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.resource = resource.clone();
    }
}

struct BufferedAsyncProcessor {
    buffer: Mutex<Vec<SpanData>>,
    exporter: Arc<ForteOtlpExporter>,
}

impl std::fmt::Debug for BufferedAsyncProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferedAsyncProcessor").finish()
    }
}

impl SpanProcessor for BufferedAsyncProcessor {
    fn on_start(&self, _span: &mut Span, _cx: &opentelemetry::Context) {}

    fn on_end(&self, span: SpanData) {
        let is_root = span.parent_span_id == SpanId::INVALID;
        if let Ok(mut buf) = self.buffer.lock() {
            buf.push(span);
            if is_root {
                let batch: Vec<SpanData> = buf.drain(..).collect();
                drop(buf);
                let exporter = self.exporter.clone();
                crate::runtime::spawn(async move {
                    if let Err(e) = exporter.export(batch).await {
                        tracing::warn!(?e, "otlp export failed");
                    }
                });
            }
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: std::time::Duration) -> OTelSdkResult {
        Ok(())
    }

    fn set_resource(&mut self, resource: &Resource) {
        if let Some(exporter) = Arc::get_mut(&mut self.exporter) {
            exporter.set_resource(resource);
        }
    }
}

static INIT: OnceLock<()> = OnceLock::new();

pub(crate) fn init_once() {
    INIT.get_or_init(|| {
        let service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "forte-app".to_string());
        let resource = Resource::builder()
            .with_attribute(KeyValue::new("service.name", service_name.clone()))
            .build();
        let exporter = Arc::new(ForteOtlpExporter {
            endpoint: ENDPOINT.to_string(),
            resource: resource.clone(),
        });
        let processor = BufferedAsyncProcessor {
            buffer: Mutex::new(Vec::new()),
            exporter,
        };
        let provider = SdkTracerProvider::builder()
            .with_span_processor(processor)
            .with_resource(resource)
            .build();
        let tracer = opentelemetry::trace::TracerProvider::tracer(&provider, service_name);
        let _ = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .try_init();
        opentelemetry::global::set_tracer_provider(provider);
    });
}
