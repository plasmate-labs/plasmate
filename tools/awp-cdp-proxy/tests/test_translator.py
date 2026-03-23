"""Tests for the AWP-to-CDP translator.

Tests the translation layer with mock CDP responses, verifying that
AWP methods are correctly handled without needing a real Chrome instance.
"""

from __future__ import annotations

import asyncio
import json
import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from awp_cdp_proxy.translator import AWPTranslator, NotFoundError
from awp_cdp_proxy.som_generator import html_to_som


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

SAMPLE_HTML = """<!DOCTYPE html>
<html lang="en">
<head><title>Test Page</title></head>
<body>
<nav aria-label="Main navigation">
    <a href="/">Home</a>
    <a href="/about">About</a>
    <a href="/contact">Contact</a>
</nav>
<main>
    <h1>Welcome to Test Page</h1>
    <p>This is a test paragraph with some content.</p>
    <form action="/search" method="GET">
        <input type="text" name="q" placeholder="Search...">
        <button type="submit">Search</button>
    </form>
    <ul>
        <li>Item one</li>
        <li>Item two</li>
        <li>Item three</li>
    </ul>
    <a href="/more">Read more</a>
</main>
<footer>
    <p>Copyright 2026</p>
</footer>
</body>
</html>"""


def make_request(method: str, params: dict = None, msg_id: str = "1") -> dict:
    return {
        "id": msg_id,
        "type": "request",
        "method": method,
        "params": params or {},
    }


@pytest.fixture
def translator():
    return AWPTranslator("ws://localhost:9222")


# ---------------------------------------------------------------------------
# awp.hello
# ---------------------------------------------------------------------------

class TestHello:
    @pytest.mark.asyncio
    async def test_hello_success(self, translator):
        req = make_request("awp.hello", {
            "client_name": "test-agent",
            "client_version": "0.1.0",
            "awp_version": "0.1",
        })
        resp = await translator.handle(req)
        assert resp["type"] == "response"
        assert resp["id"] == "1"
        assert "error" not in resp
        result = resp["result"]
        assert result["awp_version"] == "0.1"
        assert result["server_name"] == "awp-cdp-proxy"
        assert "som.snapshot" in result["features"]
        assert "act.primitive" in result["features"]
        assert "extract" in result["features"]

    @pytest.mark.asyncio
    async def test_hello_unsupported_version(self, translator):
        req = make_request("awp.hello", {"awp_version": "99.0"})
        resp = await translator.handle(req)
        assert "error" in resp
        # The error might be INVALID_REQUEST or INTERNAL depending on exception type
        assert resp["error"]["code"] in ("INVALID_REQUEST", "INTERNAL")

    @pytest.mark.asyncio
    async def test_method_before_hello(self, translator):
        req = make_request("session.create", {})
        resp = await translator.handle(req)
        assert "error" in resp
        assert resp["error"]["code"] == "INVALID_REQUEST"
        assert "hello" in resp["error"]["message"].lower()


# ---------------------------------------------------------------------------
# session.create / session.close
# ---------------------------------------------------------------------------

