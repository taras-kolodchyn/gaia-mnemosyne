use axum::Json;
use serde::Deserialize;

use crate::handlers::error::ApiError;
use mnemo_ingest::job_runner::run_job as ingest_run_job;

#[derive(Deserialize)]
pub struct RunJobRequest {
    pub job_id: String,
}

#[derive(serde::Serialize)]
pub struct RunJobResponse {
    pub status: String,
    pub job: mnemo_core::jobs::JobDTO,
}

pub async fn run_job(Json(req): Json<RunJobRequest>) -> Result<Json<RunJobResponse>, ApiError> {
    let job = ingest_run_job(&req.job_id).await.map_err(ApiError::from)?;
    Ok(Json(RunJobResponse { status: "ok".into(), job }))
}
