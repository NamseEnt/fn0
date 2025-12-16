use color_eyre::eyre::Result;
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::collections::HashMap;
use std::env;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_otlp() -> Result<SdkTracerProvider> {
    let otlp_endpoint = env::var("OTLP_ENDPOINT").expect("env var OTLP_ENDPOINT is not set");
    let auth_header = env::var("OTLP_AUTH_HEADER").expect("env var OTLP_AUTH_HEADER is not set");
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), auth_header);

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(otlp_endpoint)
        .with_headers(headers)
        .with_protocol(Protocol::HttpBinary)
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name("hq").build())
        .build();

    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer("hq-tracer");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer)) // Grafana 전송용
        .init();

    info!("otel setup completed");
    Ok(provider)
}

pub fn on_shutdown(provider: SdkTracerProvider) -> Result<()> {
    provider.shutdown()?;
    Ok(())
}
