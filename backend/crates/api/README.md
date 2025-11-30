# mnemo-api

Axum HTTP/WS API for Gaia Mnemosyne.

## Endpoints (selected)
- `GET /v1/health`, `GET /v1/version`
- `POST /v1/jobs/create`, `POST /v1/jobs/run`, `GET /v1/jobs`
- `GET /v1/ingestion/metrics`
- `POST /v1/rag/query`, `POST /v1/rag/debug`, `GET /v1/rag/metadata`
- `GET /v1/graph/snapshot`, `GET /v1/graph/node/:id`
- WS: `/ws/all` (aggregated), `/ws/status`, `/ws/jobs`, `/ws/logs`, `/ws/graph`, `/ws/rag`

## Layers
- CORS: permissive via `CorsLayer::new().allow_origin(Any)`
- TraceLayer for request logging

## Run
```bash
cargo run -p mnemo-api-service
```

## Development
```bash
cargo test -p mnemo-api
```
