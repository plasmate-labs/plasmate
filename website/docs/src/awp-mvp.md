---
tags: [project, agentic-browser, protocol, awp, mvp]
created: 2026-03-16
status: draft
version: 0.1-mvp
---

# AWP v0.1 MVP - Implementable Specification

This document is the tightened, build-ready subset of the full AWP draft. Everything here is MUST-implement for the Rust PoC. Anything not listed here is deferred to v0.2.

An agentic engineering team should be able to implement a working AWP server from this document alone.

---

## 1. Scope

### v0.1 implements

| Method | Purpose |
|---|---|
| `awp.hello` | Handshake, version and encoding negotiation |
| `session.create` | Create in-memory session |
| `session.close` | Destroy session |
| `page.navigate` | Fetch URL, parse HTML, compile SOM |
| `page.observe` | Return SOM snapshot |
| `page.act` | Execute primitive actions (click, type, select) |
| `page.extract` | Extract structured data from SOM |

### v0.1 defers (v0.2 backlog)

| Method | Reason |
|---|---|
| `page.create` / `page.close` | v0.1 uses one implicit page per session |
| `page.observe` mutations mode | Requires DOM mutation tracking (needs JS runtime) |
| `session.export` / `session.import` | Persistence layer not in PoC |
| `session.set_network` | Stealth/proxy not in PoC |
| `session.cookies.*` | Cookie jar exists internally but not exposed via protocol yet |
| `skills.list` / `skills.invoke` | Wasm runtime not in PoC |
| Telemetry events | Nice-to-have; PoC logs to tracing instead |
| `auth.bearer` / `auth.hmac` | PoC runs local-only, no auth needed |
| MessagePack encoding | v0.1 uses JSON only; MessagePack added in v0.2 |
| Compression (zstd) | Deferred |

---

## 2. Transport

**WebSocket** over TCP. Single connection, single session (v0.1 simplification).

- Server listens on configurable `host:port` (default `127.0.0.1:9222`).
- Client connects via `ws://{host}:{port}/`.
- All messages are UTF-8 JSON text frames.
- Binary frames are reserved for future MessagePack (v0.2).

### Connection lifecycle

1. Client opens WebSocket.
2. Client MUST send `awp.hello` as first message.
3. Server responds with capabilities.
4. Client sends requests; server responds and may emit events.
5. Either side may close the WebSocket.

---

## 3. Message Envelope

Every message is a JSON object with this shape:

```typescript
// Request (client -> server)
{
  id: string,           // unique per-connection, client-generated
  type: "request",
  method: string,       // e.g. "page.navigate"
  params: object        // method-specific
}

// Response (server -> client)
{
  id: string,           // matches request id
  type: "response",
  result?: object,      // present on success
  error?: {             // present on failure
    code: string,
    message: string,
    details?: object
  }
}

// Event (server -> client, unsolicited)
{
  type: "event",
  method: string,       // e.g. "page.load_complete"
  params: object
}
```

### Rules

- `id` MUST be a non-empty string. UUIDs or incrementing integers as strings are both fine.
- Exactly one of `result` or `error` MUST be present in a response.
- Events have no `id` field.
- Unknown fields MUST be ignored (forward compatibility).

### Error codes (v0.1 subset)

| Code | When |
|---|---|
| `INVALID_REQUEST` | Malformed JSON, missing required fields, unknown method |
| `NOT_FOUND` | Session or element reference doesn't exist |
| `TIMEOUT` | Navigation or wait exceeded timeout |
| `NAVIGATION_FAILED` | HTTP error, DNS failure, TLS error |
| `INTERNAL` | Unexpected server error |

---

## 4. Methods

### 4.1 awp.hello

First message after WebSocket connect. Negotiates protocol version.

**Request**

```json
{
  "id": "1",
  "type": "request",
  "method": "awp.hello",
  "params": {
    "client_name": "my-agent",
    "client_version": "0.1.0",
    "awp_version": "0.1"
  }
}
```

