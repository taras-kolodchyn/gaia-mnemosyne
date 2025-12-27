# TensorZero Gateway + UI (dev)

Flow:
```
Mnemo API -> TensorZero Gateway (:3000, /inference) -> Ollama (:11434) -> chat_default, vector_default
                         |
                         -> TensorZero UI (:4000)
```

Compose services (`make up`):
- `tensorzero` gateway on `3000`, config at `ops/tensorzero-config/tensorzero.toml`.
- `tensorzero-ui` on `4000`.
- `postgres` and `clickhouse` for TensorZero metadata/UI.
- `ollama` runs on the host (macOS).

Smoke tests:
```bash
# Gateway health
curl -s http://localhost:3000/status

# Chat (via function)
curl -X POST http://localhost:3000/inference \
  -H 'Content-Type: application/json' \
  -d '{"function_name":"chatbot","input":{"messages":[{"role":"user","content":"Hello from TensorZero"}]}}'

# Embedding fallback (direct Ollama, temporary)
curl -X POST http://localhost:11434/v1/embeddings \
  -H 'Content-Type: application/json' \
  -d '{"model":"qwen3-embedding:8b","input":"hello world"}'
```

Model aliases (avoid hardcoding model names in code):
- `chat_default` -> `qwen3:8b`
- `vector_default` -> `qwen3-embedding:8b`

Adjust aliases in `ops/tensorzero-config/tensorzero.toml`, and set:
- `MNEMO_LLM_MODEL=chat_default`
- `TENSORZERO_EMBED_MODEL=vector_default`
These aliases are required at runtime; the code does not fall back to a default.

Vector (embedding) models are used to create numeric embeddings for semantic
search, retrieval, and ranking. They are stored in Qdrant and reused by the
RAG pipeline.
