# mnemo-core

Core domain for Gaia Mnemosyne: models, traits, RAG orchestration, config, logging.

## Key Modules
- `models/` — Document, Chunk, QueryPlan, RAGContext
- `traits/` — storage/search/ontology/ranking interfaces
- `rag/` — orchestrator, pipeline API, strategies, caching hooks
- `config/` — MnemoConfig, profile loaders
- `logging/` — tracing init helpers
- `mnemosyne.rs` — top-level engine facade

## Usage
```rust
use mnemo_core::mnemosyne::MnemosyneEngine;

#[tokio::main]
async fn main() {
    let engine = MnemosyneEngine::new();
    let resp = engine.query("hello").await;
    println!("{}", resp);
}
```

## Traits
- `VectorSearch`, `KeywordSearch`, `RankingEngine`, `OntologyEngine`

## Development
```bash
cargo test -p mnemo-core
```