**Response**

```json
{
  "id": "1",
  "type": "response",
  "result": {
    "awp_version": "0.1",
    "server_name": "plasmate",
    "server_version": "0.1.0",
    "features": ["som.snapshot", "act.primitive", "extract"]
  }
}
```

**Behavior**
- If `awp_version` is not supported, respond with error `INVALID_REQUEST`.
- Server MUST NOT process any other method before `awp.hello` succeeds.

---

### 4.2 session.create

Creates a browsing session. v0.1: one session per WebSocket connection.

**Request**

```json
{
  "id": "2",
  "type": "request",
  "method": "session.create",
  "params": {
    "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
    "locale": "en-US",
    "timeout_ms": 30000
  }
}
```

All params are optional. Defaults:
- `user_agent`: Chrome 128 on macOS
- `locale`: `en-US`
- `timeout_ms`: `30000` (default navigation timeout)

**Response**

```json
{
  "id": "2",
  "type": "response",
  "result": {
    "session_id": "s_a1b2c3d4"
  }
}
```

**Behavior**
- Allocates in-memory session with empty cookie jar.
- v0.1: only one active session per connection. Creating a second closes the first.

---

### 4.3 session.close

Destroys a session and frees resources.

**Request**

```json
{
  "id": "3",
  "type": "request",
  "method": "session.close",
  "params": {
    "session_id": "s_a1b2c3d4"
  }
}
```

**Response**

```json
{
  "id": "3",
  "type": "response",
  "result": {
    "closed": true
  }
}
```

---

### 4.4 page.navigate

Fetches a URL and compiles the HTML into a SOM.

**Request**

```json
{
  "id": "10",
  "type": "request",
  "method": "page.navigate",
  "params": {
    "session_id": "s_a1b2c3d4",
    "url": "https://news.ycombinator.com",
    "timeout_ms": 15000
  }
}
```

**Response**

```json
{
  "id": "10",
  "type": "response",
  "result": {
    "url": "https://news.ycombinator.com",
    "status": 200,
    "content_type": "text/html",
    "html_bytes": 42891,
    "som_ready": true,
    "load_ms": 312
  }
}
```

**Behavior**
1. HTTP GET with session's User-Agent and cookie jar.
2. Follow redirects (max 10).
3. Store response cookies.
4. Parse HTML with `html5ever`.
5. Compile SOM (see Section 5).
6. Store SOM in session state (overwriting previous page).

**Errors**
- DNS failure, TLS error, HTTP 4xx/5xx without HTML body: `NAVIGATION_FAILED`
- Timeout: `TIMEOUT`

---

### 4.5 page.observe

Returns the current page's SOM snapshot.

**Request**

```json
{
  "id": "20",
  "type": "request",
  "method": "page.observe",
  "params": {
    "session_id": "s_a1b2c3d4"
  }
}
```

**Response**

```json
{
  "id": "20",
  "type": "response",
  "result": {
    "som": { ... }
  }
}
```

The `som` object follows the SOM schema defined in Section 5.

**Errors**
- No page loaded yet: `NOT_FOUND`

---

### 4.6 page.act

Executes an action against a SOM element. v0.1 supports primitive actions only.

**Request**

```json
{
  "id": "30",
  "type": "request",
  "method": "page.act",
  "params": {
    "session_id": "s_a1b2c3d4",
    "intent": {
      "action": "click",
      "target": {
        "ref": "e_8f3a1b"
      }
    }
  }
}
```

**Target resolution** (tried in order):

1. `ref` - direct element_id from SOM
2. `text` + `role` - semantic query (find element matching role and visible text)
3. `css` - CSS selector fallback

```json
// Target by ref
{"ref": "e_8f3a1b"}

// Target by semantic query
{"text": "Add to Cart", "role": "button"}

// Target by CSS selector
{"css": "button.btn-primary"}

// Target with fallback chain
{"ref": "e_8f3a1b", "fallback": {"text": "Submit", "role": "button"}}
```

