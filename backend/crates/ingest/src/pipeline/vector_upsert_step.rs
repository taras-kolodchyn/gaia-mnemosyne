use async_trait::async_trait;
use mnemo_core::error::MnemoResult;
use mnemo_core::ws::WS_HUB;
use mnemo_storage::vector::qdrant::QdrantVectorStore;
use mnemo_storage::vector::vector_engine::VectorEngine;
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};

use super::{data::PipelineData, step::PipelineStep};

pub struct VectorUpsertStep;

fn broadcast_step(job_id: &Option<String>, step: &str, status: &str) {
    if let Some(id) = job_id {
        let _ = WS_HUB.broadcast(
            json!({"event":"ingest_step","job_id":id,"step":step,"status":status}).to_string(),
        );
    }
}

fn stable_chunk_id(path: &str, idx: usize) -> i64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    idx.hash(&mut hasher);
    (hasher.finish() & 0x7FFF_FFFF_FFFF_FFFF) as i64
}

#[async_trait]
impl PipelineStep for VectorUpsertStep {
    async fn process(&self, mut data: PipelineData) -> MnemoResult<PipelineData> {
        let job_id = data.job_id.clone();
        broadcast_step(&job_id, "vector_upsert", "running");

        let qdrant_url =
            env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
        let engine = VectorEngine::new(QdrantVectorStore { url: qdrant_url });

        // Ensure collection exists before writing points.
        if let Err(e) = engine.store.init_schema().await {
            tracing::warn!("Qdrant init_schema failed: {}", e);
        } else {
            tracing::info!("Qdrant collection mnemo_chunks is ready");
        }

        let mut upserted = 0usize;
        for chunk in &mut data.chunks {
            let vector = chunk.embedding.clone().unwrap_or_else(|| vec![0.1_f32; 1536]);
            let chunk_id = stable_chunk_id(&chunk.document_path, chunk.chunk_index);
            let tags = chunk.tags.clone();

            let res = engine
                .upsert_chunk(
                    chunk_id,
                    &chunk.document_path,
                    &chunk.text,
                    vector,
                    &chunk.sparse_indices,
                    &chunk.sparse_values,
                    &chunk.namespace,
                    &tags,
                    chunk.chunk_index,
                )
                .await;
            if let Err(e) = res {
                tracing::warn!(
                    "Qdrant upsert failed for {}#{}: {}",
                    chunk.document_path,
                    chunk.chunk_index,
                    e
                );
                let _ = WS_HUB.broadcast(
                    json!({"event":"log","message":format!("Qdrant upsert failed {}#{}: {}", chunk.document_path, chunk.chunk_index, e),"job_id": job_id})
                        .to_string(),
                );
                continue;
            } else {
                tracing::info!(
                    "Qdrant upserted chunk {}#{}",
                    chunk.document_path,
                    chunk.chunk_index
                );
                let _ = WS_HUB.broadcast(
                    json!({"event":"log","message":format!("Qdrant upserted {}#{}", chunk.document_path, chunk.chunk_index),"job_id": job_id})
                        .to_string(),
                );
                data.metrics.qdrant_writes += 1;
                upserted += 1;
            }
        }

        if upserted == 0 {
            broadcast_step(&job_id, "vector_upsert", "failed");
            return Err(mnemo_core::error::MnemoError::Message(
                "vector upsert produced no writes".into(),
            ));
        }

        broadcast_step(&job_id, "vector_upsert", "done");
        Ok(data)
    }
}
