use crate::handlers::error::ApiError;
use crate::models::{context_request::ContextQueryRequest, context_response::ContextQueryResponse};
use axum::Json;

pub async fn context_query(
    Json(_req): Json<ContextQueryRequest>,
) -> Result<Json<ContextQueryResponse>, ApiError> {
    Ok(Json(ContextQueryResponse {
        project_chunks: vec![],
        domain_chunks: vec![],
        company_chunks: vec![],
        metadata: serde_json::json!({}),
    }))
}
