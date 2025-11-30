use mnemo_inference::traits::InferenceEngine;
use mnemo_test_utils::{fake_inference_engine::FakeInferenceEngine, fixtures::load_fixture};

#[tokio::test]
async fn fixture_and_fake_engine_can_work_together() {
    let content = load_fixture("sample.txt");
    let engine = FakeInferenceEngine;
    let vectors = engine.embed(vec![content]).await;
    assert_eq!(vectors[0], vec![1.0, 2.0, 3.0]);
}
