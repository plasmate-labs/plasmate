"""PlasmateBrowser — a Browser Use-compatible browser backed by Plasmate SOM output.

Instead of Playwright + Chrome, this uses Plasmate's MCP subprocess to fetch pages
and return them as Semantic Object Model (SOM) documents. The SOM representation
is ~10x more token-efficient than raw DOM, making it ideal for LLM-driven agents.

Browser Use normally feeds the LLM a DOM tree like::

    [123]<button type="submit" />
      Submit Order
    [456]<input name="email" placeholder="you@example.com" />

With Plasmate, the agent sees compact SOM output like::

    [1] button "Submit Order"
    [2] input(email) "Email" placeholder="you@example.com"

Same information, far fewer tokens.
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, field
from typing import Any, Optional

from plasmate import AsyncPlasmate

from .som_formatter import som_to_browser_use_state


@dataclass
class InteractiveElement:
    """An interactive element extracted from the SOM, mapped to an integer index."""

    index: int
    """Sequential integer index used by Browser Use to reference this element."""

    som_id: str
    """Original SOM element ID (e.g. 'e_a1b2c3d4e5f6')."""

    role: str
    """Element role: link, button, text_input, textarea, select, checkbox, radio."""

    text: str
    """Display text or label for the element."""

    attrs: dict[str, Any] = field(default_factory=dict)
    """Role-specific attributes (href, placeholder, value, etc.)."""


@dataclass
class PageState:
    """The current page state as seen by the LLM agent.

    This is the Plasmate equivalent of Browser Use's BrowserStateSummary.
    It provides the same information in SOM format, using ~10x fewer tokens.
    """

    url: str
    """Current page URL."""

    title: str
    """Page title."""

    text: str
    """SOM-formatted text representation for the LLM (the main output)."""

    interactive_elements: list[InteractiveElement]
    """All interactive elements with their integer indices for Browser Use."""

    som: dict[str, Any]
    """Raw SOM dict for programmatic access."""

    selector_map: dict[int, InteractiveElement]
    """Maps integer index -> InteractiveElement for quick lookup."""

    som_tokens: int
    """Estimated token count of the SOM representation."""

    html_bytes: int
    """Original HTML size in bytes."""

    som_bytes: int
    """SOM output size in bytes."""


class PlasmateBrowser:
    """A Browser Use-compatible browser that uses Plasmate for SOM output.

    Spawns a ``plasmate mcp`` subprocess and uses stateful sessions to
    navigate, click, and type — returning compact SOM state instead of
    raw DOM at each step.

    Args:
        binary: Path to the plasmate binary. Default: ``"plasmate"``
        timeout: Response timeout in seconds. Default: ``30``
        budget: Optional SOM token budget (limits output size).

    Example::

        async with PlasmateBrowser() as browser:
            state = await browser.navigate("https://example.com")
            print(state.text)
            print(f"Tokens: {state.som_tokens}")

            # Click the first link
            link = state.interactive_elements[0]
            state = await browser.click(link.index)
    """

    def __init__(
        self,
        binary: str = "plasmate",
        timeout: float = 30,
        budget: Optional[int] = None,
    ):
        self._client = AsyncPlasmate(binary=binary, timeout=timeout)
        self._budget = budget
        self._session_id: Optional[str] = None
        self._current_state: Optional[PageState] = None
        self._index_to_som_id: dict[int, str] = {}

    async def __aenter__(self) -> PlasmateBrowser:
        return self

    async def __aexit__(self, *args: Any) -> None:
        await self.close()

    # ---- Core browser interface ----

    async def navigate(self, url: str) -> PageState:
        """Navigate to a URL and return the page state as SOM.

        Opens a new persistent session (closing any previous one). The returned
        PageState contains the SOM text representation with indexed interactive
        elements that can be referenced via click() and type_text().

        Args:
            url: The URL to navigate to.

        Returns:
            PageState with SOM-formatted content and element index map.
        """
        # Close existing session if any
        if self._session_id is not None:
            try:
                await self._client.close_page(self._session_id)
            except RuntimeError:
                pass
            self._session_id = None

        result = await self._client.open_page(url)
        self._session_id = result["session_id"]

        # open_page returns the SOM directly in the result (without "som" wrapper
        # in newer versions, or with it in older ones)
        som = result.get("som", result)
        return self._build_state(som)

    async def click(self, element_index: int) -> PageState:
        """Click an element by its integer index.

        The index corresponds to the ``[N]`` prefix shown in the SOM text output.
        After clicking, Plasmate returns the updated page SOM.

        Args:
            element_index: The integer index of the element to click.

        Returns:
            Updated PageState after the click.

        Raises:
            RuntimeError: If no page is open or the element index is invalid.
        """
        if self._session_id is None:
            raise RuntimeError("No page is open. Call navigate() first.")

        som_id = self._index_to_som_id.get(element_index)
        if som_id is None:
            valid = sorted(self._index_to_som_id.keys())
            raise RuntimeError(
                f"Element index {element_index} not found. "
                f"Valid indices: {valid[:20]}{'...' if len(valid) > 20 else ''}"
            )

        som = await self._client.click(self._session_id, som_id)
        return self._build_state(som)

    async def type_text(self, element_index: int, text: str) -> PageState:
        """Type text into a form element by its integer index.

        Uses JavaScript evaluation to set the value and dispatch input events,
        since Plasmate operates at the DOM level rather than sending keystrokes.

        Args:
            element_index: The integer index of the input/textarea element.
            text: The text to type.

        Returns:
            Updated PageState after typing.

        Raises:
            RuntimeError: If no page is open or the element index is invalid.
        """
        if self._session_id is None:
            raise RuntimeError("No page is open. Call navigate() first.")

        som_id = self._index_to_som_id.get(element_index)
        if som_id is None:
            valid = sorted(self._index_to_som_id.keys())
            raise RuntimeError(
                f"Element index {element_index} not found. "
                f"Valid indices: {valid[:20]}{'...' if len(valid) > 20 else ''}"
            )

        # Use evaluate to set value and trigger input events
        escaped = text.replace("\\", "\\\\").replace("'", "\\'").replace("\n", "\\n")
        js = f"""
        (() => {{
            const el = document.querySelector('[data-som-id="{som_id}"]')
                || document.getElementById('{som_id}');
            if (!el) return false;
            el.value = '{escaped}';
            el.dispatchEvent(new Event('input', {{ bubbles: true }}));
            el.dispatchEvent(new Event('change', {{ bubbles: true }}));
            return true;
        }})()
        """
        await self._client.evaluate(self._session_id, js)

        # Re-fetch state to get updated SOM
        return await self.get_state()

    async def get_state(self) -> PageState:
        """Return the current page state as SOM.

        Re-fetches the page DOM and compiles it to SOM. Useful after JavaScript
        mutations or to get a fresh snapshot.

        Returns:
            Current PageState.

        Raises:
            RuntimeError: If no page is open.
        """
        if self._session_id is None:
            raise RuntimeError("No page is open. Call navigate() first.")

        # Evaluate to get current URL and trigger SOM recompilation
        som = await self._client.evaluate(
            self._session_id,
            "document.documentElement.outerHTML",
        )

        # If evaluate returns HTML string, we need to use click with a no-op
        # to get fresh SOM. Instead, use the stateless fetch on current URL.
        url = await self._client.evaluate(
            self._session_id,
            "window.location.href",
        )

        if self._current_state and isinstance(url, str):
            # Use fetch_page for a fresh SOM of the current URL
            som = await self._client.fetch_page(url, budget=self._budget)
            return self._build_state(som)

        # Fallback: return cached state
        if self._current_state:
            return self._current_state
        raise RuntimeError("No page state available.")

    async def screenshot(self) -> None:
        """Return a screenshot of the current page.

        Plasmate is a headless engine with no visual rendering pipeline,
        so screenshots are not supported. Returns None.

        Browser Use agents that rely on visual screenshots should use the
        default Playwright backend instead.
        """
        return None

    async def close(self) -> None:
        """Close the browser session and shut down the Plasmate process."""
        if self._session_id is not None:
            try:
                await self._client.close_page(self._session_id)
            except RuntimeError:
                pass
            self._session_id = None
        await self._client.close()

    # ---- State building ----

    def _build_state(self, som: dict[str, Any]) -> PageState:
        """Build a PageState from a SOM dict, indexing interactive elements."""
        elements: list[InteractiveElement] = []
        index_map: dict[int, str] = {}
        selector_map: dict[int, InteractiveElement] = {}

        # Walk all regions and collect interactive elements
        idx = 1
        for region in som.get("regions", []):
            for elem in region.get("elements", []):
                idx = self._collect_interactive(elem, idx, elements, index_map, selector_map)

        self._index_to_som_id = index_map

        # Generate the text representation
        text = som_to_browser_use_state(som, index_map={v: k for k, v in index_map.items()})

        meta = som.get("meta", {})
        som_text = text.encode("utf-8")
        state = PageState(
            url=som.get("url", ""),
            title=som.get("title", "Untitled"),
            text=text,
            interactive_elements=elements,
            som=som,
            selector_map=selector_map,
            som_tokens=len(som_text) // 4,
            html_bytes=meta.get("html_bytes", 0),
            som_bytes=meta.get("som_bytes", len(som_text)),
        )
        self._current_state = state
        return state

    def _collect_interactive(
        self,
        elem: dict[str, Any],
        idx: int,
        elements: list[InteractiveElement],
        index_map: dict[int, str],
        selector_map: dict[int, InteractiveElement],
    ) -> int:
        """Recursively collect interactive elements, assigning sequential indices."""
        if elem.get("actions"):
            ie = InteractiveElement(
                index=idx,
                som_id=elem["id"],
                role=elem.get("role", ""),
                text=elem.get("label") or elem.get("text") or "",
                attrs=elem.get("attrs") or {},
            )
            elements.append(ie)
            index_map[idx] = elem["id"]
            selector_map[idx] = ie
            idx += 1

        for child in elem.get("children", []) or []:
            idx = self._collect_interactive(child, idx, elements, index_map, selector_map)

        return idx
