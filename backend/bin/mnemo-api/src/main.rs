use axum::Router;
use mnemo_api::build_router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, prelude::*};

#[tokio::main]
async fn main() {
    // Logging (OTEL-ready). Uncomment the OTLP section to export traces.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_level(true)
        .compact();

    // OTEL EXAMPLE (disabled):
    // let otel = opentelemetry_otlp::new_exporter()
    //     .tonic()
    //     .with_endpoint("http://otel-collector:4317");
    // let tracer = opentelemetry_otlp::new_pipeline()
    //     .tracing()
    //     .with_exporter(otel)
    //     .install_simple()
    //     .expect("install otel");
    // tracing_subscriber::registry()
    //     .with(fmt_layer)
    //     .with(tracing_opentelemetry::layer().with_tracer(tracer))
    //     .init();

    // Use env-based filter (default: info). Set RUST_LOG=debug or trace to increase verbosity.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry().with(env_filter).with(fmt_layer).init();

    let app: Router = build_router();
    let app = app.layer(CorsLayer::permissive());
    let port = std::env::var("MNEMO_HTTP_PORT").unwrap_or("7700".into());
    let addr = format!("0.0.0.0:{}", port);
    info!("Starting Gaia Mnemosyne API service on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