**v0.1 actions**

| Action | Params | Behavior |
|---|---|---|
| `click` | `target` | Resolve target. If target is a link (`<a href>`), navigate to href and recompile SOM. If target is a button within a form, treat as form submission. Otherwise return resolved element. |
| `type` | `target`, `value` (string) | Resolve target to an input/textarea. Set its value in the SOM. Return updated element. |
| `select` | `target`, `value` (string) | Resolve target to a select/radio group. Set selected option in the SOM. Return updated element. |
| `scroll` | `direction` ("down"\|"up"), `amount` (optional int, default 1 screen) | No-op in v0.1 (no viewport), but accept and return ok for protocol compatibility. |

**Response**

```json
{
  "id": "30",
  "type": "response",
  "result": {
    "status": "ok",
    "resolved": {
      "element_id": "e_8f3a1b",
      "role": "button",
      "text": "Add to Cart"
    },
    "effects": {
      "navigated": false,
      "som_changed": true
    }
  }
}
```

**Errors**
- Target not found: `NOT_FOUND` with details including the attempted resolution strategy.

---

### 4.7 page.extract

Extracts structured data from the current SOM.

**Request**

```json
{
  "id": "40",
  "type": "request",
  "method": "page.extract",
  "params": {
    "session_id": "s_a1b2c3d4",
    "fields": {
      "title": {"role": "heading", "level": 1},
      "links": {"role": "link", "all": true, "props": ["text", "href"]},
      "price": {"text_match": "\\$\\d+\\.\\d{2}"}
    }
  }
}
```

**Response**

```json
{
  "id": "40",
  "type": "response",
  "result": {
    "data": {
      "title": "Widget Pro",
      "links": [
        {"text": "Home", "href": "/"},
        {"text": "Products", "href": "/products"}
      ],
      "price": "$49.99"
    },
    "provenance": {
      "title": "e_1a2b3c",
      "price": "e_7d8e9f"
    }
  }
}
```

**Field query types**

| Query | Meaning |
|---|---|
| `{"role": "heading", "level": 1}` | First element with role=heading and level=1 |
| `{"role": "link", "all": true}` | All elements with role=link |
| `{"text_match": "regex"}` | First element whose text matches regex |
| `{"ref": "e_xxx"}` | Specific element by ID |

When `all: true` is set, result is an array. Otherwise, result is the first match (or null).

`provenance` maps each field to the element_id it was extracted from. This allows agents to verify and re-target.

---

## 5. SOM Schema (v0.1)

### 5.1 Top-level structure

```json
{
  "som_version": "0.1",
  "url": "https://example.com/page",
  "title": "Page Title",
  "lang": "en",
  "regions": [ ... ],
  "meta": {
    "html_bytes": 42891,
    "som_bytes": 1823,
    "element_count": 47,
    "interactive_count": 12
  }
}
```

### 5.2 Regions

Regions are top-level semantic areas. Determined by HTML5 landmarks or heuristics.

```json
{
  "id": "r_nav",
  "role": "navigation",
  "label": "Main navigation",
  "elements": [ ... ]
}
```

**Region roles (v0.1)**

| Role | Sources |
|---|---|
| `navigation` | `<nav>`, `[role=navigation]`, heuristic: list of links in header |
| `main` | `<main>`, `[role=main]`, heuristic: largest content block |
| `aside` | `<aside>`, `[role=complementary]` |
| `header` | `<header>`, `[role=banner]` (page-level only) |
| `footer` | `<footer>`, `[role=contentinfo]` (page-level only) |
| `form` | `<form>` that is a direct child or prominent |
| `dialog` | `<dialog>`, `[role=dialog]`, `[role=alertdialog]` |
| `content` | Fallback: significant content blocks not matched above |

If no landmarks are found, the entire body becomes a single `content` region.

### 5.3 Elements

Every element in the SOM has:

