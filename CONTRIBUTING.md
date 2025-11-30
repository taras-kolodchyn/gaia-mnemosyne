# Contributing to Gaia Mnemosyne

## Introduction
Thanks for helping improve Gaia Mnemosyne! Please follow the workflow below so we can keep changes smooth and reviewable.

## Development workflow
- Fork the repo, create a feature branch, and open a pull request.
- Run tests before submitting: `cargo test --all`

## Code style
- Use `rustfmt` for Rust code formatting.
- Run `clippy` to catch common issues and lints.

## Test structure
- Unit tests: `backend/tests/unit`
- Integration tests: `backend/tests/integration`
- End-to-end tests: `backend/tests/e2e`
- Shared fixtures: `backend/tests/fixtures`

## PR requirements
- Keep PRs focused and reasonably small.
- Include tests or notes on testing performed.
- Describe the change, rationale, and any follow-up work.
