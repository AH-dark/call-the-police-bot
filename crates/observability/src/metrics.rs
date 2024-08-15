use std::env;
use std::net::SocketAddr;

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusRecorder};

pub fn metrics_layer() -> Result<Option<PrometheusRecorder>, Box<dyn std::error::Error>> {
    match env::var("OTEL_METRICS_EXPORTER")?.as_str() {
        "prometheus" => {
            let socket = env::var("OTEL_METRICS_EXPORTER_ENDPOINT")
                .clone()
                .unwrap_or("0.0.0.0:9090".into())
                .parse::<SocketAddr>()?;
            let builder = PrometheusBuilder::new().with_http_listener(socket);

            let (recorder, exporter_future) = builder.build()?;
            tokio::spawn(exporter_future);
            Ok(Some(recorder))
        }
        _ => Ok(None),
    }
}
