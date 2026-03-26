#!/bin/bash
# Fetch all pages in three formats and cache them locally.
# Run once to build the corpus. All evaluations use these cached files.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."
CORPUS="$ROOT/corpus"
URLS="$ROOT/urls.json"

mkdir -p "$CORPUS/html" "$CORPUS/markdown" "$CORPUS/som"

echo "=== WebTaskBench Corpus Builder ==="
echo "Fetching pages in 3 formats..."
echo ""

TOTAL=$(python3 -c "import json; print(len(json.load(open('$URLS'))))")
COUNT=0

python3 -c "
import json, subprocess, sys, os, time

urls = json.load(open('$URLS'))
corpus = '$CORPUS'
total = len(urls)

for i, entry in enumerate(urls):
    uid = entry['id']
    url = entry['url']
    print(f'[{i+1}/{total}] {uid}: {url}')

    html_path = f'{corpus}/html/{uid}.html'
    md_path = f'{corpus}/markdown/{uid}.md'
    som_path = f'{corpus}/som/{uid}.json'

    # Skip if all three already exist
    if os.path.exists(html_path) and os.path.exists(som_path):
        print(f'  SKIP (cached)')
        continue

    # 1. Raw HTML
    try:
        r = subprocess.run(
            ['curl', '-sSL', '--max-time', '20',
             '-H', 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
             url],
            capture_output=True, text=True, timeout=25
        )
        if r.returncode == 0 and len(r.stdout) > 100:
            with open(html_path, 'w') as f:
                f.write(r.stdout)
            print(f'  HTML: {len(r.stdout):,} bytes')
        else:
            print(f'  HTML: FAILED (rc={r.returncode})')
            continue
    except Exception as e:
        print(f'  HTML: ERROR {e}')
        continue

    # 2. Markdown (via readability + turndown equivalent: just strip tags for now)
    # In production, use mozilla/readability. For corpus building, a simple approach works.
    try:
        r = subprocess.run(
            ['python3', '-c', '''
import sys
from html.parser import HTMLParser
class TextExtractor(HTMLParser):
    def __init__(self):
        super().__init__()
        self.text = []
        self.skip = False
    def handle_starttag(self, tag, attrs):
        if tag in ('script', 'style', 'noscript'): self.skip = True
        if tag in ('h1','h2','h3','h4','h5','h6'): self.text.append('\\n' + '#' * int(tag[1]) + ' ')
        elif tag == 'p': self.text.append('\\n\\n')
        elif tag == 'br': self.text.append('\\n')
        elif tag == 'li': self.text.append('\\n- ')
        elif tag == 'a':
            href = dict(attrs).get('href', '')
            if href: self.text.append('[')
    def handle_endtag(self, tag):
        if tag in ('script', 'style', 'noscript'): self.skip = False
        if tag == 'a': self.text.append(']')
    def handle_data(self, data):
        if not self.skip:
            self.text.append(data)
html = open(sys.argv[1]).read()
p = TextExtractor()
p.feed(html)
text = "".join(p.text).strip()
# Collapse whitespace
import re
text = re.sub(r"\\n{3,}", "\\n\\n", text)
text = re.sub(r"[ \\t]+", " ", text)
print(text)
''', html_path],
            capture_output=True, text=True, timeout=10
        )
        if r.returncode == 0:
            with open(md_path, 'w') as f:
                f.write(r.stdout)
            print(f'  MD:   {len(r.stdout):,} chars')
    except Exception as e:
        print(f'  MD:   ERROR {e}')

    # 3. SOM via Plasmate
    try:
        plasmate = os.path.expanduser('~/Git/plasmate/target/release/plasmate')
        env = os.environ.copy()
        env['PLASMATE_ICU_DATA'] = '/tmp/icu74/icudt74l.dat'
        r = subprocess.run(
            [plasmate, 'fetch', url],
            capture_output=True, text=True, timeout=30, env=env
        )
        if r.returncode == 0 and r.stdout.startswith('{'):
            with open(som_path, 'w') as f:
                f.write(r.stdout)
            print(f'  SOM:  {len(r.stdout):,} bytes')
        else:
            print(f'  SOM:  FAILED (rc={r.returncode})')
    except Exception as e:
        print(f'  SOM:  ERROR {e}')

    time.sleep(1)  # Be polite

print('\\nDone.')
"

echo ""
echo "Corpus saved to: $CORPUS"
echo "HTML files: $(ls "$CORPUS/html" 2>/dev/null | wc -l | tr -d ' ')"
echo "Markdown files: $(ls "$CORPUS/markdown" 2>/dev/null | wc -l | tr -d ' ')"
echo "SOM files: $(ls "$CORPUS/som" 2>/dev/null | wc -l | tr -d ' ')"
