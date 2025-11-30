use axum::{Json, extract::Query};
use mnemo_core::jobs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JobsQuery {
    pub limit: Option<usize>,
}

pub async fn list_jobs(
    Query(params): Query<JobsQuery>,
) -> Result<Json<Vec<mnemo_core::jobs::JobDTO>>, crate::handlers::error::ApiError> {
    let jobs =
        jobs::list_jobs(params.limit).await.map_err(crate::handlers::error::ApiError::from)?;
    Ok(Json(jobs))
}
