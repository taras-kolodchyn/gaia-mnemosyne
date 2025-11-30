use axum::Json;
use serde_json::json;

pub async fn ingestion_metrics() -> Json<serde_json::Value> {
    let metrics = mnemo_ingest::metrics::snapshot_last_metrics();
    Json(json!({
        "documents": metrics.documents_processed,
        "chunks": metrics.chunks_produced,
        "embeddings": metrics.embedding_calls,
        "qdrant_writes": metrics.qdrant_writes,
        "duration_ms": metrics.duration_ms
    }))
}
