"""Protocol lifecycle regression tests for the Python SDK clients."""

import asyncio
import json
import os
import threading
from pathlib import Path

import pytest

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
        tool_name = request.get("params", {}).get("name")
        if tool_name == "silent_tool":
            continue
        if tool_name == "empty_error":
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {"content": [], "isError": True},
            })
            continue
        if tool_name == "oversized_error":
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "content": [{"type": "text", "text": "x" * 5000}],
                    "isError": True,
                },
            })
            continue
        if tool_name == "unparseable_tool":
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "content": [{"type": "text", "text": "Fetching fixture...\\nnot-json"}],
                    "isError": False,
                },
            })
            continue
        content = {
            "type": "text",
            "text": "Plain text fixture"
            if tool_name == "extract_text"
            else json.dumps({"ok": True}),
        }
        send({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {"content": [content]},
        })
''',
        encoding="utf-8",
    )
    os.chmod(path, 0o755)
    return str(path)


def test_sync_first_tool_call_does_not_consume_notification_response(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        assert client._call_tool("fixture_tool", {}) == {"ok": True}
    finally:
        client.close()


def test_async_first_tool_call_does_not_consume_notification_response(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            assert await client._call_tool("fixture_tool", {}) == {"ok": True}
        finally:
            await client.close()

    asyncio.run(exercise())


def test_sync_extract_text_preserves_plain_text(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        assert client.extract_text("fixture") == "Plain text fixture"
    finally:
        client.close()


def test_async_extract_text_preserves_plain_text(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            assert await client.extract_text("fixture") == "Plain text fixture"
        finally:
            await client.close()

    asyncio.run(exercise())


def test_sync_empty_error_content_has_bounded_message(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        with pytest.raises(RuntimeError, match="Unknown error"):
            client._call_tool("empty_error", {})
    finally:
        client.close()


def test_async_empty_error_content_has_bounded_message(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            with pytest.raises(RuntimeError, match="Unknown error"):
                await client._call_tool("empty_error", {})
        finally:
            await client.close()

    asyncio.run(exercise())


def test_sync_oversized_tool_error_has_bounded_message(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        with pytest.raises(RuntimeError) as error:
            client._call_tool("oversized_error", {})
        assert str(error.value) == f"{'x' * 199}…"
    finally:
        client.close()


def test_async_oversized_tool_error_has_bounded_message(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            with pytest.raises(RuntimeError) as error:
                await client._call_tool("oversized_error", {})
            assert str(error.value) == f"{'x' * 199}…"
        finally:
            await client.close()

    asyncio.run(exercise())


def test_sync_unparseable_tool_text_fails_closed(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        with pytest.raises(RuntimeError, match="Tool returned unparseable output"):
            client._call_tool("unparseable_tool", {})
    finally:
        client.close()


def test_async_unparseable_tool_text_fails_closed(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            with pytest.raises(RuntimeError, match="Tool returned unparseable output"):
                await client._call_tool("unparseable_tool", {})
        finally:
            await client.close()

    asyncio.run(exercise())


def test_sync_response_timeout_is_bounded(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path), timeout=1)
    errors: list[Exception] = []
    thread = None

    def call_tool() -> None:
        try:
            client._call_tool("silent_tool", {})
        except Exception as exc:
            errors.append(exc)

    try:
        client._ensure_started()
        client._timeout = 0.05
        thread = threading.Thread(target=call_tool)
        thread.start()
        thread.join(timeout=0.2)
        assert not thread.is_alive(), "silent MCP tool call exceeded response timeout"
        assert len(errors) == 1
        assert str(errors[0]) == "Timed out waiting for response"
    finally:
        client.close()
        if thread is not None:
            thread.join(timeout=1)


def test_async_response_timeout_is_bounded(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path), timeout=1)
        try:
            await client._ensure_started()
            client._timeout = 0.05
            with pytest.raises(RuntimeError, match="Timed out waiting for response"):
                await asyncio.wait_for(
                    client._call_tool("silent_tool", {}),
                    timeout=0.2,
                )
        finally:
            await client.close()

    asyncio.run(exercise())
