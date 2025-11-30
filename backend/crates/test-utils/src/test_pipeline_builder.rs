use mnemo_ingest::pipeline::registry::PipelineRegistry;

/// Helpers for constructing test pipeline registries.
pub struct TestPipelineBuilder;

impl TestPipelineBuilder {
    pub fn simple_pipeline() -> PipelineRegistry {
        PipelineRegistry { steps: vec![] }
    }
}
