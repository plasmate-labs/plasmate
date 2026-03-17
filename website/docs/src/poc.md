---
tags: [project, agentic-browser, poc, rust]
created: 2026-03-15
status: draft
---

# Rust Proof of Concept - Build Brief (Plasmate v0)

## Goal

Build a minimal Rust proof of concept that proves the core thesis:

1. **SOM** can represent real pages deterministically and with 10x to 30x fewer tokens than raw HTML or DOM dumps.
2. **AWP** can provide a stable, agent-friendly interface to observe and act.

This PoC is not trying to be a full browser. It is trying to be a compelling benchmark and a believable foundation.

## Scope (In)

### A. Core Engine

- CLI: `plasmate fetch <url> --som out.json`
- CLI: `plasmate serve --host 127.0.0.1 --port 9222` (AWP server)

### B. Networking (basic)

- HTTP GET navigation with redirects
- Cookie jar (in-memory)
- Basic header impersonation (User-Agent string only)

### C. HTML to SOM

- Parse HTML with `html5ever`
- Strip scripts and styles
- Extract:
  - page title
  - main regions (nav, main, aside, footer) using heuristics
  - interactive elements (links, buttons, inputs, selects, textareas)
  - labels (from `label[for]`, aria-label, placeholder)
  - text content (merged and trimmed)
- Assign stable `element_id` values

### D. AWP v0.1 subset

Implement:

- `awp.hello`
- `session.create`
- `page.create`
- `page.navigate`
- `page.observe` (snapshot only)
- `page.act` for primitives:
  - `primitive.click`
  - `primitive.type`

For `page.act`, it is acceptable to simulate behavior by:

- returning a structured result that resolves the target
- not executing real clicks unless a minimal DOM event system exists

The PoC can still be compelling even if action execution is stubbed, as long as:

- target resolution works reliably
- observation is excellent

### E. Token benchmark harness

- Build a dataset runner: input list of 50 to 200 URLs
- For each URL, output:
  - HTML byte size
  - SOM byte size
  - token estimate for HTML and SOM

Token estimation method:

- Prefer a deterministic tokenizer library.
- If exact tokenizer is not available in Rust, use:
  - approximate token count = `chars / 4` as a first pass
  - plus a Python script that computes real tokens using tiktoken for final reports

## Scope (Out)

Not included in the PoC:

- JavaScript execution (V8)
- CSS layout / rendering
- stealth TLS spoofing
- HTTP/2 fingerprint tuning
- proxy rotation
- Wasm skills runtime
- multi-page concurrency

## Acceptance Criteria

The PoC is "done" when:

1. For at least 30 of 50 target URLs, the engine produces a SOM snapshot successfully.
2. SOM output includes at least 50% of interactive elements visible to a human (links, buttons, inputs).
3. SOM output is at least 8x smaller than raw HTML by token count on average.
4. The AWP server can handle:
   - 50 sequential sessions
   - 10 concurrent sessions
   without crashes.

## Recommended Crates

- `tokio`
- `reqwest`
- `rustls`
- `html5ever`
- `serde`
- `serde_json`
- `rmp-serde` (MessagePack)
- `tokio-tungstenite`
- `tracing` and `tracing-subscriber`
- `url`

## Element ID Strategy (Deterministic)

Define element_id as a hash of:

- normalized URL origin
- element role
- accessible name (aria-label, label text, innerText)
- DOM path index chain (indices of child nodes among element siblings)

Example:

`element_id = sha256("origin|role|name|path")` then base32 encode.

This yields stable IDs unless the page structure changes, which is acceptable for a PoC.

## Test Targets (suggested)

Pick a stable set:

- wikipedia.org pages
- github.com repo pages
- news.ycombinator.com
- stripe checkout demo (if accessible)
- a few Shopify demo stores
- a few simple marketing sites

Avoid:

- heavily dynamic apps that require JS to render
- pages behind auth

## Deliverables

1. `plasmate` Rust binary
2. `AWP` WebSocket server
3. Example Python client that connects and calls observe
4. Benchmark report (markdown) with token comparisons
5. Dockerfile

## Next Step After PoC

Phase 1 Alpha:

- integrate `rusty_v8`
- implement event loop and DOM mutation tracking
- implement `page.act` real interactions
- add persistence (RocksDB)
- start first Wasm skill SDK
