use async_trait::async_trait;
use mnemo_inference::traits::InferenceEngine;

/// Test helper implementing the InferenceEngine trait with fixed outputs.
pub struct FakeInferenceEngine;

#[async_trait]
impl InferenceEngine for FakeInferenceEngine {
    async fn embed(&self, texts: Vec<String>) -> Vec<Vec<f32>> {
        vec![vec![1.0, 2.0, 3.0]; texts.len()]
    }

    async fn infer(&self, _prompt: String) -> String {
        "fake_output".into()
    }

    async fn classify(&self, text: String, _labels: Vec<String>) -> String {
        format!("fake_class_for_{}", text)
    }
}
