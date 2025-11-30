use crate::handlers;
use crate::handlers::ingestion_metrics::ingestion_metrics;
use crate::middleware::rate_limiter::rate_limit;
use crate::openapi::build_openapi;
use crate::ws::all_ws::all_ws;
use crate::ws::graph_ws::graph_ws;
use crate::ws::jobs_ws::jobs_ws;
use crate::ws::logs_ws::logs_ws;
use crate::ws::rag_ws::rag_ws;
use crate::ws::status_ws::status_ws;
use axum::{
    Json, Router, middleware,
    routing::{get, post},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing::warn;

pub fn build_router() -> Router {
    validate_env_vars();

    // Permissive CORS for local development; adjust for production.
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let rag_routes = Router::new()
        .route("/v1/rag/query", post(handlers::rag_query::rag_query))
        .route("/v1/rag/debug", post(handlers::rag_debug::rag_debug))
        .layer(middleware::from_fn(rate_limit));

    let router = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/v1/health", get(handlers::health::health))
        .route("/v1/context/query", post(handlers::context_query::context_query))
        .route("/v1/rag/test", get(handlers::rag_test::rag_test))
        .route("/v1/graph/snapshot", get(handlers::graph_snapshot::graph_snapshot))
        .route("/v1/graph/debug", get(handlers::graph_debug::graph_debug))
        .route("/v1/graph/node/:id", get(handlers::graph_node::graph_node))
        .route("/v1/graph/expand/:id", get(handlers::graph_expand::graph_expand))
        .route(
            "/v1/jobs",
            get(handlers::jobs_list::list_jobs).post(handlers::jobs_create::create_job),
        )
        .route("/v1/jobs/create", post(handlers::jobs_create::create_job))
        .route("/v1/jobs/run", post(handlers::jobs_run::run_job))
        .route("/v1/providers", get(handlers::providers_list::providers_list))
        .route(
            "/v1/config/ingestion",
            get(|| async {
                Json(serde_json::json!({
                    "mode": "filesystem",
                    "root_paths": ["./data"],
                    "frequency": "manual"
                }))
            }),
        )
        .route("/v1/docs/clusters", get(handlers::docs_clusters::docs_clusters))
        .route("/v1/rag/metadata", get(handlers::rag_metadata::rag_metadata))
        .route("/v1/ingestion/metrics", get(ingestion_metrics))
        .route("/v1/version", get(handlers::version::version))
        .route("/v1/reindex", post(handlers::reindex::reindex))
        .route("/metrics", get(handlers::metrics::metrics))
        .route("/v1/metrics", get(handlers::metrics_basic::metrics_basic))
        .route("/ws/status", get(status_ws))
        .route("/ws/jobs", get(jobs_ws))
        .route("/ws/logs", get(logs_ws))
        .route("/ws/graph", get(graph_ws))
        .route("/ws/rag", get(rag_ws))
        .route("/ws/all", get(all_ws))
        .route("/swagger.json", get(|| async { Json(build_openapi()) }))
        .merge(rag_routes);

    router.layer(TraceLayer::new_for_http()).layer(cors)
}

fn validate_env_vars() {
    fn check(name: &str) {
        match std::env::var(name) {
            Ok(val) if !val.is_empty() => info!("env {}={}", name, val),
            _ => warn!("env {} is not set; falling back to defaults", name),
        }
    }

    check("QDRANT_URL");
    check("SURREALDB_URL");
    check("REDIS_URL");
    check("POSTGRES_URL");
}
