use super::assembler::ContextAssembler;
use super::orchestrator::RAGOrchestrator;
use tracing::info_span;

pub struct RAGPipeline;

impl RAGPipeline {
    pub fn new() -> Self {
        Self
    }

    pub async fn query(&self, text: &str) -> crate::models::rag_context::RAGContext {
        let span = info_span!("rag_query", query = %text);
        let _guard = span.enter();

        let orchestrator = RAGOrchestrator::new();
        let _raw = orchestrator.run(text).await;

        let assembler = ContextAssembler::new();
        assembler.assemble().await
    }
}
