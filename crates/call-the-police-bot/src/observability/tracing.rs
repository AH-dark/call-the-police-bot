use opentelemetry::global;
use opentelemetry_otlp::{ExportConfig, HttpExporterBuilder, SpanExporterBuilder, TonicExporterBuilder, WithExportConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::Sampler;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;

use crate::observability::resource::init_resource;

pub fn init_tracer() {
    let export_config = ExportConfig {
        endpoint: std::env::var("OTEL_EXPORTER_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string()),
        ..Default::default()
    };

    let exporter = match std::env::var("OTEL_EXPORTER").unwrap_or_else(|_| "otlp_grpc".to_string()).as_str() {
        "otlp_http" => SpanExporterBuilder::Http(
            HttpExporterBuilder::default().with_export_config(export_config),
        ),
        "otlp_grpc" => SpanExporterBuilder::Tonic(
            TonicExporterBuilder::default().with_export_config(export_config),
        ),
        _ => {
            panic!("`OTEL_EXPORTER` not supported");
        }
    };

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(
                    std::env::var("OTEL_SAMPLE_RATE")
                        .unwrap_or_else(|_| "1".to_string())
                        .parse()
                        .unwrap(),
                ))
                .with_resource(init_resource()),
        )
        .install_batch(Tokio)
        .expect("Failed to install `opentelemetry` tracer.");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO"));
    let subscriber = Registry::default().with(telemetry).with(env_filter);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");
}