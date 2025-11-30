use async_trait::async_trait;
use mnemo_core::error::MnemoResult;

use super::data::PipelineData;

#[async_trait]
pub trait PipelineStep {
    async fn process(&self, data: PipelineData) -> MnemoResult<PipelineData>;
}
