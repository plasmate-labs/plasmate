"""LangChain integration for Plasmate — agent-native headless browser.

Provides LangChain tools and a document loader that return SOM (Semantic
Object Model) output instead of raw HTML, saving ~10x tokens.

Quick start::

    from langchain_plasmate import get_plasmate_tools

    tools = get_plasmate_tools()
    # tools = [plasmate_fetch, plasmate_navigate, plasmate_click, plasmate_type]

Document loader::

    from langchain_plasmate import PlasmateSOMLoader

    docs = PlasmateSOMLoader(["https://example.com"]).load()
"""

from .loader import PlasmateSOMLoader
from .som_output import som_to_text
from .tools import (
    PlasmateBrowser,
    PlasmateClickTool,
    PlasmateFetchTool,
    PlasmateNavigateTool,
    PlasmateTypeTool,
    get_plasmate_tools,
)

__all__ = [
    "PlasmateBrowser",
    "PlasmateClickTool",
    "PlasmateFetchTool",
    "PlasmateNavigateTool",
    "PlasmateSOMLoader",
    "PlasmateTypeTool",
    "get_plasmate_tools",
    "som_to_text",
]
__version__ = "0.1.0"
