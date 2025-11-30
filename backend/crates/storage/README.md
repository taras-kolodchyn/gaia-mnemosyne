# mnemo-storage

Storage adapters for Gaia Mnemosyne: Qdrant vectors, SurrealDB graph, Postgres metadata, Redis caches.

## Modules
- `vector/` — Qdrant client, vector_engine upserts/search
- `graph/` — SurrealDB 2.x helpers (`file`, `chunk`, `contains` tables)
- `metadata/` — Postgres fingerprints/documents/chunks persistence
- `cache/` — Redis cache skeletons

## Example
```rust
use mnemo_storage::vector::vector_engine::VectorEngine;
use mnemo_storage::vector::qdrant::QdrantVectorStore;

let engine = VectorEngine::new(QdrantVectorStore { url: "http://localhost:6333".into() });
```

## Notes
- Graph IDs are hashed (sha256) from paths/indices
- SurrealDB 2.x SurrealQL inserts into `file`, `chunk`, `contains`

## Development
```bash
cargo test -p mnemo-storage
```
