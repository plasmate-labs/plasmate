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


def test_sync_extract_text_forwards_javascript_opt_out() -> None:
    browser = Plasmate()
    calls = Mock(return_value="text")
    browser._call_tool = calls  # type: ignore[method-assign]

    assert browser.extract_text("fixture", javascript=False) == "text"

    assert calls.call_args == call("extract_text", {"url": "fixture", "javascript": False})


def test_async_extract_text_forwards_javascript_opt_out() -> None:
    async def exercise() -> None:
        browser = AsyncPlasmate()
        calls = AsyncMock(return_value="text")
        browser._call_tool = calls  # type: ignore[method-assign]

        assert await browser.extract_text("fixture", javascript=False) == "text"

        calls.assert_awaited_once_with(
            "extract_text", {"url": "fixture", "javascript": False}
        )

    asyncio.run(exercise())


def test_sync_extract_links_forwards_javascript_opt_out() -> None:
    browser = Plasmate()
    calls = Mock(return_value="https://example.test/a")
    browser._call_tool = calls  # type: ignore[method-assign]

    assert browser.extract_links("fixture", javascript=False) == "https://example.test/a"

    assert calls.call_args == call("extract_links", {"url": "fixture", "javascript": False})


def test_sync_extract_links_forwards_max_links() -> None:
    browser = Plasmate()
    calls = Mock(return_value="https://example.test/a")
    browser._call_tool = calls  # type: ignore[method-assign]

    assert browser.extract_links("fixture", max_links=3) == "https://example.test/a"
    assert calls.call_args == call("extract_links", {"url": "fixture", "max_links": 3})


def test_async_extract_links_forwards_javascript_opt_out() -> None:
    async def exercise() -> None:
        browser = AsyncPlasmate()
        calls = AsyncMock(return_value="https://example.test/a")
        browser._call_tool = calls  # type: ignore[method-assign]

        assert await browser.extract_links("fixture", javascript=False) == "https://example.test/a"

        calls.assert_awaited_once_with(
            "extract_links", {"url": "fixture", "javascript": False}
        )

    asyncio.run(exercise())


def test_async_extract_links_forwards_max_links() -> None:
    async def exercise() -> None:
        browser = AsyncPlasmate()
        calls = AsyncMock(return_value="https://example.test/a")
        browser._call_tool = calls  # type: ignore[method-assign]

        assert await browser.extract_links("fixture", max_links=3) == "https://example.test/a"
        calls.assert_awaited_once_with("extract_links", {"url": "fixture", "max_links": 3})

    asyncio.run(exercise())


def test_sync_open_page_forwards_selector() -> None:
    browser = Plasmate()
    calls = Mock(return_value={})
    browser._call_tool = calls  # type: ignore[method-assign]

    assert browser.open_page("fixture", selector="main") == {}

    calls.assert_called_once_with("open_page", {"url": "fixture", "selector": "main"})


def test_async_open_page_forwards_selector() -> None:
    async def exercise() -> None:
        browser = AsyncPlasmate()
        calls = AsyncMock(return_value={})
        browser._call_tool = calls  # type: ignore[method-assign]

        assert await browser.open_page("fixture", selector="main") == {}

        calls.assert_awaited_once_with(
            "open_page", {"url": "fixture", "selector": "main"}
        )

    asyncio.run(exercise())


def test_sync_navigate_to_forwards_session_and_url() -> None:
    browser = Plasmate()
    calls = Mock(side_effect=[{}])
    browser._call_tool = calls  # type: ignore[method-assign]

    assert browser.navigate_to("s1", "https://example.test/next") == {}

    assert calls.call_args_list == [
        call(
            "navigate_to",
            {"session_id": "s1", "url": "https://example.test/next"},
        ),
    ]


def test_async_navigate_to_forwards_session_and_url() -> None:
    async def exercise() -> None:
        browser = AsyncPlasmate()
        calls = AsyncMock(side_effect=[{}])
        browser._call_tool = calls  # type: ignore[method-assign]

        assert await browser.navigate_to("s1", "https://example.test/next") == {}

        assert calls.call_args_list == [
            call(
                "navigate_to",
                {"session_id": "s1", "url": "https://example.test/next"},
            ),
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
