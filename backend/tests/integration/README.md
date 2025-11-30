# Integration Tests

End-to-end flows across crates and lightweight external boundaries.

## Purpose
- Validate API wiring, pipeline assembly, and crate interoperability.
- Exercise HTTP handlers and shared types with minimal mocks.

## Structure
- `api_integration_test.rs` — router initialization sanity.
- Additional files cover cross-crate workflows as they are added.

## How to Run
```bash
cd backend
cargo test --all --tests
```

## Guidelines
- Prefer in-memory or fake services over live dependencies.
- Reuse `mnemo_test_utils` mocks where possible.
- Keep assertions stable and order-independent.
