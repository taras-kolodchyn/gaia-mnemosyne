use super::super::models::rag_context::RAGContext;

/// Placeholder context assembler combining retrieval sources.
pub struct ContextAssembler;

impl ContextAssembler {
    pub fn new() -> Self {
        Self
    }

    pub async fn assemble(&self) -> RAGContext {
        RAGContext {
            project_chunks: vec![],
            domain_chunks: vec![],
            company_chunks: vec![],
            graph_neighbors: vec![],
            ontology_tags: vec![],
            debug_candidates: vec![],
        }
    }
}
