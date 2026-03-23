---
tags: [project, agentic-browser, protocol, awp]
created: 2026-03-15
status: draft
version: 0.1
---

# AWP - Agent Web Protocol (Draft Specification v0.1)

**Project:** Plasmate

## Abstract

The Agent Web Protocol (AWP) is a purpose-built, intent-forward protocol for controlling a headless browser engine in AI agent workloads.

AWP is designed as a replacement for the Chrome DevTools Protocol (CDP) in agentic environments. CDP is a debugging interface optimized for human developers and pixel-oriented automation. AWP is optimized for:

- Semantic interaction, not coordinate clicking
- Token efficiency, not visual fidelity
- Massive concurrency, not single-user tabs
- Determinism, not rendering
- Extensibility via WebAssembly skills

AWP is transport-agnostic but specified primarily over WebSocket. Payload encoding is binary (MessagePack) by default.

## Status of This Document

This is a draft for discussion and implementation planning. It is intentionally exhaustive so an agentic engineering team can implement:

1. A minimal AWP server
2. A minimal AWP client SDK
3. A Rust proof of concept focused on SOM generation and basic action execution

## Table of Contents

1. [Goals and Non-Goals](#goals-and-non-goals)
2. [Terminology](#terminology)
3. [Architecture Overview](#architecture-overview)
4. [Versioning and Capability Negotiation](#versioning-and-capability-negotiation)
5. [Transport](#transport)
6. [Encoding](#encoding)
7. [Message Model](#message-model)
8. [Error Model](#error-model)
9. [Authentication and Authorization](#authentication-and-authorization)
10. [Resources and Identifiers](#resources-and-identifiers)
11. [Sessions](#sessions)
12. [Page Lifecycle](#page-lifecycle)
13. [Observation: SOM Snapshots and Mutations](#observation-som-snapshots-and-mutations)
14. [Actions: Intent-Based Interaction](#actions-intent-based-interaction)
15. [Data Extraction](#data-extraction)
16. [Network Controls: Proxy, Profiles, Stealth](#network-controls-proxy-profiles-stealth)
17. [State: Cookies, Storage, Downloads](#state-cookies-storage-downloads)
18. [Skills: WebAssembly Extensions](#skills-webassembly-extensions)
19. [Telemetry and Auditing](#telemetry-and-auditing)
20. [Determinism and Reproducibility](#determinism-and-reproducibility)
21. [Examples](#examples)
22. [Compatibility and Migration from CDP](#compatibility-and-migration-from-cdp)
23. [Implementation Notes for a Rust PoC](#implementation-notes-for-a-rust-poc)

---

## Goals and Non-Goals

### Goals

1. **Intent-first**: agents issue semantic actions, not pixels.
2. **Token efficiency**: primary observation output is the Semantic Object Model (SOM), not screenshots.
3. **Deterministic references**: stable element addressing across SPA mutations.
4. **Massive concurrency**: protocol supports thousands of sessions with strict resource limits.
5. **Robustness**: sessions survive reconnects, and operations are idempotent when practical.
6. **Extensibility**: site-specific logic is implemented as Wasm skills, not baked into the engine.
7. **Observability**: first-class telemetry for latency, errors, retries, costs, and provenance.
8. **Security**: clear trust boundaries and permission model for high-risk operations.

### Non-Goals

1. AWP does not define a UI, rendering pipeline, or pixel output requirements.
2. AWP does not define scraping legality or usage policy.
3. AWP does not mandate a specific SOM schema beyond normative minimums.
4. AWP does not prescribe a particular LLM, agent framework, or planning strategy.

---

## Terminology

- **Agent**: A client system (often LLM-driven) that issues commands.
- **Engine**: The headless browser implementation that executes commands.
- **AWP Client**: Library used by the Agent to speak AWP.
- **AWP Server**: Component inside the Engine that receives and executes AWP messages.
- **Session**: A durable context including cookies, storage, proxy settings, and optional persistence.
- **Page**: A browsing context within a Session, analogous to a tab.
- **Frame**: An iframe or sub-document within a Page.
- **SOM**: Semantic Object Model. A token-efficient representation of interactive and meaningful content.
- **SOM Snapshot**: Full SOM document at a point in time.
- **SOM Mutation**: Incremental changes to the SOM using JSON Patch semantics.
- **Intent**: A semantic operation request (for example, `add_to_cart`).
- **Primitive Action**: Low-level operations (click, type, navigate).
- **Skill**: WebAssembly module that provides domain or application specific automation.
- **Target**: An element reference or selection query that resolves to an element.

---

## Architecture Overview

AWP is designed around a simple loop:

1. Agent requests observation (`page.observe`) and receives a SOM snapshot or mutations.
2. Agent decides action and sends intent (`page.act`) targeting SOM elements.
3. Engine executes using primitives or skills.
4. Engine returns structured results and emits events.

AWP explicitly separates:

- **Observation**: what the Agent sees (SOM)
- **Action**: what the Agent does (Intent)
- **State**: cookies/storage/session persistence
- **Network**: proxy profiles, browser impersonation profiles
- **Skills**: extensibility

---

## Versioning and Capability Negotiation

### Protocol Version

AWP version is a semantic version string: `MAJOR.MINOR`.

- MAJOR increments indicate breaking changes.
- MINOR increments indicate backward compatible additions.

### Handshake

Upon WebSocket connection, the client MUST send `awp.hello`.

**Request**

```json
{
  "id": "1",
  "type": "request",
  "method": "awp.hello",
  "params": {
    "client": {
      "name": "plasmate-python",
      "version": "0.1.0"
    },
    "awp": {
      "versions": ["0.1"]
    },
    "wants": {
      "encoding": ["msgpack", "json"],
      "compression": ["none", "zstd"],
      "features": [
        "som.snapshots",
        "som.mutations",
        "intents.level2",
        "skills.invoke",
        "sessions.persist"
      ]
    }
  }
}
```

**Response**

```json
{
  "id": "1",
  "type": "response",
  "result": {
    "awp": {"version": "0.1"},
    "encoding": {"selected": "msgpack"},
    "compression": {"selected": "none"},
    "features": {
      "som": {
        "snapshot": true,
        "mutation": true,
        "formats": ["som+json", "som+msgpack"],
        "max_bytes": 1048576
      },
      "skills": {
        "invoke": true,
        "registry": true
      },
      "network": {
        "proxy": true,
        "profiles": ["chrome128", "firefox128", "safari17"]
      },
      "limits": {
        "max_sessions": 500,
        "max_pages_per_session": 16
      }
    },
    "server": {
      "name": "plasmate-engine",
      "version": "0.1.0",
      "build": "dev"
    }
  }
}
```

### Capability Rules

- The server MUST reject requests that require unsupported features with `error.code = "UNSUPPORTED"`.
- Clients SHOULD degrade gracefully by falling back to primitive actions when intent levels are unsupported.

---

## Transport

### WebSocket

- Default transport: WebSocket.
- One WebSocket connection may multiplex multiple sessions.
- Clients MAY open multiple connections for isolation.

### Message Ordering

- Requests and responses are matched by `id`.
- Events are out-of-band and do not require `id`.
- The server MUST preserve per-page action ordering unless the client opts into concurrency.

### Heartbeats

- Either side MAY send ping frames.
- Server SHOULD emit `awp.pong` events if application-level liveness is needed.

---

## Encoding

### Default Encoding

- Default: MessagePack.
- Optional: JSON (for debugging).

### Compression

- Default: none.
- Optional: zstd per-message.

### Canonical JSON

When JSON is used, keys MUST be lower_snake_case.

---

## Message Model

### Envelope

Every message uses this envelope:

```json
{
  "id": "string-optional",
  "type": "request|response|event",
  "method": "string-optional",
  "params": {"...": "..."},
  "result": {"...": "..."},
  "error": {"code": "...", "message": "...", "details": {}}
}
```

Rules:

- `request` MUST include `id`, `method`, and `params`.
- `response` MUST include `id` and either `result` or `error`.
- `event` MUST include `method` and `params`. It MUST NOT include `id`.

### Idempotency

Requests MAY include `meta.idempotency_key`.

```json
{
  "meta": {
    "idempotency_key": "uuid",
    "trace_id": "uuid"
  }
}
```

If provided, the server SHOULD guarantee that repeating the request yields the same effect (when feasible).

### Tracing

All responses and events SHOULD include `meta.trace_id` and `meta.span_id`.

---

## Error Model

### Standard Error Codes

| Code | Meaning |
|---|---|
| `INVALID_REQUEST` | Malformed message or missing fields |
| `UNSUPPORTED` | Feature or method not supported |
| `NOT_FOUND` | Session/page/element not found |
| `TIMEOUT` | Operation timed out |
| `CONFLICT` | Concurrent state conflict |
| `RATE_LIMITED` | Server rate limit |
| `PERMISSION_DENIED` | Action not permitted |
| `NAVIGATION_FAILED` | Failed to load page |
| `SCRIPT_ERROR` | JS evaluation error |
| `SKILL_ERROR` | Skill execution failed |
| `INTERNAL` | Unexpected error |

### Error Object

```json
{
  "code": "TIMEOUT",
  "message": "wait_for condition not met",
  "details": {
    "timeout_ms": 10000,
    "method": "page.wait_for"
  }
}
```

---

## Authentication and Authorization

AWP supports several deployment modes:

1. **Local engine**: no auth required.
2. **LAN engine**: shared secret.
3. **Cloud engine**: API key + per-tenant permissions.

### Auth Methods

- `auth.none`
- `auth.bearer` (token)
- `auth.hmac` (signed requests)

### Permissions

Actions are permissioned because some operations are dangerous:

- `page.evaluate_js` is high risk.
- file uploads/downloads can exfiltrate data.
- proxy configuration can leak traffic.

Permissions are expressed as scopes:

- `som:read`
- `page:act`
- `page:evaluate`
- `session:write`
- `network:configure`
- `skills:invoke`

The server MAY enforce per-domain allowlists.

---

## Resources and Identifiers

AWP uses stable identifiers:

- `session_id`: string
- `page_id`: string
- `frame_id`: string
- `som_id`: string
- `element_id`: string

### Element References

AWP prefers stable element IDs from the SOM.

A target is specified as one of:

1. `element_ref`: direct reference by SOM ID.
2. `query`: semantic query.
3. `fallback`: list of fallbacks.

**Example**

```json
{
  "target": {
    "element_ref": "e:9f2c...",
    "fallback": [
      {"query": {"text": "Add to Cart", "role": "button"}},
      {"query": {"css": "button[data-testid='add-to-cart']"}}
    ]
  }
}
```

---

## Sessions

### session.create

Creates a browsing session.

```json
{
  "id": "10",
  "type": "request",
  "method": "session.create",
  "params": {
    "persist": false,
    "ttl_ms": 3600000,
    "profile": {
      "impersonation": "chrome128",
      "locale": "en-US",
      "timezone": "America/New_York",
      "viewport": {"width": 1280, "height": 720}
    },
    "network": {
      "proxy": {"mode": "none"}
    },
    "limits": {
      "max_pages": 8,
      "max_js_heap_mb": 64
    }
  }
}
```

Response includes `session_id`.

### session.close

Closes and optionally persists state.

### session.export / session.import

Optional methods for portable session snapshots.

---

## Page Lifecycle

### page.create

Creates a page (tab) inside a session.

### page.navigate

```json
{
  "id": "20",
  "type": "request",
  "method": "page.navigate",
  "params": {
    "session_id": "s:1",
    "page_id": "p:1",
    "url": "https://example.com",
    "wait": {"until": "network_idle", "timeout_ms": 15000}
  }
}
```

Wait strategies:

- `dom_content_loaded`
- `load`
- `network_idle`
- `som_stable` (SOM mutation quiet period)

### page.close

---

## Observation: SOM Snapshots and Mutations

### page.observe

Returns either:

- A full SOM snapshot, or
- A stream of SOM mutations since a cursor

**Snapshot request**

```json
{
  "id": "30",
  "type": "request",
  "method": "page.observe",
  "params": {
    "session_id": "s:1",
    "page_id": "p:1",
    "mode": "snapshot",
    "format": "som+json",
    "max_bytes": 262144
  }
}
```

**Mutation request**

```json
{
  "id": "31",
  "type": "request",
  "method": "page.observe",
  "params": {
    "session_id": "s:1",
    "page_id": "p:1",
    "mode": "mutations",
    "cursor": "c:12345",
    "max_events": 100
  }
}
```

### SOM Minimum Requirements

A SOM snapshot MUST include:

- `som_version`
- `url`
- `title`
- `elements` (or `regions`) that include:
  - stable `element_id`
  - `role` (button, link, input, select, dialog, tab, etc)
  - `text` or `label` where relevant
  - `actions` supported

### page.som_mutation Event

```json
{
  "type": "event",
  "method": "page.som_mutation",
  "params": {
    "session_id": "s:1",
    "page_id": "p:1",
    "cursor": "c:12346",
    "patch": [
      {"op": "replace", "path": "/regions/0/items/2/badge", "value": "4"}
    ]
  }
}
```

---

## Actions: Intent-Based Interaction

### page.act

This is the central operation.

```json
{
  "id": "40",
  "type": "request",
  "method": "page.act",
  "params": {
    "session_id": "s:1",
    "page_id": "p:1",
    "intent": {
      "action": "click",
      "target": {"element_ref": "e:9f2c"}
    },
    "wait": {"until": "som_stable", "timeout_ms": 10000}
  }
}
```

### Intent Object

`intent` has:

- `action`: string
- `target`: optional target
- `value`: optional value
- `options`: optional action options

Examples:

- Click: `{action:"click", target:{element_ref:"e:..."}}`
- Type: `{action:"type", target:{...}, value:"hello"}`
- Select: `{action:"select", target:{...}, value:"United States"}`
- Submit: `{action:"submit_form", target:{query:{role:"form", text:"Checkout"}}}`

### Intent Levels

AWP reserves namespaces:

- `primitive.*` (lowest level)
- `semantic.*` (SOM-aware)
- `intent.*` (higher-level intents)
- `skill.*` (provided by Wasm)

Examples:

- `primitive.click`
- `semantic.fill_form`
- `intent.checkout`
- `skill.stripe_pay`

Servers MAY accept un-namespaced action strings for convenience, but MUST normalize internally.

### Result Object

```json
{
  "status": "ok",
  "effects": {
    "navigation": true,
    "som_changed": true
  },
  "target_resolved": {
    "element_id": "e:9f2c",
    "confidence": 0.98
  },
  "artifacts": {
    "som_cursor": "c:12350"
  }
}
```

---

## Data Extraction

### page.extract

Extracts structured data based on selectors or semantic queries.

```json
{
  "id": "50",
  "type": "request",
  "method": "page.extract",
  "params": {
    "session_id": "s:1",
    "page_id": "p:1",
    "schema": {
      "type": "object",
      "properties": {
        "price": {"type": "number"},
        "title": {"type": "string"}
      },
      "required": ["price", "title"]
    },
    "queries": {
      "title": {"query": {"role": "heading", "level": 1}},
      "price": {"query": {"text_regex": "\\$[0-9]+\\.[0-9]{2}"}}
    }
  }
}
```

Response returns JSON data and provenance (element IDs used).

---

## Network Controls: Proxy, Profiles, Stealth

### session.set_network

Allows selecting proxy mode and impersonation profile.

```json
{
  "id": "60",
  "type": "request",
  "method": "session.set_network",
  "params": {
    "session_id": "s:1",
    "proxy": {"mode": "sticky_per_domain", "pool": "residential-us"},
    "impersonation": "chrome128"
  }
}
```

Security: server SHOULD require `network:configure` scope.

---

## State: Cookies, Storage, Downloads

### session.cookies

Methods:

- `session.cookies.get`
- `session.cookies.set`
- `session.cookies.clear`

### page.downloads

Methods:

- `page.downloads.list`
- `page.downloads.get` (stream file)

---

## Skills: WebAssembly Extensions

### skills.list

Returns installed skills and provided actions.

### skills.invoke

```json
{
  "id": "70",
  "type": "request",
  "method": "skills.invoke",
  "params": {
    "session_id": "s:1",
    "page_id": "p:1",
    "skill": "stripe-checkout",
    "action": "stripe_pay",
    "args": {
      "card_number": "4242424242424242",
      "expiry": "12/30",
      "cvc": "123"
    }
  }
}
```

Security: server MUST treat skill invocation as privileged and permissioned.

---

## Telemetry and Auditing

### page.telemetry Event

Servers SHOULD emit structured telemetry:

- latency (network, parse, som, js)
- retries
- proxy used
- cost estimation (optional)
- errors

Telemetry MUST NOT include secrets (passwords, card numbers).

---

## Determinism and Reproducibility

AWP encourages reproducibility:

- Stable element IDs in SOM.
- Mutation cursors.
- Action logs with trace IDs.
- Optional session export.

A minimal reproducible trace should allow replaying:

- initial URL
- observed SOM snapshot
- action sequence with resolved targets
- navigation and mutation events

---

## Examples

### Example 1: Search Workflow

1. `awp.hello`
2. `session.create`
3. `page.create`
4. `page.navigate` to search engine
5. `page.observe` snapshot
6. `page.act` type into search input
7. `page.act` submit
8. `page.observe` snapshot of results
9. `page.extract` titles and links

### Example 2: Checkout Workflow (Skill + Fallback)

1. Observe page
2. If `skills.list` shows `stripe-checkout`, call `skills.invoke`
3. Else fallback to `page.act` fill_form + submit

---

## Compatibility and Migration from CDP

Migration strategy:

- Provide a thin shim that maps CDP calls into AWP primitives.
- Encourage frameworks to adopt AWP for observation and intent actions.
- Long-term: AWP becomes native and CDP shim is optional.

---

## Implementation Notes for a Rust PoC

A minimal Rust PoC can implement:

- WebSocket server
- `awp.hello`
- `session.create` (in-memory)
- `page.navigate` (HTTP fetch)
- `page.observe` (SOM snapshot only)

And defer:

- JS execution
- stealth TLS spoofing
- proxy rotation
- Wasm skills

This PoC proves the thesis: SOM is token-efficient and deterministic.

---

*End of AWP Draft v0.1*
