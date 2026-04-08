use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::time::Duration;
use tonic::metadata::MetadataMap;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub type TelemetryProviders = (SdkTracerProvider, SdkMeterProvider);

pub fn setup(
    endpoint: &str,
    basic_auth: Option<&str>,
) -> color_eyre::eyre::Result<TelemetryProviders> {
    let metadata = build_metadata(basic_auth);

    let tracer_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_protocol(Protocol::Grpc)
        .with_metadata(metadata.clone())
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(tracer_exporter)
        .with_resource(Resource::builder().with_service_name("fn0-worker").build())
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    let tracer = tracer_provider.tracer("fn0-worker-tracer");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_protocol(Protocol::Grpc)
        .with_metadata(metadata)
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
    Ok((tracer_provider, meter_provider))
}

pub fn shutdown(
    (tracer_provider, meter_provider): TelemetryProviders,
) -> color_eyre::eyre::Result<()> {
    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    Ok(())
}

fn build_metadata(basic_auth: Option<&str>) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    if let Some(auth) = basic_auth {
        metadata.insert(
            "authorization",
            format!("Basic {auth}").parse().unwrap(),
        );
    }
    metadata
}
