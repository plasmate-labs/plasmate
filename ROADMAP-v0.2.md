# Plasmate v0.2 Roadmap - The Full Engine

## Vision

Plasmate v0.1 proved SOM works: 9.4x median compression across 38 real sites.
v0.2 makes it a drop-in replacement for Lightpanda and Chrome headless.

Three pillars: **Speed**, **Memory**, **Parallelism** - all powered by SOM-native architecture.

## 2026 Direction Update

The browser-agent market has consolidated around three buyer expectations:
managed browser fleets for scale, Playwright-compatible deterministic control,
and LLM-friendly structured snapshots that avoid screenshots when possible.
Plasmate should keep the local-first semantic engine as the core wedge and
make it easy for the surrounding ecosystem to consume SOM.

Roadmap additions from the current market review:

- **SDK/schema parity as product quality**: Rust, Python, Node, Go, and parser
  packages must accept the same SOM fields. Shadow DOM, iframes, `html_id`,
  and disclosure widgets are now part of the compatibility surface.
- **Playwright MCP compatibility track**: expose SOM as a structured snapshot
  alternative for IDE agents already adopting Playwright MCP-style workflows.
- **Stagehand/Browserbase bridge**: provide an observe/extract adapter that
  lets teams keep Stagehand-style scripts while replacing verbose page state
  with SOM where possible.
- **Browser Use retention path**: keep the Browser Use integration current and
  add examples for authenticated sessions, shadow DOM, iframes, and selectors.
- **Trust and safety**: document MCP execution boundaries, audit logging, and
  local-first privacy as differentiators against hosted browser APIs.

## Architecture

```
                    +-----------------------+
                    |     CDP Gateway       |  <-- Puppeteer/Playwright compatible
                    |  (Chrome DevTools     |
                    |   Protocol subset)    |
                    +----------+------------+
                               |
                    +----------+------------+
                    |     AWP Server        |  <-- Native protocol (SOM-aware)
                    |  (WebSocket, v0.1)    |
                    +----------+------------+
                               |
              +----------------+----------------+
              |                |                |
    +---------v------+ +------v--------+ +-----v--------+
    |  Page Runtime  | | SOM Compiler  | | SOM Cache    |
    |  (V8 via       | | (v0.1, proven)| | (new)        |
    |   rusty_v8)    | |               | |              |
    +--------+-------+ +------+--------+ +-----+--------+
             |                |                |
    +--------v-------+ +-----v--------+       |
    | DOM Mutations  | | Heuristics   |       |
    | Observer       | | Engine       |       |
    +--------+-------+ +--------------+       |
             |                                |
    +--------v---------------------------------v-----+
    |              Session Manager                    |
    |  (per-tab isolation, parallel execution,        |
    |   shared cache, resource budgets)               |
    +-------------------------------------------------+
```

## Module Plan

### 1. JavaScript Execution (`src/js/`)
- **Crate**: `rusty_v8` (V8 bindings for Rust)
- **Scope**: Create V8 isolate per session, execute `<script>` tags
- **DOM bridge**: Minimal - expose `document.querySelector`, `document.getElementById`,
  `element.textContent`, `element.getAttribute`, `element.click()`,
  `window.location`, `setTimeout/setInterval`
- **Not needed for v0.2**: Full Web API (Canvas, WebRTC, WebGL, Workers)
- **Key insight**: We only need enough JS to make pages render their content.
  90% of agent-relevant JS is "fetch data, insert into DOM." We skip layout/paint.

### 2. CDP Compatibility Layer (`src/cdp/`)
- **Goal**: Puppeteer `puppeteer.connect()` works out of the box
- **Domains to implement**:
  - `Browser` (getVersion, close)
  - `Target` (createTarget, attachToTarget, getTargets)
  - `Page` (navigate, enable, getFrameTree, lifecycleEvent)
  - `Runtime` (evaluate, callFunctionOn, getProperties)
  - `DOM` (getDocument, querySelector, getOuterHTML, setAttributeValue)
  - `Network` (enable, requestWillBeSent, responseReceived - for interception)
  - `Input` (dispatchMouseEvent, dispatchKeyEvent)
  - `LP` (getMarkdown, getSemanticTree, getInteractiveElements) - Lightpanda compat!
