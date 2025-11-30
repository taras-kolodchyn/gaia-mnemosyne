#[derive(Debug, Clone, Copy)]
pub enum RagStrategy {
    KeywordHeavy,
    Semantic,
    Graph,
    Combined,
}

#[derive(Debug, Clone, Copy)]
pub struct StrategyWeights {
    pub dense: f32,
    pub sparse: f32,
    pub graph: f32,
    pub ontology: f32,
}

impl RagStrategy {
    pub fn weights(self) -> StrategyWeights {
        match self {
            RagStrategy::KeywordHeavy => {
                StrategyWeights { dense: 0.4, sparse: 0.4, graph: 0.1, ontology: 0.1 }
            }
            RagStrategy::Semantic => {
                StrategyWeights { dense: 0.75, sparse: 0.1, graph: 0.1, ontology: 0.05 }
            }
            RagStrategy::Graph => {
                StrategyWeights { dense: 0.4, sparse: 0.2, graph: 0.3, ontology: 0.1 }
            }
            RagStrategy::Combined => {
                StrategyWeights { dense: 0.6, sparse: 0.2, graph: 0.1, ontology: 0.1 }
            }
        }
    }
}

/// Heuristic strategy selection based on query length, keyword count, and tags.
pub fn select_strategy(
    query_tokens: usize,
    keyword_count: usize,
    inferred_tags: &[String],
) -> RagStrategy {
    if keyword_count > 6 && query_tokens <= 12 {
        RagStrategy::KeywordHeavy
    } else if query_tokens > 18 {
        RagStrategy::Semantic
    } else if inferred_tags.iter().any(|t| t.contains("graph")) {
        RagStrategy::Graph
    } else {
        RagStrategy::Combined
    }
}
