/// Represents the ingestion profile selection.
pub struct IngestionProfile {
    pub mode: String, // "filesystem", "github", "hybrid"
}

impl IngestionProfile {
    pub fn load(_profile: &str) -> Self {
        Self { mode: "filesystem".into() }
    }
}
