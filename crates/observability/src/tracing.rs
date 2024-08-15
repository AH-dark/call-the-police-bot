use opentelemetry::global;
use opentelemetry_otlp::{
    ExportConfig, HttpExporterBuilder, SpanExporterBuilder, TonicExporterBuilder, WithExportConfig,
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::{Sampler, Tracer};
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::Registry;

pub fn tracer_layer(
    resource: Resource,
) -> Result<Option<OpenTelemetryLayer<Registry, Tracer>>, Box<dyn std::error::Error>> {
    let export_config = ExportConfig {
        endpoint: std::env::var("OTEL_TRACE_EXPORTER_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string()),
        ..Default::default()
    };

    let exporter = match std::env::var("OTEL_TRACE_EXPORTER")
        .unwrap_or_else(|_| "otlp_grpc".to_string())
        .as_str()
    {
        "otlp_http" => SpanExporterBuilder::Http(
            HttpExporterBuilder::default().with_export_config(export_config),
        ),
        "otlp_grpc" => SpanExporterBuilder::Tonic(
            TonicExporterBuilder::default().with_export_config(export_config),
        ),
        _ => {
            return Ok(None);
        }
    };

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(
                    std::env::var("OTEL_TRACE_SAMPLE_RATE")
                        .unwrap_or_else(|_| "1".to_string())
                        .parse()
                        .unwrap(),
                ))
                .with_resource(resource),
        )
        .install_batch(Tokio)
        .expect("Failed to install `opentelemetry` tracer.");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    Ok(Some(telemetry))
}
