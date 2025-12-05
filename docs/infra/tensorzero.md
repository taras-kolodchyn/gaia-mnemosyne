# TensorZero Core + UI (dev)

Flow:
```
Mnemo API ──HTTP──> TensorZero (OpenAI compat :9090/v1) ──> Ollama (:11434) ──> Qwen2.5:7b
                              │
                              └── TensorZero UI (:4000)
```

Compose services (`make up`):
- `ollama` on `11434`, volume `ollama_data`.
- `tensorzero` (gateway image) on `9090`, with `MODEL_ADAPTER=ollama:qwen2.5:7b`, dev API key `local-dev-key`.
- `tensorzero-ui` on `4000`, reads config from `tensorzero-config/`.
- `mnemo-api` env points only to TensorZero (`MNEMO_LLM_URL=http://tensorzero:9090/v1`, `MNEMO_LLM_API_KEY=local-dev-key`).

Smoke tests:
```bash
# From host
curl -s http://localhost:9090/v1/models

curl -s http://localhost:9090/v1/chat/completions \
  -H 'Authorization: Bearer local-dev-key' \
  -H 'Content-Type: application/json' \
  -d '{"model":"qwen2.5:7b","messages":[{"role":"user","content":"Hello from TensorZero"}]}'

# In compose network
curl -s http://tensorzero:9090/v1/models
```

Add models:
1) Pull in Ollama: `ollama pull <model>`.
2) Update `MODEL_ADAPTER` (tensorzero service) and `MNEMO_LLM_MODEL` if needed.
