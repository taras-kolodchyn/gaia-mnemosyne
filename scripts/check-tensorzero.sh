#!/usr/bin/env bash

set -euo pipefail

BASE_URL="${MNEMO_LLM_URL:-http://localhost:3000}"

if curl -sS --max-time 5 "${BASE_URL}/v1/models" >/dev/null; then
  echo "TensorZero endpoint is reachable at ${BASE_URL}"
  exit 0
else
  echo "TensorZero endpoint is NOT reachable at ${BASE_URL}"
  exit 1
fi
