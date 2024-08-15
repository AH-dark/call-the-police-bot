use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::metrics::metrics_layer;

mod metrics;
mod resource;
mod tracing;

pub fn init(
    service_name: String,
    service_version: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let resource = resource::init_resource(service_name, service_version);

    match tracing::tracer_layer(resource)? {
        Some(telemetry) => {
            let subscriber = Registry::default()
                .with(telemetry)
                .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO")))
                .with(MetricsLayer::default());
            ::tracing::subscriber::set_global_default(subscriber)?;
        }
        None => {
            tracing_subscriber::fmt::init();
        }
    }

    match metrics_layer()? {
        None => {}
        Some(metrics) => {
            let recorder = TracingContextLayer::all().layer(metrics);
            ::metrics::set_global_recorder(recorder)?;
        }
    }

    Ok(())
}
