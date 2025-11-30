use async_trait::async_trait;

use crate::traits::InferenceEngine;

/// Placeholder engine for TensorZero integration.
pub struct TensorZeroEngine {
    pub model_path: String,
}

impl TensorZeroEngine {
    pub fn new(model_path: String) -> Self {
        Self { model_path }
    }
}

#[async_trait]
impl InferenceEngine for TensorZeroEngine {
    async fn embed(&self, texts: Vec<String>) -> Vec<Vec<f32>> {
        texts
            .into_iter()
            .map(|_| vec![0.0_f32; 1536]) // placeholder vector of correct length
            .collect()
    }

    async fn infer(&self, _prompt: String) -> String {
        "tensorzero-placeholder-output".into()
    }

    async fn classify(&self, text: String, _labels: Vec<String>) -> String {
        format!("tensorzero-classified: {}", text)
    }
}
