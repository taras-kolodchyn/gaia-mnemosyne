# Gaia Mnemosyne

[![Status](https://img.shields.io/badge/status-alpha-blue)](#) [![License](https://img.shields.io/badge/license-MIT-green)](#)

Gaia Mnemosyne is a knowledge orchestration stack for local-first RAG with graph context and multi-source ingestion. It combines a Rust backend, React UI, vector search (Qdrant), graph storage (SurrealDB 2.x), and pluggable ingestion pipelines.

## Key Features
- Multi-source ingestion: filesystem (local/host-mount), GitHub (planned), OpenAPI/PDF/DOCX (extensible providers)
- Chunking + ontology tagging + embeddings → vector upsert to Qdrant
- Graph projection of files/chunks in SurrealDB 2.x with live WS updates
- RAG orchestration with keyword + vector + graph signals
- Real-time job monitoring (WS), logs, and ingestion metrics
- React/Vite/Tailwind UI with Pipeline Monitor, RAG Playground, Graph Explorer

## Architecture Overview
```
+----------------+     +-----------------+     +-----------------+
|   Providers    | --> |  Ingest Pipelines| --> |  Storage Layer  |
| FS / GitHub... |     |  (fingerprint -> |     | Qdrant (vector) |
|                |     |   chunk -> emb   |     | Surreal (graph) |
+----------------+     +-----------------+     +-----------------+
         | WS events (jobs, steps, logs)  | REST (/v1/*)
         v                                 v
+-------------------------------------------------------------+
|                         API (axum)                          |
+-------------------------------------------------------------+
         |                           \
         v                            \ WS (status/jobs/logs/graph/rag)
+------------------+          +-----------------------+
| RAG Orchestrator |          | React UI (Vite/TW)    |
+------------------+          +-----------------------+
```

## Quickstart
```bash
# from repo root
make up            # start services via docker-compose
make run           # starts deps + backend + frontend (dev)
```
Or manually:
```bash
docker compose -f ops/docker-compose.dev.yml up --build -d
cd backend && cargo run -p mnemo-api-service
cd ui && npm install && npm run dev
```

## Directory Structure
- `backend/` — Rust workspace (api, core, ingest, storage, inference, cli)
- `ui/` — React/Vite/Tailwind frontend
- `ops/` — docker-compose, scripts
- `backend/tests/` — unit, integration, e2e suites
- `backend/migrations/` — database migrations (Postgres)

## Components Overview
- **API (axum)**: HTTP/WS endpoints, health, jobs, RAG, graph snapshot/node.
- **Core**: domain models, RAG orchestrator, config, logging, traits.
- **Ingest**: pipeline steps (fingerprint, chunk, ontology, embeddings, vector upsert, graph upsert), providers, controller.
- **Storage**: Qdrant client, SurrealDB graph helpers, Postgres metadata.
- **Inference**: engine traits/adapters (TensorZero/Proxy placeholders), embedding engine facade.
- **UI**: Dashboard, Pipeline Monitor, Graph Explorer, RAG Playground, System Status.

## Development Workflow
- Rust toolchain 1.91.1
- `cargo fmt && cargo clippy`
- `cargo test --workspace`
- Frontend: `npm run lint && npm run build`

## Testing
- Backend: `cd backend && cargo test --workspace`
- Frontend: `cd ui && npm run test` (add tests as needed)

## Troubleshooting
- Qdrant errors: verify `QDRANT_URL` and collection `mnemo_chunks` exists.
- Surreal 400 errors: ensure SurrealDB 2.x running and IDs are hashed; check logs in Pipeline Monitor.
- CORS/WS: server.rs uses permissive CORS; confirm API at `http://localhost:7700`.

## Local LLM (Ollama on macOS)
- Ollama runs on the host and is used by the TensorZero gateway.
- Embedding fallback (temporary workaround until TensorZero exposes embeddings):
  - `TENSORZERO_EMBED_FALLBACK_URL=http://host.docker.internal:11434`
  - `TENSORZERO_EMBED_FALLBACK_MODELS=qwen3-embedding:8b`
- See `docs/infra/ollama-local-llm.md` for setup, model pull, and health checks (`scripts/check-ollama-llm.sh`).

## TensorZero LLM Proxy
- Mnemo API is wired to use TensorZero as the LLM gateway (not Ollama directly).
- Set:
  - `MNEMO_LLM_PROVIDER=tensorzero`
  - `MNEMO_LLM_URL=http://tensorzero:3000`
  - `MNEMO_LLM_MODEL=chat_default`
  - `TENSORZERO_EMBED_MODEL=vector_default`
- These aliases are required; the code does not hardcode model names.
- Model aliases are used everywhere in code; concrete model names live only in `ops/tensorzero-config/tensorzero.toml` and `.env`.
- Vector (embedding) models produce numeric embeddings stored in Qdrant for semantic search, retrieval, ranking, and graph linking.
- See `docs/infra/tensorzero-setup.md` and `docs/infra/llm.md` for setup, wiring, and health checks (`scripts/run-tensorzero-local.sh`, `scripts/check-tensorzero.sh`).

## Roadmap
- Provider auto-discovery and GitHub/OpenAPI enrichment
- Real embedding engines integration
- Schema-aware graph traversals and RAG scoring improvements
- Production hardening, auth, multi-namespace isolation

## License
MIT
