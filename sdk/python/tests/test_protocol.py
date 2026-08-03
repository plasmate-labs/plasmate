"""Protocol lifecycle regression tests for the Python SDK clients."""

import asyncio
import json
import os
from pathlib import Path

from plasmate.client import AsyncPlasmate, Plasmate


def _fixture(tmp_path: Path) -> str:
    path = tmp_path / "mcp-fixture.py"
    path.write_text(
        '''#!/usr/bin/env python3
import json
import sys

def send(value):
    sys.stdout.write(json.dumps(value, separators=(",", ":")) + "\\n")
    sys.stdout.flush()

for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request.get("id"), "result": {"protocolVersion": "2024-11-05"}})
    elif method == "notifications/initialized":
        if "id" in request:
            send({"jsonrpc": "2.0", "id": request["id"], "result": {"stale": True}})
    elif method == "tools/call":
        send({"jsonrpc": "2.0", "method": "notifications/progress", "params": {"progress": 1}})
        send({"jsonrpc": "2.0", "id": request.get("id") + 1000, "result": {"stale": True}})
        send({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {"content": [{"type": "text", "text": json.dumps({"ok": True})}]},
        })
''',
        encoding="utf-8",
    )
    os.chmod(path, 0o755)
    return str(path)


def test_sync_first_tool_call_does_not_consume_notification_response(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    process = None
    try:
        assert client._call_tool("fixture_tool", {}) == {"ok": True}
        process = client._process
    finally:
        client.close()
    assert process is not None
    assert process.poll() is not None


def test_async_first_tool_call_does_not_consume_notification_response(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        process = None
        try:
            assert await client._call_tool("fixture_tool", {}) == {"ok": True}
            process = client._process
        finally:
            await client.close()
        assert process is not None
        assert process.returncode is not None

    asyncio.run(exercise())
