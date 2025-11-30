use super::vector_engine::VectorEngine;

/// Wrapper providing a unified vector search interface.
pub struct VectorSearchEngine {
    pub engine: VectorEngine,
}

impl VectorSearchEngine {
    pub fn new(engine: VectorEngine) -> Self {
        Self { engine }
    }

    pub async fn search(
        &self,
        query: Vec<f32>,
        top_k: usize,
        namespace: &str,
        tag: Option<&str>,
    ) -> mnemo_core::error::MnemoResult<Vec<String>> {
        self.engine.search(query, top_k, namespace, tag).await
    }
}
