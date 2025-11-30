use chrono::Utc;
use serde::Serialize;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::error::{MnemoError, MnemoResult};

#[derive(Clone, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Success,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Success => "success",
            JobStatus::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Serialize)]
pub struct JobDTO {
    pub id: String,
    pub job_type: String,
    pub status: JobStatus,
    pub created_at: String,
    pub updated_at: String,
    pub progress: u8,
}

pub fn can_transition(from: &JobStatus, to: &JobStatus) -> bool {
    matches!(
        (from, to),
        (JobStatus::Pending, JobStatus::Running)
            | (JobStatus::Running, JobStatus::Success)
            | (JobStatus::Running, JobStatus::Failed)
    )
}

static PG_POOL: OnceCell<PgPool> = OnceCell::const_new();

async fn pool() -> MnemoResult<&'static PgPool> {
    PG_POOL
        .get_or_try_init(|| async {
            let url = std::env::var("DATABASE_URL")
                .map_err(|_| MnemoError::Message("DATABASE_URL must be set".into()))?;
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
                .map_err(|e| MnemoError::Message(format!("pg pool connect failed: {e}")))
        })
        .await
}

fn status_from_str(s: &str) -> JobStatus {
    match s {
        "running" => JobStatus::Running,
        "success" => JobStatus::Success,
        "failed" => JobStatus::Failed,
        _ => JobStatus::Pending,
    }
}

pub async fn list_jobs(limit: Option<usize>) -> MnemoResult<Vec<JobDTO>> {
    let pool = pool().await?;
    let lim = limit.unwrap_or(100).min(500) as i64;
    let sql = format!(
        "SELECT id, job_type, status, created_at, updated_at, progress \
         FROM jobs ORDER BY created_at DESC LIMIT {}",
        lim
    );
    let rows = sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .map_err(|e| MnemoError::Message(format!("list_jobs failed: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let id: uuid::Uuid = r.try_get("id").unwrap_or_else(|_| Uuid::nil());
            let job_type: String = r.try_get("job_type").unwrap_or_default();
            let status_str: String = r.try_get("status").unwrap_or_else(|_| "pending".into());
            let created_at: chrono::DateTime<Utc> =
                r.try_get("created_at").unwrap_or_else(|_| Utc::now());
            let updated_at: chrono::DateTime<Utc> =
                r.try_get("updated_at").unwrap_or_else(|_| Utc::now());
            let progress: u8 = r.try_get::<i32, _>("progress").unwrap_or(0).clamp(0, 100) as u8;
            JobDTO {
                id: id.to_string(),
                job_type,
                status: status_from_str(&status_str),
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
                progress,
            }
        })
        .collect())
}

pub async fn create_job(job_type: String) -> MnemoResult<JobDTO> {
    let pool = pool().await?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query(
        r#"INSERT INTO jobs (id, job_type, status, progress, created_at, updated_at)
            VALUES ($1, $2, 'pending', 0, $3, $3)"#,
    )
    .bind(id)
    .bind(&job_type)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| MnemoError::Message(format!("create_job failed: {e}")))?;

    Ok(JobDTO {
        id: id.to_string(),
        job_type,
        status: JobStatus::Pending,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        progress: 0,
    })
}

pub async fn update_job_status(
    job_id: &str,
    status: JobStatus,
    progress: u8,
) -> MnemoResult<Option<JobDTO>> {
    let pool = pool().await?;
    let uuid =
        Uuid::parse_str(job_id).map_err(|e| MnemoError::Message(format!("invalid job id: {e}")))?;
    let now = Utc::now();

    // Load current status for transition validation.
    let current: Option<String> = sqlx::query_scalar("SELECT status FROM jobs WHERE id = $1")
        .bind(uuid)
        .fetch_optional(pool)
        .await
        .map_err(|e| MnemoError::Message(format!("fetch job status failed: {e}")))?;
    if let Some(cur) = current.as_ref() {
        let from = status_from_str(cur);
        if !can_transition(&from, &status) && from != status {
            return Err(MnemoError::Message(format!(
                "invalid status transition from {} to {}",
                from, status
            )));
        }
    } else {
        return Ok(None);
    }
    sqlx::query("UPDATE jobs SET status = $1, progress = $2, updated_at = $3 WHERE id = $4")
        .bind(status.to_string())
        .bind(progress as i32)
        .bind(now)
        .bind(uuid)
        .execute(pool)
        .await
        .map_err(|e| MnemoError::Message(format!("update_job_status failed: {e}")))?;

    let row = sqlx::query(
        r#"SELECT id, job_type, status, created_at, updated_at, progress FROM jobs WHERE id = $1"#,
    )
    .bind(uuid)
    .fetch_optional(pool)
    .await
    .map_err(|e| MnemoError::Message(format!("fetch job after update failed: {e}")))?;

    Ok(row.map(|r| {
        let id: uuid::Uuid = r.try_get("id").unwrap_or_else(|_| Uuid::nil());
        let job_type: String = r.try_get("job_type").unwrap_or_default();
        let status_str: String = r.try_get("status").unwrap_or_else(|_| "pending".into());
        let created_at: chrono::DateTime<Utc> =
            r.try_get("created_at").unwrap_or_else(|_| Utc::now());
        let updated_at: chrono::DateTime<Utc> =
            r.try_get("updated_at").unwrap_or_else(|_| Utc::now());
        let progress_db: u8 =
            r.try_get::<i32, _>("progress").unwrap_or(progress as i32).clamp(0, 100) as u8;
        JobDTO {
            id: id.to_string(),
            job_type,
            status: status_from_str(&status_str),
            created_at: created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
            progress: progress_db,
        }
    }))
}
