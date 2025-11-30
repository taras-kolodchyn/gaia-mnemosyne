use async_trait::async_trait;

#[async_trait]
pub trait InferenceEngine {
    async fn embed(&self, texts: Vec<String>) -> Vec<Vec<f32>>;
    async fn infer(&self, prompt: String) -> String;
    async fn classify(&self, text: String, labels: Vec<String>) -> String;
}
