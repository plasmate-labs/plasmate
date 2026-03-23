"""AWP WebSocket server that proxies to Chrome DevTools Protocol.

This is the entry point for the awp-cdp-proxy. It listens for AWP
WebSocket connections and translates them to CDP commands sent to
any Chrome/Chromium browser with remote debugging enabled.

Usage:
    awp-cdp-proxy --cdp-url ws://localhost:9222 --port 9333
"""

from __future__ import annotations

import argparse
import asyncio
import json
import logging
import signal
import sys
from typing import Optional

import websockets

from .translator import AWPTranslator

logger = logging.getLogger(__name__)


class AWPServer:
    """AWP WebSocket server that proxies requests to Chrome via CDP."""

    def __init__(
        self,
        cdp_url: str = "ws://localhost:9222",
        host: str = "127.0.0.1",
        port: int = 9333,
    ):
        self.cdp_url = cdp_url
        self.host = host
        self.port = port
        self.clients: set = set()

    async def start(self) -> None:
        """Start the AWP server."""
        logger.info(f"AWP-CDP Proxy listening on ws://{self.host}:{self.port}")
        logger.info(f"CDP target: {self.cdp_url}")
        logger.info("Waiting for AWP client connections...")

        async with websockets.serve(self.handle_client, self.host, self.port):
            await asyncio.Future()  # run forever

    async def handle_client(self, websocket) -> None:
        """Handle an AWP client connection."""
        addr = websocket.remote_address
        logger.info(f"Client connected from {addr}")
        self.clients.add(websocket)

        translator = AWPTranslator(self.cdp_url)
        try:
            async for message in websocket:
                try:
                    request = json.loads(message)
                except json.JSONDecodeError:
                    error_response = {
                        "id": "",
                        "type": "response",
                        "error": {
                            "code": "INVALID_REQUEST",
                            "message": "Invalid JSON",
                        },
                    }
                    await websocket.send(json.dumps(error_response))
                    continue

                logger.debug(f"<- {request.get('method', '?')} (id={request.get('id', '?')})")
                response = await translator.handle(request)
                logger.debug(f"-> {response.get('type', '?')} (id={response.get('id', '?')})")

                await websocket.send(json.dumps(response))

        except websockets.exceptions.ConnectionClosed:
            logger.info(f"Client disconnected: {addr}")
        except Exception:
            logger.exception(f"Error handling client {addr}")
        finally:
            self.clients.discard(websocket)
            await translator.cleanup()


def main() -> None:
    """CLI entry point for awp-cdp-proxy."""
    parser = argparse.ArgumentParser(
        description="AWP-to-CDP Proxy: use the Agent Web Protocol with any Chrome browser",
    )
    parser.add_argument(
        "--cdp-url",
        default="ws://localhost:9222",
        help="Chrome DevTools Protocol WebSocket URL (default: ws://localhost:9222)",
    )
    parser.add_argument(
        "--host",
        default="127.0.0.1",
        help="Host to bind the AWP server to (default: 127.0.0.1)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=9333,
        help="Port for the AWP server (default: 9333)",
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Enable verbose (debug) logging",
    )
    args = parser.parse_args()

    log_level = logging.DEBUG if args.verbose else logging.INFO
    logging.basicConfig(
        level=log_level,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
        datefmt="%H:%M:%S",
    )

    server = AWPServer(
        cdp_url=args.cdp_url,
        host=args.host,
        port=args.port,
    )

    try:
        asyncio.run(server.start())
    except KeyboardInterrupt:
        logger.info("Shutting down...")


if __name__ == "__main__":
    main()
