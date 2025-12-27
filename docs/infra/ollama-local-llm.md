# Ollama Qwen3 Local LLM (macOS)

This guide prepares a local Ollama endpoint on macOS for Gaia Mnemosyne. TensorZero uses Ollama as the backend, and embeddings temporarily fall back to Ollama directly.

## 1) Install Ollama (macOS)
- Download and install from https://ollama.com/download (official .dmg).
- Start Ollama (launch the app). It runs on `http://localhost:11434`.
- Verify it is running:
  ```bash
  ps aux | grep ollama
  curl -s http://localhost:11434/api/version
  ```

## 2) Pull models
```bash
ollama pull qwen3:8b
ollama pull qwen3-embedding:8b
```
- Check models:
  ```bash
  ollama list
  ```
- Chat smoke test:
  ```bash
  ollama run qwen3:8b <<'EOF'
  You are a concise assistant. Say hello in one sentence.
  EOF
  ```
- Embedding smoke test:
  ```bash
  curl -X POST http://localhost:11434/v1/embeddings \
    -H 'Content-Type: application/json' \
    -d '{"model":"qwen3-embedding:8b","input":"hello world"}'
  ```

## 3) Resource notes
- Qwen3 8B typically needs ~6-8 GB RAM; allow headroom for Docker containers.
- CPU-only is fine; GPU use depends on your hardware/drivers.

## 4) Networking from Mnemo containers
- On macOS, Docker containers can reach the host via `http://host.docker.internal`.
- For Ollama, use: `http://host.docker.internal:11434`.

## 5) Mnemo env vars (embedding fallback)
TensorZero handles chat, while embeddings fall back to Ollama for now:
```bash
export TENSORZERO_EMBED_FALLBACK_URL=http://host.docker.internal:11434
export TENSORZERO_EMBED_FALLBACK_MODELS=qwen3-embedding:8b
```
Set `TENSORZERO_EMBED_MODEL` (alias) in your env or compose config; concrete
model names live only in config/env.

## 6) Health check helper
```bash
scripts/check-ollama-llm.sh
```
If you just cloned, mark executable once:
```bash
chmod +x scripts/check-ollama-llm.sh
```
