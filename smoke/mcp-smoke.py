#!/usr/bin/env python3
"""MCP integration smoke test.

Tests the full agent workflow: initialize, open_page, evaluate, click, close_page.
Runs against a local HTTP server with a test fixture to avoid network dependencies.
"""

import subprocess
import json
import sys
import http.server
import threading
import os

FIXTURE_HTML = """<!doctype html>
<html>
<head><title>MCP Smoke Test</title></head>
<body>
  <h1>Welcome</h1>
  <p>This is a test page for MCP integration.</p>
  <a href="/page2">Go to page 2</a>
  <button id="btn">Click me</button>
</body>
</html>"""

FIXTURE_PAGE2 = """<!doctype html>
<html>
<head><title>Page Two</title></head>
<body>
  <h1>Page 2</h1>
  <p>You navigated here via click.</p>
  <a href="/">Back home</a>
</body>
</html>"""


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/page2":
            html = FIXTURE_PAGE2
        else:
            html = FIXTURE_HTML
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.end_headers()
        self.wfile.write(html.encode())

    def log_message(self, *args):
        pass  # Silence logs


def start_server():
    server = http.server.HTTPServer(("127.0.0.1", 8766), Handler)
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    return server


def main():
    # Find binary
    binary = os.environ.get("PLASMATE_BIN", "./target/release/plasmate")
    if not os.path.exists(binary):
        print(f"Binary not found: {binary}")
        sys.exit(1)

    server = start_server()

    proc = subprocess.Popen(
        [binary, "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    _id = [0]

    def rpc(method, params=None):
        _id[0] += 1
        req = {"jsonrpc": "2.0", "id": _id[0], "method": method}
        if params:
            req["params"] = params
        proc.stdin.write((json.dumps(req) + "\n").encode())
        proc.stdin.flush()
        while True:
            line = proc.stdout.readline().decode().strip()
            if not line:
                continue
            try:
                resp = json.loads(line)
                if resp.get("id") == _id[0]:
                    return resp
            except json.JSONDecodeError:
                continue

    passed = 0
    failed = 0

    def check(name, condition, detail=""):
        nonlocal passed, failed
        if condition:
            print(f"  PASS: {name}")
            passed += 1
        else:
            print(f"  FAIL: {name} {detail}")
            failed += 1

    print("=== MCP Integration Smoke Test ===\n")

    # 1. Initialize
    print("1. Initialize")
    r = rpc("initialize", {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "smoke-test", "version": "1.0"},
    })
    check("server responds", r.get("result") is not None)
    check("server name", r["result"]["serverInfo"]["name"] == "plasmate")

    # 2. fetch_page (stateless)
    print("\n2. fetch_page (stateless)")
    r = rpc("tools/call", {"name": "fetch_page", "arguments": {"url": "http://127.0.0.1:8766/"}})
    som = json.loads(r["result"]["content"][0]["text"])
    check("title", som.get("title") == "MCP Smoke Test", f'got: {som.get("title")}')
    check("has regions", len(som.get("regions", [])) > 0)
    all_elements = []
    for region in som.get("regions", []):
        all_elements.extend(region.get("elements", []))
    check("has elements", len(all_elements) > 0, f"count: {len(all_elements)}")

    # 3. extract_text (stateless)
    print("\n3. extract_text (stateless)")
    r = rpc("tools/call", {"name": "extract_text", "arguments": {"url": "http://127.0.0.1:8766/"}})
    text = r["result"]["content"][0]["text"]
    check("contains content", "test page" in text.lower(), f"got: {text[:100]}")

    # 4. open_page (stateful)
    print("\n4. open_page (stateful)")
    r = rpc("tools/call", {"name": "open_page", "arguments": {"url": "http://127.0.0.1:8766/"}})
    page = json.loads(r["result"]["content"][0]["text"])
    session_id = page.get("session_id")
    check("has session_id", session_id is not None)
    check("title matches", page.get("title") == "MCP Smoke Test", f'got: {page.get("title")}')

    # 5. evaluate
    print("\n5. evaluate")
    r = rpc("tools/call", {"name": "evaluate", "arguments": {
        "session_id": session_id,
        "expression": "document.title",
    }})
    result = json.loads(r["result"]["content"][0]["text"])
    check("returns title", result.get("result") == "MCP Smoke Test", f'got: {result.get("result")}')

    # 6. click (navigate to page 2)
    print("\n6. click (navigation)")
    all_elements = []
    for region in page.get("regions", []):
        all_elements.extend(region.get("elements", []))
    link = next((e for e in all_elements if e.get("text") == "Go to page 2"), None)
    check("found link element", link is not None)
    if link:
        r = rpc("tools/call", {"name": "click", "arguments": {
            "session_id": session_id,
            "element_id": link["id"],
        }})
        click_text = r.get("result", {}).get("content", [{}])[0].get("text", "")
        is_error = r.get("result", {}).get("isError", False)
        if is_error or not click_text:
            check("click response", False, f"error={is_error} text={click_text[:200] if click_text else 'empty'} full={json.dumps(r)[:300]}")
        else:
            click_result = json.loads(click_text)
        check("navigated", "page2" in click_result.get("url", ""), f'url: {click_result.get("url")}')
        check("new title", click_result.get("title") == "Page Two", f'got: {click_result.get("title")}')

    # 7. close_page
    print("\n7. close_page")
    r = rpc("tools/call", {"name": "close_page", "arguments": {"session_id": session_id}})
    check("closed", r.get("result") is not None)

    # Summary
    proc.terminate()
    server.shutdown()

    print(f"\n=== Results: {passed} passed, {failed} failed ===")
    if failed > 0:
        sys.exit(1)
    print("PASS")


if __name__ == "__main__":
    main()
