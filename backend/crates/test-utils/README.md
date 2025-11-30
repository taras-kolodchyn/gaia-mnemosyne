# mnemo-test-utils

> Shared testing helpers for Gaia Mnemosyne (mock engines, fixtures, pipelines).

## Key Features
- Fake inference/vector engines for isolated tests.
- Fixture loader helpers.
- Pipeline builders for ingest tests.

## Quickstart
```bash
cd backend
cargo test -p mnemo_test_utils
```

## Modules
- `fake_vector_store` — mock vector store.
- `fake_inference_engine` — mock inference engine.
- `fixtures` — fixture loader utilities.
- `test_pipeline_builder` — simple pipeline registry builders.

## Usage Example
```rust
use mnemo_test_utils::fake_inference_engine::FakeInferenceEngine;

let engine = FakeInferenceEngine;
let vectors = engine.embed(vec!["hello".into()]).await;
assert_eq!(vectors[0], vec![1.0, 2.0, 3.0]);
```

## Development
- Requires Rust 1.91.1 (see root README).
- Run `cargo fmt` and `cargo clippy` before pushing.

## License
MIT © Gaia Mnemosyne Contributors
