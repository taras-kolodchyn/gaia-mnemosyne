use super::pipeline::{data::PipelineData, registry::PipelineRegistry};
use crate::jobs::IngestJob;

pub struct IngestionRunner {
    pub pipelines: PipelineRegistry,
}

impl IngestionRunner {
    pub fn new(pipelines: PipelineRegistry) -> Self {
        Self { pipelines }
    }

    pub async fn run(&self, _job: IngestJob) {
        let data = PipelineData::new();
        let mut current = data;

        for step in &self.pipelines.steps {
            current = step.process(current).await.unwrap();
        }
    }
}
