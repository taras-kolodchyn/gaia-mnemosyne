use async_trait::async_trait;

use crate::error::MnemoResult;

#[async_trait]
pub trait GraphStore {
    async fn initialize(&self) -> MnemoResult<()>;
    async fn add_node(&self, node_id: &str, properties: Option<&str>) -> MnemoResult<()>;
    async fn add_edge(&self, from: &str, to: &str, label: &str) -> MnemoResult<()>;
    async fn get_node(&self, node_id: &str) -> MnemoResult<bool>;
    async fn remove_node(&self, node_id: &str) -> MnemoResult<()>;
    async fn remove_edge(&self, from: &str, to: &str, label: &str) -> MnemoResult<()>;
    async fn neighbors(&self, node_id: &str) -> MnemoResult<Vec<String>>;
    async fn query_path(&self, start: &str, end: &str, max_hops: usize)
    -> MnemoResult<Vec<String>>;
}
