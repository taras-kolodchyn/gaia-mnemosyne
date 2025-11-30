# Gaia Mnemosyne Backend

[![Status](https://img.shields.io/badge/status-alpha-blue)](#) [![License](https://img.shields.io/badge/license-MIT-green)](#)

Rust workspace providing APIs, ingestion pipelines, storage adapters, and RAG orchestration for Gaia Mnemosyne.

## Key Features
- Axum HTTP/WS API (`mnemo-api-service`)
- Ingestion pipeline: fingerprint → chunk → ontology → embeddings → Qdrant upsert → SurrealDB graph upsert
- Pluggable providers (filesystem live, GitHub/OpenAPI/PDF/DOCX ready)
- Metadata in Postgres; vectors in Qdrant; graph in SurrealDB 2.x
- RAG orchestrator combining vector + keyword + graph signals

## Architecture
```
crates/
  api         -> axum endpoints, WS hubs
  core        -> models, traits, RAG orchestration, config, logging
  ingest      -> pipeline steps, providers, metrics, jobs
  storage     -> Qdrant client, Surreal graph, Postgres metadata
  inference   -> engine traits/adapters, embedding engine facade
  cli         -> mnemo CLI commands
bin/
  mnemo-api   -> runtime binary wiring API + services
```

## Getting Started
### Prerequisites
- Rust 1.91.1
- Qdrant, SurrealDB 2.x, Postgres (docker-compose provided)

### Run
```bash
cd backend
cargo run -p mnemo-api-service
```

### Tests
```bash
cargo test --workspace
```

## Environment Variables
- `SURREALDB_URL` (default `http://localhost:8000`)
- `SURREALDB_NS` / `SURREALDB_DB` (default `mnemo`)
- `SURREALDB_USER` / `SURREALDB_PASS`
- `QDRANT_URL` (default `http://localhost:6333`)
- `DATABASE_URL` (Postgres)
- `INGESTION_ROOT` (filesystem provider root)

## Pipeline Overview
1. **Fingerprint**: skip unchanged files via Postgres fingerprints.
2. **Chunking**: split into 800–1200 char chunks.
3. **Ontology**: tag chunks (project/domain/company).
4. **Embeddings**: placeholder dense vectors (1536 dims, normalized).
5. **Vector Upsert**: Qdrant collection `mnemo_chunks`.
6. **Graph Upsert**: SurrealDB tables `file`, `chunk`, `contains` (hashed IDs).
7. **Metrics & WS**: steps, logs, job updates broadcast over WS.

## API Surface (selected)
- `GET /v1/health`
- `GET /v1/version`
- `POST /v1/jobs/create`, `POST /v1/jobs/run`, `GET /v1/jobs`
- `POST /v1/rag/query`, `POST /v1/rag/debug`, `GET /v1/rag/metadata`
- `GET /v1/graph/snapshot`, `GET /v1/graph/node/:id`
- WS: `/ws/all` (aggregated), `/ws/jobs`, `/ws/logs`, `/ws/graph`, `/ws/rag`, `/ws/status`

## Development Workflow
- Format: `cargo fmt`
- Lint: `cargo clippy --workspace`
- Test: `cargo test --workspace`
- Run migrations: `make migrate` (sqlx + Postgres)

## Troubleshooting
- Surreal errors: ensure 2.x image, hashed IDs, check logs for SQL.
- Qdrant 400: confirm payload has `id` and collection exists.
- CORS/WS: server uses `CorsLayer::new().allow_origin(Any)`.

## License
MIT
