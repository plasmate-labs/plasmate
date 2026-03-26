#!/usr/bin/env python3
"""Fetch all pages in three formats and cache locally."""
import json
import os
import re
import subprocess
import sys
import time
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).parent.parent
CORPUS = ROOT / "corpus"
URLS_FILE = ROOT / "urls.json"

class SimpleMarkdownExtractor(HTMLParser):
    def __init__(self):
        super().__init__()
        self.parts = []
        self.skip = False
    def handle_starttag(self, tag, attrs):
        if tag in ("script", "style", "noscript"):
            self.skip = True
        if not self.skip:
            if tag in ("h1","h2","h3","h4","h5","h6"):
                self.parts.append("\n" + "#" * int(tag[1]) + " ")
            elif tag == "p":
                self.parts.append("\n\n")
            elif tag == "br":
                self.parts.append("\n")
            elif tag == "li":
                self.parts.append("\n- ")
    def handle_endtag(self, tag):
        if tag in ("script", "style", "noscript"):
            self.skip = False
    def handle_data(self, data):
        if not self.skip:
            self.parts.append(data)
    def get_text(self):
        text = "".join(self.parts).strip()
        text = re.sub(r"\n{3,}", "\n\n", text)
        text = re.sub(r"[ \t]+", " ", text)
        return text

def html_to_markdown(html_text):
    p = SimpleMarkdownExtractor()
    p.feed(html_text)
    return p.get_text()

def main():
    for d in ["html", "markdown", "som"]:
        (CORPUS / d).mkdir(parents=True, exist_ok=True)

    urls = json.loads(URLS_FILE.read_text())
    total = len(urls)
    print(f"=== WebTaskBench Corpus Builder ===")
    print(f"Fetching {total} pages in 3 formats...\n")

    for i, entry in enumerate(urls):
        uid = entry["id"]
        url = entry["url"]
        print(f"[{i+1}/{total}] {uid}: {url}")

        html_path = CORPUS / "html" / f"{uid}.html"
        md_path = CORPUS / "markdown" / f"{uid}.md"
        som_path = CORPUS / "som" / f"{uid}.json"

        if html_path.exists() and som_path.exists() and md_path.exists():
            print(f"  SKIP (cached)")
            continue

        # 1. Raw HTML
        try:
            r = subprocess.run(
                ["curl", "-sSL", "--max-time", "20",
                 "-H", "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
                 url],
                capture_output=True, text=True, timeout=25
            )
            if r.returncode == 0 and len(r.stdout) > 100:
                html_path.write_text(r.stdout)
                print(f"  HTML: {len(r.stdout):,} bytes")
            else:
                print(f"  HTML: FAILED (rc={r.returncode}, {len(r.stdout)} bytes)")
                continue
        except Exception as e:
            print(f"  HTML: ERROR {e}")
            continue

        # 2. Markdown
        try:
            html_text = html_path.read_text(errors="replace")
            md_text = html_to_markdown(html_text)
            md_path.write_text(md_text)
            print(f"  MD:   {len(md_text):,} chars")
        except Exception as e:
            print(f"  MD:   ERROR {e}")

        # 3. SOM via Plasmate
        try:
            plasmate = os.path.expanduser("~/Git/plasmate/target/release/plasmate")
            env = os.environ.copy()
            env["PLASMATE_ICU_DATA"] = "/tmp/icu74/icudt74l.dat"
            r = subprocess.run(
                [plasmate, "fetch", url],
                capture_output=True, text=True, timeout=30, env=env
            )
            if r.returncode == 0 and r.stdout.startswith("{"):
                som_path.write_text(r.stdout)
                print(f"  SOM:  {len(r.stdout):,} bytes")
            else:
                print(f"  SOM:  FAILED (rc={r.returncode})")
        except Exception as e:
            print(f"  SOM:  ERROR {e}")

        time.sleep(1)

    html_count = len(list((CORPUS / "html").glob("*.html")))
    md_count = len(list((CORPUS / "markdown").glob("*.md")))
    som_count = len(list((CORPUS / "som").glob("*.json")))
    print(f"\nDone. HTML: {html_count}, Markdown: {md_count}, SOM: {som_count}")

if __name__ == "__main__":
    main()
