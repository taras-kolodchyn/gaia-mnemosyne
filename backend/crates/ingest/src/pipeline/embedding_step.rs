use async_trait::async_trait;
use futures::future::join_all;
use mnemo_core::error::MnemoResult;
use mnemo_core::rag::keyword::sparse_vector;
use mnemo_core::ws::WS_HUB;
use mnemo_inference::model_router::select_model;
use serde_json::json;
use tokio::task;

use super::{data::PipelineData, step::PipelineStep};

pub struct EmbeddingStep;

fn broadcast_step(job_id: &Option<String>, step: &str, status: &str) {
    if let Some(id) = job_id {
        let _ = WS_HUB.broadcast(
            json!({"event":"ingest_step","job_id":id,"step":step,"status":status}).to_string(),
        );
    }
}

#[async_trait]
impl PipelineStep for EmbeddingStep {
    async fn process(&self, mut data: PipelineData) -> MnemoResult<PipelineData> {
        let job_id = data.job_id.clone();
        broadcast_step(&job_id, "embeddings", "running");

        if data.chunks.is_empty() {
            broadcast_step(&job_id, "embeddings", "failed");
            return Err(mnemo_core::error::MnemoError::Message(
                "no chunks available for embedding".into(),
            ));
        }

        // Prepare lookup for document metadata
        let mut doc_lookup = std::collections::HashMap::new();
        for doc in &data.documents {
            doc_lookup.insert(
                doc.path.clone(),
                (
                    doc.file_type.clone(),
                    doc.namespace.clone(),
                    doc.file_size.unwrap_or(0),
                    doc.language.clone(),
                ),
            );
        }

        // Group chunks by selected model to minimize engine switching.
        let mut by_model: std::collections::HashMap<String, Vec<(usize, String)>> =
            std::collections::HashMap::new();
        for (idx, chunk) in data.chunks.iter().enumerate() {
            let (file_type, namespace, size, language) = doc_lookup
                .get(&chunk.document_path)
                .cloned()
                .unwrap_or((None, "default".to_string(), 0, None));
            let model = select_model(
                file_type.as_deref(),
                Some(namespace.as_str()),
                language.as_deref(),
                Some(size),
            );
            by_model.entry(model).or_default().push((idx, chunk.text.clone()));
        }

        let mut embeddings: Vec<Vec<f32>> = vec![Vec::new(); data.chunks.len()];

        let mut tasks = Vec::new();
        for (_model, entries) in by_model {
            let indices: Vec<usize> = entries.iter().map(|(i, _)| *i).collect();
            let texts: Vec<String> = entries.into_iter().map(|(_, t)| t).collect();
            tasks.push(task::spawn(async move {
                // Temporary: fake embeddings until real TensorZero is wired.
                let fake: Vec<Vec<f32>> = texts.iter().map(|_| vec![0.1_f32; 1536]).collect();
                (indices, fake)
            }));
        }

        for res in join_all(tasks).await {
            match res {
                Ok((indices, vecs)) => {
                    for (i, emb) in indices.into_iter().zip(vecs.into_iter()) {
                        if i < embeddings.len() {
                            embeddings[i] = emb;
                        }
                    }
                }
                Err(e) => {
                    broadcast_step(&job_id, "embeddings", "failed");
                    return Err(mnemo_core::error::MnemoError::Message(e.to_string()));
                }
            }
        }

        for (idx, emb) in embeddings.into_iter().enumerate() {
            if let Some(chunk) = data.chunks.get_mut(idx) {
                chunk.embedding = Some(emb);
                let (indices, values) = sparse_vector(&chunk.text);
                chunk.sparse_indices = indices;
                chunk.sparse_values = values;
            }
        }
        data.metrics.embedding_calls += data.chunks.len();
        tracing::info!("Embeddings generated for {} chunks", data.chunks.len());
        let _ = WS_HUB.broadcast(
            json!({"event":"log","message":format!("Embeddings generated for {} chunks", data.chunks.len()), "job_id": job_id})
                .to_string(),
        );
        broadcast_step(&job_id, "embeddings", "done");
        Ok(data)
    }
}
