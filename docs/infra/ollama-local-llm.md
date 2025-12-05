# Ollama Qwen2.5 Local LLM (macOS M1)

This guide prepares a local Ollama endpoint on a Mac M1 for Gaia Mnemosyne. No Mnemo code changes are required beyond setting environment variables and ensuring Docker containers can reach the host.

## 1) Install Ollama (macOS)
- Download and install from https://ollama.com/download (official .dmg).  
  *Do not use Homebrew for this setup.*
- Start Ollama (launch the app). It runs a local HTTP service on `http://localhost:11434`.
- Verify it is running:
  ```bash
  ps aux | grep ollama
  curl -s http://localhost:11434/api/version
  ```

## 2) Pull Qwen2.5 7B
```bash
ollama pull qwen2.5:7b
```
- Check models:
  ```bash
  ollama list
  ```
- Smoke test:
  ```bash
  ollama run qwen2.5:7b <<'EOF'
  You are a concise assistant. Say hello in one sentence.
  EOF
  ```
- API tag listing:
  ```bash
  curl -s http://localhost:11434/api/tags
  ```

## 3) Resource notes (M1)
- Qwen2.5:7B typically needs ~6–8 GB RAM; allow headroom for Docker containers.
- CPU-only is fine; GPU use depends on your hardware/drivers.

## 4) Networking from Mnemo containers
- On macOS, Docker containers can reach the host via `http://host.docker.internal`.
- For Ollama, use: `http://host.docker.internal:11434`.

## 5) Mnemo environment variables (LLM)
Set these before running the stack (also injected via docker-compose):
```bash
export MNEMO_LLM_BASE_URL=http://host.docker.internal:11434
export MNEMO_LLM_MODEL=qwen2.5:7b
```

## 6) Health check helper
After setting the env vars:
```bash
scripts/check-ollama-llm.sh
```
It will report whether the Ollama endpoint is reachable. If you just cloned, mark executable once:
```bash
chmod +x scripts/check-ollama-llm.sh
```
