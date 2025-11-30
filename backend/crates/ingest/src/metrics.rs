/// Simple ingestion metrics collected during a pipeline run.
#[derive(Default, Clone, Debug)]
pub struct IngestionMetrics {
    pub documents_processed: usize,
    pub chunks_produced: usize,
    pub embedding_calls: usize,
    pub qdrant_writes: usize,
    pub duration_ms: u64,
}

impl IngestionMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

use once_cell::sync::Lazy;
use std::sync::Mutex;

static LAST_METRICS: Lazy<Mutex<IngestionMetrics>> =
    Lazy::new(|| Mutex::new(IngestionMetrics::default()));

/// Store metrics from the most recent run.
pub fn store_last_metrics(metrics: IngestionMetrics) {
    if let Ok(mut guard) = LAST_METRICS.lock() {
        *guard = metrics;
    }
}

/// Snapshot of the last recorded metrics.
pub fn snapshot_last_metrics() -> IngestionMetrics {
    LAST_METRICS.lock().map(|m| m.clone()).unwrap_or_default()
}
