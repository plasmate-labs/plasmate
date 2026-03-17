# JS-Rendered Content: Build Brief

## Goal
Make Plasmate produce correct SOMs for JS-rendered pages (React, Vue, Angular, dynamic content).
After this, fix Puppeteer compatibility so `browser.newPage()` works end-to-end.

## Current State

### What Works
- V8 integrated via `rusty_v8`, persistent context per page
- DOM shim with ~200 lines of JS providing `document`, `window`, `console`, timers, fetch/XHR stubs
- Script extraction from HTML (inline + external with `--fetch-external`)
- Mutations captured in `__plasmate_mutations` array
- `document.write` and `appendChild` mutations injected back into HTML before SOM compile
- Timer draining (short `setTimeout` callbacks executed)
- 20/20 real sites passing script execution (284/738 scripts across 20 sites)

### What's Missing (The Gap)
1. **DOM shim doesn't build a real tree** - `document.createElement` creates detached `PlasElement` objects but `querySelector`/`getElementById` always return `null`. JS that queries the DOM after creating elements gets nothing back.
2. **No HTML parsing of the source into the shim tree** - The shim starts with empty `<html><head><body>`. It doesn't reflect the actual page structure. Scripts that do `document.getElementById('app')` or `document.querySelector('.content')` fail.
3. **Mutations don't actually modify the HTML** properly - The pipeline re-executes scripts in a fresh context to collect mutations, then does string insertion before `</body>`. This misses: DOM node removal, attribute changes, innerHTML replacement, textContent updates on existing nodes.
4. **No async execution model** - `fetch()` returns empty stubs. React/Vue hydration calls `fetch()` for data, gets nothing, renders empty. Real JS-rendered pages need network responses to produce content.
5. **SOM is compiled from original HTML, not from the mutated DOM** - Even when mutations are captured, the SOM compiler parses the original (or lightly patched) HTML string, not a proper post-JS DOM tree.

## Architecture Decision

**Option A: Build a full DOM in Rust, expose it to V8 via bindings**
- Most correct approach (what real browsers do)
- Massive effort: need Rust DOM tree + V8 C++ bindings for every DOM API
- 10,000+ lines minimum, weeks of work

**Option B: Build a richer DOM tree in JS, serialize back to HTML after execution**
- The DOM shim becomes a real (but minimal) DOM implementation in JS
- Parse the source HTML into the JS DOM tree before running page scripts
- After all scripts run, serialize the JS DOM tree back to HTML
- Feed that HTML to the existing SOM compiler
- Much faster to build, leverages existing SOM compiler
- Trade-off: JS DOM won't be 100% spec-compliant, but covers 80-90% of real-world patterns

**Decision: Option B.** Ship fast, iterate. The JS DOM tree in the shim needs to be good enough for React/Vue/Angular hydration and common dynamic content patterns, not a full W3C DOM implementation.

## Implementation Plan

### Phase 1: Rich DOM Shim (the JS side)

Replace the current `PlasElement`/shim with a proper DOM tree implementation in JS:

1. **Node types**: Element, Text, Comment, DocumentFragment with proper `nodeType`, `parentNode`, `childNodes`, `firstChild`, `lastChild`, `nextSibling`, `previousSibling`
2. **Tree operations**: `appendChild`, `removeChild`, `insertBefore`, `replaceChild`, `cloneNode(deep)` - all maintaining parent/child/sibling pointers
3. **Query methods**: `getElementById`, `querySelector`, `querySelectorAll` with basic CSS selector support (tag, #id, .class, [attr], combinators)
4. **innerHTML/outerHTML**: Parse HTML strings into DOM nodes (mini HTML parser in JS), serialize DOM subtree back to HTML string
5. **Element specifics**: `classList`, `style.cssText`, `dataset`, form element `value`/`checked`/`selected`
6. **Events**: `DOMContentLoaded`, `load`, `readystatechange` fire at correct lifecycle points
7. **Serialize**: `document.documentElement.outerHTML` produces the full post-JS HTML

### Phase 2: HTML-to-DOM Bootstrap

Before running any page scripts:

1. Parse the fetched HTML into the JS DOM tree (using the innerHTML parser from Phase 1)
2. Set `document.title`, `document.head`, `document.body` from parsed tree
3. Set `window.location` from the page URL
4. Inject `<script>` elements are already extracted - just need the tree to reflect the rest of the page structure

### Phase 3: Async-Capable Fetch

Replace the empty fetch stub with one that actually makes HTTP requests:

1. Expose a Rust function to V8 that does synchronous HTTP fetch (via `reqwest::blocking` or `tokio::runtime::Handle::block_on`)
2. JS `fetch()` calls this Rust function, gets real response body
3. XHR gets the same treatment
4. This is critical for React apps that fetch data during hydration/render

### Phase 4: Pipeline Integration

Update `pipeline.rs`:

1. After all scripts execute + timers drain, serialize the DOM tree: `document.documentElement.outerHTML`
2. Pass the serialized HTML (which now includes JS-created elements, modified attributes, updated text) to the SOM compiler
3. The SOM compiler doesn't need to change - it already handles HTML

### Phase 5: Puppeteer Compatibility

Fix the CDP layer for multi-target routing:

1. The `setAutoAttach` / `runIfWaitingForDebugger` infinite loop - root cause is that `attachedToTarget` events trigger more `setAutoAttach` calls which trigger more events
2. Each `Target.createTarget` needs a truly independent session with its own page state
3. `Page.navigate` on a child target should use that target's session, not the parent's
4. Frame tree needs to be per-target, with proper `executionContextCreated` events per frame

## Files to Modify

- `src/js/runtime.rs` - New DOM shim (DOM_SHIM const), Rust-side fetch bridge
- `src/js/pipeline.rs` - Serialize JS DOM -> HTML -> SOM compiler
- `src/js/extract.rs` - May need to preserve non-script HTML structure for bootstrap
- `src/cdp/handler.rs` - Fix multi-target session routing
- `src/cdp/session.rs` - Per-target state isolation
- `src/cdp/domains.rs` - Fix `setAutoAttach` event logic

## Test Strategy

### JS Rendering Tests
- Static HTML (no JS) - SOM unchanged
- `document.write()` - content appears in SOM
- `document.createElement` + `appendChild` - new elements in SOM
- `innerHTML` replacement - updated content in SOM
- `getElementById` + modify - changes reflected in SOM
- React-style hydration pattern (create elements, set textContent from data)
- Timer-based rendering (`setTimeout(render, 0)`)
- `DOMContentLoaded` handler that modifies DOM

### Puppeteer Compatibility Tests
- `browser.newPage()` creates a working page
- `page.goto(url)` navigates and returns
- `page.content()` returns HTML
- `page.evaluate()` runs JS
- `page.$eval()` queries DOM
- Multiple pages don't interfere

## Success Criteria

1. `plasmate fetch https://react-site.com` produces a SOM with the rendered content, not just the empty `<div id="root"></div>`
2. Puppeteer smoke test passes: `browser.newPage()` -> `page.goto()` -> `page.content()` returns rendered HTML
3. No regression on existing 184 tests
4. Throughput stays under 50ms/page for JS-light pages (acceptable to go to ~200ms for JS-heavy pages that need execution)