- **Bonus**: Also expose `Plasmate` CDP domain with native SOM access

### 3. SOM Cache (`src/cache/`)
- **The paradigm shift**: Why re-parse a page you already understand?
- **Architecture**:
  ```
  URL + content_hash -> cached SOM snapshot
  ```
- **Three tiers**:
  1. **Hot cache** (in-memory): Last N pages, instant retrieval, ~0ms
  2. **Warm cache** (RocksDB on disk): Thousands of pages, <1ms retrieval
  3. **Cold cache** (shared/networked): Cross-session, cross-agent SOM sharing
- **Differential SOM**: When revisiting a URL:
  1. Fetch HTML, compute content hash
  2. If hash matches cache: return cached SOM instantly (zero parse time)
  3. If hash differs: compile new SOM, diff against cached version,
     return only changed elements + full SOM
- **Cache-aware navigation**: Agent says "go to HN" - if cached SOM is <60s old,
  return it WITHOUT fetching. Agent gets instant page understanding.
- **Prewarming**: Background thread fetches + caches URLs the agent is likely
  to visit (based on links in current SOM)

### 4. Parallel Session Manager (`src/sessions/`)
- **Rust advantage**: tokio green threads + zero-cost async
- **Per-session isolation**: Each session gets its own V8 isolate + cookie jar
- **Shared resources**: SOM cache, DNS cache, connection pool
- **Budget enforcement**: Max memory per session, max JS execution time
- **Benchmarks target**:
  - 500+ concurrent sessions per 8GB RAM (vs Lightpanda's 140, Chrome's 15)
  - <50ms cold start per session (vs Lightpanda's 100ms, Chrome's 3-5s)
  - <10ms warm start (cached SOM, no fetch needed)

### 5. Network Layer Upgrades (`src/network/`)
- **Connection pooling**: Reuse TCP/TLS connections across sessions
- **HTTP/2 multiplexing**: Multiple requests per connection
- **DNS caching**: Shared across sessions
- **Request interception**: Block ads, tracking, unnecessary resources
- **Resource budgets**: Skip images, fonts, media by default (agent doesn't need them)

## Performance Targets (vs Lightpanda)

| Metric | Chrome | Lightpanda | Plasmate Target |
|--------|--------|------------|-----------------|
| 100-page benchmark | 25.2s | 2.3s | <1.0s |
| Memory per instance | 207MB | 24MB | <8MB |
| Concurrent (8GB) | 15 | 140 | 500+ |
| Cold start | 3-5s | <100ms | <50ms |
| Token output | Raw DOM | Markdown/Tree (bolt-on) | SOM (native) |
| Warm page load | N/A | N/A | <10ms (cached SOM) |

## Why SOM Cache Changes Everything

Lightpanda and Chrome re-render every page from scratch every time.
Plasmate with SOM Cache creates a fundamentally different paradigm:

1. **First visit**: Fetch -> Parse -> JS -> Compile SOM -> Cache (normal speed)
2. **Revisit (same content)**: Cache hit -> Return SOM (~0ms)
3. **Revisit (changed content)**: Fetch -> Diff -> Update cache -> Return delta
4. **Predicted navigation**: Cache prewarms links from current page

For an AI agent that navigates 50 pages in a workflow, pages 2-50 are often
revisits or predictable next-pages. SOM Cache makes those effectively free.

## Build Order

1. **V8 integration** (unblocks JS execution, biggest gap vs Lightpanda)
2. **CDP compatibility** (unblocks Puppeteer/Playwright, enables benchmarking vs LP)
3. **SOM Cache** (the differentiator, makes us categorically faster)
4. **Parallel session manager** (proves the concurrency story)
5. **Network upgrades** (polish, production readiness)

## Dependencies to Add

```toml
# V8 JavaScript engine
rusty_v8 = "0.106"

# On-disk cache
rocksdb = { version = "0.22", default-features = false }

# Faster hashing for cache keys
xxhash-rust = { version = "0.8", features = ["xxh3"] }

# Memory tracking
jemalloc-ctl = "0.5"
jemallocator = "0.5"

# HTTP/2
hyper = { version = "1", features = ["http2", "server"] }
```
