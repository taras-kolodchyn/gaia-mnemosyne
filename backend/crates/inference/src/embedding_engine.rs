use super::traits::InferenceEngine;

pub struct EmbeddingEngine<E: InferenceEngine> {
    pub engine: E,
}

impl<E: InferenceEngine> EmbeddingEngine<E> {
    pub fn new(engine: E) -> Self {
        Self { engine }
    }

    fn batch_size_from_env() -> usize {
        std::env::var("MNEMO_EMBED_BATCH_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|v: &usize| *v > 0)
            .unwrap_or(16)
    }

    pub async fn embed(&self, texts: Vec<String>) -> Vec<Vec<f32>> {
        let raw = self.engine.embed(texts).await;
        raw.into_iter().map(normalize_vec).collect()
    }

    /// Batch embeddings to reduce per-request overhead.
    pub async fn embed_batch(&self, texts: Vec<String>, batch_size: usize) -> Vec<Vec<f32>> {
        let mut all = Vec::new();
        let mut start = 0;
        let bs = batch_size.max(1);
        while start < texts.len() {
            let end = (start + bs).min(texts.len());
            let batch = texts[start..end].to_vec();
            let mut vecs: Vec<Vec<f32>> =
                self.engine.embed(batch).await.into_iter().map(normalize_vec).collect();
            all.append(&mut vecs);
            start = end;
        }
        all
    }

    /// Batch embeddings using configured env size (MNEMO_EMBED_BATCH_SIZE, default 16).
    pub async fn embed_batch_env(&self, texts: Vec<String>) -> Vec<Vec<f32>> {
        let bs = Self::batch_size_from_env();
        self.embed_batch(texts, bs).await
    }
}

fn normalize_vec(mut v: Vec<f32>) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| (*x as f64).powi(2) as f32).sum::<f32>().sqrt();
    if norm.is_finite() && norm > 0.0 {
        for x in v.iter_mut() {
            *x = (*x / norm).clamp(-1.0, 1.0);
            if !x.is_finite() {
                *x = 0.0;
            }
        }
    } else {
        for x in v.iter_mut() {
            if !x.is_finite() {
                *x = 0.0;
            }
        }
    }
    v
}
