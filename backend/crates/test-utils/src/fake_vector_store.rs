use async_trait::async_trait;
use mnemo_core::error::MnemoResult;
use mnemo_core::traits::vector_store::VectorStore;

/// Test helper implementing the VectorStore trait without external dependencies.
pub struct FakeVectorStore;

#[async_trait]
impl VectorStore for FakeVectorStore {
    async fn initialize(&self) -> MnemoResult<()> {
        Ok(())
    }

    async fn upsert_vector(
        &self,
        _id: &str,
        _values: &[f32],
        _metadata: Option<&str>,
    ) -> MnemoResult<()> {
        Ok(())
    }

    async fn get_vector(&self, _id: &str) -> MnemoResult<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn delete_vector(&self, _id: &str) -> MnemoResult<()> {
        Ok(())
    }

    async fn search_similar(&self, _query: &[f32], _top_k: usize) -> MnemoResult<Vec<String>> {
        Ok(vec!["fake_result".into()])
    }
}

impl FakeVectorStore {
    /// Convenience helper to mimic a simple search call in tests.
    pub async fn search(&self, query: Vec<f32>, top_k: usize) -> Vec<String> {
        self.search_similar(&query, top_k).await.unwrap_or_default()
    }
}
