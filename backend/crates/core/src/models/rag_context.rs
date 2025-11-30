use crate::rag::orchestrator::DebugCandidate;
use serde::{Deserialize, Serialize};

/// Aggregated context pulled from various retrieval sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGContext {
    pub project_chunks: Vec<String>,
    pub domain_chunks: Vec<String>,
    pub company_chunks: Vec<String>,
    pub graph_neighbors: Vec<String>,
    pub ontology_tags: Vec<String>,
    #[serde(default)]
    pub debug_candidates: Vec<DebugCandidate>,
}
