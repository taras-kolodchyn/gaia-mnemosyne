use crate::config::MnemoConfig;

/// Central orchestrator placeholder for Gaia Mnemosyne.
pub struct MnemosyneEngine {
    pub config: MnemoConfig,
    pub qdrant: Option<String>,
    pub surreal: Option<String>,
    pub redis: Option<String>,
}

impl MnemosyneEngine {
    pub fn new() -> Self {
        // Placeholder clients; wire real SDKs later.
        Self {
            config: MnemoConfig::load(),
            qdrant: Some("qdrant_client_placeholder".into()),
            surreal: Some("surreal_client_placeholder".into()),
            redis: Some("redis_client_placeholder".into()),
        }
    }

    pub fn print_mode(&self) -> String {
        self.config.mode.clone()
    }

    pub async fn query(&self, text: &str) -> String {
        let pipeline = crate::rag::api::RAGPipeline::new();
        let ctx = pipeline.query(text).await;
        tracing::info!(
            vector_hits = ctx.project_chunks.len(),
            ontology_tags = ctx.ontology_tags.len(),
            selected_project_chunks = ?ctx.project_chunks,
            "RAG query completed"
        );
        format!("rag_pipeline_executed_for: {}", text)
    }
}
