/// Placeholder SurrealDB graph store adapter.
pub struct SurrealGraphStore {
    pub url: String,
}

impl SurrealGraphStore {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Surreal 2.0 is schemaless by default; no-op initializer.
    pub async fn init_schema(&self) -> mnemo_core::error::MnemoResult<()> {
        Ok(())
    }
}
