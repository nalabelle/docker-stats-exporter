mod convert;
mod docker;
mod error;

use crate::convert::Measurements;
use crate::docker::DockerContainerStats;
use crate::error::ApiResult;
use anyhow::Result;
use axum::{routing::get, Router};
use convert::ContainerMetrics;
use prometheus::{Encoder, Registry, TextEncoder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn get_prometheus_format(stats: Vec<DockerContainerStats>) -> Result<String> {
    let registry = Registry::new();
    let gauges = Measurements::new();
    for container_stats in &stats {
        let measurements = ContainerMetrics::new(container_stats);
        measurements.set_gauges(&gauges);
        gauges.register(&registry)?;
    }

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    encoder.encode(&metric_families, &mut buffer)?;

    let str = String::from_utf8(buffer)?;
    Ok(str)
}

async fn docker_stats_metrics() -> ApiResult<String> {
    let stats = docker::stats()?;
    let prometheus_stuff = get_prometheus_format(stats)?;
    Ok(prometheus_stuff)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "docker_stats_exporter=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let bind_address = "0.0.0.0:3069";
    tracing::info!("Starting docker stats exporter on {}", bind_address);

    let app = Router::new()
        .route("/docker-stats/metrics", get(docker_stats_metrics))
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
