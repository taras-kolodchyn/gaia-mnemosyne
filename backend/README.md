# Gaia Mnemosyne Backend

Rust workspace for API, ingestion pipeline, storage, inference, and CLI.

- crates/api      — HTTP + MCP API
- crates/core     — models, traits, shared logic
- crates/ingest   — ingestion workers
- crates/scheduler — local scheduler implementation
- crates/storage  — vector, graph, metadata, cache
- crates/inference — LLM + Embedding engines
- crates/cli      — mnemo CLI