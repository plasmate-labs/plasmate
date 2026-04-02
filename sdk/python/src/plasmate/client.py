"""
Plasmate MCP client - communicates with `plasmate mcp` over stdio.

Provides both synchronous (Plasmate) and asynchronous (AsyncPlasmate) clients.
"""

from __future__ import annotations

import asyncio
import json
import subprocess
import threading
from typing import Any, Optional


def _extract_last_json(text: str) -> Any:
    """Extract the last complete JSON object from text that may contain mixed output.

    Three-phase approach — each phase is progressively more expensive but handles
    messier input:

    1. **Fast path** — ``json.loads`` on the stripped text directly.  Handles
       clean output from the MCP stdio transport (the common case).
    2. **Line scan** — try each non-empty line from the end.  Handles output
       where a progress/info line precedes the JSON on a separate line.
    3. **Brace walk** — scan for every ``{`` position right-to-left and call
       ``raw_decode`` at each.  Handles JSON embedded within a longer string
       (e.g. a log message that includes the serialised response).

    Returns the parsed value (usually a ``dict``), or ``None`` if no valid JSON
    object is found.  Malformed or absent JSON silently returns ``None`` rather
    than raising, so callers can decide whether to fall back or surface an error.
    """
    if not text:
        return None

    stripped = text.strip()

    # Phase 1: clean output — try the whole string first
    try:
        return json.loads(stripped)
    except (json.JSONDecodeError, ValueError):
        pass

    # Phase 2: JSON on its own line (common when Plasmate emits a status line
    # such as "Fetching …" before outputting the SOM)
    for line in reversed(stripped.splitlines()):
        line = line.strip()
        if line.startswith(("{", "[")):
            try:
                return json.loads(line)
            except (json.JSONDecodeError, ValueError):
                pass

    # Phase 3: JSON embedded inside a longer string — try raw_decode from each
    # '{' position, rightmost first, so we return the *last* complete object
    decoder = json.JSONDecoder()
    for pos in reversed([i for i, ch in enumerate(stripped) if ch == "{"]):
        try:
            value, _ = decoder.raw_decode(stripped, pos)
            return value
        except (json.JSONDecodeError, ValueError):
            continue

    return None


