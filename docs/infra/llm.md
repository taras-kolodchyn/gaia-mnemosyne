# Mnemosyne LLM Path: Mnemo API → TensorZero → Ollama

```
Mnemo API ──(HTTP)──> TensorZero (http://tensorzero:9090) ──> Ollama (http://ollama:11434) ──> Qwen2.5 7B
```

## What’s deployed (make up)
- `ollama` service on port `11434` (pulls/runs `qwen2.5:7b`).
- `tensorzero` service on port `9090`, configured to proxy Ollama via `infra/tensorzero/tensorzero.yaml`.
- `mnemo-api` receives `MNEMO_LLM_PROVIDER=tensorzero`, `MNEMO_LLM_URL=http://tensorzero:9090`, `MNEMO_LLM_MODEL=qwen2.5:7b`.

## Verify TensorZero
From host:
```bash
curl -s http://localhost:9090/v1/models
```

From container network (inside compose):
```bash
curl -s http://tensorzero:9090/v1/models
```

Helper scripts:
```bash
scripts/run-tensorzero-local.sh    # starts TensorZero (if using host tooling)
scripts/check-tensorzero.sh        # healthcheck against MNEMO_LLM_URL (default http://localhost:9090)
```

## Quick TensorZero API smoke (no Mnemo logic yet)
```bash
curl -s http://localhost:9090/v1/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"qwen2.5:7b","prompt":"Say hello in one sentence."}'
```

## Mnemo API reminder
- Mnemo must NEVER call Ollama directly. All LLM traffic goes through TensorZero.
- Current work only wires config and healthcheck logging; actual LLM calls will be implemented later.
