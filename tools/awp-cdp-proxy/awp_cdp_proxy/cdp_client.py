"""Minimal CDP client for communicating with Chrome/Chromium.

Connects to Chrome's DevTools Protocol via WebSocket, sends commands,
and dispatches responses back to callers.
"""

from __future__ import annotations

import asyncio
import json
import logging
from typing import Any, Optional

import httpx
import websockets

logger = logging.getLogger(__name__)


class CDPClient:
    """Low-level Chrome DevTools Protocol WebSocket client."""

    def __init__(self, browser_ws_url: str):
        self.browser_ws_url = browser_ws_url
        self.ws: Optional[websockets.WebSocketClientProtocol] = None
        self.msg_id = 0
        self.pending: dict[int, asyncio.Future] = {}
        self.event_handlers: dict[str, list] = {}
        self._receive_task: Optional[asyncio.Task] = None

    async def connect(self) -> None:
        """Connect to a Chrome page target via CDP WebSocket."""
        self.ws = await websockets.connect(
            self.browser_ws_url,
            max_size=50 * 1024 * 1024,  # 50MB for large DOMs
        )
        self._receive_task = asyncio.create_task(self._receive_loop())
        logger.info(f"Connected to CDP: {self.browser_ws_url}")

    async def disconnect(self) -> None:
        """Close the CDP WebSocket connection."""
        if self._receive_task:
            self._receive_task.cancel()
            try:
                await self._receive_task
            except asyncio.CancelledError:
                pass
        if self.ws:
            await self.ws.close()
            self.ws = None

    async def send(self, method: str, params: Optional[dict] = None, timeout: float = 30) -> dict:
        """Send a CDP command and wait for the response."""
        if not self.ws:
            raise RuntimeError("Not connected to CDP")

        self.msg_id += 1
        msg_id = self.msg_id
        msg = {"id": msg_id, "method": method, "params": params or {}}

        future: asyncio.Future = asyncio.get_running_loop().create_future()
        self.pending[msg_id] = future

        await self.ws.send(json.dumps(msg))
        logger.debug(f"CDP -> {method} (id={msg_id})")

        try:
            result = await asyncio.wait_for(future, timeout=timeout)
        except asyncio.TimeoutError:
            self.pending.pop(msg_id, None)
            raise TimeoutError(f"CDP command {method} timed out after {timeout}s")

        if "error" in result:
            err = result["error"]
            raise CDPError(err.get("code", -1), err.get("message", "Unknown CDP error"))

        return result.get("result", {})

    def on_event(self, method: str, handler) -> None:
        """Register a handler for a CDP event."""
        self.event_handlers.setdefault(method, []).append(handler)

    async def _receive_loop(self) -> None:
        """Background task that receives CDP messages and dispatches them."""
        try:
            async for message in self.ws:
                data = json.loads(message)

                # Response to a pending command
                if "id" in data and data["id"] in self.pending:
                    self.pending.pop(data["id"]).set_result(data)

                # Event
                elif "method" in data:
                    method = data["method"]
                    for handler in self.event_handlers.get(method, []):
                        try:
                            handler(data.get("params", {}))
                        except Exception:
                            logger.exception(f"Error in CDP event handler for {method}")
        except websockets.exceptions.ConnectionClosed:
            logger.info("CDP connection closed")
        except asyncio.CancelledError:
            pass


class CDPError(Exception):
    """Error returned by the Chrome DevTools Protocol."""

    def __init__(self, code: int, message: str):
        self.code = code
        super().__init__(f"CDP error {code}: {message}")


async def connect_to_target(cdp_url: str) -> CDPClient:
    """Connect to a Chrome instance and open a new target (tab).

    Args:
        cdp_url: The Chrome debugging URL, e.g. ws://localhost:9222.
                 Can be a WebSocket URL or HTTP URL.

    Returns:
        A connected CDPClient attached to a new page target.
    """
    # Derive HTTP URL for the /json endpoints
    http_url = cdp_url.replace("ws://", "http://").replace("wss://", "https://")
    # Strip any path
    if "//" in http_url:
        parts = http_url.split("/")
        http_url = "/".join(parts[:3])

    async with httpx.AsyncClient() as client:
        # Create a new target (tab)
        resp = await client.put(f"{http_url}/json/new")
        if resp.status_code != 200:
            # Some Chrome versions use POST
            resp = await client.post(f"{http_url}/json/new")
        target_info = resp.json()

    ws_url = target_info.get("webSocketDebuggerUrl")
    if not ws_url:
        raise RuntimeError(f"Chrome did not return a webSocketDebuggerUrl: {target_info}")

    cdp = CDPClient(ws_url)
    await cdp.connect()

    # Enable required CDP domains
    await cdp.send("Page.enable")
    await cdp.send("DOM.enable")
    await cdp.send("Runtime.enable")

    return cdp


async def close_target(cdp_url: str, target_id: str) -> None:
    """Close a Chrome target (tab) by its target ID."""
    http_url = cdp_url.replace("ws://", "http://").replace("wss://", "https://")
    if "//" in http_url:
        parts = http_url.split("/")
        http_url = "/".join(parts[:3])

    async with httpx.AsyncClient() as client:
        await client.get(f"{http_url}/json/close/{target_id}")
