# TensorZero Gateway (local dev)

Mnemosyne should call TensorZero, not Ollama directly. TensorZero proxies the local Ollama-backed models and exposes a stable HTTP API.

## Prereqs
- Ollama installed and running (`http://localhost:11434`)
- Models pulled:
  ```bash
  ollama pull qwen3:8b
  ollama pull qwen3-embedding:8b
  ```
- TensorZero installed (choose one):
  ```bash
  pip install tensorzero
  # or
  uv pip install tensorzero
  # or check releases: https://github.com/tensorzero/tensorzero
  ```

## Start TensorZero (local)
Helper script (make executable once: `chmod +x scripts/run-tensorzero-local.sh`):
```bash
scripts/run-tensorzero-local.sh
```
It runs:
```
tensorzero serve --config ops/tensorzero-config/tensorzero.toml
```
Default endpoint: `http://localhost:3000`

Manual run:
```bash
tensorzero serve --config ops/tensorzero-config/tensorzero.toml
```

### Health check
```bash
scripts/check-tensorzero.sh          # uses MNEMO_LLM_URL or defaults to http://localhost:3000
curl -s http://localhost:3000/status
```

## Env vars for Mnemosyne
```bash
export MNEMO_LLM_PROVIDER=tensorzero
export MNEMO_LLM_URL=http://host.docker.internal:3000
export MNEMO_LLM_MODEL=chat_default
export TENSORZERO_EMBED_MODEL=vector_default
export TENSORZERO_EMBED_FALLBACK_URL=http://host.docker.internal:11434
export TENSORZERO_EMBED_FALLBACK_MODELS=qwen3-embedding:8b
```
These aliases are required by the code; there is no hardcoded fallback.

## Notes
- TensorZero -> Ollama -> Qwen3 via model aliases (`chat_default`, `vector_default`).
- Mnemo API should not call Ollama directly except for embedding fallback (temporary).
- Keep concrete model names in config/`.env` only; code should use aliases.