class Plasmate:
    """
    Synchronous Plasmate client.

    Spawns a `plasmate mcp` child process and communicates via JSON-RPC 2.0.

    Args:
        binary: Path to the plasmate binary. Default: "plasmate"
        timeout: Response timeout in seconds. Default: 30

    Example::

        browser = Plasmate()
        som = browser.fetch_page("https://example.com")
        print(som["title"])
        browser.close()
    """

    def __init__(self, binary: str = "plasmate", timeout: float = 30):
        self._binary = binary
        self._timeout = timeout
        self._process: Optional[subprocess.Popen] = None
        self._next_id = 1
        self._lock = threading.Lock()
        self._initialized = False

    def _ensure_started(self) -> None:
        if self._initialized:
            return
        self._start()

    def _start(self) -> None:
        self._process = subprocess.Popen(
            [self._binary, "mcp"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

        # Initialize MCP session
        self._rpc("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "plasmate-python-sdk", "version": "0.2.0"},
        })

        # Send initialized notification
        self._send({
            "jsonrpc": "2.0",
            "id": self._next_id,
            "method": "notifications/initialized",
        })
        self._next_id += 1
        # Read the notification response (if any)
        # Some servers respond, some don't - read with timeout
        self._initialized = True

    def close(self) -> None:
        """Shut down the plasmate process."""
        if self._process:
            self._process.terminate()
            try:
                self._process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._process.kill()
            self._process = None
        self._initialized = False

    def __enter__(self) -> "Plasmate":
        return self

    def __exit__(self, *args: Any) -> None:
        self.close()

    def __del__(self) -> None:
        self.close()

    # ---- Transport ----

    def _send(self, request: dict) -> None:
        if not self._process or not self._process.stdin:
            raise RuntimeError("Plasmate process is not running")
        line = json.dumps(request) + "\n"
        self._process.stdin.write(line.encode())
        self._process.stdin.flush()

    def _read_response(self) -> dict:
        if not self._process or not self._process.stdout:
            raise RuntimeError("Plasmate process is not running")
        while True:
            line = self._process.stdout.readline()
            if not line:
                raise RuntimeError("Plasmate process closed unexpectedly")
            line = line.decode().strip()
            if not line:
                continue
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue  # Skip non-JSON lines (tracing output)

    def _rpc(self, method: str, params: Any = None) -> Any:
        with self._lock:
            request_id = self._next_id
            self._next_id += 1
            request = {"jsonrpc": "2.0", "id": request_id, "method": method}
            if params is not None:
                request["params"] = params
            self._send(request)
            response = self._read_response()

        if "error" in response and response["error"]:
            raise RuntimeError(response["error"]["message"])
        return response.get("result")

    def _call_tool(self, name: str, arguments: dict) -> Any:
        self._ensure_started()
        result = self._rpc("tools/call", {"name": name, "arguments": arguments})

        if result and result.get("isError"):
            msg = result.get("content", [{}])[0].get("text", "Unknown error")
            raise RuntimeError(msg)

        text = result.get("content", [{}])[0].get("text", "")
        if not text:
            return None

        return _extract_last_json(text)

    # ---- Stateless Tools ----

    def fetch_page(
        self,
        url: str,
        *,
        budget: Optional[int] = None,
        javascript: bool = True,
    ) -> dict:
        """
        Fetch a page and return its Semantic Object Model.

        Args:
            url: URL to fetch
            budget: Maximum output tokens (SOM will be truncated)
            javascript: Enable JS execution (default: True)

        Returns:
            SOM dict with title, url, regions, and meta
        """
        args: dict[str, Any] = {"url": url}
        if budget is not None:
            args["budget"] = budget
        if not javascript:
            args["javascript"] = False
        return self._call_tool("fetch_page", args)

    def extract_text(
        self,
        url: str,
        *,
        max_chars: Optional[int] = None,
    ) -> str:
        """
        Fetch a page and return clean, readable text only.

        Args:
            url: URL to fetch
            max_chars: Maximum characters to return

        Returns:
            Clean text content
        """
        args: dict[str, Any] = {"url": url}
        if max_chars is not None:
            args["max_chars"] = max_chars
        return self._call_tool("extract_text", args)

    # ---- Stateful Tools ----

    def open_page(self, url: str) -> dict:
        """
        Open a page in a persistent browser session.

        Args:
            url: URL to open

        Returns:
            Dict with session_id and som
        """
        return self._call_tool("open_page", {"url": url})

    def evaluate(self, session_id: str, expression: str) -> Any:
        """
        Execute JavaScript in the page context.

        Args:
            session_id: Session ID from open_page
            expression: JavaScript expression to evaluate

        Returns:
            The evaluation result
        """
        return self._call_tool("evaluate", {
            "session_id": session_id,
            "expression": expression,
        })

    def click(self, session_id: str, element_id: str) -> dict:
        """
        Click an element by its SOM element ID.

        Args:
            session_id: Session ID from open_page
            element_id: Element ID from SOM (e.g. 'e5')

        Returns:
            Updated SOM after the click
        """
        return self._call_tool("click", {
            "session_id": session_id,
            "element_id": element_id,
        })

    def close_page(self, session_id: str) -> None:
        """
        Close a browser session and free resources.

        Args:
            session_id: Session ID to close
        """
        self._call_tool("close_page", {"session_id": session_id})


class AsyncPlasmate:
    """
    Async Plasmate client.

    Same API as Plasmate but with async/await support.

    Example::

        async with AsyncPlasmate() as browser:
            som = await browser.fetch_page("https://example.com")
            print(som["title"])
    """

    def __init__(self, binary: str = "plasmate", timeout: float = 30):
        self._binary = binary
        self._timeout = timeout
        self._process: Optional[asyncio.subprocess.Process] = None
        self._next_id = 1
        self._lock = asyncio.Lock()
        self._initialized = False

    async def _ensure_started(self) -> None:
        if self._initialized:
            return
        await self._start()

    async def _start(self) -> None:
        self._process = await asyncio.create_subprocess_exec(
            self._binary, "mcp",
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )

        await self._rpc("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "plasmate-python-sdk", "version": "0.2.0"},
        })

        await self._send({
            "jsonrpc": "2.0",
            "id": self._next_id,
            "method": "notifications/initialized",
        })
        self._next_id += 1
        self._initialized = True

    async def close(self) -> None:
        """Shut down the plasmate process."""
        if self._process:
            self._process.terminate()
            try:
                await asyncio.wait_for(self._process.wait(), timeout=5)
            except asyncio.TimeoutError:
                self._process.kill()
            self._process = None
        self._initialized = False

    async def __aenter__(self) -> "AsyncPlasmate":
        return self

    async def __aexit__(self, *args: Any) -> None:
        await self.close()

    async def _send(self, request: dict) -> None:
        if not self._process or not self._process.stdin:
            raise RuntimeError("Plasmate process is not running")
        line = json.dumps(request) + "\n"
        self._process.stdin.write(line.encode())
        await self._process.stdin.drain()

    async def _read_response(self) -> dict:
        if not self._process or not self._process.stdout:
            raise RuntimeError("Plasmate process is not running")
        while True:
            line = await self._process.stdout.readline()
            if not line:
                raise RuntimeError("Plasmate process closed unexpectedly")
            text = line.decode().strip()
            if not text:
                continue
            try:
                return json.loads(text)
            except json.JSONDecodeError:
                continue

    async def _rpc(self, method: str, params: Any = None) -> Any:
        async with self._lock:
            request_id = self._next_id
            self._next_id += 1
            request = {"jsonrpc": "2.0", "id": request_id, "method": method}
            if params is not None:
                request["params"] = params
            await self._send(request)
            response = await self._read_response()

        if "error" in response and response["error"]:
            raise RuntimeError(response["error"]["message"])
        return response.get("result")

    async def _call_tool(self, name: str, arguments: dict) -> Any:
        await self._ensure_started()
        result = await self._rpc("tools/call", {"name": name, "arguments": arguments})

        if result and result.get("isError"):
            msg = result.get("content", [{}])[0].get("text", "Unknown error")
            raise RuntimeError(msg)

        text = result.get("content", [{}])[0].get("text", "")
        if not text:
            return None

        return _extract_last_json(text)

    async def fetch_page(self, url: str, *, budget: Optional[int] = None, javascript: bool = True) -> dict:
        """Fetch a page and return its Semantic Object Model."""
        args: dict[str, Any] = {"url": url}
        if budget is not None:
            args["budget"] = budget
        if not javascript:
            args["javascript"] = False
        return await self._call_tool("fetch_page", args)

    async def extract_text(self, url: str, *, max_chars: Optional[int] = None) -> str:
        """Fetch a page and return clean, readable text only."""
        args: dict[str, Any] = {"url": url}
        if max_chars is not None:
            args["max_chars"] = max_chars
        return await self._call_tool("extract_text", args)

    async def open_page(self, url: str) -> dict:
        """Open a page in a persistent browser session."""
        return await self._call_tool("open_page", {"url": url})

    async def evaluate(self, session_id: str, expression: str) -> Any:
        """Execute JavaScript in the page context."""
        return await self._call_tool("evaluate", {
            "session_id": session_id,
            "expression": expression,
        })

    async def click(self, session_id: str, element_id: str) -> dict:
        """Click an element by its SOM element ID. Returns updated SOM."""
        return await self._call_tool("click", {
            "session_id": session_id,
            "element_id": element_id,
        })

    async def close_page(self, session_id: str) -> None:
        """Close a browser session and free resources."""
        await self._call_tool("close_page", {"session_id": session_id})
