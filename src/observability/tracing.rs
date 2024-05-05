use opentelemetry::global;
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace;
use opentelemetry_sdk::trace::{Sampler, TracerProvider};

use crate::observability::resource::init_resource;

pub fn init_tracer() {
    let export_config = ExportConfig {
        endpoint: std::env::var("OTEL_EXPORTER_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string()),
        ..Default::default()
    };

    let exporter = match std::env::var("OTEL_EXPORTER").unwrap_or_else(|_| "otlp_grpc".to_string()).as_str() {
        "otlp_http" => opentelemetry_otlp::new_exporter()
            .http()
            .with_export_config(export_config)
            .build_span_exporter()
            .unwrap(),
        "otlp_grpc" => opentelemetry_otlp::new_exporter()
            .tonic()
            .with_export_config(export_config)
            .build_span_exporter()
            .unwrap(),
        _ => {
            panic!("`OTEL_EXPORTER` not supported");
        }
    };

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        .with_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(
                    std::env::var("OTEL_SAMPLE_RATE")
                        .unwrap_or_else(|_| "1".to_string())
                        .parse()
                        .unwrap(),
                ))
                .with_resource(init_resource()),
        )
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider);
}