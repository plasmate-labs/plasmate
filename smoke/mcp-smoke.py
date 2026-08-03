#!/usr/bin/env python3
"""MCP integration smoke test.

Tests the full agent workflow: initialize, open_page, evaluate, click, close_page.
Runs against a local HTTP server with a test fixture to avoid network dependencies.
"""

import json
import http.server
import os
import selectors
import subprocess
import sys
import tempfile
import threading
import time

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

RPC_TIMEOUT_SECONDS = float(os.environ.get("MCP_SMOKE_RPC_TIMEOUT", "20"))
OVERALL_TIMEOUT_SECONDS = float(os.environ.get("MCP_SMOKE_OVERALL_TIMEOUT", "120"))

if RPC_TIMEOUT_SECONDS <= 0 or OVERALL_TIMEOUT_SECONDS <= 0:
    raise ValueError("MCP smoke timeouts must be positive")


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
    server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
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
    base_url = f"http://127.0.0.1:{server.server_port}"

    stderr_log = tempfile.TemporaryFile(mode="w+")
    child_env = os.environ.copy()
    child_env["PLASMATE_UNSAFE_ALLOW_PRIVATE_NETWORK"] = "1"
    proc = subprocess.Popen(
        [binary, "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=stderr_log,
        text=True,
        bufsize=1,
        env=child_env,
    )
    selector = selectors.DefaultSelector()
    selector.register(proc.stdout, selectors.EVENT_READ)
    overall_deadline = time.monotonic() + OVERALL_TIMEOUT_SECONDS

    _id = [0]

    def stderr_tail():
        stderr_log.flush()
        stderr_log.seek(0, os.SEEK_END)
        size = stderr_log.tell()
        stderr_log.seek(max(0, size - 4000))
        tail = stderr_log.read().strip()
        stderr_log.seek(0, os.SEEK_END)
        return tail

    def rpc(method, params=None):
        if proc.poll() is not None:
            raise RuntimeError(
                f"MCP process exited before {method} (status {proc.returncode}): "
                f"{stderr_tail() or '<no stderr>'}"
            )

        _id[0] += 1
        req = {"jsonrpc": "2.0", "id": _id[0], "method": method}
        if params is not None:
            req["params"] = params

        try:
            proc.stdin.write(json.dumps(req) + "\n")
            proc.stdin.flush()
        except (BrokenPipeError, OSError) as exc:
            raise RuntimeError(
                f"MCP process closed stdin during {method}: "
                f"{stderr_tail() or '<no stderr>'}"
            ) from exc

        deadline = min(
            time.monotonic() + RPC_TIMEOUT_SECONDS,
            overall_deadline,
        )
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(
                    f"Timed out waiting for MCP response to {method} "
                    f"(request id {_id[0]}): {stderr_tail() or '<no stderr>'}"
                )

            if not selector.select(remaining):
                continue

            line = proc.stdout.readline()
            if not line:
                status = proc.poll()
                raise RuntimeError(
                    f"MCP stdout closed during {method}"
                    + (f" (status {status})" if status is not None else "")
                    + f": {stderr_tail() or '<no stderr>'}"
                )
            line = line.strip()
            if not line:
                continue
            try:
                resp = json.loads(line)
                if resp.get("id") == _id[0]:
                    return resp
            except json.JSONDecodeError:
                continue

    def notify(method, params=None):
        """Send an MCP notification, which intentionally has no response id."""
        if proc.poll() is not None:
            raise RuntimeError(
                f"MCP process exited before {method} (status {proc.returncode}): "
                f"{stderr_tail() or '<no stderr>'}"
            )

        req = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            req["params"] = params
        try:
            proc.stdin.write(json.dumps(req) + "\n")
            proc.stdin.flush()
        except (BrokenPipeError, OSError) as exc:
            raise RuntimeError(
                f"MCP process closed stdin during {method}: "
                f"{stderr_tail() or '<no stderr>'}"
            ) from exc

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

    exit_code = 1

    try:
        print("=== MCP Integration Smoke Test ===\n")

        # 1. Initialize
        print("1. Initialize")
        r = rpc("initialize", {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "smoke-test", "version": "1.0"},
        })
        check("server responds", r.get("result") is not None)
        check("server name", r["result"]["serverInfo"]["name"] == "plasmate")
        check("Codex protocol version", r["result"]["protocolVersion"] == "2025-06-18")
        notify("notifications/initialized")

        # 2. repeated fetch_page calls on one long-lived transport
        print("\n2. fetch_page (10 consecutive stateless calls)")
        fetch_responses = [
            rpc("tools/call", {
                "name": "fetch_page",
                "arguments": {"url": f"{base_url}/", "javascript": False},
            })
            for _ in range(10)
        ]
        check(
            "10 calls keep transport healthy",
            all(not response.get("result", {}).get("isError", False) for response in fetch_responses),
        )
        r = fetch_responses[-1]
        som = json.loads(r["result"]["content"][0]["text"])
        check("title", som.get("title") == "MCP Smoke Test", f'got: {som.get("title")}')
        check("has regions", len(som.get("regions", [])) > 0)
        all_elements = []
        for region in som.get("regions", []):
            all_elements.extend(region.get("elements", []))
        check("has elements", len(all_elements) > 0, f"count: {len(all_elements)}")

        # 3. extract_text (stateless)
        print("\n3. extract_text (stateless)")
        r = rpc("tools/call", {"name": "extract_text", "arguments": {"url": f"{base_url}/"}})
        text = r["result"]["content"][0]["text"]
        check("contains content", "test page" in text.lower(), f"got: {text[:100]}")

        # 4. open_page (stateful)
        print("\n4. open_page (stateful)")
        r = rpc("tools/call", {"name": "open_page", "arguments": {"url": f"{base_url}/", "trace": True}})
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
            click_result = {}
            if is_error or not click_text:
                check("click response", False, f"error={is_error} text={click_text[:200] if click_text else 'empty'} full={json.dumps(r)[:300]}")
            else:
                click_result = json.loads(click_text)
            check("navigated", "page2" in click_result.get("url", ""), f'url: {click_result.get("url")}')
            check("new title", click_result.get("title") == "Page Two", f'got: {click_result.get("title")}')

        # 7. privacy-safe trace and validation-only replay
        print("\n7. trace export and replay validation")
        r = rpc("tools/call", {"name": "trace_status", "arguments": {"session_id": session_id}})
        trace_status = json.loads(r["result"]["content"][0]["text"])
        trace_id = trace_status.get("trace_id")
        check("trace enabled", trace_status.get("enabled") is True)
        check("trace has actions", trace_status.get("retained_events", 0) >= 3)

        r = rpc("tools/call", {"name": "trace_export", "arguments": {"session_id": session_id}})
        trace_text = r["result"]["content"][0]["text"]
        trace_export = json.loads(trace_text)
        click_event = next((event for event in trace_export.get("events", []) if event.get("action") == "click"), None)
        check("trace schema", trace_export.get("schema") == "plasmate.trace.v1")
        check("trace omits evaluate source", "document.title" not in trace_text)
        check("trace has click", click_event is not None)
        if click_event:
            r = rpc("tools/call", {"name": "replay_validate", "arguments": {
                "session_id": session_id,
                "trace_id": trace_id,
                "sequence": click_event["sequence"],
                "confirmed": True,
            }})
            replay = json.loads(r["result"]["content"][0]["text"])
            check("stale replay refused", replay.get("status") == "refused", f"got: {replay}")
            check("replay has no side effects", replay.get("side_effects") is False)
            check("replay execution unavailable", replay.get("execution_available") is False)

        # 8. close_page returns the last in-memory trace before destruction
        print("\n8. close_page")
        r = rpc("tools/call", {"name": "close_page", "arguments": {"session_id": session_id}})
        close_payload = json.loads(r["result"]["content"][0]["text"])
        check("closed", close_payload.get("closed") is True)
        final_events = close_payload.get("final_trace", {}).get("events", [])
        check("close returns final trace", bool(final_events) and final_events[-1].get("action") == "close_page")

        # 9. A tool error is structured and does not poison the transport.
        print("\n9. tool error recovery")
        error_response = rpc("tools/call", {"name": "fetch_page", "arguments": {}})
        check("fetch error is structured", error_response.get("result", {}).get("isError") is True)
        healthy = rpc("tools/call", {"name": "cache_status", "arguments": {}})
        check("cache_status works after error", healthy.get("result", {}).get("isError") is not True)

        # 10. Closing the client pipe must let the stdio child exit cleanly.
        print("\n10. client disconnect")
        proc.stdin.close()
        proc.wait(timeout=5)
        check("stdio child exits on EOF", proc.returncode == 0, f"status: {proc.returncode}")

        print(f"\n=== Results: {passed} passed, {failed} failed ===")
        if failed == 0:
            print("PASS")
            exit_code = 0
    except Exception as exc:
        print(f"\nFATAL: {exc}", file=sys.stderr)
    finally:
        selector.close()
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)
        server.shutdown()
        server.server_close()
        stderr_log.close()

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
