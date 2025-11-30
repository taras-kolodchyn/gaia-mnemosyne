use axum::Json;
use mnemo_core::rag::orchestrator::RAGOrchestrator;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RagDebugRequest {
    pub query: String,
}

#[derive(Serialize)]
pub struct CandidateDebug {
    pub chunk: String,
    pub vector_score: f32,
    pub keyword_score: f32,
    pub graph_score: f32,
    pub knowledge_score: f32,
    pub final_score: f32,
    pub tags: Vec<String>,
    pub neighbors_count: usize,
}

#[derive(Serialize)]
pub struct RagDebugResponse {
    pub candidates: Vec<CandidateDebug>,
}

pub async fn rag_debug(Json(req): Json<RagDebugRequest>) -> Json<RagDebugResponse> {
    let orchestrator = RAGOrchestrator::new();
    let raw = orchestrator.gather_candidates(&req.query, None).await;
    let candidates = raw
        .into_iter()
        .map(|c| CandidateDebug {
            chunk: c.chunk,
            vector_score: c.vector_score,
            keyword_score: c.keyword_score,
            graph_score: c.graph_score,
            knowledge_score: c.ontology_score,
            final_score: c.final_score,
            tags: c.tags,
            neighbors_count: c.neighbors_count,
        })
        .collect();

    Json(RagDebugResponse { candidates })
}
