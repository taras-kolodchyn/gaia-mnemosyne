# Mnemosyne LLM Path (dev)

```
Mnemo API -> TensorZero Gateway (http://tensorzero:3000) -> Ollama (http://host.docker.internal:11434)
```

## What is deployed (make up)
- `tensorzero` gateway on `3000`, config at `ops/tensorzero-config/tensorzero.toml`.
- `tensorzero-ui` on `4000`.
- `postgres` and `clickhouse` for TensorZero metadata/UI.
- `ollama` runs on the host (macOS).

## Model aliases (no hardcoded model names in code)
- Chat: `chat_default`
- Embeddings: `vector_default`

Aliases map to concrete Ollama models in `ops/tensorzero-config/tensorzero.toml`.
Concrete model names should only live in config or `.env`, never in code.
Set the aliases via env (required by code):
- `MNEMO_LLM_MODEL` (chat alias)
- `TENSORZERO_EMBED_MODEL` or `TENSORZERO_EMBED_MODELS` (embedding alias)

## Why a vector (embedding) model?
The vector model converts text into numeric embeddings. These embeddings power:
- semantic similarity search in Qdrant,
- retrieval and re-ranking for RAG,
- deduplication / clustering,
- graph linking and relevance scoring.

## Verify TensorZero
```bash
curl -s http://localhost:3000/status

curl -X POST http://localhost:3000/inference \
  -H 'Content-Type: application/json' \
  -d '{"function_name":"chatbot","input":{"messages":[{"role":"user","content":"Hello"}]}}'
```

## Embedding fallback (temporary)
TensorZero 2025.11.6 does not expose OpenAI-style `/v1/embeddings` for Ollama.
For now, embeddings fall back to Ollama directly:

```bash
curl -X POST http://localhost:11434/v1/embeddings \
  -H 'Content-Type: application/json' \
  -d '{"model":"qwen3-embedding:8b","input":"hello world"}'
```

Set env vars:
- `TENSORZERO_EMBED_FALLBACK_URL=http://host.docker.internal:11434`
- `TENSORZERO_EMBED_FALLBACK_MODELS=qwen3-embedding:8b`

## Helper scripts
```bash
scripts/run-tensorzero-local.sh    # optional local run
scripts/check-tensorzero.sh        # healthcheck against MNEMO_LLM_URL (default http://localhost:3000)
```
