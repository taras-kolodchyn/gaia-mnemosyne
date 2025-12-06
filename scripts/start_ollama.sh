#!/bin/bash
set -euo pipefail

check_up() {
  curl -fsS --max-time 2 http://localhost:11434/api/tags >/dev/null
}

echo "[INFO] Checking for Ollama CLI..."
if ! command -v ollama >/dev/null 2>&1; then
  echo "[ERROR] Ollama CLI not found. Please install from https://ollama.com/download and launch it once." >&2
  exit 1
fi

echo "[INFO] Starting Ollama daemon…"
# Try user GUI service first (typical on macOS), then system. Ignore failures and fall back.
launchctl kickstart -k "gui/$(id -u)/com.ollama" >/dev/null 2>&1 || \
launchctl kickstart -k system/com.ollama >/dev/null 2>&1 || true

# If still not up, fall back to foreground server in background.
if ! check_up; then
  echo "[INFO] Falling back to 'ollama serve' in background..."
  nohup ollama serve >/tmp/ollama-serve.log 2>&1 &
  sleep 2
fi

if ! check_up; then
  echo "[ERROR] Ollama daemon is not responding at http://localhost:11434. Ensure the macOS Ollama app is installed and running." >&2
  exit 1
fi

echo "[INFO] Pulling model qwen3:8b…"
ollama pull qwen3:8b

echo "[INFO] Warming model qwen3:8b…"
echo "hello" | ollama run qwen3:8b >/dev/null 2>&1 || true

echo "[INFO] Pulling model qwen3-embedding…"
ollama pull qwen3-embedding:8b || true

echo "[INFO] Warming model qwen3-embedding…"
echo "ping" | ollama run qwen3-embedding >/dev/null 2>&1 || true

echo "[INFO] Ollama ready."
