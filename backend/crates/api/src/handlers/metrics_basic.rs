use axum::Json;
use serde_json::json;

pub async fn metrics_basic() -> Json<serde_json::Value> {
    Json(json!({
        "qdrant_latency": null,
        "redis_ping": null,
        "surreal_latency": null
    }))
}
