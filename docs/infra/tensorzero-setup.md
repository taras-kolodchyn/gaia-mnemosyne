# TensorZero LLM Proxy (local dev, macOS M1)

Mnemosyne should call TensorZero, not Ollama directly. TensorZero proxies the local Ollama-backed model (Qwen2.5 7B) and exposes a stable HTTP API.

## Prereqs
- Ollama installed and running (`http://localhost:11434`)
- Qwen2.5 7B pulled:
  ```bash
  ollama pull qwen2.5:7b
  ```
- TensorZero installed (choose one):
  ```bash
  # pip/uv
  pip install tensorzero
  # or uv
  uv pip install tensorzero
  # or check releases for binaries: https://github.com/tensorzero/tensorzero
  ```

## Start TensorZero (local)
Use the helper script (after making it executable once: `chmod +x scripts/run-tensorzero-local.sh`):
```bash
scripts/run-tensorzero-local.sh
```
It checks Ollama availability then runs:
```
tensorzero serve --config infra/tensorzero/tensorzero.yaml
```
Default endpoint: `http://localhost:9090`

Manual run (optional):
```bash
tensorzero serve --config infra/tensorzero/tensorzero.yaml
```

### Health check
```bash
scripts/check-tensorzero.sh          # uses MNEMO_LLM_URL or defaults to http://localhost:9090
curl -s http://localhost:9090/v1/models
```

## Env vars for Mnemosyne
Set before running the stack (docker-compose passes these through):
```bash
export MNEMO_LLM_PROVIDER=tensorzero
export MNEMO_LLM_URL=http://host.docker.internal:9090   # for Docker on macOS
export MNEMO_LLM_MODEL=qwen2.5:7b
```

## Notes
- TensorZero → Ollama → Qwen2.5 7B. Mnemo API must never call Ollama directly.
- Ensure Ollama is running on the host; TensorZero proxies it for container access.
