use mnemo_core::error::{MnemoError, MnemoResult};
use reqwest::StatusCode;

/// Placeholder Qdrant vector store adapter.
pub struct QdrantVectorStore {
    pub url: String,
}

impl QdrantVectorStore {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Initialize the Qdrant collection schema if it does not exist.
    pub async fn init_schema(&self) -> MnemoResult<()> {
        let client = reqwest::Client::new();
        let collection_url = format!("{}/collections/mnemo_chunks", self.url);

        let exists = client
            .get(&collection_url)
            .send()
            .await
            .map(|res| res.status().is_success())
            .unwrap_or(false);

        if exists {
            return Ok(());
        }

        let payload = serde_json::json!({
            "vectors": {
                // Name the vector so payloads can use "dense"
                "dense": {
                    "size": 1536,
                    "distance": "Cosine"
                }
            },
            "optimizers_config": {
                "default_segment_number": 2
            }
        });

        let resp = client
            .put(&collection_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| MnemoError::Message(e.to_string()))?;

        if resp.status() == StatusCode::OK || resp.status() == StatusCode::CREATED {
            Ok(())
        } else {
            Err(MnemoError::Message(format!(
                "Failed to init Qdrant schema: status {}",
                resp.status()
            )))
        }
    }
}
