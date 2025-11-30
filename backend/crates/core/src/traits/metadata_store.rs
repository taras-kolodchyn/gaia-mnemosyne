use async_trait::async_trait;

use crate::error::MnemoResult;

#[async_trait]
pub trait MetadataStore {
    async fn initialize(&self) -> MnemoResult<()>;
    async fn put(&self, key: &str, value: &str) -> MnemoResult<()>;
    async fn get(&self, key: &str) -> MnemoResult<Option<String>>;
    async fn delete(&self, key: &str) -> MnemoResult<()>;
    async fn list_by_prefix(&self, prefix: &str) -> MnemoResult<Vec<(String, String)>>;
}
