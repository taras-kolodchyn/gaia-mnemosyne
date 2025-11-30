/// Abstraction for vector search backends.
pub trait VectorSearch {
    fn search(&self, query: Vec<f32>, top_k: usize) -> Vec<String>;
}
