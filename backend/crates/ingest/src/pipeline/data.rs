use std::collections::HashMap;

use crate::metrics::IngestionMetrics;
pub use mnemo_core::models::{chunk::Chunk, document::Document};

/// Data passed between pipeline steps.
#[derive(Clone)]
pub struct PipelineData {
    pub documents: Vec<Document>,
    pub chunks: Vec<Chunk>,
    pub metadata: HashMap<String, String>,
    pub job_id: Option<String>,
    pub metrics: IngestionMetrics,
}

impl PipelineData {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            chunks: Vec::new(),
            metadata: HashMap::new(),
            job_id: None,
            metrics: IngestionMetrics::new(),
        }
    }
}
