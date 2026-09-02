"""Tests for public SDK read-option forwarding."""

import asyncio
from unittest.mock import AsyncMock, Mock, call

from plasmate.client import AsyncPlasmate, Plasmate


def test_sync_read_methods_forward_selector() -> None:
    browser = Plasmate()
    calls = Mock(side_effect=[{}, "text", "https://example.test/a"])
    browser._call_tool = calls  # type: ignore[method-assign]

    assert browser.fetch_page("fixture", selector="main") == {}
    assert browser.extract_text("fixture", selector="content") == "text"
    assert browser.extract_links("fixture", selector="nav") == "https://example.test/a"

    assert calls.call_args_list == [
        call("fetch_page", {"url": "fixture", "selector": "main"}),
        call("extract_text", {"url": "fixture", "selector": "content"}),
        call("extract_links", {"url": "fixture", "selector": "nav"}),
    ]


def test_async_read_methods_forward_selector() -> None:
    async def exercise() -> None:
        browser = AsyncPlasmate()
        calls = AsyncMock(side_effect=[{}, "text", "https://example.test/a"])
        browser._call_tool = calls  # type: ignore[method-assign]

        assert await browser.fetch_page("fixture", selector="main") == {}
        assert await browser.extract_text("fixture", selector="content") == "text"
        assert await browser.extract_links("fixture", selector="nav") == "https://example.test/a"

        assert calls.call_args_list == [
            call("fetch_page", {"url": "fixture", "selector": "main"}),
            call("extract_text", {"url": "fixture", "selector": "content"}),
            call("extract_links", {"url": "fixture", "selector": "nav"}),
        ]

    asyncio.run(exercise())


def test_sync_type_text_forwards_session_target_and_append() -> None:
    browser = Plasmate()
    calls = Mock(side_effect=[{}, {}])
    browser._call_tool = calls  # type: ignore[method-assign]

    assert browser.type_text("s1", "e5", "hello") == {}
    assert browser.type_text("s1", "e5", "!", append=True) == {}

    assert calls.call_args_list == [
        call("type_text", {"session_id": "s1", "element_id": "e5", "text": "hello"}),
        call(
            "type_text",
            {"session_id": "s1", "element_id": "e5", "text": "!", "append": True},
        ),
    ]


def test_async_type_text_forwards_session_target_and_append() -> None:
    async def exercise() -> None:
        browser = AsyncPlasmate()
        calls = AsyncMock(side_effect=[{}, {}])
        browser._call_tool = calls  # type: ignore[method-assign]

        assert await browser.type_text("s1", "e5", "hello") == {}
        assert await browser.type_text("s1", "e5", "!", append=True) == {}

        assert calls.call_args_list == [
            call("type_text", {"session_id": "s1", "element_id": "e5", "text": "hello"}),
            call(
                "type_text",
                {"session_id": "s1", "element_id": "e5", "text": "!", "append": True},
            ),
        ]

    asyncio.run(exercise())
