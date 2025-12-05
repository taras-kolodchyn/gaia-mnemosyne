#!/usr/bin/env bash

set -euo pipefail

BASE_URL="${MNEMO_LLM_BASE_URL:-http://localhost:11434}"

resp="$(curl -sS --max-time 5 "${BASE_URL}/api/tags" || true)"

if [ -n "${resp}" ]; then
  echo "Ollama LLM endpoint is reachable at ${BASE_URL}"
  exit 0
else
  echo "Ollama LLM endpoint is NOT reachable at ${BASE_URL}"
  exit 1
fi
