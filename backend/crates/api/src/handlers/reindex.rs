use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ReindexResponse {
    pub scanned: usize,
    pub reindexed: usize,
    pub skipped: usize,
    pub chunks: usize,
    pub success: bool,
}

/// Trigger a lightweight reindex that only processes modified documents.
pub async fn reindex() -> Json<ReindexResponse> {
    let mut controller = mnemo_ingest::controller::IngestionController::new();

    // Load all sources, but pipeline will drop unchanged docs via fingerprint step.
    let docs = controller.load_sources();
    let scanned = docs.len();

    let result = controller.run_pipeline(docs).await;
    let reindexed = controller.metrics.documents_processed;
    let chunks = controller.metrics.chunks_produced;
    let skipped = scanned.saturating_sub(reindexed);

    Json(ReindexResponse { scanned, reindexed, skipped, chunks, success: result.is_ok() })
}
