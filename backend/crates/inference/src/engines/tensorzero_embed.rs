use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::InferenceError;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

/// TensorZero embedder using `/inference` with an embedding model.
#[derive(Clone)]
pub struct TensorZeroEmbedder {
    client: Client,
    url: String,
    models: Vec<String>, // priority-ordered list
    api_key: Option<String>,
}

impl TensorZeroEmbedder {
    pub fn new(url: String, models: Vec<String>) -> Self {
        Self {
            client: Client::new(),
            url,
            models,
            api_key: None,
        }
    }

    pub fn from_env() -> Result<Self, InferenceError> {
        let url = std::env::var("TENSORZERO_URL")
            .or_else(|_| std::env::var("MNEMO_LLM_URL"))
            .unwrap_or_else(|_| "http://tensorzero:3000".to_string());
        let models_env = std::env::var("TENSORZERO_EMBED_MODELS")
            .or_else(|_| std::env::var("MNEMO_EMBED_MODELS"))
            .unwrap_or_else(|_| "".into());
        let mut models: Vec<String> = models_env
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .collect();
        if models.is_empty() {
            models.push(
                std::env::var("TENSORZERO_EMBED_MODEL")
                    .unwrap_or_else(|_| "qwen3_embedding".into()),
            );
        }
        let api_key = std::env::var("TENSORZERO_API_KEY")
            .or_else(|_| std::env::var("MNEMO_LLM_API_KEY"))
            .ok()
            .filter(|k| !k.is_empty() && k.to_lowercase() != "none");

        let mut embedder = Self::new(url, models);
        embedder.api_key = api_key;
        Ok(embedder)
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, InferenceError> {
        let url = format!("{}/v1/embeddings", self.url.trim_end_matches('/'));
        let mut last_err: Option<InferenceError> = None;

        for model in &self.models {
            let body = EmbeddingRequest {
                model: model.clone(),
                input: text.to_string(),
            };

            let mut req = self.client.post(&url).json(&body);
            if let Some(key) = &self.api_key {
                req = req.bearer_auth(key);
            }

            let resp = req.send().await;
            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(e.into());
                    continue;
                }
            };

            if !resp.status().is_success() {
                last_err = Some(InferenceError::Status(format!(
                    "TensorZero embed error: model={} status={}",
                    model,
                    resp.status()
                )));
                continue;
            }

            let parsed: Result<EmbeddingResponse, _> = resp.json().await;
            match parsed {
                Ok(parsed) => {
                    if let Some(first) = parsed.data.first() {
                        return Ok(first.embedding.clone());
                    }
                    last_err = Some(InferenceError::Other(
                        "embedding response had empty data".into(),
                    ));
                }
                Err(e) => {
                    last_err = Some(e.into());
                    continue;
                }
            }
        }

        Err(last_err.unwrap_or_else(|| InferenceError::Other("no models tried".into())))
    }
}
