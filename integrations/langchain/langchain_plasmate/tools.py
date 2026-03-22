"""LangChain tools for browsing the web with Plasmate.

Provides four tools:

- ``PlasmateFetchTool`` — stateless one-shot page fetch, returns SOM text
- ``PlasmateNavigateTool`` — open a URL in a persistent browser session
- ``PlasmateClickTool`` — click an interactive element by SOM ID
- ``PlasmateTypeTool`` — type text into a form field by SOM ID

The session-based tools (navigate, click, type) share a browser session via
a ``PlasmateBrowser`` wrapper. Use ``get_plasmate_tools()`` to create a
pre-configured tool set.
"""

from __future__ import annotations

from typing import Any, Optional, Type

from langchain_core.tools import BaseTool
from pydantic import BaseModel, Field

from plasmate import Plasmate

from .som_output import som_to_text


# ---------------------------------------------------------------------------
# Shared browser session state
# ---------------------------------------------------------------------------


class PlasmateBrowser:
    """Manages a Plasmate client and persistent browser session.

    Shared between session-based tools so click/type operations target
    the page opened by the most recent navigate call.
    """

    def __init__(self, client: Optional[Plasmate] = None, **kwargs: Any):
        self.client = client or Plasmate(**kwargs)
        self.session_id: Optional[str] = None

    def navigate(self, url: str) -> dict:
        """Open *url* in a new or replacement session. Returns the SOM."""
        if self.session_id:
            try:
                self.client.close_page(self.session_id)
            except Exception:
                pass
        result = self.client.open_page(url)
        self.session_id = result["session_id"]
        return result.get("som", result)

    def click(self, element_id: str) -> dict:
        """Click an element in the current session. Returns updated SOM."""
        self._require_session()
        return self.client.click(self.session_id, element_id)  # type: ignore[arg-type]

    def type_text(self, element_id: str, text: str) -> dict:
        """Type *text* into an element in the current session."""
        self._require_session()
        return self.client._call_tool(
            "type",
            {
                "session_id": self.session_id,
                "element_id": element_id,
                "text": text,
            },
        )

    def close(self) -> None:
        """Close the session and the underlying Plasmate process."""
        if self.session_id:
            try:
                self.client.close_page(self.session_id)
            except Exception:
                pass
            self.session_id = None
        self.client.close()

    def _require_session(self) -> None:
        if not self.session_id:
            raise RuntimeError(
                "No active browser session. Use plasmate_navigate first."
            )


# ---------------------------------------------------------------------------
# Input schemas
# ---------------------------------------------------------------------------


class _ClickInput(BaseModel):
    element_id: str = Field(description="The SOM element ID to click (e.g. 'e_a1b2c3d4e5f6').")


class _TypeInput(BaseModel):
    element_id: str = Field(description="The SOM element ID of the input field.")
    text: str = Field(description="The text to type into the element.")


# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------


class PlasmateFetchTool(BaseTool):
    """Fetch a web page and return its semantic structure.

    This is a **stateless** tool — each call fetches a fresh page with no
    persistent session. Use ``PlasmateNavigateTool`` when you need to
    interact with the page afterwards (click, type).
    """

    name: str = "plasmate_fetch"
    description: str = (
        "Fetch a web page and return its semantic structure "
        "(navigation, content, forms, interactive elements). "
        "Returns a compact text representation with element IDs "
        "that can be referenced by other tools. "
        "Input: the URL to fetch."
    )
    client: Any = None  # Plasmate instance

    model_config = {"arbitrary_types_allowed": True}

    def __init__(self, client: Optional[Plasmate] = None, **kwargs: Any):
        super().__init__(**kwargs)
        self.client = client or Plasmate()

    def _run(self, url: str) -> str:
        som = self.client.fetch_page(url)
        return som_to_text(som)

    async def _arun(self, url: str) -> str:
        return self._run(url)


class PlasmateNavigateTool(BaseTool):
    """Navigate to a URL in a persistent browser session.

    Opens (or replaces) the current browser session. Subsequent
    ``plasmate_click`` and ``plasmate_type`` calls will target this page.
    """

    name: str = "plasmate_navigate"
    description: str = (
        "Navigate to a URL in a persistent browser session. "
        "Returns the page's semantic structure with element IDs. "
        "Use this before click or type tools. "
        "Input: the URL to navigate to."
    )
    browser: Any = None  # PlasmateBrowser instance

    model_config = {"arbitrary_types_allowed": True}

    def _run(self, url: str) -> str:
        som = self.browser.navigate(url)
        return som_to_text(som)

    async def _arun(self, url: str) -> str:
        return self._run(url)


class PlasmateClickTool(BaseTool):
    """Click an interactive element by its SOM ID."""

    name: str = "plasmate_click"
    description: str = (
        "Click an interactive element on the current page by its SOM element ID "
        "(e.g. 'e_a1b2c3d4e5f6'). Returns the updated page structure. "
        "You must call plasmate_navigate first to open a page."
    )
    args_schema: Type[BaseModel] = _ClickInput
    browser: Any = None  # PlasmateBrowser instance

    model_config = {"arbitrary_types_allowed": True}

    def _run(self, element_id: str) -> str:
        som = self.browser.click(element_id)
        return som_to_text(som)

    async def _arun(self, element_id: str) -> str:
        return self._run(element_id)


class PlasmateTypeTool(BaseTool):
    """Type text into a form field by its SOM ID."""

    name: str = "plasmate_type"
    description: str = (
        "Type text into a form input or textarea on the current page. "
        "Requires the SOM element ID and the text to type. "
        "You must call plasmate_navigate first to open a page."
    )
    args_schema: Type[BaseModel] = _TypeInput
    browser: Any = None  # PlasmateBrowser instance

    model_config = {"arbitrary_types_allowed": True}

    def _run(self, element_id: str, text: str) -> str:
        som = self.browser.type_text(element_id, text)
        return som_to_text(som)

    async def _arun(self, element_id: str, text: str) -> str:
        return self._run(element_id, text)


# ---------------------------------------------------------------------------
# Convenience constructor
# ---------------------------------------------------------------------------


def get_plasmate_tools(
    client: Optional[Plasmate] = None,
) -> list[BaseTool]:
    """Create a full set of Plasmate browsing tools with shared session state.

    Args:
        client: An existing Plasmate client. If ``None``, a new one is
            created automatically.

    Returns:
        A list of four LangChain tools: fetch, navigate, click, and type.
    """
    client = client or Plasmate()
    browser = PlasmateBrowser(client=client)
    return [
        PlasmateFetchTool(client=client),
        PlasmateNavigateTool(browser=browser),
        PlasmateClickTool(browser=browser),
        PlasmateTypeTool(browser=browser),
    ]
