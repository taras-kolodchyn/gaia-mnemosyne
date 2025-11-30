use mnemo_core::error::MnemoResult;

use super::{data::PipelineData, registry::PipelineRegistry};

/// Executes a pipeline registry from start to finish.
pub struct PipelineExecutor {
    pub registry: PipelineRegistry,
}

impl PipelineExecutor {
    pub fn new(registry: PipelineRegistry) -> Self {
        Self { registry }
    }

    pub async fn execute(&self) -> MnemoResult<PipelineData> {
        let mut data = PipelineData::new();
        for step in &self.registry.steps {
            data = step.process(data).await?;
        }
        Ok(data)
    }

    pub async fn execute_with_data(&self, mut data: PipelineData) -> MnemoResult<PipelineData> {
        for step in &self.registry.steps {
            data = step.process(data).await?;
        }
        Ok(data)
    }
}
