use mnemo_inference::engines::tensorzero::{TensorZeroConfig, TensorZeroEngine};
use mnemo_inference::traits::InferenceEngine;

#[tokio::test]
async fn test_qwen3_chat() {
    let cfg = TensorZeroConfig::from_env().unwrap_or_else(|_| TensorZeroConfig {
        base_url: "http://localhost:3000".into(),
        model_name: "qwen3_8b".into(),
        api_key: "none".into(),
        timeout_ms: 30_000,
    });

    let engine = TensorZeroEngine::new(cfg).expect("failed to init tensorzero engine");
    let reply = engine.infer("hello from test".into()).await;

    assert!(
        !reply.trim().is_empty() && reply != "tensorzero-error",
        "unexpected reply: {reply}"
    );
}
