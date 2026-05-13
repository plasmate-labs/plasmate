#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GO_CACHE="${GOCACHE:-$ROOT/target/go-cache}"

run_in() {
  local dir="$1"
  local label="$2"
  shift 2

  printf '\n==> %s\n' "$label"
  (
    cd "$ROOT/$dir"
    "$@"
  )
}

run_in "packages/som-parser-python" \
  "Python parser action manifest" \
  env PYTHONPATH=. python3 -m pytest tests/test_parser.py -q

run_in "packages/som-parser-node" \
  "Node parser action manifest" \
  npm test

run_in "sdk/go" \
  "Go SDK action manifest" \
  env GOCACHE="$GO_CACHE" go test ./...

run_in "sdk/python" \
  "Python SDK action manifest" \
  env PYTHONPATH=src python3 -m pytest tests/test_query.py -q

run_in "sdk/node" \
  "Node SDK action manifest" \
  npm test

run_in "integrations/browser-use" \
  "Browser Use adapter action manifest" \
  env PYTHONPATH="$ROOT/packages/som-parser-python:$ROOT/integrations/browser-use" \
    python3 -m pytest tests/test_extractor.py -q

run_in "integrations/langchain" \
  "LangChain adapter action manifest" \
  env PYTHONPATH="$ROOT/packages/som-parser-python:$ROOT/integrations/langchain" \
    python3 -m pytest tests/test_som_output.py -q

run_in "integrations/vercel-ai" \
  "Vercel AI adapter action manifest" \
  npm test

printf '\nAction manifest conformance passed.\n'
