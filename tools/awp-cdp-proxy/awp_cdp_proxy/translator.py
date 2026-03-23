"""AWP-to-CDP translation layer.

Maps AWP v0.1 methods to Chrome DevTools Protocol commands.
This is the core logic that makes AWP work with any CDP-compatible browser.
"""

from __future__ import annotations

import asyncio
import logging
import re
import uuid
from typing import Any, Optional

from .cdp_client import CDPClient, CDPError, connect_to_target, close_target
from .som_generator import html_to_som

logger = logging.getLogger(__name__)


class SessionState:
    """Tracks state for a single AWP session backed by a CDP target."""

    def __init__(self, session_id: str, cdp: CDPClient, target_id: str):
        self.session_id = session_id
        self.cdp = cdp
        self.target_id = target_id
        self.current_url: Optional[str] = None
        self.current_som = None
        self.current_html: Optional[str] = None
        self.timeout_ms: int = 30000
        self.user_agent: Optional[str] = None
        self.locale: str = "en-US"


class AWPTranslator:
    """Translates AWP requests into CDP commands and returns AWP responses."""

    def __init__(self, cdp_url: str = "ws://localhost:9222"):
        self.cdp_url = cdp_url
        self.sessions: dict[str, SessionState] = {}
        self.hello_done = False

    async def handle(self, request: dict) -> dict:
        """Handle an AWP request and return an AWP response."""
        msg_id = request.get("id", "")
        msg_type = request.get("type", "")
        method = request.get("method", "")
        params = request.get("params", {})

        if msg_type != "request":
            return self._error(msg_id, "INVALID_REQUEST", "Expected type 'request'")

        if not method:
            return self._error(msg_id, "INVALID_REQUEST", "Missing 'method' field")

        # awp.hello must be first
        if not self.hello_done and method != "awp.hello":
            return self._error(msg_id, "INVALID_REQUEST", "Must send awp.hello first")

        try:
            handler = self._get_handler(method)
            if not handler:
                return self._error(msg_id, "INVALID_REQUEST", f"Unknown method: {method}")
            result = await handler(params)
            return self._response(msg_id, result)
        except TimeoutError as e:
            return self._error(msg_id, "TIMEOUT", str(e))
        except CDPError as e:
            return self._error(msg_id, "INTERNAL", str(e))
        except NotFoundError as e:
            return self._error(msg_id, "NOT_FOUND", str(e))
        except NavigationError as e:
            return self._error(msg_id, "NAVIGATION_FAILED", str(e))
        except Exception as e:
            logger.exception(f"Unexpected error handling {method}")
            return self._error(msg_id, "INTERNAL", str(e))

    def _get_handler(self, method: str):
        handlers = {
            "awp.hello": self._handle_hello,
            "session.create": self._handle_session_create,
            "session.close": self._handle_session_close,
            "page.navigate": self._handle_page_navigate,
            "page.observe": self._handle_page_observe,
            "page.act": self._handle_page_act,
            "page.extract": self._handle_page_extract,
        }
        return handlers.get(method)

    async def _handle_hello(self, params: dict) -> dict:
        awp_version = params.get("awp_version", "")
        if awp_version and awp_version != "0.1":
            raise InvalidRequestError(f"Unsupported AWP version: {awp_version}")

        self.hello_done = True
        return {
            "awp_version": "0.1",
            "server_name": "awp-cdp-proxy",
            "server_version": "0.1.0",
            "features": ["som.snapshot", "act.primitive", "extract"],
        }

    async def _handle_session_create(self, params: dict) -> dict:
        # Close existing session if any (v0.1: one session per connection)
        for sid in list(self.sessions.keys()):
            await self._close_session(sid)

        # Connect to Chrome via CDP and create a new tab
        cdp = await connect_to_target(self.cdp_url)

        session_id = f"s_{uuid.uuid4().hex[:8]}"

        # Extract target_id from the WebSocket URL
        target_id = ""
        if cdp.browser_ws_url:
            # URL format: ws://host:port/devtools/page/TARGET_ID
            parts = cdp.browser_ws_url.split("/")
            if parts:
                target_id = parts[-1]

        session = SessionState(session_id, cdp, target_id)
        session.timeout_ms = params.get("timeout_ms", 30000)
        session.user_agent = params.get("user_agent")
        session.locale = params.get("locale", "en-US")

        # Set user agent if specified
        if session.user_agent:
            await cdp.send("Network.setUserAgentOverride", {
                "userAgent": session.user_agent,
            })

        self.sessions[session_id] = session
        return {"session_id": session_id}

    async def _handle_session_close(self, params: dict) -> dict:
        session_id = params.get("session_id", "")
        if session_id not in self.sessions:
            raise NotFoundError(f"Session not found: {session_id}")

        await self._close_session(session_id)
        return {"closed": True}

    async def _close_session(self, session_id: str) -> None:
        session = self.sessions.pop(session_id, None)
        if session:
            await session.cdp.disconnect()
            try:
                await close_target(self.cdp_url, session.target_id)
            except Exception:
                logger.debug(f"Failed to close target {session.target_id}")

    async def _handle_page_navigate(self, params: dict) -> dict:
        session = self._get_session(params)
        url = params.get("url", "")
        if not url:
            raise InvalidRequestError("Missing 'url' parameter")

        timeout_ms = params.get("timeout_ms", session.timeout_ms)
        timeout_s = timeout_ms / 1000.0

        import time
        start = time.monotonic()

        # Set up load event waiter
        load_event = asyncio.Event()
        session.cdp.on_event("Page.loadEventFired", lambda p: load_event.set())

        # Navigate
        nav_result = await session.cdp.send("Page.navigate", {"url": url}, timeout=timeout_s)

        error_text = nav_result.get("errorText")
        if error_text:
            raise NavigationError(f"Navigation failed: {error_text}")

        # Wait for load event
        try:
            await asyncio.wait_for(load_event.wait(), timeout=timeout_s)
        except asyncio.TimeoutError:
            raise TimeoutError(f"Page load timed out after {timeout_ms}ms")

        # Small delay for any final rendering
        await asyncio.sleep(0.1)

        # Get the page HTML
        eval_result = await session.cdp.send("Runtime.evaluate", {
            "expression": "document.documentElement.outerHTML",
            "returnByValue": True,
        })
        html = eval_result.get("result", {}).get("value", "")

        # Get the actual URL (after redirects)
        actual_url = url
        try:
            frame_tree = await session.cdp.send("Page.getFrameTree")
            actual_url = frame_tree.get("frameTree", {}).get("frame", {}).get("url", url)
        except Exception:
            pass

        # Get content type
        content_type = "text/html"

        # Generate SOM
        som = html_to_som(html, url=actual_url)
        session.current_url = actual_url
        session.current_html = html
        session.current_som = som

        elapsed_ms = int((time.monotonic() - start) * 1000)

        return {
            "url": actual_url,
            "status": 200,
            "content_type": content_type,
            "html_bytes": len(html.encode("utf-8")),
            "som_ready": True,
            "load_ms": elapsed_ms,
        }

    async def _handle_page_observe(self, params: dict) -> dict:
        session = self._get_session(params)

        if not session.current_som:
            raise NotFoundError("No page loaded yet")

        return {"som": session.current_som.model_dump(exclude_none=True)}

    async def _handle_page_act(self, params: dict) -> dict:
        session = self._get_session(params)

        if not session.current_som:
            raise NotFoundError("No page loaded yet")

        intent = params.get("intent", {})
        action = intent.get("action", "")
        target = intent.get("target", {})
        value = intent.get("value")

        # Resolve the target element
        resolved = self._resolve_target(session, target)
        if not resolved:
            raise NotFoundError(f"Target not found: {target}")

        element_id, element = resolved
        effects = {"navigated": False, "som_changed": False}

        if action == "click":
            await self._act_click(session, element)
            # Check if navigation happened (e.g., clicked a link)
            if element.role.value == "link" and element.attrs and element.attrs.href:
                # Re-navigate to get updated SOM
                effects["navigated"] = True
                effects["som_changed"] = True
            else:
                # Refresh the SOM
                await self._refresh_som(session)
                effects["som_changed"] = True

        elif action == "type":
            if value is None:
                raise InvalidRequestError("'type' action requires 'value' parameter")
            await self._act_type(session, element, value)
            await self._refresh_som(session)
            effects["som_changed"] = True

        elif action == "select":
            if value is None:
                raise InvalidRequestError("'select' action requires 'value' parameter")
            await self._act_select(session, element, value)
            await self._refresh_som(session)
            effects["som_changed"] = True

        elif action == "scroll":
            # No-op in v0.1 per spec
            pass

        else:
            raise InvalidRequestError(f"Unknown action: {action}")

        return {
            "status": "ok",
            "resolved": {
                "element_id": element_id,
                "role": element.role.value,
                "text": element.text,
            },
            "effects": effects,
        }

    async def _handle_page_extract(self, params: dict) -> dict:
        session = self._get_session(params)

        if not session.current_som:
            raise NotFoundError("No page loaded yet")

        fields = params.get("fields", {})
        data = {}
        provenance = {}

        som = session.current_som
        all_elements = []
        for region in som.regions:
            for el in region.elements:
                all_elements.append(el)
                if el.children:
                    all_elements.extend(el.children)

        for field_name, query in fields.items():
            result, prov = self._extract_field(all_elements, query)
            data[field_name] = result
            if prov:
                provenance[field_name] = prov

        return {"data": data, "provenance": provenance}

    def _extract_field(self, elements, query):
        """Extract a field value from SOM elements based on a query."""
        match_all = query.get("all", False)
        props = query.get("props", ["text"])
        matches = []
        prov_ids = []

        for el in elements:
            if self._element_matches_query(el, query):
                if match_all:
                    item = {}
                    for prop in props:
                        if prop == "text":
                            item["text"] = el.text
                        elif prop == "href" and el.attrs:
                            item["href"] = el.attrs.href
                        elif prop == "value" and el.attrs:
                            item["value"] = el.attrs.value
                        elif prop == "src" and el.attrs:
                            item["src"] = el.attrs.src
                    matches.append(item)
                    prov_ids.append(el.id)
                else:
                    # Return first match
                    if "text_match" in query:
                        return el.text, el.id
                    return el.text, el.id

        if match_all:
            return matches, prov_ids
        return None, None

    def _element_matches_query(self, element, query) -> bool:
        """Check if a SOM element matches an extraction query."""
        if "ref" in query:
            return element.id == query["ref"]

        if "role" in query:
            if element.role.value != query["role"]:
                return False
            if "level" in query and element.attrs:
                if element.attrs.level != query["level"]:
                    return False
            return True

        if "text_match" in query:
            if element.text:
                return bool(re.search(query["text_match"], element.text))
            return False

        return False

    def _resolve_target(self, session: SessionState, target: dict):
        """Resolve an AWP target to a SOM element."""
        som = session.current_som
        all_elements = []
        for region in som.regions:
            for el in region.elements:
                all_elements.append(el)
                if el.children:
                    all_elements.extend(el.children)

        # Try ref first
        ref = target.get("ref")
        if ref:
            for el in all_elements:
                if el.id == ref:
                    return (el.id, el)

        # Try text + role
        text = target.get("text")
        role = target.get("role")
        if text or role:
            for el in all_elements:
                role_match = (not role) or el.role.value == role
                text_match = (not text) or (el.text and text.lower() in el.text.lower())
                if role_match and text_match:
                    return (el.id, el)

        # Try CSS selector (we can't resolve CSS against SOM, use CDP)
        css = target.get("css")
        if css:
            # Return a placeholder; actual CSS resolution happens via CDP
            return None

        # Try fallback
        fallback = target.get("fallback")
        if fallback:
            return self._resolve_target(session, fallback)

        return None

    async def _act_click(self, session: SessionState, element) -> None:
        """Translate a click action to CDP commands."""
        cdp = session.cdp

        # Use Runtime.evaluate to find and click the element
        # We use querySelector with a strategy based on the element's properties
        selector = self._build_css_selector(element)

        if selector:
            # Get the element's bounding box
            result = await cdp.send("Runtime.evaluate", {
                "expression": f"""
                    (function() {{
                        var el = document.querySelector({repr(selector)});
                        if (!el) return null;
                        var rect = el.getBoundingClientRect();
                        return {{x: rect.x + rect.width/2, y: rect.y + rect.height/2}};
                    }})()
                """,
                "returnByValue": True,
            })

            coords = result.get("result", {}).get("value")
            if coords and coords.get("x") is not None:
                x, y = coords["x"], coords["y"]
                # Dispatch mouse events
                await cdp.send("Input.dispatchMouseEvent", {
                    "type": "mousePressed",
                    "x": x, "y": y,
                    "button": "left",
                    "clickCount": 1,
                })
                await cdp.send("Input.dispatchMouseEvent", {
                    "type": "mouseReleased",
                    "x": x, "y": y,
                    "button": "left",
                    "clickCount": 1,
                })
                return

        # Fallback: use JS click
        await cdp.send("Runtime.evaluate", {
            "expression": f"""
                (function() {{
                    var el = document.querySelector({repr(selector or 'body')});
                    if (el) el.click();
                }})()
            """,
        })

    async def _act_type(self, session: SessionState, element, value: str) -> None:
        """Translate a type action to CDP commands."""
        cdp = session.cdp
        selector = self._build_css_selector(element)

        if selector:
            # Focus the element
            await cdp.send("Runtime.evaluate", {
                "expression": f"""
                    (function() {{
                        var el = document.querySelector({repr(selector)});
                        if (el) {{ el.focus(); el.value = ''; }}
                    }})()
                """,
            })

        # Insert text
        await cdp.send("Input.insertText", {"text": value})

    async def _act_select(self, session: SessionState, element, value: str) -> None:
        """Translate a select action to CDP commands."""
        cdp = session.cdp
        selector = self._build_css_selector(element)

        if selector:
            await cdp.send("Runtime.evaluate", {
                "expression": f"""
                    (function() {{
                        var el = document.querySelector({repr(selector)});
                        if (el) {{
                            el.value = {repr(value)};
                            el.dispatchEvent(new Event('change', {{bubbles: true}}));
                        }}
                    }})()
                """,
            })

    def _build_css_selector(self, element) -> Optional[str]:
        """Build a CSS selector to find an element in the live DOM."""
        role = element.role.value

        if role == "link" and element.attrs and element.attrs.href:
            href = element.attrs.href
            # Escape quotes in href
            href = href.replace("'", "\\'")
            return f"a[href='{href}']"

        if role == "button" and element.text:
            # Use XPath-like approach via JS; return simple selector
            return f"button"

        if role == "text_input" and element.attrs:
            input_type = element.attrs.input_type or "text"
            if element.label:
                return f"input[type='{input_type}']"
            if element.attrs.placeholder:
                placeholder = element.attrs.placeholder.replace("'", "\\'")
                return f"input[placeholder='{placeholder}']"
            return f"input[type='{input_type}']"

        if role == "textarea":
            return "textarea"

        if role == "select":
            return "select"

        return None

    async def _refresh_som(self, session: SessionState) -> None:
        """Re-fetch the page HTML and regenerate the SOM."""
        try:
            eval_result = await session.cdp.send("Runtime.evaluate", {
                "expression": "document.documentElement.outerHTML",
                "returnByValue": True,
            })
            html = eval_result.get("result", {}).get("value", "")

            actual_url = session.current_url or ""
            try:
                frame_tree = await session.cdp.send("Page.getFrameTree")
                actual_url = frame_tree.get("frameTree", {}).get("frame", {}).get("url", actual_url)
            except Exception:
                pass

            session.current_html = html
            session.current_url = actual_url
            session.current_som = html_to_som(html, url=actual_url)
        except Exception:
            logger.exception("Failed to refresh SOM")

    def _get_session(self, params: dict) -> SessionState:
        """Get a session by ID from params."""
        session_id = params.get("session_id", "")
        session = self.sessions.get(session_id)
        if not session:
            raise NotFoundError(f"Session not found: {session_id}")
        return session

    async def cleanup(self) -> None:
        """Clean up all sessions."""
        for session_id in list(self.sessions.keys()):
            await self._close_session(session_id)

    @staticmethod
    def _response(msg_id: str, result: dict) -> dict:
        return {"id": msg_id, "type": "response", "result": result}

    @staticmethod
    def _error(msg_id: str, code: str, message: str) -> dict:
        return {"id": msg_id, "type": "response", "error": {"code": code, "message": message}}


class NotFoundError(Exception):
    pass


class NavigationError(Exception):
    pass


class InvalidRequestError(Exception):
    pass
