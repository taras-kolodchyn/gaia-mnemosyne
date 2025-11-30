/// Simple keyword search abstraction for fallback retrieval.
pub trait KeywordSearch {
    fn search(&self, query: &str) -> Vec<String>;
}
