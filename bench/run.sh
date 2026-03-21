#!/usr/bin/env bash
#
# bench/run.sh — One-command reproducible benchmark runner for Plasmate.
#
# Usage:
#   ./bench/run.sh                    # default: bench/urls-100.txt
#   ./bench/run.sh bench/urls.txt     # use original 38-URL list
#   TIMEOUT=20000 ./bench/run.sh      # custom timeout (ms)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Configuration ──
URLS_FILE="${1:-$SCRIPT_DIR/urls-100.txt}"
TIMEOUT="${TIMEOUT:-15000}"
REPORT_MD="$SCRIPT_DIR/report-100.md"
RESULTS_JSON="$SCRIPT_DIR/results.json"
BINARY="$ROOT_DIR/target/release/plasmate"

# ── Helpers ──
info()  { printf "\033[1;34m==>\033[0m %s\n" "$*"; }
warn()  { printf "\033[1;33mWARN:\033[0m %s\n" "$*"; }
error() { printf "\033[1;31mERROR:\033[0m %s\n" "$*" >&2; }

# ── Validate inputs ──
if [ ! -f "$URLS_FILE" ]; then
    error "URL file not found: $URLS_FILE"
    exit 1
fi

URL_COUNT=$(grep -v '^#' "$URLS_FILE" | grep -v '^$' | wc -l | tr -d ' ')
info "URL file: $URLS_FILE ($URL_COUNT URLs)"
info "Timeout per URL: ${TIMEOUT}ms"

# ── Step 1: Build in release mode ──
info "Building plasmate in release mode..."
cd "$ROOT_DIR"
cargo build --release 2>&1 | tail -1
if [ ! -f "$BINARY" ]; then
    error "Build failed — binary not found at $BINARY"
    exit 1
fi
info "Build complete: $BINARY"

# ── Step 2: Run benchmark ──
info "Running benchmark against $URL_COUNT URLs..."
echo ""

# The bench subcommand writes a markdown report directly.
# We run it and capture the output.
"$BINARY" bench --urls "$URLS_FILE" --output "$REPORT_MD" --timeout "$TIMEOUT" || {
    warn "Benchmark exited with non-zero status (some URLs may have failed)"
}

if [ ! -f "$REPORT_MD" ]; then
    error "Benchmark did not produce report at $REPORT_MD"
    exit 1
fi

# ── Step 3: Generate machine-readable JSON ──
info "Generating $RESULTS_JSON..."

# Parse the markdown table into JSON.
# Extract per-URL rows from the report (lines starting with "| " that contain a URL).
{
    echo '{'
    echo '  "meta": {'
    echo "    \"date\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "    \"urls_file\": \"$(basename "$URLS_FILE")\","
    echo "    \"url_count\": $URL_COUNT,"
    echo "    \"timeout_ms\": $TIMEOUT,"
    echo "    \"plasmate_version\": \"$("$BINARY" --version 2>/dev/null | head -1 || echo unknown)\","
    echo "    \"os\": \"$(uname -s) $(uname -r)\","
    echo "    \"arch\": \"$(uname -m)\""
    echo '  },'
    echo '  "results": ['

    FIRST=true
    # Read per-URL result rows from the markdown report.
    # Format: | url | html_bytes | som_bytes | ratio | grade | elements | interactive | fetch_ms | parse_ms | status |
    while IFS='|' read -r _ url html_bytes som_bytes ratio grade elements interactive fetch_ms parse_ms status _; do
        # Skip header/separator rows
        url=$(echo "$url" | xargs)
        [[ "$url" == "URL" ]] && continue
        [[ "$url" == -* ]] && continue
        [[ -z "$url" ]] && continue

        # Clean fields
        html_bytes=$(echo "$html_bytes" | tr -d ', ' | xargs)
        som_bytes=$(echo "$som_bytes" | tr -d ', ' | xargs)
        ratio=$(echo "$ratio" | tr -d 'x ' | xargs)
        grade=$(echo "$grade" | xargs)
        elements=$(echo "$elements" | tr -d ', ' | xargs)
        interactive=$(echo "$interactive" | tr -d ', ' | xargs)
        fetch_ms=$(echo "$fetch_ms" | tr -d ', ' | xargs)
        parse_ms=$(echo "$parse_ms" | tr -d ', ' | xargs)
        status=$(echo "$status" | xargs)

        # Handle N/A ratio
        if [ "$ratio" = "N/A" ]; then
            ratio_json="null"
        else
            ratio_json="$ratio"
        fi

        if [ "$FIRST" = true ]; then
            FIRST=false
        else
            echo ','
        fi

        printf '    {"url": "%s", "html_bytes": %s, "som_bytes": %s, "ratio": %s, "grade": "%s", "elements": %s, "interactive": %s, "fetch_ms": %s, "parse_ms": %s, "status": "%s"}' \
            "$url" "${html_bytes:-0}" "${som_bytes:-0}" "${ratio_json:-null}" "${grade:-F}" "${elements:-0}" "${interactive:-0}" "${fetch_ms:-0}" "${parse_ms:-0}" "${status:-error}"
    done < "$REPORT_MD"

    echo ''
    echo '  ]'
    echo '}'
} > "$RESULTS_JSON"

info "JSON results written to $RESULTS_JSON"

# ── Step 4: Summary ──
echo ""
info "Benchmark complete!"
echo "  Report:  $REPORT_MD"
echo "  JSON:    $RESULTS_JSON"
echo ""

# Print the summary table from the report (first table)
awk '/^## Summary/{found=1} found{print} /^$/{if(found) exit}' "$REPORT_MD"
