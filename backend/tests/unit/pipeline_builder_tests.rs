use mnemo_test_utils::test_pipeline_builder::TestPipelineBuilder;

#[test]
fn simple_pipeline_builds() {
    let pipeline = TestPipelineBuilder::simple_pipeline();
    assert_eq!(pipeline.steps.len(), 0);
}
