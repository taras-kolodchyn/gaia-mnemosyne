use axum::Json;
use mnemo_core::jobs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateJobRequest {
    pub job_type: String,
}

#[derive(serde::Serialize)]
pub struct CreateJobResponse {
    pub id: String,
}

pub async fn create_job(
    Json(req): Json<CreateJobRequest>,
) -> Result<Json<CreateJobResponse>, crate::handlers::error::ApiError> {
    let job =
        jobs::create_job(req.job_type).await.map_err(crate::handlers::error::ApiError::from)?;
    Ok(Json(CreateJobResponse { id: job.id }))
}
