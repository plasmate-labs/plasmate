"""
Plasmate - Agent-native headless browser SDK for Python.

Communicates with the `plasmate mcp` process over stdio using JSON-RPC 2.0.

Example::

    from plasmate import Plasmate

    browser = Plasmate()

    # One-shot: fetch a page as SOM
    som = browser.fetch_page("https://example.com")
    print(som["title"], len(som["regions"]))

    # Interactive: open, click, evaluate, close
    session = browser.open_page("https://news.ycombinator.com")
    title = browser.evaluate(session["session_id"], "document.title")
    browser.close_page(session["session_id"])

    # Clean up
    browser.close()
"""

from .client import Plasmate, AsyncPlasmate
from .types import (
    ElementAttrs,
    ElementRole,
    LinkElement,
    ListItem,
    RegionRole,
    SelectOption,
    SemanticHint,
    Som,
    SomElement,
    SomMeta,
    SomRegion,
    SomShadowRoot,
    StructuredData,
)
from .query import (
    find_action_target_by_cache_key,
    find_action_target_by_html_id,
    find_action_target_by_id,
    find_by_id,
    find_by_html_id,
    find_by_role,
    find_by_tag,
    find_by_text,
    find_interactive,
    flat_elements,
    get_action_plan,
    get_action_plan_cache_key,
    get_action_plan_fingerprint,
    get_action_plan_index,
    get_action_plan_summary,
    get_enabled_action_plan,
    get_token_estimate,
)

__all__ = [
    # Client
    "Plasmate",
    "AsyncPlasmate",
    # Types
    "ElementAttrs",
    "ElementRole",
    "LinkElement",
    "ListItem",
    "RegionRole",
    "SelectOption",
    "SemanticHint",
    "Som",
    "SomElement",
    "SomMeta",
    "SomRegion",
    "SomShadowRoot",
    "StructuredData",
    # Query helpers
    "find_action_target_by_cache_key",
    "find_action_target_by_html_id",
    "find_action_target_by_id",
    "find_by_id",
    "find_by_html_id",
    "find_by_role",
    "find_by_tag",
    "find_by_text",
    "find_interactive",
    "flat_elements",
    "get_action_plan",
    "get_action_plan_cache_key",
    "get_action_plan_fingerprint",
    "get_action_plan_index",
    "get_action_plan_summary",
    "get_enabled_action_plan",
    "get_token_estimate",
]
__version__ = "0.3.0"
