use bytes::Bytes;
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_http::{HttpClient, HttpError};
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Handle;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub type TelemetryProviders = (SdkTracerProvider, SdkMeterProvider, SdkLoggerProvider);

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
                let request = request.try_into()?;
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

pub fn setup(
    endpoint: &str,
    basic_auth: Option<&str>,
) -> color_eyre::eyre::Result<TelemetryProviders> {
    let headers = build_headers(basic_auth);
    let traces_endpoint = format!("{}/v1/traces", endpoint.trim_end_matches('/'));
    let metrics_endpoint = format!("{}/v1/metrics", endpoint.trim_end_matches('/'));
    let logs_endpoint = format!("{}/v1/logs", endpoint.trim_end_matches('/'));

    let http_client = TokioHttpClient {
        client: reqwest::Client::builder().build()?,
        handle: Handle::current(),
    };

    let tracer_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_http_client(http_client.clone())
        .with_endpoint(&traces_endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_headers(headers.clone())
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(tracer_exporter)
        .with_resource(Resource::builder().with_service_name("fn0-worker").build())
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .with_http_client(http_client.clone())
        .with_endpoint(&logs_endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_headers(headers.clone())
        .build()?;

    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(Resource::builder().with_service_name("fn0-worker").build())
        .build();

    let log_bridge = OpenTelemetryTracingBridge::new(&logger_provider);

    let tracer = tracer_provider.tracer("fn0-worker-tracer");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(log_bridge)
        .init();

    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_http_client(http_client)
        .with_endpoint(&metrics_endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_headers(headers)
        .build()?;

    let reader = PeriodicReader::builder(metric_exporter)
        .with_interval(Duration::from_secs(10))
        .build();

    let meter_provider = SdkMeterProvider::builder()
        .with_resource(Resource::builder().with_service_name("fn0-worker").build())
        .with_reader(reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    info!("telemetry setup completed with OTLP endpoint: {}", endpoint);
    Ok((tracer_provider, meter_provider, logger_provider))
}

pub fn shutdown(
    (tracer_provider, meter_provider, logger_provider): TelemetryProviders,
) -> color_eyre::eyre::Result<()> {
    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    logger_provider.shutdown()?;
    Ok(())
}

fn build_headers(basic_auth: Option<&str>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    if let Some(auth) = basic_auth {
        headers.insert("authorization".to_string(), format!("Basic {auth}"));
    }
    headers
}
