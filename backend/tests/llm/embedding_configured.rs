use mnemo_inference::TensorZeroEmbedder;

#[tokio::test]
async fn test_embedding_configured() {
    let model_alias = std::env::var("TENSORZERO_EMBED_MODEL")
        .or_else(|_| std::env::var("MNEMO_EMBED_MODEL"))
        .ok()
        .filter(|m| !m.trim().is_empty())
        .or_else(|| {
            std::env::var("TENSORZERO_EMBED_MODELS")
                .or_else(|_| std::env::var("MNEMO_EMBED_MODELS"))
                .ok()
                .and_then(|list| {
                    list.split(',')
                        .map(|s| s.trim())
                        .find(|s| !s.is_empty())
                        .map(|s| s.to_string())
                })
        });
    if model_alias.is_none() {
        eprintln!(
            "Skipping embedding test; set TENSORZERO_EMBED_MODEL or TENSORZERO_EMBED_MODELS."
        );
        return;
    }

    if std::env::var("TENSORZERO_URL").is_err() && std::env::var("MNEMO_LLM_URL").is_err() {
        unsafe {
            std::env::set_var("TENSORZERO_URL", "http://localhost:3000");
        }
    }

    let fallback_url = std::env::var("TENSORZERO_EMBED_FALLBACK_URL")
        .or_else(|_| std::env::var("MNEMO_EMBED_FALLBACK_URL"))
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    unsafe {
        std::env::set_var("TENSORZERO_EMBED_FALLBACK_URL", fallback_url);
    }

    let fallback_models = std::env::var("TENSORZERO_EMBED_FALLBACK_MODELS")
        .or_else(|_| std::env::var("MNEMO_EMBED_FALLBACK_MODELS"))
        .ok();
    if let Some(models) = fallback_models {
        unsafe {
            std::env::set_var("TENSORZERO_EMBED_FALLBACK_MODELS", models);
        }
    } else {
        eprintln!(
            "Skipping embedding test; set TENSORZERO_EMBED_FALLBACK_MODELS or MNEMO_EMBED_FALLBACK_MODELS to run it."
        );
        return;
    }

    let embedder = TensorZeroEmbedder::from_env().expect("failed to init embedder");

    let vec = embedder.embed("hello world").await.expect("embedding failed");
    assert!(
        vec.len() > 100,
        "embedding length too small: {}",
        vec.len()
    );
}
