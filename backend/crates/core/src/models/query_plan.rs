/// Represents an execution plan for a RAG-style query.
pub struct QueryPlan {
    pub ranking_profile: String,
    pub knowledge_levels: Vec<String>,
    pub graph_expansion_depth: u8,
    pub top_k: usize,
}
