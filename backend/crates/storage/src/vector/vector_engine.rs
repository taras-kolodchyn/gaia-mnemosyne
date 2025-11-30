use super::qdrant::QdrantVectorStore;
use mnemo_core::error::MnemoResult;
use serde_json::Map;
use serde_json::Value;
use serde_json::json;

/// Placeholder wrapper around a Qdrant vector store.
pub struct VectorEngine {
    pub store: QdrantVectorStore,
}

impl VectorEngine {
    pub fn new(store: QdrantVectorStore) -> Self {
        Self { store }
    }

    pub async fn search(
        &self,
        query: Vec<f32>,
        top_k: usize,
        namespace: &str,
        tag: Option<&str>,
    ) -> MnemoResult<Vec<String>> {
        if top_k == 0 {
            return Ok(Vec::new());
        }

        let client = reqwest::Client::new();
        let url = format!("{}/collections/mnemo_chunks/points/search", self.store.url);

        // Sparse query is not provided here (legacy call); dense-only fallback.
        let mut must = vec![json!({
            "key": "namespace",
            "match": { "value": namespace }
        })];
        if let Some(t) = tag {
            must.push(json!({
                "key": "tags",
                "match": { "value": t }
            }));
        }

        let payload = json!({
            "vector": {
                "name": "dense",
                "vector": query
            },
            "limit": top_k,
            "with_payload": true,
            "filter": {
                "must": must
            }
        });

        let res = client.post(url).json(&payload).send().await;
        let mut results = Vec::new();

        match res {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| mnemo_core::error::MnemoError::Message(e.to_string()))?;
                if let Some(arr) = body.get("result").and_then(|r| r.as_array()) {
                    for item in arr {
                        if let Some(payload) = item.get("payload") {
                            if let Some(text) = payload.get("text").and_then(|v| v.as_str()) {
                                results.push(text.to_string());
                            }
                        }
                    }
                }
            }
            _ => {
                // Fallback placeholder when Qdrant is unavailable.
                results.push("vector_result_placeholder".to_string());
            }
        }

        Ok(results)
    }

    pub async fn upsert_chunk(
        &self,
        chunk_id: i64,
        document_path: &str,
        chunk_text: &str,
        vector: Vec<f32>,
        _sparse_indices: &[u32],
        _sparse_values: &[f32],
        namespace: &str,
        tags: &[String],
        chunk_index: usize,
    ) -> MnemoResult<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/collections/mnemo_chunks/points?wait=true", self.store.url);
        // Avoid overlarge payloads to Qdrant by truncating text.
        let text_preview: String = chunk_text.chars().take(2000).collect();

        // Use PointsBatch format to avoid “missing field `ids`” parsing errors.
        let payload = json!({
            "ids": [chunk_id],
            "vectors": [
                { "dense": vector }
            ],
            "payloads": [
                {
                    "text": text_preview,
                    "path": document_path,
                    "namespace": namespace,
                    "chunk_index": chunk_index as i32,
                    "tags": tags,
                }
            ]
        });

        let res = client.post(url).json(&payload).send().await;
        match res {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                Err(mnemo_core::error::MnemoError::Message(format!(
                    "Qdrant upsert failed with status {} body: {}",
                    status, body
                )))
            }
            Err(e) => Err(mnemo_core::error::MnemoError::Message(e.to_string())),
        }
    }

    /// Update only metadata fields for an existing point without touching vectors.
    /// Allows refreshing tags / namespace / file_path when ontology or file metadata changes.
    pub async fn update_metadata(
        &self,
        point_id: &str,
        namespace: Option<&str>,
        tags: Option<&[String]>,
        file_path: Option<&str>,
    ) -> MnemoResult<()> {
        let mut payload: Map<String, Value> = Map::new();
        if let Some(ns) = namespace {
            payload.insert("namespace".into(), Value::String(ns.to_string()));
        }
        if let Some(t) = tags {
            payload.insert(
                "tags".into(),
                Value::Array(t.iter().cloned().map(Value::String).collect()),
            );
        }
        if let Some(path) = file_path {
            payload.insert("file_path".into(), Value::String(path.to_string()));
        }

        // Nothing to update.
        if payload.is_empty() {
            return Ok(());
        }

        let client = reqwest::Client::new();
        let url = format!("{}/collections/mnemo_chunks/points", self.store.url);
        let body = json!({
            "points": [
                {
                    "id": point_id,
                    "payload": Value::Object(payload)
                }
            ]
        });

        let res = client.patch(url).json(&body).send().await.map_err(|e| {
            mnemo_core::error::MnemoError::Message(format!("Qdrant patch failed: {e}"))
        })?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(mnemo_core::error::MnemoError::Message(format!(
                "Qdrant patch failed with status {}",
                res.status()
            )))
        }
    }
}
