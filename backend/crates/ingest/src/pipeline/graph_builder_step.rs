use async_trait::async_trait;
use mnemo_core::error::MnemoResult;
use mnemo_core::ws::WS_HUB;
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{data::PipelineData, step::PipelineStep};
use mnemo_storage::surreal_rpc_client::SurrealRpcClient;

fn broadcast_step(job_id: &Option<String>, step: &str, status: &str) {
    if let Some(id) = job_id {
        let _ = WS_HUB.broadcast(
            json!({"event":"ingest_step","job_id":id,"step":step,"status":status}).to_string(),
        );
    }
}

pub struct GraphBuilderStep {
    pub surreal_url: String,
    pub metadata_store: mnemo_storage::metadata::postgres::PostgresMetadataStore,
}

impl GraphBuilderStep {
    pub fn new(
        url: String,
        metadata_store: mnemo_storage::metadata::postgres::PostgresMetadataStore,
    ) -> Self {
        Self { surreal_url: url, metadata_store }
    }
}

fn hash_id(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[async_trait]
impl PipelineStep for GraphBuilderStep {
    async fn process(&self, data: PipelineData) -> MnemoResult<PipelineData> {
        let job_id = data.job_id.clone();
        broadcast_step(&job_id, "graph_upsert", "running");
        let result: MnemoResult<PipelineData> = async {
            let client = SurrealRpcClient::get().await?;
            let mut statements: Vec<String> = Vec::new();

            for doc in &data.documents {
                let file_id = format!("file:{}", hash_id(&doc.path));
                let path = doc.path.replace('\'', "''");
                let ns = doc.namespace.replace('\'', "''");
                // Use delete+insert to avoid Surreal 2.x upsert syntax issues.
                statements.push(format!("DELETE FROM file WHERE id = '{file_id}';"));
                statements.push(format!(
                    "INSERT INTO file (id, path, namespace) VALUES ('{file_id}', '{path}', '{ns}');",
                    file_id = file_id,
                    path = path,
                    ns = ns
                ));
            }

            for chunk in &data.chunks {
                let file_id = format!("file:{}", hash_id(&chunk.document_path));
                let chunk_id = format!(
                    "chunk:{}",
                    hash_id(&format!("{}#{}", chunk.document_path, chunk.chunk_index))
                );
                let path = chunk.document_path.replace('\'', "''");
                let ns = chunk.namespace.replace('\'', "''");
                statements.push(format!("DELETE FROM chunk WHERE id = '{chunk_id}';", chunk_id = chunk_id));
                statements.push(format!(
                    "INSERT INTO chunk (id, path, namespace, chunk_index) VALUES ('{chunk_id}', '{path}', '{ns}', {idx});",
                    chunk_id = chunk_id,
                    path = path,
                    ns = ns,
                    idx = chunk.chunk_index as i32,
                ));
                let edge_id = hash_id(&format!("{file_id}->{chunk_id}"));
                statements.push(format!("DELETE FROM contains WHERE id = '{edge_id}';", edge_id = edge_id));
                statements.push(format!(
                    "INSERT INTO contains (id, in, out, relation) VALUES ('{edge_id}', '{file_id}', '{chunk_id}', 'contains');",
                    edge_id = edge_id,
                    file_id = file_id,
                    chunk_id = chunk_id
                ));
            }

            tracing::info!(
                "Surreal graph upsert: {} files, {} chunks",
                data.documents.len(),
                data.chunks.len()
            );
            let _ = WS_HUB.broadcast(
                json!({"event":"log","message":format!("Graph upsert: {} files, {} chunks", data.documents.len(), data.chunks.len()),"job_id": job_id})
                    .to_string(),
            );
            let _ = WS_HUB.broadcast(
                json!({"event":"log","message":"Graph upsert statements prepared","job_id": job_id})
                    .to_string(),
            );

            for stmt in statements {
                tracing::debug!("Surreal RPC exec: {}", stmt);
                if let Err(err) = client.query(&stmt).await {
                    tracing::error!("Surreal RPC statement failed: {err}");
                    let _ = WS_HUB.broadcast(
                        json!({"event":"ingest_step","job_id": job_id,"step":"graph_upsert","status":"failed"}).to_string(),
                    );
                    return Err(err);
                }
            }

            // Debug counts to confirm graph records are actually persisted.
            let file_count = client
                .query("SELECT count() AS c FROM file;")
                .await
                .unwrap_or_default()
                .get(0)
                .and_then(|v| v.get("c"))
                .and_then(|c| c.as_i64())
                .unwrap_or(0);
            let chunk_count = client
                .query("SELECT count() AS c FROM chunk;")
                .await
                .unwrap_or_default()
                .get(0)
                .and_then(|v| v.get("c"))
                .and_then(|c| c.as_i64())
                .unwrap_or(0);
            let edge_count = client
                .query("SELECT count() AS c FROM contains;")
                .await
                .unwrap_or_default()
                .get(0)
                .and_then(|v| v.get("c"))
                .and_then(|c| c.as_i64())
                .unwrap_or(0);
            tracing::info!(
                "Surreal graph counts after upsert: files={}, chunks={}, edges={}",
                file_count,
                chunk_count,
                edge_count
            );

            let _ = WS_HUB.broadcast(
                json!({"event":"ingest_step","job_id": job_id,"step":"graph_upsert","status":"done"}).to_string(),
            );
            let _ = WS_HUB.broadcast(
                json!({"event":"log","message":"Graph upsert completed","job_id": job_id}).to_string(),
            );
            Ok(data)
        }
        .await;

        match result {
            Ok(out) => Ok(out),
            Err(e) => Err(e),
        }
    }
}