```json
{
  "id": "e_8f3a1b",
  "role": "button",
  "text": "Add to Cart",
  "actions": ["click"],
  "attrs": {}
}
```

**Required fields:**
- `id` - stable element identifier (see Section 6)
- `role` - semantic role (see table below)

**Optional fields:**
- `text` - visible text content (trimmed, max 200 chars)
- `label` - accessible label if different from text (aria-label, title, label[for])
- `actions` - array of available actions: `["click"]`, `["type", "clear"]`, `["select"]`, etc.
- `attrs` - role-specific attributes (see below)
- `children` - nested elements (for lists, tables, forms)

**Element roles (v0.1)**

| Role | HTML Sources | attrs |
|---|---|---|
| `link` | `<a href>` | `href` |
| `button` | `<button>`, `input[type=submit\|button\|reset]`, `[role=button]` | `disabled`, `form_id` |
| `text_input` | `input[type=text\|email\|password\|search\|tel\|url\|number]` | `input_type`, `placeholder`, `value`, `required`, `disabled` |
| `textarea` | `<textarea>` | `placeholder`, `value`, `required` |
| `select` | `<select>` | `options: [{value, text, selected}]`, `multiple`, `required` |
| `checkbox` | `input[type=checkbox]`, `[role=checkbox]` | `checked`, `disabled` |
| `radio` | `input[type=radio]`, `[role=radio]` | `checked`, `group`, `disabled` |
| `heading` | `<h1>` through `<h6>` | `level` (1-6) |
| `image` | `<img>`, `<picture>`, `[role=img]` | `alt`, `src` |
| `list` | `<ul>`, `<ol>` | `ordered`, `items: [{text}]` |
| `table` | `<table>` | `headers: [string]`, `rows: [[string]]` (max 20 rows in snapshot) |
| `paragraph` | `<p>`, bare text blocks | (none) |
| `section` | `<section>`, `<article>` | `section_label` |
| `separator` | `<hr>` | (none) |

### 5.4 Text handling

- All text is trimmed and collapsed (consecutive whitespace becomes single space).
- Text longer than 200 characters is truncated with `...` suffix.
- Hidden text (`display:none`, `visibility:hidden`, `aria-hidden=true`) is excluded.
  - v0.1 caveat: without CSS computation, hidden detection uses heuristics (inline `style="display:none"`, `hidden` attribute, `aria-hidden`).

### 5.5 Form grouping

When a `<form>` contains interactive elements, those elements are nested:

```json
{
  "id": "r_form1",
  "role": "form",
  "label": "Login",
  "action": "/api/login",
  "method": "POST",
  "elements": [
    {"id": "e_1", "role": "text_input", "label": "Email", "attrs": {"input_type": "email", "required": true}},
    {"id": "e_2", "role": "text_input", "label": "Password", "attrs": {"input_type": "password", "required": true}},
    {"id": "e_3", "role": "button", "text": "Sign In", "actions": ["click"]}
  ]
}
```

### 5.6 Omitted content

SOM v0.1 omits:
- `<script>`, `<style>`, `<noscript>`, `<template>`
- HTML comments
- SVG content (except `<svg>` with `role=img` and an accessible name)
- `<meta>`, `<link>` (consumed for page metadata, not output)
- Inline styles and CSS classes
- `data-*` attributes (except `data-testid` which is preserved for fallback targeting)
- Empty elements (no text, no interactive role, no children with content)
- Decorative images (`alt=""` or `role=presentation`)

---

## 6. Element ID Generation

### Algorithm

Element IDs must be deterministic: same page structure produces same IDs.

```
input = "{origin}|{role}|{accessible_name}|{dom_path}"
element_id = "e_" + hex(sha256(input))[0..12]
```

Where:
- `origin` = URL origin (`https://example.com`)
- `role` = SOM role string (`button`, `link`, etc.)
- `accessible_name` = normalized: lowercased, trimmed, first 100 chars
- `dom_path` = slash-separated indices from `<body>`: `0/3/1/0` (0th child of body, 3rd child of that, etc.)

