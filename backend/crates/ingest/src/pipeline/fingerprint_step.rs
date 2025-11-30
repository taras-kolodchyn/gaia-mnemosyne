use async_trait::async_trait;
use mnemo_core::error::MnemoResult;
use mnemo_core::ws::WS_HUB;
use mnemo_storage::metadata::postgres::PostgresMetadataStore;
use serde_json::json;

use super::{data::PipelineData, step::PipelineStep};

fn broadcast_step(job_id: &Option<String>, step: &str, status: &str) {
    if let Some(id) = job_id {
        let _ = WS_HUB.broadcast(
            json!({"event":"ingest_step","job_id":id,"step":step,"status":status}).to_string(),
        );
    }
}

/// Skips documents whose fingerprints are unchanged based on metadata store.
pub struct FingerprintStep {
    pub store: PostgresMetadataStore,
}

impl FingerprintStep {
    pub fn new(store: PostgresMetadataStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl PipelineStep for FingerprintStep {
    async fn process(&self, mut data: PipelineData) -> MnemoResult<PipelineData> {
        let job_id = data.job_id.clone();
        broadcast_step(&job_id, "fingerprints", "running");

        let mut filtered = Vec::new();
        for doc in data.documents.drain(..) {
            let existing = self.store.get_fingerprint(&doc.path).await?;
            if let Some(hash) = existing {
                if hash == doc.fingerprint {
                    tracing::info!("Skipping unchanged document: {}", doc.path);
                    let _ = WS_HUB.broadcast(
                        json!({"event":"log","message":format!("Skip unchanged {}", doc.path), "job_id": job_id})
                            .to_string(),
                    );
                    continue;
                }
            }
            self.store.set_fingerprint(&doc.path, &doc.fingerprint).await?;
            let _ = self.store.insert_document_metadata(&doc).await;
            tracing::info!("Document fingerprint stored: {}", doc.path);
            let _ = WS_HUB.broadcast(
                json!({"event":"log","message":format!("Fingerprint stored {}", doc.path), "job_id": job_id})
                    .to_string(),
            );
            filtered.push(doc);
        }
        data.documents = filtered;
        broadcast_step(&job_id, "fingerprints", "done");
        Ok(data)
    }
}
