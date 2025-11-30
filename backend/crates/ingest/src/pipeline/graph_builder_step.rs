use async_trait::async_trait;
use mnemo_core::error::MnemoResult;
use mnemo_core::ws::WS_HUB;
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{data::PipelineData, step::PipelineStep};

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
            let client = Client::new();
            let mut statements: Vec<String> = Vec::new();

            for doc in &data.documents {
                let file_id = format!("file:{}", hash_id(&doc.path));
                statements.push(format!(
                    "DELETE FROM file WHERE id = '{file_id}'; INSERT INTO file (id, path, namespace) VALUES ('{file_id}', '{path}', '{ns}');",
                    file_id=file_id,
                    path=doc.path.replace('\'', "''"),
                    ns=doc.namespace.replace('\'', "''")
                ));
            }

            for chunk in &data.chunks {
                let file_id = format!("file:{}", hash_id(&chunk.document_path));
                let chunk_id = format!(
                    "chunk:{}",
                    hash_id(&format!("{}#{}", chunk.document_path, chunk.chunk_index))
                );
                statements.push(format!(
                    "DELETE FROM chunk WHERE id = '{chunk_id}'; INSERT INTO chunk (id, path, namespace, chunk_index) VALUES ('{chunk_id}', '{path}', '{ns}', {idx});",
                    chunk_id=chunk_id,
                    path=chunk.document_path.replace('\'', "''"),
                    ns=chunk.namespace.replace('\'', "''"),
                    idx=chunk.chunk_index as i32,
                ));
                statements.push(format!(
                    "DELETE FROM contains WHERE id = '{edge_id}'; INSERT INTO contains (id, in, out, relation) VALUES ('{edge_id}', '{file_id}', '{chunk_id}', 'contains');",
                    edge_id=hash_id(&format!("{file_id}->{chunk_id}")),
                    file_id=file_id,
                    chunk_id=chunk_id
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

            let sql = statements.join("\n");
            let user = std::env::var("SURREALDB_USER").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "root".into());
            let pass = std::env::var("SURREALDB_PASS").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "root".into());
            let ns = std::env::var("SURREALDB_NS").unwrap_or_else(|_| "mnemo".into());
            let db = std::env::var("SURREALDB_DB").unwrap_or_else(|_| "mnemo".into());
            tracing::info!("Surreal graph query:\n{}", sql);

            let mut req = client
                .post(format!("{}/sql", self.surreal_url))
                .header("NS", ns)
                .header("DB", db)
                .header("Content-Type", "text/plain")
                .header("Accept", "application/json")
                .body(sql);
            req = req.basic_auth(user, Some(pass));

            let res = req
                .send()
                .await
                .map_err(|e| mnemo_core::error::MnemoError::Message(e.to_string()))?;

            let status_code = res.status();
            if status_code.is_success() {
                let _ = WS_HUB.broadcast(
                    json!({"event":"ingest_step","job_id": job_id,"step":"graph_upsert","status":"done"}).to_string(),
                );
                let _ = WS_HUB.broadcast(
                    json!({"event":"log","message":"Graph upsert completed","job_id": job_id}).to_string(),
                );
                Ok(data)
            } else {
                let body = res.text().await.unwrap_or_else(|_| "<no body>".into());
                tracing::error!("Surreal graph upsert failed: status={} body={}", status_code, body);
                let _ = WS_HUB.broadcast(
                    json!({"event":"ingest_step","job_id": job_id,"step":"graph_upsert","status":"failed"}).to_string(),
                );
                Err(mnemo_core::error::MnemoError::Message(format!(
                    "Surreal schema write failed: {} body: {}",
                    status_code,
                    body
                )))
            }
        }
        .await;

        match result {
            Ok(out) => Ok(out),
            Err(e) => Err(e),
        }
    }
}
