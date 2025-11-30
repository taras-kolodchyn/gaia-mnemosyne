/// Trait for scoring relevance using multiple signals.
pub trait RankingEngine {
    fn score(
        &self,
        vector_score: f32,
        keyword_score: f32,
        graph_score: f32,
        knowledge_score: f32,
    ) -> f32;
}
