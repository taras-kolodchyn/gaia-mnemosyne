use async_trait::async_trait;
use mnemo_core::error::MnemoResult;
use mnemo_core::models::Job;

#[async_trait]
pub trait JobQueue {
    async fn push(&self, job: Job) -> MnemoResult<()>;
    async fn pop(&self) -> MnemoResult<Option<Job>>;
}
