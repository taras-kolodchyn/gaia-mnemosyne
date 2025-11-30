/// Represents a chunk of text tied to a source document.
#[derive(Clone)]
pub struct Chunk {
    pub document_path: String,
    pub text: String,
    pub tags: Vec<String>,
    pub embedding: Option<Vec<f32>>,
    pub chunk_index: usize,
    pub vector_id: Option<String>,
    pub namespace: String,
    pub sparse_indices: Vec<u32>,
    pub sparse_values: Vec<f32>,
}
