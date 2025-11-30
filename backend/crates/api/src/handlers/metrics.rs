use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct MetricsResponse {
    pub qdrant_latency: Option<f64>,
    pub redis_ping: Option<f64>,
    pub surreal_latency: Option<f64>,
}

pub async fn metrics() -> Json<MetricsResponse> {
    Json(MetricsResponse { qdrant_latency: None, redis_ping: None, surreal_latency: None })
}
