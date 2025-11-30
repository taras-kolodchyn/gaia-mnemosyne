# Gaia Mnemosyne UI

React + TypeScript + Vite + Tailwind UI for monitoring ingestion, exploring the graph, and running RAG queries.

## Features
- Dashboard and System Status (API/Qdrant/SurrealDB/Redis/Postgres)
- Pipeline Monitor with live WS updates, logs, timelines
- Graph Explorer (ReactFlow) with node inspector
- RAG Playground (query → backend RAG pipeline)
- Global WS context consolidating `/ws/all`

## Getting Started
```bash
cd ui
npm install
npm run dev
```
Open http://localhost:5173.

## Pages
- **Dashboard**: quick metrics and cards
- **Pipeline Monitor**: jobs list, history, steps, logs
- **Graph Explorer**: renders nodes/edges from `/v1/graph/snapshot`, node inspector from `/v1/graph/node/:id`
- **RAG Playground**: POST `/v1/rag/query`, shows chunks & metadata
- **System Status**: health panel using `/v1/health` and metrics endpoints

## WS Event Model
Handled via `WSContext` (single connection):
- `job_update`, `ingest_step`, `ingest_log`, `ingest_error`, `pipeline_failed`
- `graph_update`
- `rag_processing` / `rag_done`
- `log` (backend logs stream)

## Development Workflow
- Lint: `npm run lint`
- Build: `npm run build`

## Troubleshooting
- Black screen: ensure API reachable at `VITE_API_URL` (default http://localhost:7700)
- WS issues: check `/ws/all` reachable, CORS open on backend

## License
MIT