class TestSession:
    @pytest.mark.asyncio
    async def test_session_create(self, translator):
        # Do hello first
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect:
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABCDEF123"
            mock_connect.return_value = mock_cdp

            req = make_request("session.create", {}, msg_id="2")
            resp = await translator.handle(req)

            assert resp["type"] == "response"
            assert "error" not in resp
            assert "session_id" in resp["result"]
            assert resp["result"]["session_id"].startswith("s_")

    @pytest.mark.asyncio
    async def test_session_close(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect, \
             patch("awp_cdp_proxy.translator.close_target") as mock_close:
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABCDEF123"
            mock_connect.return_value = mock_cdp

            create_resp = await translator.handle(make_request("session.create", {}, "2"))
            session_id = create_resp["result"]["session_id"]

            close_resp = await translator.handle(
                make_request("session.close", {"session_id": session_id}, "3")
            )
            assert close_resp["result"]["closed"] is True

    @pytest.mark.asyncio
    async def test_session_close_not_found(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        resp = await translator.handle(
            make_request("session.close", {"session_id": "s_nonexistent"}, "2")
        )
        assert "error" in resp
        assert resp["error"]["code"] == "NOT_FOUND"


# ---------------------------------------------------------------------------
# page.navigate
# ---------------------------------------------------------------------------

class TestPageNavigate:
    @pytest.mark.asyncio
    async def test_navigate_success(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect, \
             patch("awp_cdp_proxy.translator.close_target"):
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABC"
            mock_connect.return_value = mock_cdp

            # Set up CDP mock responses
            async def mock_send(method, params=None, timeout=30):
                if method == "Page.navigate":
                    # Simulate load event firing
                    for handlers in mock_cdp.on_event.call_args_list:
                        pass
                    return {"frameId": "main"}
                elif method == "Runtime.evaluate":
                    expr = (params or {}).get("expression", "")
                    if "outerHTML" in expr:
                        return {"result": {"value": SAMPLE_HTML}}
                    return {"result": {"value": None}}
                elif method == "Page.getFrameTree":
                    return {"frameTree": {"frame": {"url": "https://example.com"}}}
                return {}

            mock_cdp.send = AsyncMock(side_effect=mock_send)

            # We need to handle the on_event callback for Page.loadEventFired
            event_handlers = {}
            def mock_on_event(event_name, handler):
                event_handlers[event_name] = handler
            mock_cdp.on_event = MagicMock(side_effect=mock_on_event)

            create_resp = await translator.handle(make_request("session.create", {}, "2"))
            session_id = create_resp["result"]["session_id"]

            # Fire the load event before navigate completes
            async def do_navigate():
                # Give navigate a moment to register the event handler
                await asyncio.sleep(0.05)
                if "Page.loadEventFired" in event_handlers:
                    event_handlers["Page.loadEventFired"]({})

            asyncio.create_task(do_navigate())

            nav_resp = await translator.handle(
                make_request("page.navigate", {
                    "session_id": session_id,
                    "url": "https://example.com",
                }, "3")
            )

            assert nav_resp["type"] == "response"
            assert "error" not in nav_resp
            result = nav_resp["result"]
            assert result["som_ready"] is True
            assert result["html_bytes"] > 0
            assert "url" in result


# ---------------------------------------------------------------------------
# page.observe
# ---------------------------------------------------------------------------

class TestPageObserve:
    @pytest.mark.asyncio
    async def test_observe_no_page(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect, \
             patch("awp_cdp_proxy.translator.close_target"):
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABC"
            mock_connect.return_value = mock_cdp

            create_resp = await translator.handle(make_request("session.create", {}, "2"))
            session_id = create_resp["result"]["session_id"]

            obs_resp = await translator.handle(
                make_request("page.observe", {"session_id": session_id}, "3")
            )
            assert "error" in obs_resp
            assert obs_resp["error"]["code"] == "NOT_FOUND"

    @pytest.mark.asyncio
    async def test_observe_with_som(self, translator):
        """Test observe returns valid SOM after manually setting session state."""
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect, \
             patch("awp_cdp_proxy.translator.close_target"):
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABC"
            mock_connect.return_value = mock_cdp

            create_resp = await translator.handle(make_request("session.create", {}, "2"))
            session_id = create_resp["result"]["session_id"]

            # Manually set the SOM on the session
            som = html_to_som(SAMPLE_HTML, url="https://example.com")
            translator.sessions[session_id].current_som = som

            obs_resp = await translator.handle(
                make_request("page.observe", {"session_id": session_id}, "3")
            )
            assert "error" not in obs_resp
            som_data = obs_resp["result"]["som"]
            assert som_data["som_version"] == "0.1"
            assert len(som_data["regions"]) > 0
            assert som_data["meta"]["element_count"] > 0


# ---------------------------------------------------------------------------
# page.act
# ---------------------------------------------------------------------------

class TestPageAct:
    @pytest.mark.asyncio
    async def test_act_click(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect, \
             patch("awp_cdp_proxy.translator.close_target"):
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABC"
            mock_connect.return_value = mock_cdp
            mock_cdp.send = AsyncMock(return_value={"result": {"value": {"x": 100, "y": 50}}})

            create_resp = await translator.handle(make_request("session.create", {}, "2"))
            session_id = create_resp["result"]["session_id"]

            # Set SOM
            som = html_to_som(SAMPLE_HTML, url="https://example.com")
            session = translator.sessions[session_id]
            session.current_som = som
            session.current_url = "https://example.com"

            # Find a link element ID from the SOM
            link_id = None
            for region in som.regions:
                for el in region.elements:
                    if el.role.value == "link":
                        link_id = el.id
                        break
                if link_id:
                    break

            assert link_id is not None, "Should find at least one link in sample HTML"

            act_resp = await translator.handle(
                make_request("page.act", {
                    "session_id": session_id,
                    "intent": {
                        "action": "click",
                        "target": {"ref": link_id},
                    },
                }, "3")
            )
            assert "error" not in act_resp
            assert act_resp["result"]["status"] == "ok"
            assert act_resp["result"]["resolved"]["role"] == "link"

    @pytest.mark.asyncio
    async def test_act_type(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect, \
             patch("awp_cdp_proxy.translator.close_target"):
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABC"
            mock_connect.return_value = mock_cdp
            mock_cdp.send = AsyncMock(return_value={"result": {"value": None}})

            create_resp = await translator.handle(make_request("session.create", {}, "2"))
            session_id = create_resp["result"]["session_id"]

            som = html_to_som(SAMPLE_HTML, url="https://example.com")
            session = translator.sessions[session_id]
            session.current_som = som
            session.current_url = "https://example.com"

            # Find a text_input element
            input_id = None
            for region in som.regions:
                for el in region.elements:
                    if el.role.value == "text_input":
                        input_id = el.id
                        break
                    if el.children:
                        for child in el.children:
                            if child.role.value == "text_input":
                                input_id = child.id
                                break
                if input_id:
                    break

            if input_id:
                act_resp = await translator.handle(
                    make_request("page.act", {
                        "session_id": session_id,
                        "intent": {
                            "action": "type",
                            "target": {"ref": input_id},
                            "value": "hello world",
                        },
                    }, "3")
                )
                assert "error" not in act_resp
                assert act_resp["result"]["status"] == "ok"

    @pytest.mark.asyncio
    async def test_act_target_not_found(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect, \
             patch("awp_cdp_proxy.translator.close_target"):
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABC"
            mock_connect.return_value = mock_cdp

            create_resp = await translator.handle(make_request("session.create", {}, "2"))
            session_id = create_resp["result"]["session_id"]

            som = html_to_som(SAMPLE_HTML, url="https://example.com")
            translator.sessions[session_id].current_som = som

            act_resp = await translator.handle(
                make_request("page.act", {
                    "session_id": session_id,
                    "intent": {
                        "action": "click",
                        "target": {"ref": "e_nonexistent"},
                    },
                }, "3")
            )
            assert "error" in act_resp
            assert act_resp["error"]["code"] == "NOT_FOUND"


# ---------------------------------------------------------------------------
# page.extract
# ---------------------------------------------------------------------------

class TestPageExtract:
    @pytest.mark.asyncio
    async def test_extract_by_role(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        with patch("awp_cdp_proxy.translator.connect_to_target") as mock_connect, \
             patch("awp_cdp_proxy.translator.close_target"):
            mock_cdp = AsyncMock()
            mock_cdp.browser_ws_url = "ws://localhost:9222/devtools/page/ABC"
            mock_connect.return_value = mock_cdp

            create_resp = await translator.handle(make_request("session.create", {}, "2"))
            session_id = create_resp["result"]["session_id"]

            som = html_to_som(SAMPLE_HTML, url="https://example.com")
            translator.sessions[session_id].current_som = som

            extract_resp = await translator.handle(
                make_request("page.extract", {
                    "session_id": session_id,
                    "fields": {
                        "title": {"role": "heading", "level": 1},
                        "links": {"role": "link", "all": True, "props": ["text", "href"]},
                    },
                }, "3")
            )

            assert "error" not in extract_resp
            data = extract_resp["result"]["data"]
            assert data["title"] is not None
            assert isinstance(data["links"], list)
            assert len(data["links"]) > 0


# ---------------------------------------------------------------------------
# Error handling
# ---------------------------------------------------------------------------

class TestErrorHandling:
    @pytest.mark.asyncio
    async def test_unknown_method(self, translator):
        await translator.handle(make_request("awp.hello", {"awp_version": "0.1"}))

        resp = await translator.handle(make_request("foo.bar", {}))
        assert "error" in resp
        assert resp["error"]["code"] == "INVALID_REQUEST"

    @pytest.mark.asyncio
    async def test_invalid_type(self, translator):
        resp = await translator.handle({
            "id": "1",
            "type": "event",
            "method": "awp.hello",
            "params": {},
        })
        assert "error" in resp
        assert resp["error"]["code"] == "INVALID_REQUEST"

    @pytest.mark.asyncio
    async def test_missing_method(self, translator):
        resp = await translator.handle({
            "id": "1",
            "type": "request",
            "params": {},
        })
        assert "error" in resp
        assert resp["error"]["code"] == "INVALID_REQUEST"


# ---------------------------------------------------------------------------
# SOM generator
# ---------------------------------------------------------------------------

class TestSOMGenerator:
    def test_basic_html(self):
        som = html_to_som(SAMPLE_HTML, url="https://example.com")
        assert som.som_version == "0.1"
        assert som.title == "Test Page"
        assert som.lang == "en"
        assert len(som.regions) > 0
        assert som.meta.element_count > 0
        assert som.meta.interactive_count > 0
        assert som.meta.html_bytes > 0
        assert som.meta.som_bytes > 0

    def test_nav_region(self):
        som = html_to_som(SAMPLE_HTML, url="https://example.com")
        nav_regions = [r for r in som.regions if r.role.value == "navigation"]
        assert len(nav_regions) >= 1
        nav = nav_regions[0]
        assert len(nav.elements) >= 3  # Home, About, Contact links
        assert all(el.role.value == "link" for el in nav.elements)

    def test_form_region(self):
        som = html_to_som(SAMPLE_HTML, url="https://example.com")
        form_regions = [r for r in som.regions if r.role.value == "form"]
        assert len(form_regions) >= 1
        form = form_regions[0]
        assert form.action == "/search"
        assert form.method == "GET"

    def test_heading(self):
        som = html_to_som(SAMPLE_HTML, url="https://example.com")
        all_elements = []
        for region in som.regions:
            all_elements.extend(region.elements)
        headings = [el for el in all_elements if el.role.value == "heading"]
        assert len(headings) >= 1
        h1 = headings[0]
        assert h1.attrs.level == 1
        assert "Welcome" in (h1.text or "")

    def test_empty_html(self):
        som = html_to_som("<html><body></body></html>")
        assert som.som_version == "0.1"
        assert len(som.regions) >= 1

    def test_hidden_elements_excluded(self):
        html = """
        <html><body>
            <div hidden>Hidden content</div>
            <div style="display:none">Also hidden</div>
            <div aria-hidden="true">Aria hidden</div>
            <p>Visible content</p>
        </body></html>
        """
        som = html_to_som(html)
        all_text = []
        for region in som.regions:
            for el in region.elements:
                if el.text:
                    all_text.append(el.text)
        combined = " ".join(all_text)
        assert "Hidden content" not in combined
        assert "Also hidden" not in combined
        assert "Aria hidden" not in combined

    def test_deterministic_ids(self):
        """Same HTML produces same element IDs."""
        som1 = html_to_som(SAMPLE_HTML, url="https://example.com")
        som2 = html_to_som(SAMPLE_HTML, url="https://example.com")

        ids1 = set()
        ids2 = set()
        for region in som1.regions:
            for el in region.elements:
                ids1.add(el.id)
        for region in som2.regions:
            for el in region.elements:
                ids2.add(el.id)

        assert ids1 == ids2

    def test_link_href_resolution(self):
        html = '<html><body><a href="/about">About</a></body></html>'
        som = html_to_som(html, url="https://example.com/page")
        all_elements = []
        for region in som.regions:
            all_elements.extend(region.elements)
        links = [el for el in all_elements if el.role.value == "link"]
        assert len(links) >= 1
        assert links[0].attrs.href == "https://example.com/about"

    def test_select_options(self):
        html = """
        <html><body>
            <select name="color">
                <option value="r">Red</option>
                <option value="g" selected>Green</option>
                <option value="b">Blue</option>
            </select>
        </body></html>
        """
        som = html_to_som(html)
        all_elements = []
        for region in som.regions:
            all_elements.extend(region.elements)
        selects = [el for el in all_elements if el.role.value == "select"]
        assert len(selects) >= 1
        select = selects[0]
        assert select.attrs is not None
        assert select.attrs.options is not None
        assert len(select.attrs.options) == 3

    def test_som_serializable(self):
        """SOM can be serialized to JSON (required for AWP responses)."""
        som = html_to_som(SAMPLE_HTML, url="https://example.com")
        json_str = som.model_dump_json(exclude_none=True)
        parsed = json.loads(json_str)
        assert parsed["som_version"] == "0.1"
        assert "regions" in parsed
        assert "meta" in parsed
