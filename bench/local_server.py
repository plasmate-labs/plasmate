#!/usr/bin/env python3
"""Local HTTP server for benchmarking. Serves pre-fetched HTML pages.
This matches Lightpanda's benchmark methodology (local server, no network latency)."""

import http.server
import json
import os
import sys
import threading

PAGES_DIR = os.path.join(os.path.dirname(__file__), "pages")

class BenchHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        # Map /1, /2, /3... to page files
        path = self.path.strip("/")
        if path == "":
            path = "index"

        filepath = os.path.join(PAGES_DIR, f"{path}.html")
        if os.path.exists(filepath):
            with open(filepath, "rb") as f:
                content = f.read()
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(content)))
            self.end_headers()
            self.wfile.write(content)
        else:
            # Generate a simple page
            html = f"<html><body><h1>Page {path}</h1><p>Content for page {path}</p></body></html>"
            content = html.encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(content)))
            self.end_headers()
            self.wfile.write(content)

    def log_message(self, format, *args):
        pass  # Suppress logging for clean benchmark output

def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
    server = http.server.HTTPServer(("127.0.0.1", port), BenchHandler)
    print(f"Bench server on http://127.0.0.1:{port}")
    server.serve_forever()

if __name__ == "__main__":
    main()
