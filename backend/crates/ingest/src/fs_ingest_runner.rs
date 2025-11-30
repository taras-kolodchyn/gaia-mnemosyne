use std::sync::Arc;
use std::time::Instant;

use crate::metrics::store_last_metrics;
use crate::pipeline::graph_builder_step::GraphBuilderStep;
use crate::pipeline::{
    chunk_step::ChunkStep, embedding_step::EmbeddingStep, ontology_step::OntologyStep,
    vector_upsert_step::VectorUpsertStep,
};
use crate::pipeline::{executor::PipelineExecutor, registry::PipelineRegistry};
use crate::providers::filesystem::FilesystemProvider;
use mnemo_core::error::MnemoResult;
use mnemo_core::ws::WS_HUB;
use serde_json::json;
use std::env;

/// Runs a filesystem-only ingestion pipeline.
pub async fn run_filesystem_ingestion(job_id: Option<&str>) -> MnemoResult<()> {
    println!("Starting filesystem ingestion...");
    let _ = WS_HUB.broadcast(
        json!({"event":"log","message":"Starting filesystem ingestion...","job_id":job_id})
            .to_string(),
    );
    let _ = WS_HUB.broadcast(
        json!({"event":"ingest_step","job_id":job_id,"step":"start","status":"running"})
            .to_string(),
    );
    let started = Instant::now();

    let root_path = std::env::var("INGESTION_ROOT").unwrap_or_else(|_| "/app/data".to_string());
    let provider = FilesystemProvider::new(vec![root_path.clone()]);
    tracing::info!("Filesystem runner scanning root: {}", root_path);
    let docs = provider.scan();
    let _ = WS_HUB.broadcast(
        json!({"event":"log","message":format!("Found {} documents", docs.len()),"job_id":job_id})
            .to_string(),
    );
    if docs.is_empty() {
        let msg = format!("No documents found in {}", root_path);
        tracing::error!("{}", msg);
        let _ = WS_HUB
            .broadcast(json!({"event":"ingest_error","job_id":job_id,"message":msg}).to_string());
        let _ = WS_HUB.broadcast(
            json!({"event":"ingest_step","job_id":job_id,"step":"completed","status":"failed"})
                .to_string(),
        );
        return Err(mnemo_core::error::MnemoError::Message("no documents found".into()));
    }

    let pg_url = env::var("MNEMO_METADATA_PG")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/mnemo".into());
    let meta_store = mnemo_storage::metadata::postgres::PostgresMetadataStore::new(pg_url)
        .map_err(|e| mnemo_core::error::MnemoError::Message(e.to_string()))?;

    let mut registry = PipelineRegistry::new();
    registry.add_step(Arc::new(super::pipeline::fingerprint_step::FingerprintStep::new(
        meta_store.clone(),
    )));
    registry.add_step(Arc::new(ChunkStep));
    registry.add_step(Arc::new(OntologyStep));
    registry.add_step(Arc::new(EmbeddingStep));
    registry.add_step(Arc::new(VectorUpsertStep));
    registry.add_step(Arc::new(GraphBuilderStep::new(
        std::env::var("SURREALDB_URL").unwrap_or_else(|_| "http://localhost:8000".into()),
        meta_store.clone(),
    )));

    let executor = PipelineExecutor::new(registry);

    let mut data = crate::pipeline::data::PipelineData::new();
    data.documents = docs;
    data.job_id = job_id.map(|s| s.to_string());
    data.metrics.documents_processed = data.documents.len();

    // Mark start step as completed before entering the pipeline sequence.
    let _ = WS_HUB.broadcast(
        json!({"event":"ingest_step","job_id":data.job_id,"step":"start","status":"done"})
            .to_string(),
    );

    let result = executor.execute_with_data(data).await;

    match result {
        Ok(mut output) => {
            let elapsed = started.elapsed().as_millis() as u64;
            output.metrics.duration_ms = elapsed;
            store_last_metrics(output.metrics.clone());
            println!("Filesystem ingestion completed");
            let _ = WS_HUB.broadcast(
                json!({"event":"log","message":"Filesystem ingestion completed","job_id":job_id})
                    .to_string(),
            );
            let _ = WS_HUB.broadcast(
                json!({"event":"ingest_step","job_id":job_id,"step":"completed","status":"done"})
                    .to_string(),
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("Filesystem ingestion failed: {}", e);
            let _ = WS_HUB.broadcast(
                json!({"event":"log","message":format!("Filesystem ingestion failed: {}", e),"job_id":job_id})
                    .to_string(),
            );
            let _ = WS_HUB.broadcast(
                json!({"event":"ingest_step","job_id":job_id,"step":"completed","status":"failed"})
                    .to_string(),
            );
            Err(e)
        }
    }
}
