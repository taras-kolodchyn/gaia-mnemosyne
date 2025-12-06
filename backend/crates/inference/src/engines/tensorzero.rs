use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

use crate::traits::InferenceEngine;

#[derive(Debug, Error)]
pub enum TensorZeroError {
    #[error("TensorZero request failed: {0}")]
    Http(String),
    #[error("TensorZero returned non-success status={status} body={body}")]
    Status { status: reqwest::StatusCode, body: String },
    #[error("TensorZero response missing text content")]
    NoTextContent,
    #[error("Config error: {0}")]
    Config(String),
}

/// Minimal config for TensorZero /inference calls.
#[derive(Clone, Debug)]
pub struct TensorZeroConfig {
    pub base_url: String,
    pub model_name: String,
    pub api_key: String,
    pub timeout_ms: u64,
}

impl TensorZeroConfig {
    pub fn from_env() -> Result<Self, TensorZeroError> {
        let base_url = std::env::var("MNEMO_LLM_URL")
            .or_else(|_| std::env::var("TENSORZERO_URL"))
            .unwrap_or_else(|_| "http://tensorzero:3000".to_string());
        let model_name = std::env::var("MNEMO_LLM_MODEL")
            .or_else(|_| std::env::var("TENSORZERO_MODEL"))
            .unwrap_or_else(|_| "qwen3_8b".to_string());
        let api_key =
            std::env::var("MNEMO_LLM_API_KEY").unwrap_or_else(|_| "none".to_string());
        let timeout_ms = std::env::var("TENSORZERO_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30_000);

        Ok(Self {
            base_url,
            model_name,
            api_key,
            timeout_ms,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InferenceRequest {
    model_name: String,
    input: InferenceInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InferenceInput {
    messages: Vec<InferenceMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InferenceMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InferenceResponse {
    content: Vec<InferenceContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InferenceContent {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

/// TensorZero-backed inference engine. Maps the generic inference trait to
/// TensorZero's `/inference` API (chat-style).
pub struct TensorZeroEngine {
    client: Client,
    cfg: TensorZeroConfig,
}

impl TensorZeroEngine {
    pub fn new(cfg: TensorZeroConfig) -> Result<Self, TensorZeroError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(cfg.timeout_ms))
            .build()
            .map_err(|e| TensorZeroError::Http(e.to_string()))?;
        Ok(Self { client, cfg })
    }

    async fn chat(&self, prompt: String) -> Result<String, TensorZeroError> {
        let url = format!("{}/inference", self.cfg.base_url.trim_end_matches('/'));

        let req_body = InferenceRequest {
            model_name: self.cfg.model_name.clone(),
            input: InferenceInput {
                messages: vec![InferenceMessage {
                    role: "user".into(),
                    content: prompt,
                }],
            },
        };

        info!(
            target: "mnemo_llm",
            "TensorZero infer model={} url={}",
            self.cfg.model_name,
            url
        );

        let mut request = self.client.post(&url).json(&req_body);
        if self.cfg.api_key.to_lowercase() != "none" && !self.cfg.api_key.is_empty() {
            request = request.bearer_auth(self.cfg.api_key.clone());
        }

        let resp = request
            .send()
            .await
            .map_err(|e| TensorZeroError::Http(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TensorZeroError::Status { status, body });
        }

        let parsed: InferenceResponse = resp
            .json()
            .await
            .map_err(|e| TensorZeroError::Http(e.to_string()))?;

        parsed
            .content
            .iter()
            .find(|c| c.kind == "text")
            .map(|c| c.text.clone())
            .ok_or(TensorZeroError::NoTextContent)
    }
}

#[async_trait]
impl InferenceEngine for TensorZeroEngine {
    async fn embed(&self, texts: Vec<String>) -> Vec<Vec<f32>> {
        // TensorZero Gateway doesn't expose embeddings yet; return zeros but log once.
        warn!("TensorZero embed is not implemented; returning zero vectors");
        texts.into_iter().map(|_| vec![0.0_f32; 1536]).collect()
    }

    async fn infer(&self, prompt: String) -> String {
        self.chat(prompt).await.unwrap_or_else(|e| {
            warn!("TensorZero infer failed: {}", e);
            "tensorzero-error".into()
        })
    }

    async fn classify(&self, text: String, labels: Vec<String>) -> String {
        let prompt = if labels.is_empty() {
            text
        } else {
            format!(
                "Classify the following text into one of these labels {:?}: {}",
                labels, text
            )
        };
        self.chat(prompt).await.unwrap_or_else(|e| {
            warn!("TensorZero classify failed: {}", e);
            "tensorzero-error".into()
        })
    }
}
