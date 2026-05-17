#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GO_CACHE="${GOCACHE:-$ROOT/target/go-cache}"
MODE="${1:---full}"

usage() {
  cat <<'USAGE'
Usage: ./scripts/action-manifest-conformance.sh [--full|--quick]

Runs the shared action-availability expectation manifest across parser
packages, SDKs, and framework adapters, including grouped role/action target
buckets used to scope replay plans.

  --full   Run each package's normal action-plan test suite. Default.
  --quick  Run the narrow shared-manifest checks for faster CI feedback.
USAGE
}

case "$MODE" in
  --full | --quick)
    ;;
  -h | --help)
    usage
    exit 0
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

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

if [ "$MODE" = "--quick" ]; then
  run_in "packages/som-parser-python" \
    "Python parser action manifest" \
    env PYTHONPATH=. python3 -m pytest \
      tests/test_parser.py::TestGetActionPlan::test_matches_shared_action_availability_manifest -q

  run_in "packages/som-parser-node" \
    "Node parser action manifest" \
    npm test -- tests/parser.test.ts -t "matches the shared action availability manifest"

  run_in "sdk/go" \
    "Go SDK action manifest" \
    env GOCACHE="$GO_CACHE" go test ./... \
      -run 'Test(GetActionPlanMatchesSharedAvailabilityManifest|ActionPlanLookupHelpers|EnabledActionPlanIndexFiltersBlockedTargets)'

  run_in "sdk/python" \
    "Python SDK action manifest" \
    env PYTHONPATH=src python3 -m pytest \
      tests/test_query.py::TestGetActionPlan::test_matches_shared_action_availability_manifest -q

  run_in "sdk/node" \
    "Node SDK action manifest" \
    sh -c 'npm run build && node --test --test-name-pattern "matches the shared action availability manifest" dist/query.test.js'
else
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
fi

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
