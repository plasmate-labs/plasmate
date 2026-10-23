"""Protocol lifecycle regression tests for the Python SDK clients."""

import asyncio
import json
import os
from pathlib import Path
import subprocess
import threading
from unittest.mock import AsyncMock, Mock, patch

import pytest

from plasmate.client import AsyncPlasmate, Plasmate


def test_sync_start_closes_child_when_initialization_fails() -> None:
    client = Plasmate(binary="fixture")
    process = Mock()
    client._rpc = Mock(side_effect=RuntimeError("initialize failed"))  # type: ignore[method-assign]

    with patch("plasmate.client.subprocess.Popen", return_value=process):
        with pytest.raises(RuntimeError, match="initialize failed"):
            client._start()

    process.terminate.assert_called_once_with()
    assert client._process is None
    assert not client._initialized


def test_async_start_closes_child_when_initialization_fails() -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary="fixture")
        process = Mock()
        process.wait = AsyncMock(return_value=None)
        client._rpc = AsyncMock(side_effect=RuntimeError("initialize failed"))  # type: ignore[method-assign]

        with patch(
            "plasmate.client.asyncio.create_subprocess_exec",
            new=AsyncMock(return_value=process),
        ):
            with pytest.raises(RuntimeError, match="initialize failed"):
                await client._start()

        process.terminate.assert_called_once_with()
        assert client._process is None
        assert not client._initialized

    asyncio.run(exercise())


def test_sync_close_is_safe_when_process_already_exited() -> None:
    client = Plasmate()
    process = Mock()
    process.terminate.side_effect = ProcessLookupError
    client._process = process
    client._initialized = True

    client.close()

    assert client._process is None
    assert not client._initialized


def test_sync_close_ignores_exit_during_kill() -> None:
    client = Plasmate()
    process = Mock()
    process.wait.side_effect = [
        subprocess.TimeoutExpired(cmd="fixture", timeout=5),
        None,
    ]
    process.kill.side_effect = ProcessLookupError
    client._process = process

    client.close()

    assert client._process is None
    assert process.wait.call_count == 2


def test_sync_close_reaps_child_after_kill() -> None:
    client = Plasmate()
    process = Mock()
    process.wait.side_effect = [
        subprocess.TimeoutExpired(cmd="fixture", timeout=5),
        None,
    ]
    client._process = process

    client.close()

    process.kill.assert_called_once_with()
    assert process.wait.call_count == 2


def test_async_close_is_safe_when_process_already_exited() -> None:
    async def exercise() -> None:
        client = AsyncPlasmate()
        process = Mock()
        process.terminate.side_effect = ProcessLookupError
        client._process = process
        client._initialized = True

        await client.close()

        assert client._process is None
        assert not client._initialized

    asyncio.run(exercise())


def test_async_close_ignores_exit_during_kill() -> None:
    async def exercise() -> None:
        client = AsyncPlasmate()
        process = Mock()
        process.wait = AsyncMock(side_effect=[asyncio.TimeoutError, None])
        process.kill.side_effect = ProcessLookupError
        client._process = process

        await client.close()

        assert client._process is None
        assert process.wait.await_count == 2

    asyncio.run(exercise())


def test_async_close_reaps_child_after_kill() -> None:
    async def exercise() -> None:
        client = AsyncPlasmate()
        process = Mock()
        process.wait = AsyncMock(side_effect=[asyncio.TimeoutError, None])
        client._process = process

        await client.close()

        process.kill.assert_called_once_with()
        assert process.wait.await_count == 2

    asyncio.run(exercise())


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
        if tool_name == "out_of_order_tool":
            send({"jsonrpc": "2.0", "method": "notifications/progress", "params": {"done": False}})
            send({"jsonrpc": "2.0", "id": 999, "result": {"stale": True}})
            send({"jsonrpc": "2.0", "id": True, "result": {"stale": True}})
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {"content": [{"type": "text", "text": "{\\"ok\\":true}"}]},
            })
            continue
        if tool_name == "empty_error":
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {"content": [], "isError": True},
            })
            continue
        if tool_name == "structured_error":
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "content": [],
                    "isError": True,
                    "structuredContent": {"error": {"message": "structured diagnostic"}},
                },
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
        if tool_name == "image_then_tool":
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "content": [
                        {"type": "image", "data": "ignored"},
                        {"type": "text", "text": json.dumps({"ok": True})},
                    ]
                },
            })
            continue
        if tool_name == "split_text_tool":
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "content": [
                        {"type": "image", "data": "ignored"},
                        {"type": "text", "text": "progress"},
                        {"type": "text", "text": json.dumps({"ok": True})},
                    ]
                },
            })
            continue
        if tool_name == "structured_only":
            send({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "content": [],
                    "structuredContent": {"result": {"ok": True}},
                },
            })
            continue
        if tool_name == "extract_text":
            payload = "Plain text fixture"
        elif tool_name == "extract_links":
            payload = "https://example.test/a\\nhttps://example.test/b"
        else:
            payload = json.dumps({"ok": True})
        content = {
            "type": "text",
            "text": payload,
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


def test_sync_tool_matches_response_by_request_id(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        assert client._call_tool("out_of_order_tool", {}) == {"ok": True}
    finally:
        client.close()


def test_async_tool_matches_response_by_request_id(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            assert await client._call_tool("out_of_order_tool", {}) == {"ok": True}
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


def test_sync_extract_links_preserves_plain_text(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        assert client.extract_links("fixture") == (
            "https://example.test/a\nhttps://example.test/b"
        )
    finally:
        client.close()


def test_async_extract_links_preserves_plain_text(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            assert await client.extract_links("fixture") == (
                "https://example.test/a\nhttps://example.test/b"
            )
        finally:
            await client.close()

    asyncio.run(exercise())


def test_sync_tool_reads_text_after_non_text_content(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        assert client._call_tool("image_then_tool", {}) == {"ok": True}
    finally:
        client.close()


def test_async_tool_reads_text_after_non_text_content(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            assert await client._call_tool("image_then_tool", {}) == {"ok": True}
        finally:
            await client.close()

    asyncio.run(exercise())


def test_sync_tool_combines_text_content_blocks(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        assert client._call_tool("split_text_tool", {}) == {"ok": True}
    finally:
        client.close()


def test_async_tool_combines_text_content_blocks(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            assert await client._call_tool("split_text_tool", {}) == {"ok": True}
        finally:
            await client.close()

    asyncio.run(exercise())


def test_sync_tool_reads_structured_content_without_text(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        assert client._call_tool("structured_only", {}) == {"ok": True}
    finally:
        client.close()


def test_async_tool_reads_structured_content_without_text(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            assert await client._call_tool("structured_only", {}) == {"ok": True}
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


def test_sync_structured_error_without_text_uses_diagnostic(tmp_path: Path) -> None:
    client = Plasmate(binary=_fixture(tmp_path))
    try:
        with pytest.raises(RuntimeError, match="structured diagnostic"):
            client._call_tool("structured_error", {})
    finally:
        client.close()


def test_async_structured_error_without_text_uses_diagnostic(tmp_path: Path) -> None:
    async def exercise() -> None:
        client = AsyncPlasmate(binary=_fixture(tmp_path))
        try:
            with pytest.raises(RuntimeError, match="structured diagnostic"):
                await client._call_tool("structured_error", {})
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
