use mnemo_core::error::MnemoResult;
use mnemo_core::jobs::{JobDTO, JobStatus, update_job_status};
use mnemo_core::ws::WS_HUB;

use crate::fs_ingest_runner::run_filesystem_ingestion;
use futures::FutureExt;
use serde_json::json;
use std::panic::AssertUnwindSafe;

pub async fn run_job(job_id: &str) -> MnemoResult<JobDTO> {
    let running = update_job_status(job_id, JobStatus::Running, 0).await?;
    let current = running.ok_or(mnemo_core::error::MnemoError::Message("job not found".into()))?;
    let payload = json!({
        "event": "job_update",
        "job_id": current.id,
        "status": current.status.to_string(),
        "progress": current.progress
    });
    WS_HUB.broadcast(payload.to_string());

    let result = AssertUnwindSafe(run_filesystem_ingestion(Some(job_id)))
        .catch_unwind()
        .await
        .map_err(|_| mnemo_core::error::MnemoError::Message("pipeline panicked".into()))
        .and_then(|r| r);

    // Only mark success if every pipeline step completed without error.
    let all_steps_ok = result.is_ok();
    let final_status = if all_steps_ok { JobStatus::Success } else { JobStatus::Failed };
    let final_progress = if all_steps_ok { 100 } else { 0 };
    let updated = update_job_status(job_id, final_status, final_progress).await?;
    if let Some(job) = updated {
        let payload = json!({
            "event": "job_update",
            "job_id": job.id,
            "status": job.status.to_string(),
            "progress": job.progress
        });
        WS_HUB.broadcast(payload.to_string());
        Ok(job)
    } else {
        Err(mnemo_core::error::MnemoError::Message("job not found after execution".into()))
    }
}
