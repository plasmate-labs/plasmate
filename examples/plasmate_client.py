"""
plasmate_client.py - AWP v0.1 Python client

Usage:
    client = PlasmateClient("ws://127.0.0.1:9222")
    await client.connect()
    session = await client.create_session()
    await client.navigate(session, "https://example.com")
    som = await client.observe(session)
    print(json.dumps(som, indent=2))
"""

import asyncio
import json
import uuid
import websockets


class PlasmateClient:
    def __init__(self, url: str = "ws://127.0.0.1:9222"):
        self.url = url
        self.ws = None
        self._pending = {}

    async def connect(self):
        self.ws = await websockets.connect(self.url)
        asyncio.create_task(self._reader())
        result = await self._request("awp.hello", {
            "client_name": "plasmate-python",
            "client_version": "0.1.0",
            "awp_version": "0.1"
        })
        return result

    async def create_session(self, **kwargs) -> str:
        result = await self._request("session.create", kwargs)
        return result["session_id"]

    async def close_session(self, session_id: str):
        return await self._request("session.close", {"session_id": session_id})

    async def navigate(self, session_id: str, url: str, timeout_ms: int = 15000):
        return await self._request("page.navigate", {
            "session_id": session_id,
            "url": url,
            "timeout_ms": timeout_ms
        })

    async def observe(self, session_id: str) -> dict:
        result = await self._request("page.observe", {"session_id": session_id})
        return result["som"]

    async def act(self, session_id: str, action: str, target: dict, value: str = None):
        intent = {"action": action, "target": target}
        if value is not None:
            intent["value"] = value
        return await self._request("page.act", {
            "session_id": session_id,
            "intent": intent
        })

    async def extract(self, session_id: str, fields: dict) -> dict:
        result = await self._request("page.extract", {
            "session_id": session_id,
            "fields": fields
        })
        return result["data"]

    async def _request(self, method: str, params: dict) -> dict:
        msg_id = str(uuid.uuid4())[:8]
        msg = {"id": msg_id, "type": "request", "method": method, "params": params}
        future = asyncio.get_event_loop().create_future()
        self._pending[msg_id] = future
        await self.ws.send(json.dumps(msg))
        result = await asyncio.wait_for(future, timeout=30)
        return result

    async def _reader(self):
        async for raw in self.ws:
            msg = json.loads(raw)
            if msg.get("type") == "response" and msg.get("id") in self._pending:
                future = self._pending.pop(msg["id"])
                if "error" in msg:
                    future.set_exception(
                        PlasmateError(msg["error"]["code"], msg["error"]["message"])
                    )
                else:
                    future.set_result(msg.get("result", {}))
            # Events (v0.2) would be handled here


class PlasmateError(Exception):
    def __init__(self, code: str, message: str):
        self.code = code
        super().__init__(f"[{code}] {message}")


# --- Quick test ---
async def main():
    client = PlasmateClient()
    await client.connect()
    session = await client.create_session()

    nav = await client.navigate(session, "https://news.ycombinator.com")
    print(f"Loaded {nav['url']} ({nav['html_bytes']} bytes HTML, {nav['load_ms']}ms)")

    som = await client.observe(session)
    print(f"SOM: {som['meta']['som_bytes']} bytes, "
          f"{som['meta']['element_count']} elements, "
          f"{som['meta']['interactive_count']} interactive")

    print(f"\nToken ratio: {som['meta']['html_bytes'] / som['meta']['som_bytes']:.1f}x reduction")

    links = await client.extract(session, {
        "stories": {"role": "link", "all": True, "props": ["text", "href"]}
    })
    for link in links.get("stories", [])[:5]:
        print(f"  - {link['text'][:60]}")

    await client.close_session(session)

if __name__ == "__main__":
    asyncio.run(main())
