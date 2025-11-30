/// Unified representation of ingestion sources.
pub enum DataSource {
    GitHub { repo: String },
    Filesystem { path: String },
}
