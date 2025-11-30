use std::sync::Arc;

use super::step::PipelineStep;

pub struct PipelineRegistry {
    pub steps: Vec<Arc<dyn PipelineStep + Send + Sync>>,
}

impl PipelineRegistry {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, step: Arc<dyn PipelineStep + Send + Sync>) {
        self.steps.push(step);
    }
}
