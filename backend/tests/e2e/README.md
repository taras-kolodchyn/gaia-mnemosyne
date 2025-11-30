# End-to-End Tests

High-level scenarios that mirror real usage (ingestion + RAG + APIs).

## Purpose
- Validate complete workflows across services and storage layers.
- Catch regressions that unit/integration tests may miss.

## Structure
- `basic_flow_test.rs` and future scenario files.

## How to Run
```bash
cd backend
cargo test --all --tests
```

## Guidelines
- Prefer stable, deterministic inputs from `tests/fixtures`.
- Keep external dependencies (Qdrant/SurrealDB/Redis) isolated via docker-compose when needed.
