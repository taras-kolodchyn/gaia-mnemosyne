use async_trait::async_trait;
use futures::future::join_all;
use mnemo_core::error::MnemoResult;
use mnemo_core::models::chunk::Chunk;
use mnemo_core::ws::WS_HUB;
use serde_json::json;
use tokio::task;

use super::{chunk_builder::ChunkBuilder, data::PipelineData, step::PipelineStep};

pub struct ChunkStep;

fn broadcast_step(job_id: &Option<String>, step: &str, status: &str) {
    if let Some(id) = job_id {
        let _ = WS_HUB.broadcast(
            json!({"event":"ingest_step","job_id":id,"step":step,"status":status}).to_string(),
        );
    }
}

#[async_trait]
impl PipelineStep for ChunkStep {
    async fn process(&self, mut data: PipelineData) -> MnemoResult<PipelineData> {
        let job_id = data.job_id.clone();
        broadcast_step(&job_id, "chunking", "running");
        // Build chunks per document in parallel; pure CPU work so safe to spawn tasks.
        let docs = data.documents.clone();
        tracing::info!("Chunking {} documents", docs.len());
        let _ = WS_HUB.broadcast(
            json!({"event":"log","message":format!("Chunking {} documents", docs.len()), "job_id": job_id})
                .to_string(),
        );
        let handles = docs.into_iter().map(|doc| {
            let ftype = doc
                .language
                .as_deref()
                .map(|lang| ChunkBuilder::detect_language(lang))
                .unwrap_or_else(|| ChunkBuilder::detect(&doc.path));
            task::spawn(async move {
                ChunkBuilder::build(&doc.content, ftype)
                    .into_iter()
                    .enumerate()
                    .map(|(idx, chunk_text)| Chunk {
                        document_path: doc.path.clone(),
                        text: chunk_text,
                        tags: Vec::new(),
                        embedding: None,
                        chunk_index: idx,
                        vector_id: None,
                        namespace: doc.namespace.clone(),
                        sparse_indices: Vec::new(),
                        sparse_values: Vec::new(),
                    })
                    .collect::<Vec<_>>()
            })
        });

        let mut out = Vec::new();
        for res in join_all(handles).await {
            match res {
                Ok(mut chunks) => out.append(&mut chunks),
                Err(e) => {
                    broadcast_step(&job_id, "chunking", "failed");
                    return Err(mnemo_core::error::MnemoError::Message(e.to_string()));
                }
            }
        }

        tracing::info!("Created {} chunks", out.len());
        let _ = WS_HUB.broadcast(
            json!({"event":"log","message":format!("Created {} chunks", out.len()), "job_id": job_id})
                .to_string(),
        );
        data.metrics.chunks_produced = out.len();
        data.chunks = out;
        broadcast_step(&job_id, "chunking", "done");
        if data.chunks.is_empty() {
            tracing::error!("Chunking produced 0 chunks");
            broadcast_step(&job_id, "chunking", "failed");
            let _ = WS_HUB.broadcast(
                json!({"event":"log","message":"Chunking produced 0 chunks","job_id": job_id})
                    .to_string(),
            );
            return Err(mnemo_core::error::MnemoError::Message(
                "chunking produced no chunks".into(),
            ));
        } else {
            let _ = WS_HUB.broadcast(
                json!({"event":"log","message":format!("Chunking produced {} chunks", data.chunks.len()),"job_id": job_id})
                    .to_string(),
            );
        }
        Ok(data)
    }
}
