# mnemo-inference

Inference abstraction layer for embeddings / LLM / classification engines.

## Modules
- `traits.rs` — `InferenceEngine` async trait (embed, infer, classify)
- `embedding_engine.rs` — thin wrapper to call engines
- `engines/` — TensorZero, Proxy skeletons

## Example
```rust
use mnemo_inference::traits::InferenceEngine;

struct Fake;
#[async_trait::async_trait]
impl InferenceEngine for Fake {
    async fn embed(&self, texts: Vec<String>) -> Vec<Vec<f32>> { vec![vec![1.0; 3]; texts.len()] }
    async fn infer(&self, _p: String) -> String { "ok".into() }
    async fn classify(&self, t: String, _l: Vec<String>) -> String { t }
}
```

## Development
```bash
cargo test -p mnemo-inference
```
