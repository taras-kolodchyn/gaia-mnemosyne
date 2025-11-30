use std::sync::Arc;

use crate::metrics::IngestionMetrics;
use crate::pipeline::{
    chunk_step::ChunkStep, embedding_step::EmbeddingStep, executor::PipelineExecutor,
    graph_builder_step::GraphBuilderStep, ontology_step::OntologyStep, registry::PipelineRegistry,
    vector_upsert_step::VectorUpsertStep,
};
use crate::providers::filesystem::FilesystemProvider;
use futures::FutureExt;
use mnemo_core::config::MnemoConfig;
use mnemo_core::error::{MnemoError, MnemoResult};
use mnemo_core::models::document::Document;
use mnemo_core::ws::WS_HUB;
use serde_json::json;
use std::env;

fn broadcast_error(job_id: Option<&str>, message: &str) {
    let _ = WS_HUB
        .broadcast(json!({"event":"ingest_error","job_id":job_id,"message":message}).to_string());
}

fn broadcast_error_summary(errors: Vec<String>) {
    let _ = WS_HUB.broadcast(json!({"event":"ingest_error_summary","errors":errors}).to_string());
}

fn broadcast_panic(job_id: Option<&str>) {
    let _ = WS_HUB.broadcast(
        json!({"event":"ingest_step","job_id":job_id,"step":"panic","status":"failed","message":"Ingestion crashed due to PDF parser panic"}).to_string(),
    );
}

/// High-level controller for orchestrating ingestion across providers and pipelines.
pub struct IngestionController {
    pub pipelines: PipelineRegistry,
    pub config: Option<MnemoConfig>,
    pub metrics: IngestionMetrics,
    pub meta_store: mnemo_storage::metadata::postgres::PostgresMetadataStore,
}

impl IngestionController {
    /// Build a controller with a default pipeline stack.
    pub fn new() -> Self {
        let pg_url = env::var("MNEMO_METADATA_PG")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/mnemo".into());
        let meta_store = mnemo_storage::metadata::postgres::PostgresMetadataStore::new(pg_url)
            .expect("failed to create Postgres metadata store");

        let mut pipelines = PipelineRegistry::new();
        pipelines.add_step(Arc::new(super::pipeline::fingerprint_step::FingerprintStep::new(
            meta_store.clone(),
        )));
        pipelines.add_step(Arc::new(ChunkStep));
        pipelines.add_step(Arc::new(OntologyStep));
        pipelines.add_step(Arc::new(EmbeddingStep));
        pipelines.add_step(Arc::new(VectorUpsertStep));
        pipelines.add_step(Arc::new(GraphBuilderStep::new(
            env::var("SURREALDB_URL").unwrap_or_else(|_| "http://localhost:8000".into()),
            meta_store.clone(),
        )));

        Self { pipelines, config: None, metrics: IngestionMetrics::new(), meta_store }
    }

    /// Gather documents from configured providers (placeholder: filesystem only).
    pub fn load_sources(&self) -> Vec<Document> {
        let root_path = std::env::var("INGESTION_ROOT").unwrap_or_else(|_| "/app/data".to_string());
        let roots = vec![root_path.into()];
        tracing::info!("Scanning root path: {}", roots[0]);
        let provider = FilesystemProvider::new(roots);
        let docs = provider.scan();
        tracing::info!("Total documents loaded: {}", docs.len());
        docs
    }

    /// Run the pipeline stack against provided documents.
    pub async fn run_pipeline(&mut self, documents: Vec<Document>) -> MnemoResult<()> {
        let mut data = crate::pipeline::data::PipelineData::new();
        data.documents = documents;
        self.metrics.documents_processed = data.documents.len();
        tracing::info!("Ingestion: loaded {} documents", self.metrics.documents_processed);

        // clone steps so controller can be reused
        let registry = PipelineRegistry { steps: self.pipelines.steps.clone() };
        let executor = PipelineExecutor::new(registry);
        let job_id = data.job_id.clone();
        let result = std::panic::AssertUnwindSafe(executor.execute_with_data(data))
            .catch_unwind()
            .await
            .map_err(|_| {
                broadcast_error(job_id.as_deref(), "internal panic");
                broadcast_panic(job_id.as_deref());
                let _ = WS_HUB.broadcast(
                    json!({"event":"pipeline_failed","job_id":job_id,"message":"internal panic"})
                        .to_string(),
                );
                MnemoError::Message("pipeline panicked".into())
            })?;
        if let Ok(ref out) = result {
            self.metrics.chunks_produced = out.chunks.len();
            tracing::info!("Ingestion: produced {} chunks", out.chunks.len());
            // Validate critical stages
            if out.chunks.is_empty() {
                let msg = "chunking produced no chunks";
                broadcast_error(job_id.as_deref(), msg);
                return Err(MnemoError::Message(msg.into()));
            }
            if out.metrics.qdrant_writes == 0 {
                let msg = "vector upsert produced no writes";
                broadcast_error(job_id.as_deref(), msg);
                return Err(MnemoError::Message(msg.into()));
            }
            // Placeholder: treat chunk count as graph nodes inserted; fail if none.
            if out.chunks.is_empty() {
                let msg = "graph upsert produced no nodes";
                broadcast_error(job_id.as_deref(), msg);
                return Err(MnemoError::Message(msg.into()));
            }
        }
        result.map(|_| ())
    }

    /// Execute full ingestion: load sources then process through pipeline.
    pub async fn run(&mut self) -> MnemoResult<()> {
        let namespace = std::env::var("MNEMO_NAMESPACE").unwrap_or_else(|_| "local".to_string());

        let _ = WS_HUB.broadcast(
            json!({"event":"ingest_step","job_id":null,"step":"start","status":"running"})
                .to_string(),
        );

        let lock_acquired = self.meta_store.try_advisory_lock(&namespace).await.unwrap_or(false);
        if !lock_acquired {
            // Another worker is ingesting this namespace; skip.
            return Ok(());
        }

        let docs = self.load_sources();
        if docs.is_empty() {
            let msg = "no documents found at configured roots";
            broadcast_error(None, msg);
            return Err(MnemoError::Message(msg.into()));
        }
        let res = self.run_pipeline(docs).await;
        if res.is_ok() {
            crate::metrics::store_last_metrics(self.metrics.clone());
            let _ = WS_HUB.broadcast(
                json!({"event":"ingest_step","job_id":null,"step":"completed","status":"done"})
                    .to_string(),
            );
        } else {
            if let Err(err) = &res {
                let msg = format!("Ingestion failed: {}", err);
                broadcast_error(None, &msg);
                broadcast_error_summary(vec![msg]);
            }
            let _ = WS_HUB.broadcast(
                json!({"event":"ingest_step","job_id":null,"step":"completed","status":"failed"})
                    .to_string(),
            );
        }

        let _ = self.meta_store.advisory_unlock(&namespace).await;
        res
    }

    /// Access collected metrics.
    pub fn metrics(&self) -> &IngestionMetrics {
        &self.metrics
    }
}
