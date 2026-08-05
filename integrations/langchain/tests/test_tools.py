import asyncio
import time
from unittest.mock import Mock, call

import pytest

from langchain_plasmate import (
    PlasmateClickTool,
    PlasmateFetchTool,
    PlasmateNavigateTool,
    PlasmateTypeTool,
)


def _som() -> dict:
    return {
        "title": "Fixture",
        "url": "fixture",
        "regions": [],
        "meta": {
            "html_bytes": 0,
            "som_bytes": 0,
            "element_count": 0,
            "interactive_count": 0,
        },
    }


def test_fetch_tool_forwards_selector_and_keeps_string_input() -> None:
    client = Mock()
    client.fetch_page.return_value = _som()
    tool = PlasmateFetchTool(client=client)

    tool.invoke({"url": "fixture", "selector": "interactive"})
    tool.invoke("fixture")

    assert client.fetch_page.call_args_list == [
        call("fixture", selector="interactive"),
        call("fixture"),
    ]


def test_fetch_tool_forwards_budget_javascript_and_selector() -> None:
    client = Mock()
    client.fetch_page.return_value = _som()
    tool = PlasmateFetchTool(client=client)

    tool.invoke(
        {
            "url": "fixture",
            "budget": 128,
            "javascript": False,
            "selector": "main",
        }
    )

    client.fetch_page.assert_called_once_with(
        "fixture",
        budget=128,
        javascript=False,
        selector="main",
    )


def _blocking_som(*args: object, **kwargs: object) -> dict:
    time.sleep(0.08)
    return _som()


def _fetch_tool() -> PlasmateFetchTool:
    client = Mock()
    client.fetch_page.side_effect = _blocking_som
    return PlasmateFetchTool(client=client)


def _navigate_tool() -> PlasmateNavigateTool:
    browser = Mock()
    browser.navigate.side_effect = _blocking_som
    return PlasmateNavigateTool(browser=browser)


def _click_tool() -> PlasmateClickTool:
    browser = Mock()
    browser.click.side_effect = _blocking_som
    return PlasmateClickTool(browser=browser)


def _type_tool() -> PlasmateTypeTool:
    browser = Mock()
    browser.type_text.side_effect = _blocking_som
    return PlasmateTypeTool(browser=browser)


@pytest.mark.parametrize(
    ("factory", "tool_input"),
    [
        (_fetch_tool, {"url": "fixture"}),
        (_navigate_tool, "fixture"),
        (_click_tool, {"element_id": "e1"}),
        (_type_tool, {"element_id": "e1", "text": "x"}),
    ],
    ids=["fetch", "navigate", "click", "type"],
)
def test_async_tools_do_not_block_event_loop(factory, tool_input) -> None:
    async def exercise() -> None:
        tool = factory()
        call_task = asyncio.create_task(tool.ainvoke(tool_input))
        await asyncio.sleep(0.01)
        assert not call_task.done()
        await asyncio.wait_for(call_task, timeout=1)

    asyncio.run(exercise())


def test_fetch_tool_async_forwards_fetch_options() -> None:
    import asyncio

    client = Mock()
    client.fetch_page.return_value = _som()
    tool = PlasmateFetchTool(client=client)

    asyncio.run(
        tool.ainvoke(
            {
                "url": "fixture",
                "budget": 128,
                "javascript": False,
                "selector": "main",
            }
        )
    )

    client.fetch_page.assert_called_once_with(
        "fixture",
        budget=128,
        javascript=False,
        selector="main",
    )
