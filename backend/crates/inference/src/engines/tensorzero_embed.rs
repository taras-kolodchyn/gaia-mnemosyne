use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

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

/// Embedding client that targets OpenAI-compatible `/v1/embeddings` endpoints.
/// TensorZero is used when it exposes embeddings; otherwise we can fall back
/// to a direct Ollama-compatible endpoint.
#[derive(Clone)]
pub struct TensorZeroEmbedder {
    client: Client,
    url: String,
    models: Vec<String>, // priority-ordered list
    api_key: Option<String>,
    fallback_url: Option<String>,
    fallback_models: Vec<String>,
    fallback_api_key: Option<String>,
}

impl TensorZeroEmbedder {
    pub fn new(url: String, models: Vec<String>) -> Self {
        Self {
            client: Client::new(),
            url,
            models,
            api_key: None,
            fallback_url: None,
            fallback_models: Vec::new(),
            fallback_api_key: None,
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
            if let Ok(single) = std::env::var("TENSORZERO_EMBED_MODEL")
                .or_else(|_| std::env::var("MNEMO_EMBED_MODEL"))
            {
                let trimmed = single.trim();
                if !trimmed.is_empty() {
                    models.push(trimmed.to_string());
                }
            }
        }
        if models.is_empty() {
            return Err(InferenceError::Other(
                "embedding model alias is not configured (set TENSORZERO_EMBED_MODEL or TENSORZERO_EMBED_MODELS)"
                    .into(),
            ));
        }
        let api_key = std::env::var("TENSORZERO_API_KEY")
            .or_else(|_| std::env::var("MNEMO_LLM_API_KEY"))
            .ok()
            .filter(|k| !k.is_empty() && k.to_lowercase() != "none");

        let fallback_url = std::env::var("TENSORZERO_EMBED_FALLBACK_URL")
            .or_else(|_| std::env::var("MNEMO_EMBED_FALLBACK_URL"))
            .ok()
            .filter(|u| !u.trim().is_empty());
        let fallback_models_env = std::env::var("TENSORZERO_EMBED_FALLBACK_MODELS")
            .or_else(|_| std::env::var("MNEMO_EMBED_FALLBACK_MODELS"))
            .unwrap_or_else(|_| "".into());
        let fallback_models: Vec<String> = fallback_models_env
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
        let fallback_api_key = std::env::var("TENSORZERO_EMBED_FALLBACK_API_KEY")
            .or_else(|_| std::env::var("MNEMO_EMBED_FALLBACK_API_KEY"))
            .ok()
            .filter(|k| !k.is_empty() && k.to_lowercase() != "none");

        let mut embedder = Self::new(url, models);
        embedder.api_key = api_key;
        embedder.fallback_url = fallback_url;
        embedder.fallback_models = fallback_models;
        embedder.fallback_api_key = fallback_api_key;
        Ok(embedder)
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, InferenceError> {
        let mut last_err: Option<InferenceError> = None;

        if let Some(vec) = self
            .embed_with(
                &self.url,
                &self.models,
                self.api_key.as_ref(),
                text,
                &mut last_err,
            )
            .await
        {
            return Ok(vec);
        }

        if let Some(fallback_url) = &self.fallback_url {
            if self.fallback_models.is_empty() {
                warn!("TensorZero embed fallback URL set but no fallback models configured");
            } else if let Some(vec) = self
                .embed_with(
                    fallback_url,
                    &self.fallback_models,
                    self.fallback_api_key.as_ref(),
                    text,
                    &mut last_err,
                )
                .await
            {
                return Ok(vec);
            }
        }

        Err(last_err.unwrap_or_else(|| InferenceError::Other("no models tried".into())))
    }

    async fn embed_with(
        &self,
        base_url: &str,
        models: &[String],
        api_key: Option<&String>,
        text: &str,
        last_err: &mut Option<InferenceError>,
    ) -> Option<Vec<f32>> {
        let url = embeddings_url(base_url);

        for model in models {
            let body = EmbeddingRequest {
                model: model.clone(),
                input: text.to_string(),
            };

            let mut req = self.client.post(&url).json(&body);
            if let Some(key) = api_key {
                req = req.bearer_auth(key);
            }

            let resp = req.send().await;
            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    *last_err = Some(e.into());
                    continue;
                }
            };

            let status = resp.status();
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                *last_err = Some(InferenceError::Status(format!(
                    "embed error: model={} status={} body={}",
                    model, status, body
                )));
                continue;
            }

            let parsed: Result<EmbeddingResponse, _> = resp.json().await;
            match parsed {
                Ok(parsed) => {
                    if let Some(first) = parsed.data.first() {
                        return Some(first.embedding.clone());
                    }
                    *last_err = Some(InferenceError::Other(
                        "embedding response had empty data".into(),
                    ));
                }
                Err(e) => {
                    *last_err = Some(e.into());
                    continue;
                }
            }
        }

        None
    }
}

fn embeddings_url(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if trimmed.ends_with("/v1/embeddings") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1") {
        format!("{}/embeddings", trimmed)
    } else {
        format!("{}/v1/embeddings", trimmed)
    }
}