### Collisions

If two elements produce the same ID (rare but possible), append a counter: `e_8f3a1b_2`.

### Stability

IDs are stable across fetches of the same page as long as:
- The DOM structure doesn't change
- The element text doesn't change

This is acceptable for static and server-rendered pages. SPA-driven changes (v0.2 with JS execution) will require mutation tracking.

---

## 7. Full Worked Example

### Agent wants to search Hacker News

**Step 1: Connect and handshake**

```
-> {"id":"1","type":"request","method":"awp.hello","params":{"client_name":"demo","client_version":"0.1.0","awp_version":"0.1"}}
<- {"id":"1","type":"response","result":{"awp_version":"0.1","server_name":"plasmate","server_version":"0.1.0","features":["som.snapshot","act.primitive","extract"]}}
```

**Step 2: Create session**

```
-> {"id":"2","type":"request","method":"session.create","params":{}}
<- {"id":"2","type":"response","result":{"session_id":"s_1"}}
```

**Step 3: Navigate**

```
-> {"id":"3","type":"request","method":"page.navigate","params":{"session_id":"s_1","url":"https://news.ycombinator.com"}}
<- {"id":"3","type":"response","result":{"url":"https://news.ycombinator.com","status":200,"content_type":"text/html; charset=utf-8","html_bytes":42891,"som_ready":true,"load_ms":287}}
```

**Step 4: Observe**

```
-> {"id":"4","type":"request","method":"page.observe","params":{"session_id":"s_1"}}
<- {"id":"4","type":"response","result":{"som":{"som_version":"0.1","url":"https://news.ycombinator.com","title":"Hacker News","lang":"en","regions":[{"id":"r_nav","role":"navigation","elements":[{"id":"e_a1b2c3","role":"link","text":"Hacker News","attrs":{"href":"https://news.ycombinator.com"},"actions":["click"]},{"id":"e_d4e5f6","role":"link","text":"new","attrs":{"href":"newest"},"actions":["click"]},{"id":"e_g7h8i9","role":"link","text":"past","attrs":{"href":"front"},"actions":["click"]},{"id":"e_j0k1l2","role":"link","text":"comments","attrs":{"href":"newcomments"},"actions":["click"]},{"id":"e_m3n4o5","role":"link","text":"ask","attrs":{"href":"ask"},"actions":["click"]},{"id":"e_p6q7r8","role":"link","text":"show","attrs":{"href":"show"},"actions":["click"]},{"id":"e_s9t0u1","role":"link","text":"jobs","attrs":{"href":"jobs"},"actions":["click"]},{"id":"e_v2w3x4","role":"link","text":"submit","attrs":{"href":"submit"},"actions":["click"]}]},{"id":"r_main","role":"main","elements":[{"id":"e_story1","role":"link","text":"Show HN: Plasmate - an agent-native headless browser engine","attrs":{"href":"https://plasmate.app"},"actions":["click"]},{"id":"e_meta1","role":"paragraph","text":"142 points by thinker 3 hours ago | 89 comments"}]}],"meta":{"html_bytes":42891,"som_bytes":1847,"element_count":94,"interactive_count":38}}}}
```

**Step 5: Extract all story links**

```
-> {"id":"5","type":"request","method":"page.extract","params":{"session_id":"s_1","fields":{"stories":{"role":"link","all":true,"props":["text","href"]}}}}
<- {"id":"5","type":"response","result":{"data":{"stories":[{"text":"Show HN: Plasmate...","href":"https://plasmate.app"},{"text":"Why Rust is eating the world","href":"https://..."}]},"provenance":{"stories":["e_story1","e_story2"]}}}
```

**Step 6: Click a link**

```
-> {"id":"6","type":"request","method":"page.act","params":{"session_id":"s_1","intent":{"action":"click","target":{"ref":"e_story1"}}}}
<- {"id":"6","type":"response","result":{"status":"ok","resolved":{"element_id":"e_story1","role":"link","text":"Show HN: Plasmate..."},"effects":{"navigated":true,"som_changed":true}}}
```

