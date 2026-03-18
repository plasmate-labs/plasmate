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

__all__ = ["Plasmate", "AsyncPlasmate"]
__version__ = "0.2.0"
