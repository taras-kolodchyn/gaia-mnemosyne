use super::jobs::{IngestJob, IngestJobType};
use super::runner::IngestionRunner;

/// Dispatcher that directs ingest jobs to the runner based on job type.
pub struct IngestionDispatcher {
    pub runner: IngestionRunner,
}

impl IngestionDispatcher {
    pub fn new(runner: IngestionRunner) -> Self {
        Self { runner }
    }

    pub async fn dispatch(&self, job: IngestJob) {
        match job.job_type {
            IngestJobType::FilesystemScan => self.runner.run(job).await,
            IngestJobType::GitHubScan => self.runner.run(job).await,
            IngestJobType::Reindex => self.runner.run(job).await,
        }
    }
}