After navigation, `page.observe` would return the SOM for the new page.

---

## 8. Python Client Example

This is the reference client implementation that ships with the PoC.

```python
"""
plasmate_client.py - AWP v0.1 Python client

Usage:
    client = PlasmateClient("ws://127.0.0.1:9222")
    await client.connect()
    session = await client.create_session()
    await client.navigate(session, "https://example.com")
    som = await client.observe(session)
    print(json.dumps(som, indent=2))
"""

import asyncio
import json
import uuid
import websockets


class PlasmateClient:
    def __init__(self, url: str = "ws://127.0.0.1:9222"):
        self.url = url
        self.ws = None
        self._pending = {}

    async def connect(self):
        self.ws = await websockets.connect(self.url)
        asyncio.create_task(self._reader())
        result = await self._request("awp.hello", {
            "client_name": "plasmate-python",
            "client_version": "0.1.0",
            "awp_version": "0.1"
        })
        return result

    async def create_session(self, **kwargs) -> str:
        result = await self._request("session.create", kwargs)
        return result["session_id"]

    async def close_session(self, session_id: str):
        return await self._request("session.close", {"session_id": session_id})

    async def navigate(self, session_id: str, url: str, timeout_ms: int = 15000):
        return await self._request("page.navigate", {
            "session_id": session_id,
            "url": url,
            "timeout_ms": timeout_ms
        })

    async def observe(self, session_id: str) -> dict:
        result = await self._request("page.observe", {"session_id": session_id})
        return result["som"]

    async def act(self, session_id: str, action: str, target: dict, value: str = None):
        intent = {"action": action, "target": target}
        if value is not None:
            intent["value"] = value
        return await self._request("page.act", {
            "session_id": session_id,
            "intent": intent
        })

    async def extract(self, session_id: str, fields: dict) -> dict:
        result = await self._request("page.extract", {
            "session_id": session_id,
            "fields": fields
        })
        return result["data"]

    async def _request(self, method: str, params: dict) -> dict:
        msg_id = str(uuid.uuid4())[:8]
        msg = {"id": msg_id, "type": "request", "method": method, "params": params}
        future = asyncio.get_event_loop().create_future()
        self._pending[msg_id] = future
        await self.ws.send(json.dumps(msg))
        result = await asyncio.wait_for(future, timeout=30)
        return result

    async def _reader(self):
        async for raw in self.ws:
            msg = json.loads(raw)
            if msg.get("type") == "response" and msg.get("id") in self._pending:
                future = self._pending.pop(msg["id"])
                if "error" in msg:
                    future.set_exception(
                        PlasmateError(msg["error"]["code"], msg["error"]["message"])
                    )
                else:
                    future.set_result(msg.get("result", {}))
            # Events (v0.2) would be handled here


class PlasmateError(Exception):
    def __init__(self, code: str, message: str):
        self.code = code
        super().__init__(f"[{code}] {message}")


# --- Quick test ---
async def main():
    client = PlasmateClient()
    await client.connect()
    session = await client.create_session()

    nav = await client.navigate(session, "https://news.ycombinator.com")
    print(f"Loaded {nav['url']} ({nav['html_bytes']} bytes HTML, {nav['load_ms']}ms)")

    som = await client.observe(session)
    print(f"SOM: {som['meta']['som_bytes']} bytes, "
          f"{som['meta']['element_count']} elements, "
          f"{som['meta']['interactive_count']} interactive")

    print(f"\nToken ratio: {som['meta']['html_bytes'] / som['meta']['som_bytes']:.1f}x reduction")

    links = await client.extract(session, {
        "stories": {"role": "link", "all": True, "props": ["text", "href"]}
    })
    for link in links.get("stories", [])[:5]:
        print(f"  - {link['text'][:60]}")

    await client.close_session(session)

if __name__ == "__main__":
    asyncio.run(main())
```

