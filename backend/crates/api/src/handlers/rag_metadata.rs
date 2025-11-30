use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct RagMetadata {
    pub vector_hits: u64,
    pub graph_depth: u8,
    pub response_time_ms: u64,
}

pub async fn rag_metadata() -> Json<RagMetadata> {
    Json(RagMetadata { vector_hits: 0, graph_depth: 0, response_time_ms: 12 })
}
