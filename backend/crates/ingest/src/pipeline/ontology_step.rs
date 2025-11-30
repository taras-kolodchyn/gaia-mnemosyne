use async_trait::async_trait;
use futures::future::join_all;
use mnemo_core::error::MnemoResult;
use mnemo_core::traits::ontology_engine::OntologyEngine;
use mnemo_core::ws::WS_HUB;
use serde_json::json;
use tokio::task;

use super::{data::PipelineData, step::PipelineStep};

#[derive(Clone)]
struct SimpleOntologyEngine;

impl SimpleOntologyEngine {
    fn new() -> Self {
        Self
    }
}

impl OntologyEngine for SimpleOntologyEngine {
    fn classify(&self, text: &str) -> Vec<String> {
        let lower = text.to_lowercase();
        let mut tags = Vec::new();
        if lower.contains("project") {
            tags.push("project".into());
        }
        if lower.contains("domain") {
            tags.push("domain".into());
        }
        if lower.contains("company") {
            tags.push("company".into());
        }
        if tags.is_empty() {
            tags.push("misc".into());
        }
        tags
    }
}

pub struct OntologyStep;

fn broadcast_step(job_id: &Option<String>, status: &str) {
    if let Some(id) = job_id {
        let _ = WS_HUB.broadcast(
            json!({"event":"ingest_step","job_id":id,"step":"ontology","status":status})
                .to_string(),
        );
    }
}

#[async_trait]
impl PipelineStep for OntologyStep {
    async fn process(&self, mut data: PipelineData) -> MnemoResult<PipelineData> {
        broadcast_step(&data.job_id, "running");
        let engine = SimpleOntologyEngine::new();
        let handles = data.chunks.iter().enumerate().map(|(idx, chunk)| {
            let text = chunk.text.clone();
            let eng = engine.clone();
            task::spawn(async move { (idx, eng.classify(&text)) })
        });

        for res in join_all(handles).await {
            if let Ok((idx, tags)) = res {
                let mut tags = tags;
                if tags.is_empty() {
                    tags.push("misc".into());
                }
                if let Some(chunk) = data.chunks.get_mut(idx) {
                    chunk.tags = tags.clone();
                }
                data.metadata.insert(format!("ontology_tags_{}", idx), tags.join(","));
            }
        }
        let _ = WS_HUB.broadcast(
            json!({"event":"log","message":"Ontology applied to chunks","job_id": data.job_id})
                .to_string(),
        );
        broadcast_step(&data.job_id, "done");
        Ok(data)
    }
}
