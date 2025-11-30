use async_trait::async_trait;

use crate::error::MnemoResult;

#[async_trait]
pub trait CacheStore {
    async fn initialize(&self) -> MnemoResult<()>;
    async fn set(&self, key: &str, value: &[u8], ttl_seconds: Option<u64>) -> MnemoResult<()>;
    async fn get(&self, key: &str) -> MnemoResult<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> MnemoResult<()>;
    async fn exists(&self, key: &str) -> MnemoResult<bool>;
    async fn purge_expired(&self) -> MnemoResult<usize>;
}
