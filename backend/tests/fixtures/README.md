# Test Fixtures

Shared assets reused across unit/integration/E2E tests.

## Purpose
- Provide deterministic sample inputs (documents, configs, JSON).
- Avoid duplicating inline strings inside tests.

## Usage
- Access via `mnemo_test_utils::fixtures::load_fixture("sample.txt")`.
- Keep fixtures small and clearly named.

## Contents
- `sample.txt` — basic text example.
- Add more domain fixtures as scenarios expand.
