# Backend Unit Tests

Lightweight, fast checks for isolated modules (core, storage, inference).

## Purpose
- Validate small surfaces without external services.
- Catch regressions before integration/E2E stages.

## Structure
- `core_tests.rs`
- `storage_tests.rs`
- Additional unit files as they are added.

## How to Run
```bash
cd backend
cargo test --test '*'
```

## Conventions
- No network or external DB calls.
- Keep fixtures minimal; reuse `backend/tests/fixtures`.
- Prefer deterministic assertions over timing-based checks.
