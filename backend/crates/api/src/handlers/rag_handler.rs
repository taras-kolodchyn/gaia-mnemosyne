use axum::Json;
use mnemo_core::rag::api::RAGPipeline;

use crate::models::context_request::ContextQueryRequest;
use crate::models::context_response::ContextQueryResponse;

pub async fn rag_query(Json(req): Json<ContextQueryRequest>) -> Json<ContextQueryResponse> {
    let pipeline = RAGPipeline::new();
    let ctx = pipeline.query(&req.query).await;

    Json(ContextQueryResponse {
        project_chunks: ctx.project_chunks,
        domain_chunks: ctx.domain_chunks,
        company_chunks: ctx.company_chunks,
        metadata: serde_json::json!({
            "graph_neighbors": ctx.graph_neighbors,
            "ontology_tags": ctx.ontology_tags,
            "debug_candidates": ctx.debug_candidates,
        }),
    })
}
