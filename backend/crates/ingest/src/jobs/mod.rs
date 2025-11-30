/// Types representing ingestion-specific jobs.
pub enum IngestJobType {
    FilesystemScan,
    GitHubScan,
    Reindex,
}

/// A typed ingestion job carrying its target.
pub struct IngestJob {
    pub job_type: IngestJobType,
    pub target: String,
}
