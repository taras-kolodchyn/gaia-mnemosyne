#!/usr/bin/env bash

set -euo pipefail

CONFIG_PATH="${CONFIG_PATH:-infra/tensorzero/tensorzero.yaml}"
OLLAMA_URL="${OLLAMA_URL:-http://localhost:11434}"
TENSORZERO_PORT="${TENSORZERO_PORT:-9090}"

echo "Checking Ollama at ${OLLAMA_URL}..."
if curl -sS --max-time 3 "${OLLAMA_URL}/api/tags" >/dev/null; then
  echo "Ollama is running."
else
  echo "Ollama is NOT reachable at ${OLLAMA_URL}. Start Ollama and try again."
  exit 1
fi

echo "Starting TensorZero with config ${CONFIG_PATH}..."
echo "Endpoint will be http://localhost:${TENSORZERO_PORT}"
tensorzero serve --config "${CONFIG_PATH}"
