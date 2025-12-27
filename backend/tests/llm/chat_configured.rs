use mnemo_inference::engines::tensorzero::{TensorZeroConfig, TensorZeroEngine};
use mnemo_inference::traits::InferenceEngine;

#[tokio::test]
async fn test_chat_configured() {
    let model_name = std::env::var("MNEMO_LLM_MODEL")
        .or_else(|_| std::env::var("TENSORZERO_MODEL"))
        .ok()
        .filter(|m| !m.trim().is_empty());
    let model_name = match model_name {
        Some(model) => model,
        None => {
            eprintln!("Skipping chat test; set MNEMO_LLM_MODEL or TENSORZERO_MODEL.");
            return;
        }
    };

    let base_url = std::env::var("MNEMO_LLM_URL")
        .or_else(|_| std::env::var("TENSORZERO_URL"))
        .unwrap_or_else(|_| "http://localhost:3000".into());
    let api_key = std::env::var("MNEMO_LLM_API_KEY").unwrap_or_else(|_| "none".into());

    let cfg = TensorZeroConfig {
        base_url,
        model_name,
        api_key,
        timeout_ms: 30_000,
    };

    let engine = TensorZeroEngine::new(cfg).expect("failed to init tensorzero engine");
    let reply = engine.infer("hello from test".into()).await;

    assert!(
        !reply.trim().is_empty() && reply != "tensorzero-error",
        "unexpected reply: {reply}"
    );
}
