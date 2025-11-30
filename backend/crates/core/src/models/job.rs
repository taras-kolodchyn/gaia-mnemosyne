/// Represents a unit of work scheduled in the system.
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub payload: String,
    pub status: String,
}

/// Categories of jobs handled by the scheduler.
pub enum JobType {
    IngestRepo,
    IngestFS,
    Reindex,
    Custom(String),
}
