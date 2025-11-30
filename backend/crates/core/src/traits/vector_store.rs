use async_trait::async_trait;

use crate::error::MnemoResult;

#[async_trait]
pub trait VectorStore {
    async fn initialize(&self) -> MnemoResult<()>;
    async fn upsert_vector(
        &self,
        id: &str,
        values: &[f32],
        metadata: Option<&str>,
    ) -> MnemoResult<()>;
    async fn get_vector(&self, id: &str) -> MnemoResult<Option<Vec<f32>>>;
    async fn delete_vector(&self, id: &str) -> MnemoResult<()>;
    async fn search_similar(&self, query: &[f32], top_k: usize) -> MnemoResult<Vec<String>>;
}
