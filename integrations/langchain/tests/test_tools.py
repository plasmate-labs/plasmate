from unittest.mock import Mock, call

from langchain_plasmate import PlasmateFetchTool


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
