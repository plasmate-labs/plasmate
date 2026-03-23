# Agentic Browser Engine - Product Specification v1.0

**Working Name:** Plasmate
**Tagline:** "The browser built for machines."
**Author:** David Hurley / DBH Ventures
**Date:** March 15, 2026
**Status:** Draft - Pre-Seed

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [The Problem](#the-problem)
3. [The Vision](#the-vision)
4. [Architecture Overview](#architecture-overview)
5. [Semantic Object Model (SOM)](#semantic-object-model)
6. [Agent Web Protocol (AWP)](#agent-web-protocol)
7. [Stealth Networking Layer](#stealth-networking-layer)
8. [Sandboxed JS Runtime](#sandboxed-js-runtime)
9. [Wasm Skill System](#wasm-skill-system)
10. [Session & State Management](#session-and-state-management)
11. [Fleet Orchestration (Commercial)](#fleet-orchestration)

---

## 1. Executive Summary

Every AI agent that interacts with the web today does so through a hack. They control Chrome (or a Chrome derivative) through the Chrome DevTools Protocol (CDP) - a debugging interface designed for human developers inspecting CSS and setting breakpoints. This is the equivalent of driving a car by reaching through the window and turning the steering wheel with a broomstick.

The result: agents that are slow, brittle, expensive (in tokens), easily detected, and fundamentally limited by an architecture that was never designed for them.

Plasmate is a headless browser engine built from scratch in Rust, purpose-designed for AI agents. It introduces three foundational technologies:

1. **The Semantic Object Model (SOM)** - A new way to represent web pages that drops visual rendering entirely and outputs a clean, deterministic, token-efficient structure that LLMs can directly reason about.

2. **The Agent Web Protocol (AWP)** - A new communication standard between agents and browsers that replaces CDP's coordinate-based commands with intent-based actions.

3. **A WebAssembly Skill System** - A community-extensible plugin architecture where developers write site-specific "Skills" (in any language) that teach the browser how to navigate complex web applications.

The business model follows the Netscape playbook: open-source the engine and the protocol aggressively (Apache 2.0), establish AWP as the industry standard, and monetize the commercial fleet - orchestration, persistent sessions, anti-detection proxy network, and analytics.

**Target market size:** The browser automation market is projected at $15B+ by 2028. The AI agent infrastructure market (broader) is estimated at $50B+ by 2030. Every company building AI agents needs a browser. Today they all rent Chrome. Tomorrow they should run Plasmate.

---

## 2. The Problem

### 2.1 The CDP Bottleneck

The Chrome DevTools Protocol was created in 2011 for one purpose: letting developers debug websites. Fifteen years later, it has become the de facto interface for AI agents interacting with the web - a role it was never designed for.

**Problems with CDP for agents:**

| Issue | Description | Impact |
|---|---|---|
| Pixel-level commands | CDP operates on screen coordinates (`click(x:342, y:891)`) | Agents must understand visual layout to interact with pages |
| DOM verbosity | Returns full DOM trees with styling, attributes, event listeners | A simple page can produce 500KB+ of DOM data, consuming thousands of LLM tokens |
| Visual rendering overhead | Chrome renders CSS, computes layout, paints pixels, composites layers | 80-90% of compute is wasted on visual output no agent needs |
| Memory consumption | Each Chrome tab uses 50-300MB RAM | Fleet of 1,000 agents = 50-300GB RAM |
| Session fragility | CDP WebSocket connections drop, state is lost, recovery is manual | Agents fail silently mid-task |
| Detection surface | Headless Chrome has dozens of detectable fingerprints | Bot detection services block agents within seconds |
| No native concurrency | Chrome was designed for one human user | Running 100+ sessions requires complex orchestration |
| No semantic understanding | DOM is a rendering tree, not a meaning tree | Agents must infer purpose from HTML tag names and CSS classes |

### 2.2 The Stealth Arms Race

Every headless browser automation tool fights the same battle: appearing human to anti-bot systems. The current approach is bolting stealth patches onto Chrome after the fact:

- `puppeteer-extra-plugin-stealth` patches ~15 known detection vectors
- Browserbase and Steel Browser maintain proprietary stealth layers
- Lightpanda avoids detection by not being Chrome (different TLS fingerprint)

But all of these are reactive. Cloudflare, DataDome, PerimeterX, and Akamai continuously add new detection vectors. The patch-and-pray approach is fundamentally losing.

A native engine can solve this differently: by controlling the entire network stack from TLS handshake to HTTP headers, stealth becomes a first-class architectural feature rather than an afterthought.

### 2.3 The Token Tax

LLMs process web pages as text. The more tokens a page representation consumes, the more expensive and slow the agent becomes. Current approaches:

| Method | Tokens for a typical e-commerce page | Notes |
|---|---|---|
| Raw HTML | 15,000-50,000 | Includes scripts, styles, metadata |
| Cleaned HTML | 5,000-15,000 | Strip scripts/styles, still verbose |
| Accessibility tree | 2,000-8,000 | Better, but inconsistent across sites |
| Screenshot + vision | 1,000-3,000 (image tokens) | Expensive, can't interact with elements |
| **SOM (proposed)** | **500-2,000** | Semantic elements with affordances only |

A 10x reduction in tokens per page interaction means:
- 10x cheaper per agent action
- 10x more context available for reasoning
- 10x faster response times
- 10x more pages processable within context windows

### 2.4 What Exists Today

| Product | Approach | Engine | Protocol | Stealth | Open Source |
|---|---|---|---|---|---|
| Puppeteer/Playwright | Chrome automation | Chromium | CDP | Plugin-based | Yes |
| Browserbase/Stagehand | Cloud browsers | Chromium | CDP + SDK | Managed | SDK open, infra closed |
| Lightpanda | New engine (Zig) | Custom (Zig) | CDP-compatible | Inherent (not Chrome) | Yes (AGPL) |
| Steel Browser | Stealth Chrome | Chromium | CDP | Deep patches | Partial |
| Browser Use | Agent framework | Chromium | CDP + LLM | Basic | Yes |
| Manus Browser Operator | Extension overlay | Chrome | CDP + extension | None | No |
| ChatGPT Atlas | Consumer product | Chromium-based | Proprietary | Unknown | No |

**Gap:** Nobody has built a native agent protocol. Nobody outputs semantic models instead of DOM. Nobody has a Wasm skill ecosystem. Nobody has stealth at the network stack level.

---

## 3. The Vision

### 3.1 The Netscape Parallel

In 1994, the web existed but was unusable for most people. Netscape didn't just build a better viewer - they invented the infrastructure that made the web work:

| Netscape Innovation | Web Impact | Plasmate Equivalent | Agent Web Impact |
|---|---|---|---|
| SSL/TLS | Secure commerce | AWP | Standardized agent-browser communication |
| JavaScript | Dynamic pages | Wasm Skills | Extensible browser capabilities |
| Cookies | Session persistence | Persistent Agent State | Agents maintain context across sessions |
| Navigator browser | Mass adoption | Plasmate engine | Standard engine for all agent frameworks |

### 3.2 The End State

In 3 years, the agentic browser landscape should look like this:

- **AWP** is the standard protocol for agent-to-browser communication, like HTTP is for human-to-server
- **Plasmate** is the default engine embedded in LangChain, CrewAI, AutoGen, and every major agent framework
- **SOM** is the standard page representation that LLMs consume, replacing screenshots and DOM dumps
- **Wasm Skills** are a thriving marketplace, with thousands of community-contributed site navigators
- **The commercial fleet** (Plasmate Cloud) is a $100M+ ARR business selling orchestration to enterprises

### 3.3 What Plasmate Is NOT

- **Not a consumer browser** - no UI, no tabs, no bookmarks. Agents don't need chrome (lowercase c).
- **Not a scraping tool** - scraping is a use case, not the product. Plasmate is infrastructure.
- **Not a Chrome fork** - zero Chromium code. Clean-room Rust implementation.
- **Not an agent framework** - Plasmate doesn't decide what to do. It executes what agents tell it to do via AWP.
- **Not a proxy service** - though it has proxy capabilities built in. The proxy is a feature, not the product.

---
## 4. Architecture Overview

### 4.1 System Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    AGENT (LLM)                           │
│  LangChain / CrewAI / AutoGen / Custom                  │
└────────────────────┬────────────────────────────────────┘
                     │ AWP (Agent Web Protocol)
                     │ WebSocket + MessagePack
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  PLASMATE ENGINE (Rust)                       │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │   AWP    │  │  Session  │  │  Skill   │              │
│  │  Server  │  │  Manager  │  │  Runtime │              │
│  │          │  │           │  │  (Wasm)  │              │
│  └────┬─────┘  └─────┬─────┘  └────┬─────┘              │
│       │              │              │                    │
│  ┌────▼──────────────▼──────────────▼─────┐              │
│  │            Core Pipeline                │              │
│  │                                         │              │
│  │  ┌─────────┐  ┌─────────┐  ┌────────┐ │              │
│  │  │ Network │  │  HTML   │  │  SOM   │ │              │
│  │  │  Layer  │→│  Parser │→│ Compiler│ │              │
│  │  │(rustls) │  │(html5ever│  │        │ │              │
│  │  └─────────┘  └─────────┘  └────────┘ │              │
│  │                                         │              │
│  │  ┌─────────┐  ┌─────────┐              │              │
│  │  │   JS    │  │  State  │              │              │
│  │  │ Runtime │  │  Store  │              │              │
│  │  │(rusty_v8│  │(RocksDB)│              │              │
│  │  └─────────┘  └─────────┘              │              │
│  └─────────────────────────────────────────┘              │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Data Flow

1. **Agent sends AWP intent** → `{action: "fill_form", target: "#email", value: "user@example.com"}`
2. **AWP Server** validates intent, resolves target via SOM
3. **Session Manager** loads page state, cookies, auth context
4. **Skill Runtime** checks if a Wasm skill is registered for the current domain
5. **If skill exists:** Skill handles the interaction directly (fast path)
6. **If no skill:** Core pipeline processes:
   a. **Network Layer** fetches the page (with stealth TLS, proxy rotation)
   b. **HTML Parser** (html5ever) produces a raw DOM
   c. **SOM Compiler** transforms DOM into Semantic Object Model
   d. **JS Runtime** (V8) executes scripts that modify the SOM (SPAs, dynamic content)
7. **SOM returned to agent** via AWP with action results

### 4.3 Language & Dependencies (Rust Crates)

| Component | Crate | Purpose |
|---|---|---|
| HTTP client | `reqwest` + `hyper` | HTTP/1.1 and HTTP/2 with full header control |
| TLS | `rustls` (custom fork) | TLS 1.3 with JA3/JA4 fingerprint control |
| HTML parsing | `html5ever` + `markup5ever` | Spec-compliant HTML5 parser (same as Servo) |
| CSS parsing | `cssparser` (minimal) | Only parse selectors for SOM mapping, no layout |
| JavaScript | `rusty_v8` | V8 engine with Rust bindings, sandboxed per session |
| WebAssembly | `wasmtime` | Wasm runtime for skills |
| Async runtime | `tokio` | Async I/O, task scheduling |
| Serialization | `serde` + `rmp-serde` | MessagePack serialization for AWP |
| WebSocket | `tokio-tungstenite` | AWP transport |
| State storage | `rocksdb` (via `rust-rocksdb`) | Persistent session/cookie store |
| DNS | `trust-dns-resolver` | Custom DNS resolution (DoH support) |
| Proxy | `tokio-socks` + custom | SOCKS5/HTTP proxy with rotation |
| Compression | `flate2` + `brotli` | HTTP content decoding |
| URL parsing | `url` | Spec-compliant URL handling |
| Regex | `regex` | Pattern matching for SOM rules |
| Logging | `tracing` + `tracing-subscriber` | Structured logging and telemetry |
| CLI | `clap` | Command-line interface |
| Config | `toml` + `serde` | Configuration files |

---

## 5. Semantic Object Model (SOM)

### 5.1 What Is SOM

The Semantic Object Model is a new page representation format purpose-built for LLM consumption. It replaces the DOM (Document Object Model) by stripping away everything an agent doesn't need (visual styling, layout information, rendering hints) and adding everything an agent does need (element purpose, affordances, interaction methods, semantic relationships).

### 5.2 Design Principles

1. **Token-minimal** - Represent a page in the fewest possible tokens
2. **Deterministic** - Same page always produces the same SOM (unlike screenshots which vary by viewport)
3. **Actionable** - Every element includes its available interactions
4. **Hierarchical** - Preserve meaningful page structure (navigation, main content, sidebar, footer)
5. **Semantic** - Elements described by purpose, not by HTML tag name
6. **Stable references** - Element IDs persist across page mutations (SPA navigation)

### 5.3 SOM Format

```json
{
  "url": "https://shop.example.com/products/widget-pro",
  "title": "Widget Pro - Example Shop",
  "timestamp": "2026-03-15T23:00:00Z",
  "som_version": "1.0",
  "regions": [
    {
      "role": "navigation",
      "id": "nav-main",
      "items": [
        {"type": "link", "id": "n1", "text": "Home", "href": "/", "actions": ["click"]},
        {"type": "link", "id": "n2", "text": "Products", "href": "/products", "actions": ["click"]},
        {"type": "link", "id": "n3", "text": "Cart (3)", "href": "/cart", "actions": ["click"], "badge": "3"},
        {"type": "search", "id": "n4", "placeholder": "Search products...", "actions": ["type", "submit"]}
      ]
    },
    {
      "role": "main",
      "id": "content",
      "sections": [
        {
          "type": "product",
          "id": "p1",
          "name": "Widget Pro",
          "price": {"amount": 49.99, "currency": "USD"},
          "rating": {"score": 4.7, "count": 342},
          "description": "Professional-grade widget with titanium core and lifetime warranty.",
          "images": ["https://shop.example.com/img/widget-pro-1.jpg"],
          "variants": [
            {"id": "v1", "label": "Color", "options": ["Black", "Silver", "Blue"], "selected": "Black"},
            {"id": "v2", "label": "Size", "options": ["S", "M", "L", "XL"], "selected": null}
          ],
          "actions": [
            {"id": "a1", "type": "button", "text": "Add to Cart", "primary": true, "requires": ["v2"]},
            {"id": "a2", "type": "button", "text": "Buy Now", "primary": false},
            {"id": "a3", "type": "button", "text": "Add to Wishlist", "primary": false}
          ]
        }
      ]
    },
    {
      "role": "complementary",
      "id": "reviews",
      "type": "review_list",
      "count": 342,
      "visible": 5,
      "items": [
        {"author": "Jane D.", "rating": 5, "text": "Best widget I've ever used. The titanium core makes all the difference.", "date": "2026-03-10"},
        {"author": "Mike R.", "rating": 4, "text": "Great quality, shipping was slow though.", "date": "2026-03-08"}
      ],
      "actions": [
        {"id": "r1", "type": "pagination", "text": "Load more reviews", "actions": ["click"]}
      ]
    }
  ],
  "forms": [],
  "dialogs": [],
  "alerts": []
}
```

### 5.4 Token Comparison

For the above product page:

| Representation | Approximate Tokens | Ratio |
|---|---|---|
| Raw HTML | ~18,000 | 1x (baseline) |
| Cleaned HTML | ~6,500 | 2.8x better |
| Playwright accessibility tree | ~3,200 | 5.6x better |
| **SOM** | **~900** | **20x better** |

### 5.5 SOM Compilation Pipeline

```
Raw HTML (html5ever)
    │
    ▼
DOM Tree
    │
    ├─ Strip: <script>, <style>, <svg>, <noscript>, <meta>, comments
    ├─ Strip: CSS classes, inline styles, data-* attributes (unless semantic)
    ├─ Strip: ARIA attributes (consume for semantics, don't output)
    │
    ▼
Cleaned DOM
    │
    ├─ Identify regions: <nav>, <main>, <aside>, <footer>, <header>
    ├─ Identify semantic blocks: <form>, <table>, <article>, <section>
    ├─ Identify interactive elements: <a>, <button>, <input>, <select>, <textarea>
    ├─ Identify content elements: <h1-6>, <p>, <img>, <video>
    ├─ Consume ARIA roles/labels for element purpose
    │
    ▼
Semantic Tree
    │
    ├─ Merge adjacent text nodes
    ├─ Collapse wrapper divs (divs with single child, no semantic meaning)
    ├─ Extract structured data (JSON-LD, microdata, Open Graph)
    ├─ Resolve relative URLs
    ├─ Assign stable IDs to interactive elements
    ├─ Determine affordances (what actions each element supports)
    │
    ▼
SOM Output (JSON / MessagePack)
```

### 5.6 SOM Element Types

| Type | HTML Sources | Properties | Actions |
|---|---|---|---|
| `link` | `<a>`, `[role=link]` | text, href, target | click |
| `button` | `<button>`, `<input[type=submit]>`, `[role=button]` | text, disabled, primary | click |
| `text_input` | `<input[type=text\|email\|password\|...]>` | placeholder, value, required, pattern | type, clear, submit |
| `textarea` | `<textarea>` | placeholder, value, maxlength | type, clear |
| `select` | `<select>`, `[role=listbox]` | options[], selected, multiple | select |
| `checkbox` | `<input[type=checkbox]>`, `[role=checkbox]` | label, checked | toggle |
| `radio` | `<input[type=radio]>`, `[role=radio]` | label, checked, group | select |
| `search` | `<input[type=search]>`, `[role=search]` | placeholder, value | type, submit |
| `image` | `<img>` | alt, src, dimensions | - |
| `video` | `<video>`, `<iframe[youtube\|vimeo]>` | title, duration, src | play, pause, seek |
| `table` | `<table>` | headers[], rows[][] | sort, paginate |
| `list` | `<ul>`, `<ol>` | items[] | - |
| `heading` | `<h1>`-`<h6>` | text, level | - |
| `paragraph` | `<p>` | text | - |
| `form` | `<form>` | fields[], action, method | submit |
| `dialog` | `<dialog>`, `[role=dialog]` | title, content | accept, dismiss |
| `tab` | `[role=tab]` | label, selected, panel_id | click |
| `menu` | `<nav>`, `[role=menu]` | items[] | - |
| `alert` | `[role=alert]`, `.toast`, `.notification` | text, type(info\|warning\|error) | dismiss |
| `pagination` | `.pagination`, `[role=navigation][aria-label*=page]` | current, total, links[] | click |
| `product` | `[itemtype=Product]`, heuristic | name, price, rating, variants | add_to_cart |
| `article` | `<article>` | title, author, date, content | - |

### 5.7 SOM Heuristics

For pages without semantic HTML or ARIA, SOM uses heuristics:

1. **Form detection** - Cluster of input elements within a shared ancestor = form
2. **Navigation detection** - Lists of links in header/top of page = navigation
3. **Product detection** - Co-occurrence of price pattern + image + heading = product
4. **Article detection** - Large text block with heading + date/author = article
5. **Modal detection** - Fixed/absolute positioned overlay with close button = dialog
6. **Pagination detection** - Sequential number links or "next/prev" buttons = pagination
7. **Search detection** - Text input with magnifying glass icon or "search" in placeholder = search

### 5.8 SOM Mutation Tracking

When JS modifies the page (SPA navigation, dynamic content loading), SOM tracks changes:

```json
{
  "type": "som_mutation",
  "timestamp": "2026-03-15T23:00:01Z",
  "changes": [
    {"op": "add", "path": "/regions/1/sections/0/reviews/5", "value": {"author": "New Review..."}},
    {"op": "remove", "path": "/dialogs/0"},
    {"op": "replace", "path": "/regions/0/items/2/badge", "old": "3", "new": "4"}
  ]
}
```

This uses JSON Patch (RFC 6902) semantics, so agents can track page evolution without re-reading the entire SOM.

---

## 6. Agent Web Protocol (AWP)

*Full specification in separate document: AWP-SPEC.md*

### 6.1 Summary

AWP replaces CDP for agent-browser communication. Key differences:

| Feature | CDP | AWP |
|---|---|---|
| Addressing | CSS selectors + coordinates | SOM element IDs + semantic targets |
| Commands | Low-level (click, type, evaluate) | Intent-based (add_to_cart, login, search) |
| Responses | Raw DOM + screenshots | SOM + structured results |
| Transport | WebSocket (JSON) | WebSocket (MessagePack) - 30-50% smaller |
| Sessions | Ephemeral (lost on disconnect) | Persistent (survive reconnection) |
| Concurrency | Single-page focus | Multi-page, multi-tab native |
| Observability | Debug logging | Structured telemetry with cost tracking |
| Extensibility | Limited to Chrome internals | Wasm skills for any site-specific logic |

### 6.2 Intent Hierarchy

AWP commands form a hierarchy from low-level to high-level:

**Level 0 - Primitive Actions** (always available)
```
navigate, click, type, select, scroll, wait, screenshot, evaluate_js
```

**Level 1 - Semantic Actions** (SOM-aware)
```
fill_form, submit_form, select_option, toggle, dismiss_dialog, 
paginate, sort_table, expand_section, close_tab
```

**Level 2 - Intent Actions** (requires SOM understanding)
```
search, login, logout, add_to_cart, checkout, 
read_content, extract_data, follow_link_by_text
```

**Level 3 - Skill Actions** (loaded via Wasm)
```
stripe_checkout, salesforce_navigate, google_sheets_edit,
linkedin_send_message, github_create_issue
```

Agents can operate at any level. Most will use Level 1-2 for general browsing and Level 3 for domain-specific tasks.

---

## 7. Stealth Networking Layer

### 7.1 Architecture

The networking layer is the lowest level of the engine, handling all HTTP communication with full fingerprint control.

```
┌─────────────────────────────────────────────┐
│             Stealth Network Layer             │
│                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │   TLS    │  │  HTTP/2  │  │  Proxy   │  │
│  │ Spoofer  │  │ Tuner    │  │ Rotator  │  │
│  │          │  │          │  │          │  │
│  │ JA3/JA4  │  │ SETTINGS │  │ SOCKS5   │  │
│  │ ALPN     │  │ WINDOW   │  │ HTTP     │  │
│  │ Ciphers  │  │ PRIORITY │  │ Rotating │  │
│  │ Extensions│ │ HEADERS  │  │ Sticky   │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Header  │  │  Cookie  │  │   DNS    │  │
│  │ Manager  │  │  Jar     │  │ Resolver │  │
│  │          │  │          │  │          │  │
│  │ Ordering │  │ Per-site │  │ DoH      │  │
│  │ Casing   │  │ Profiles │  │ DoT      │  │
│  │ Realistic│  │ Rotation │  │ Custom   │  │
│  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────────────────────────────────┘
```

### 7.2 TLS Fingerprint Control

Bot detection services fingerprint TLS connections via:
- **JA3** - hash of TLS Client Hello parameters (cipher suites, extensions, curves)
- **JA4** - next-gen fingerprint including ALPN, signature algorithms, SNI
- **HTTP/2 fingerprint** - SETTINGS frame values, WINDOW_UPDATE, PRIORITY frames

Plasmate's custom `rustls` fork allows setting every parameter:

```rust
let tls_config = StealthTlsConfig::new()
    .ja3_profile(BrowserProfile::Chrome128)  // Mimic Chrome 128
    .cipher_suites(CHROME_128_CIPHERS)
    .extensions(CHROME_128_EXTENSIONS)
    .alpn(&["h2", "http/1.1"])
    .curves(&[X25519, SECP256R1, SECP384R1])
    .signature_algorithms(CHROME_128_SIGS);
```

**Built-in profiles:**
- Chrome 120-130 (Windows, Mac, Linux variants)
- Firefox 120-130
- Safari 17-18
- Edge 120-130
- Custom (full manual control)

### 7.3 HTTP/2 Fingerprint Control

Beyond TLS, HTTP/2 connections have their own fingerprint:

```rust
let http2_config = Http2Config::new()
    .initial_window_size(6291456)         // Chrome default
    .max_concurrent_streams(1000)          // Chrome default
    .header_table_size(65536)              // Chrome default
    .max_header_list_size(262144)          // Chrome default
    .priority_frames(CHROME_PRIORITY)      // Frame ordering
    .pseudo_header_order(&[":method", ":authority", ":scheme", ":path"]);
```

### 7.4 Header Management

Headers must match the impersonated browser exactly:

- **Ordering** - Chrome sends headers in a specific order (`:method` before `:path`)
- **Casing** - Some headers use different capitalization per browser
- **Content** - `Accept`, `Accept-Language`, `Accept-Encoding` must match browser version
- **User-Agent** - Automatically matches the selected TLS profile

### 7.5 Proxy Architecture

Built-in proxy support with intelligent rotation:

```rust
let proxy_pool = ProxyPool::new()
    .add_residential("provider_a", 1000)    // 1000 residential IPs
    .add_datacenter("provider_b", 500)      // 500 datacenter IPs
    .strategy(RotationStrategy::StickyPerDomain)  // Same IP per domain
    .failover(FailoverPolicy::NextProxy)          // Auto-rotate on failure
    .health_check_interval(Duration::from_secs(30));
```

**Rotation strategies:**
- `RoundRobin` - cycle through proxies sequentially
- `Random` - random proxy selection
- `StickyPerDomain` - same proxy for same domain (avoids session issues)
- `StickyPerSession` - same proxy for entire agent session
- `GeoTargeted` - select proxy by geographic region
- `LeastLatency` - pick fastest available proxy

---

## 8. Sandboxed JS Runtime

### 8.1 Why Execute JS at All

Modern web applications (SPAs) render content client-side. Without JS execution:
- React/Vue/Angular/Svelte apps show blank pages
- Dynamic content (lazy loading, infinite scroll) never loads
- Login flows that depend on JS fail
- CSRF tokens generated by JS are unavailable

A headless browser for agents MUST execute JavaScript. But it doesn't need to execute ALL JavaScript.

### 8.2 Selective Execution

Plasmate's JS runtime is selective. It executes:
- ✅ Scripts that modify DOM (content rendering)
- ✅ XHR/fetch calls (data loading)
- ✅ Event handlers triggered by agent actions
- ✅ Form validation scripts
- ✅ Authentication flows (OAuth, CSRF)

It skips:
- ❌ Analytics scripts (Google Analytics, Mixpanel, Segment)
- ❌ Ad scripts (Google Ads, Facebook Pixel)
- ❌ Tracking pixels
- ❌ Animation/transition scripts
- ❌ Service worker registration
- ❌ WebGL/Canvas rendering
- ❌ Web Workers for non-essential tasks

### 8.3 Memory Isolation

Each agent session gets an isolated V8 context:

```rust
let isolate = v8::Isolate::new(v8::CreateParams::default()
    .heap_size_limit(64 * 1024 * 1024)   // 64MB max per session
    .external_memory_limit(32 * 1024 * 1024));

let context = v8::Context::new(&mut isolate);
// Session-scoped: destroyed when session ends
// No cross-session memory leaks
```

**Memory budget per session: 64MB** (vs Chrome's 150-300MB per tab)

At 64MB per session:
- 1 server with 32GB RAM = **500 concurrent agent sessions**
- 1 server with 128GB RAM = **2,000 concurrent agent sessions**

### 8.4 Script Classification

Plasmate classifies scripts before execution:

| Category | Strategy | Examples |
|---|---|---|
| Essential | Execute immediately | Framework runtime (React, Vue) |
| Data-loading | Execute, capture responses | API calls, GraphQL queries |
| Interactive | Execute on agent action | Event handlers, form validation |
| Analytics | Block entirely | GA, Mixpanel, Hotjar |
| Advertising | Block entirely | Google Ads, FB Pixel |
| Unknown | Execute with timeout (2s) | Unclassified scripts |

Classification uses:
1. **URL pattern matching** - known analytics/ad domains
2. **AST analysis** - detect tracking patterns (pixel insertion, beacon sending)
3. **Community blocklists** - maintained like ad-blocker lists
4. **Wasm skills** - site-specific script classification

---

## 9. Wasm Skill System

### 9.1 Concept

Skills are WebAssembly modules that teach Plasmate how to interact with specific websites or web applications. They're the equivalent of browser extensions but for agents.

### 9.2 Skill Interface

Every skill implements a standard interface:

```rust
// Skill trait (Rust SDK)
pub trait Skill {
    /// Domains this skill handles
    fn domains(&self) -> Vec<String>;
    
    /// Actions this skill provides
    fn actions(&self) -> Vec<SkillAction>;
    
    /// Handle an AWP intent
    fn handle(&self, ctx: &SkillContext, intent: &Intent) -> Result<ActionResult>;
    
    /// Transform SOM for this domain (optional)
    fn transform_som(&self, som: &mut SomDocument) -> Result<()>;
    
    /// Classify scripts for this domain (optional)
    fn classify_script(&self, url: &str, content: &str) -> ScriptClass;
}
```

### 9.3 Example: Stripe Checkout Skill

```rust
use plasmate_skill_sdk::prelude::*;

struct StripeCheckoutSkill;

impl Skill for StripeCheckoutSkill {
    fn domains(&self) -> Vec<String> {
        vec!["checkout.stripe.com".into(), "*.stripe.com".into()]
    }
    
    fn actions(&self) -> Vec<SkillAction> {
        vec![
            SkillAction::new("stripe_pay")
                .description("Complete a Stripe checkout")
                .params(&[
                    Param::new("card_number", ParamType::String).required(),
                    Param::new("expiry", ParamType::String).required(),
                    Param::new("cvc", ParamType::String).required(),
                    Param::new("email", ParamType::String).optional(),
                    Param::new("name", ParamType::String).optional(),
                ]),
            SkillAction::new("stripe_apply_coupon")
                .description("Apply a coupon code")
                .params(&[Param::new("code", ParamType::String).required()]),
        ]
    }
    
    fn handle(&self, ctx: &SkillContext, intent: &Intent) -> Result<ActionResult> {
        match intent.action.as_str() {
            "stripe_pay" => {
                // Navigate through Stripe's multi-step checkout
                let email = intent.param("email")?;
                let card = intent.param("card_number")?;
                let expiry = intent.param("expiry")?;
                let cvc = intent.param("cvc")?;
                
                // Fill email field
                ctx.fill_by_label("Email", email)?;
                ctx.click_by_text("Pay")?;
                ctx.wait_for_navigation()?;
                
                // Fill card details
                ctx.fill_by_label("Card number", card)?;
                ctx.fill_by_label("Expiration", expiry)?;
                ctx.fill_by_label("CVC", cvc)?;
                
                // Submit payment
                ctx.click_by_text("Pay")?;
                ctx.wait_for_url_contains("/success")?;
                
                Ok(ActionResult::success()
                    .with_data("status", "payment_completed"))
            }
            "stripe_apply_coupon" => {
                let code = intent.param("code")?;
                ctx.click_by_text("Add promotion code")?;
                ctx.fill_by_label("Promotion code", code)?;
                ctx.click_by_text("Apply")?;
                ctx.wait_for_element("[data-testid=discount]")?;
                
                Ok(ActionResult::success())
            }
            _ => Err(SkillError::UnknownAction)
        }
    }
}
```

### 9.4 Skill Distribution

Skills are distributed as `.wasm` files via a package registry:

```bash
# Install a skill
plasmate skill install stripe-checkout
plasmate skill install salesforce-navigator
plasmate skill install google-sheets

# List installed skills
plasmate skill list

# Publish a skill
plasmate skill publish ./my-skill.wasm --name "shopify-admin" --version "1.0.0"

# Skills auto-activate when visiting matching domains
```

**Registry:** `skills.plasmate.app` (npm-like registry for Wasm skills)

### 9.5 Skill SDK

SDKs in multiple languages (all compile to Wasm):

| Language | SDK | Status |
|---|---|---|
| Rust | `plasmate-skill-sdk` (crate) | Primary, full featured |
| TypeScript | `@plasmate/skill-sdk` (npm) | Via AssemblyScript or wasm-bindgen |
| Python | `plasmate-skill-sdk` (pip) | Via componentize-py |
| Go | `plasmate-skill-sdk` (module) | Via TinyGo |
| C/C++ | `plasmate_skill.h` (header) | Via Emscripten |

### 9.6 Community Marketplace

The skill marketplace is a critical network effect:

1. **Free skills** - community-contributed, open source, anyone can publish
2. **Verified skills** - reviewed by Plasmate team, guaranteed quality
3. **Premium skills** - monetized by authors (revenue share: 70% author / 30% Plasmate)
4. **Enterprise skills** - private, company-specific, deployed to their fleet only

As the skill library grows, Plasmate becomes more capable without core engine changes. This is the Chrome Web Store model applied to agent automation.

---

## 10. Session & State Management

### 10.1 Persistent Sessions

Unlike CDP (where sessions die when the WebSocket disconnects), Plasmate sessions are durable:

```rust
let session = plasmate.session_create(SessionConfig {
    id: "user-checkout-flow",
    ttl: Duration::from_hours(24),
    persist: PersistMode::Disk,      // Survive engine restart
    cookies: CookiePolicy::Inherit,  // Carry forward from previous sessions
    auth: Some(AuthState::load("user-123")?),
});
```

**Session state includes:**
- Cookies (per-domain)
- Authentication tokens
- Local storage / session storage
- Navigation history
- SOM snapshots (for resumption)
- Agent memory context
- Proxy assignment (sticky)

### 10.2 Session Pools

For fleet operations, sessions can be pooled:

```rust
let pool = plasmate.session_pool(PoolConfig {
    name: "authenticated-gmail",
    min_sessions: 10,
    max_sessions: 100,
    warmup: WarmupStrategy::PreAuthenticate {
        url: "https://mail.google.com",
        credentials: vault.get("gmail-creds"),
    },
    idle_timeout: Duration::from_minutes(30),
    recycle_after: 50,  // New session after 50 uses
});

// Agents check out pre-authenticated sessions
let session = pool.acquire().await?;
// ... use session ...
pool.release(session).await;
```

### 10.3 State Encryption

Sensitive session data (credentials, tokens, cookies) is encrypted at rest:
- AES-256-GCM encryption
- Per-session encryption keys
- Key derivation from master secret via HKDF
- Optional HSM/TPM integration for enterprise

---

## 11. Fleet Orchestration (Commercial)

### 11.1 The Control Plane

The commercial product. While the engine is free, running 10,000 agents requires orchestration:

```
┌─────────────────────────────────────────────────────┐
│              Plasmate Cloud Control Plane                 │
│                                                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Scheduler │  │ Autoscaler│  │  Cost Tracker   │  │
│  │           │  │           │  │                  │  │
│  │ Priority  │  │ Demand-   │  │ Per-session     │  │
│  │ queues    │  │ based     │  │ Per-domain      │  │
│  │ Rate      │  │ scaling   │  │ Per-agent        │  │
│  │ limiting  │  │           │  │ LLM + compute   │  │
│  └──────────┘  └──────────┘  └──────────────────┘  │
│                                                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Session   │  │  Proxy   │  │  Observability  │  │
│  │ Store     │  │  Network │  │                  │  │
│  │           │  │           │  │ Traces          │  │
│  │ Redis +   │  │ Residential│ │ Metrics         │  │
│  │ S3        │  │ Datacenter│  │ Logs            │  │
│  │ Encrypted │  │ Rotating  │  │ Dashboards      │  │
│  └──────────┘  └──────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 11.2 Fleet API

```python
from plasmate_cloud import Fleet

fleet = Fleet(api_key="plasmate_live_xxx")

# Run 100 agents concurrently
results = await fleet.run_batch(
    agents=100,
    task="extract_pricing",
    urls=["https://competitor1.com", "https://competitor2.com", ...],
    skill="pricing-extractor",
    proxy_type="residential",
    region="us-east",
    budget=Budget(max_cost_usd=50.00),
)

# Real-time monitoring
async for event in fleet.stream("batch-123"):
    print(f"Agent {event.agent_id}: {event.status} - {event.url}")
```

### 11.3 Pricing (Fleet)

| Tier | Price | Sessions | Proxy | Support |
|---|---|---|---|---|
| Starter | $49/mo | 1,000/mo | Datacenter | Community |
| Growth | $199/mo | 10,000/mo | Datacenter + Residential | Email |
| Scale | $799/mo | 100,000/mo | Premium residential | Priority |
| Enterprise | Custom | Unlimited | Dedicated | 24/7 + SLA |

**Usage-based pricing on top:**
- Datacenter proxy: $0.001/request
- Residential proxy: $0.01/request
- Session storage: $0.10/GB-month
- Persistent sessions: $0.001/hour

---
