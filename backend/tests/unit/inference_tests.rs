use mnemo_inference::traits::InferenceEngine;
use mnemo_test_utils::fake_inference_engine::FakeInferenceEngine;

#[tokio::test]
async fn fake_inference_embed_returns_fixed_vector() {
    let engine = FakeInferenceEngine;
    let vectors = engine.embed(vec!["hello".into()]).await;
    assert_eq!(vectors[0], vec![1.0, 2.0, 3.0]);
}
