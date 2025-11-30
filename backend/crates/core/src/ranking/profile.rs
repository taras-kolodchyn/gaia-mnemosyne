/// Ranking weights for combining multiple signals.
pub struct RankingProfile {
    pub vector: f32,
    pub keyword: f32,
    pub graph: f32,
    pub knowledge: f32,
}

impl RankingProfile {
    /// Placeholder loader with default weights.
    pub fn load(_path: &str) -> Self {
        Self { vector: 0.55, keyword: 0.20, graph: 0.15, knowledge: 0.10 }
    }
}
