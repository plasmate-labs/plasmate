"""
Browser Use integration for Plasmate.

Provides a Plasmate-backed browser for Browser Use that returns SOM output
instead of raw DOM — typically 10x fewer tokens for the same page.

Example::

    from plasmate_browser_use import PlasmateBrowser

    async with PlasmateBrowser() as browser:
        state = await browser.navigate("https://news.ycombinator.com")
        print(state.text)  # SOM-formatted page state
        state = await browser.click(state.interactive_elements[0].index)
"""

from .browser import PlasmateBrowser, PageState, InteractiveElement
from .som_formatter import som_to_browser_use_state, token_count_comparison

__all__ = [
    "PlasmateBrowser",
    "PageState",
    "InteractiveElement",
    "som_to_browser_use_state",
    "token_count_comparison",
]
__version__ = "0.1.0"
