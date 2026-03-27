#!/bin/bash
# Backward-compatible wrapper. The corpus builder lives in fetch_corpus.py.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
exec python3 "$SCRIPT_DIR/fetch_corpus.py"
