/// Represents a source document discovered by ingestion.
#[derive(Clone)]
pub struct Document {
    pub path: String,
    pub content: String,
    pub fingerprint: String,
    pub namespace: String,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub language: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
