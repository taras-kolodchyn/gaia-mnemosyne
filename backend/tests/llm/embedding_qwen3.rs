use mnemo_inference::TensorZeroEmbedder;

#[tokio::test]
async fn test_qwen3_embedding() {
    let embedder = TensorZeroEmbedder::new(
        std::env::var("TENSORZERO_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
        vec![std::env::var("TENSORZERO_EMBED_MODEL").unwrap_or_else(|_| "qwen3_embedding".into())],
    );

    let vec = embedder.embed("hello world").await.expect("embedding failed");
    assert!(
        vec.len() > 100,
        "embedding length too small: {}",
        vec.len()
    );
}
