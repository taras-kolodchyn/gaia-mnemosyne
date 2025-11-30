# Backend Benchmarks

> Criterion-based micro-benchmarks for Gaia Mnemosyne backend crates.

## Contents
- `basic_bench.rs` — placeholder Criterion benchmark.

## Run
```bash
cd backend
cargo bench
```

## Adding Benchmarks
1. Create a new `*.rs` file under `backend/benches/`.
2. Use `criterion::{criterion_group, criterion_main}`.
3. Keep benchmarks short and deterministic.

## Notes
- Requires Rust 1.91.1.
- Benchmarks are opt-in; not run during `cargo test`.
