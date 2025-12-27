use tracing::{info, warn};

#[derive(Clone, Debug)]
pub enum LlmProvider {
    TensorZero,
}

/// API-level configuration parsed from environment variables.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub llm: LlmConfig,
}

#[derive(Clone, Debug)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub url: String,
    pub api_key: String,
    pub model: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let provider_raw =
            std::env::var("MNEMO_LLM_PROVIDER").unwrap_or_else(|_| "tensorzero".into());
        let provider = match provider_raw.to_lowercase().as_str() {
            "tensorzero" => LlmProvider::TensorZero,
            _ => LlmProvider::TensorZero,
        };

        let url =
            std::env::var("MNEMO_LLM_URL").unwrap_or_else(|_| "http://tensorzero:3000".into());
        let api_key = std::env::var("MNEMO_LLM_API_KEY").unwrap_or_else(|_| "none".into());
        let model = std::env::var("MNEMO_LLM_MODEL")
            .or_else(|_| std::env::var("TENSORZERO_MODEL"))
            .unwrap_or_default();

        let llm = LlmConfig { provider, url, api_key, model };
        if llm.model.trim().is_empty() {
            warn!(
                "[LLM] TensorZero model alias is not set (MNEMO_LLM_MODEL/TENSORZERO_MODEL)"
            );
        }
        info!(
            "[LLM] Using TensorZero -> {} model={} (provider=tensorzero)",
            llm.url, llm.model
        );
        Self { llm }
    }
}
