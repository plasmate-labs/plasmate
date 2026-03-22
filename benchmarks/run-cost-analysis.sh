#!/bin/bash
# SOM Cost Analysis Benchmark
# Reproducible benchmark comparing HTML vs SOM token costs
# Run: ./benchmarks/run-cost-analysis.sh
# Requires: plasmate binary in PATH or target/release/plasmate

set -e

PLASMATE="${PLASMATE_BIN:-$(command -v plasmate 2>/dev/null || echo target/release/plasmate)}"
if [ ! -x "$PLASMATE" ]; then
  echo "Error: plasmate binary not found. Install with: cargo install plasmate"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
URLS_FILE="$SCRIPT_DIR/urls.txt"
OUTPUT="$SCRIPT_DIR/results-$(date +%Y-%m-%d).json"

echo "SOM Cost Analysis Benchmark"
echo "Plasmate: $($PLASMATE --version 2>/dev/null || echo 'unknown')"
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "URLs: $(wc -l < "$URLS_FILE" | tr -d ' ')"
echo "Output: $OUTPUT"
echo ""

echo '[]' > "$OUTPUT"
total=0
success=0

while IFS= read -r url; do
  [ -z "$url" ] && continue
  [[ "$url" == \#* ]] && continue
  total=$((total + 1))

  result=$("$PLASMATE" fetch "$url" 2>/dev/null | python3 -c "
import sys,json
try:
  d=json.load(sys.stdin)
  m=d.get('meta',{})
  html=m.get('html_bytes',0)
  som=m.get('som_bytes',0)
  ratio=html/max(som,1)
  print(json.dumps({
    'url': sys.argv[1],
    'html_bytes': html,
    'som_bytes': som,
    'html_tokens': html//4,
    'som_tokens': som//4,
    'ratio': round(ratio,1),
    'elements': m.get('element_count',0),
    'interactive': m.get('interactive_count',0)
  }))
except:
  print(json.dumps({'url': sys.argv[1], 'error': 'fetch_failed'}))
" "$url" 2>/dev/null)

  if echo "$result" | python3 -c "import sys,json; d=json.load(sys.stdin); exit(0 if 'error' not in d else 1)" 2>/dev/null; then
    ratio=$(echo "$result" | python3 -c "import sys,json; print(json.load(sys.stdin)['ratio'])")
    echo "  OK  ${ratio}x  $url"
    success=$((success + 1))
  else
    echo "  FAIL     $url"
  fi

  python3 -c "
import json, sys
with open(sys.argv[1],'r') as f: data=json.load(f)
data.append(json.loads(sys.argv[2]))
with open(sys.argv[1],'w') as f: json.dump(data,f,indent=2)
" "$OUTPUT" "$result"

done < "$URLS_FILE"

echo ""
echo "Done: $success/$total succeeded"
echo ""

# Print summary
python3 -c "
import json, sys
with open(sys.argv[1]) as f: data = json.load(f)
valid = [d for d in data if 'error' not in d]
if not valid:
    print('No valid results')
    sys.exit(1)

total_html = sum(d['html_tokens'] for d in valid)
total_som = sum(d['som_tokens'] for d in valid)
ratios = sorted([d['ratio'] for d in valid])
median = ratios[len(ratios)//2]

print(f'Sites analyzed: {len(valid)}')
print(f'Total HTML tokens: {total_html:,}')
print(f'Total SOM tokens:  {total_som:,}')
print(f'Overall compression: {total_html/total_som:.1f}x')
print(f'Median compression:  {median:.1f}x')
print(f'Token savings:       {(1-total_som/total_html)*100:.0f}%')
print()
savings_per_page = (total_html - total_som) / len(valid) * 2.50 / 1_000_000
print(f'Cost savings at GPT-4o (\$2.50/M input tokens):')
print(f'  Per page:    \${savings_per_page:.4f}')
print(f'  Per 1K pages: \${savings_per_page*1000:.2f}')
print(f'  Per 1M pages: \${savings_per_page*1e6:.0f}')
" "$OUTPUT"