---

## 9. Benchmark Harness Specification

The PoC MUST ship with a benchmark tool.

### CLI

```bash
plasmate bench --urls urls.txt --output report.md
```

### urls.txt format

One URL per line. Comments with `#`.

```
# Static sites
https://news.ycombinator.com
https://en.wikipedia.org/wiki/Rust_(programming_language)
https://github.com/nickel-org/nickel.rs
https://stripe.com/docs/api

# E-commerce
https://www.shopify.com/shop-themes/free
https://store.steampowered.com

# News
https://www.bbc.com/news
https://www.reuters.com
```

### Output (report.md)

```markdown
# Plasmate SOM Benchmark Report

Date: 2026-03-XX
Engine: plasmate v0.1.0
URLs tested: 50
Successful: 47 (94%)

## Summary

| Metric | Mean | Median | P95 |
|---|---|---|---|
| HTML bytes | 38,412 | 32,100 | 89,200 |
| SOM bytes | 2,147 | 1,830 | 4,910 |
| Byte ratio | 17.9x | 17.5x | 18.2x |
| HTML tokens (est) | 9,603 | 8,025 | 22,300 |
| SOM tokens (est) | 537 | 458 | 1,228 |
| Token ratio | 17.9x | 17.5x | 18.2x |
| Elements found | 84 | 72 | 198 |
| Interactive found | 23 | 18 | 52 |
| Fetch time (ms) | 412 | 318 | 1,240 |
| Parse+SOM time (ms) | 8 | 6 | 18 |

## Per-URL Results

| URL | HTML bytes | SOM bytes | Ratio | Elements | Interactive | Status |
|---|---|---|---|---|---|---|
| news.ycombinator.com | 42,891 | 1,847 | 23.2x | 94 | 38 | ok |
| en.wikipedia.org/... | 218,445 | 8,932 | 24.5x | 312 | 89 | ok |
| ... | | | | | | |
```

### Token estimation

v0.1 uses `bytes / 4` as rough token estimate (works within 20% for English text and cl100k_base). A companion Python script using `tiktoken` produces exact counts for the final report.

---

## 10. v0.2 Backlog

Methods and features explicitly deferred. Listed here so the protocol is designed to accommodate them without breaking changes.

### v0.2 Methods

| Method | Description |
|---|---|
| `page.create` / `page.close` | Multi-page support within a session |
| `page.observe` with `mode: "mutations"` | Incremental SOM updates via JSON Patch |
| `session.cookies.get/set/clear` | Explicit cookie management |
| `session.export` / `session.import` | Portable session state |
| `session.set_network` | Proxy and impersonation configuration |
| `skills.list` / `skills.invoke` | Wasm skill system |
| `page.screenshot` | Optional pixel output (not primary, but useful for debugging) |
| `page.evaluate_js` | Raw JS evaluation (requires V8, privileged action) |

### v0.2 Transport

| Feature | Description |
|---|---|
| MessagePack encoding | Binary serialization (30-50% smaller than JSON) |
| zstd compression | Per-message compression |
| Connection multiplexing | Multiple sessions per WebSocket |

### v0.2 SOM Enhancements

| Feature | Description |
|---|---|
| Mutation tracking | DOM MutationObserver equivalent, JSON Patch output |
| Dynamic content | Content loaded by JS execution |
| Shadow DOM | Web Components support |
| iframe extraction | Nested browsing context SOM |

### v0.2 Engine Features

| Feature | Description |
|---|---|
| `rusty_v8` JS execution | SPA rendering |
| Stealth TLS (rustls fork) | JA3/JA4 fingerprint control |
| HTTP/2 fingerprinting | SETTINGS, WINDOW_UPDATE, PRIORITY tuning |
| Proxy rotation | SOCKS5/HTTP with strategies |
| RocksDB persistence | Durable sessions, cookies, state |

---

*End of AWP v0.1 MVP Specification*
