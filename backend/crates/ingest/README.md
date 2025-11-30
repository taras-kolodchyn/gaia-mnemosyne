# mnemo-ingest

Ingestion pipeline for Gaia Mnemosyne: providers, pipeline steps, jobs, metrics, WS events.

## Pipeline
1. Start / Fingerprints
2. Chunking (~800–1200 chars)
3. Ontology tagging
4. Embeddings (placeholder dense vectors)
5. Vector upsert (Qdrant)
6. Graph upsert (SurrealDB 2.x `file`, `chunk`, `contains`)
7. Completed / metrics

## Providers
- `filesystem` (INGESTION_ROOT paths)
- `github`, `openapi`, `pdf`, `docx` (extensible skeletons)

## Key Types
- `PipelineData` (documents, chunks, metadata, job_id)
- Steps: `fingerprint_step`, `chunk_step`, `ontology_step`, `embedding_step`, `vector_upsert_step`, `graph_builder_step`
- `job_runner`, `controller`, `metrics`

## WS Events
- `job_update`, `ingest_step`, `ingest_log`, `ingest_error`, `pipeline_failed`

## Example (runner)
```rust
use mnemo_ingest::fs_ingest_runner::run_filesystem_ingestion;
#[tokio::main]
async fn main() {
    run_filesystem_ingestion(None).await.unwrap();
}
```

## Development
```bash
cargo test -p mnemo-ingest
```
