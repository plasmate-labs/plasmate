# plasmate-browser-use

SOM-based content extraction for [Browser Use](https://github.com/browser-use/browser-use).
It uses Plasmate's Semantic Object Model (SOM) to provide structured page
context as an alternative to Browser Use's configured page representation.

Instead of sending raw markup, Plasmate emits a compact semantic
representation. Output size, tokenization, retained information, cost, latency,
and task quality depend on the page and workflow. Retained v0.5.1 Plasmate
snapshots measured serialized bytes, not Browser Use token usage; see the
[benchmark policy](../../docs/BENCHMARKING.md) before citing a result.

## Install

```bash
pip install plasmate-browser-use
```

## Prerequisites

You need the `plasmate` binary installed:

```bash
# Via cargo
cargo install plasmate

# Or via install script
curl -fsSL https://plasmate.app/install.sh | sh
```

Verify it works:

```bash
plasmate --version
```

## Quick Start

### Basic extraction

```python
from plasmate_browser_use import PlasmateExtractor

extractor = PlasmateExtractor()

# Get raw SOM data as a dict
som = extractor.extract("https://news.ycombinator.com")
print(f"Elements: {som['meta']['element_count']}")
print(f"Compression: {som['meta']['html_bytes'] / som['meta']['som_bytes']:.1f}x")
```

### Get page context for an LLM

The `get_page_context()` method returns a formatted string optimized for LLM consumption, with interactive elements, links, content, and compression stats:

```python
context = extractor.get_page_context("https://example.com")
print(context)

# Ask Plasmate for only the semantic region needed by the agent.
interactive_context = extractor.get_page_context(
    "https://example.com/settings",
    selector="interactive",
)
```

Output:

```
# Example Domain
URL: https://example.com
Language: en

## Interactive Elements (1)
  [e1] link "More information..." (click) [enabled]

## Content
This domain is for use in illustrative examples in documents...

---
Compression: 15.2x (1256 HTML bytes -> 83 SOM bytes)
Elements: 5 (1 interactive)
```

### Get an action plan

Use `extract_action_plan()` when an agent needs reusable targets without the rest of the page text. Targets include `cache_key` for local action memory. Disabled, inert, and read-only controls include `enabled: false` plus `blocked_reason` so Browser Use agents can skip unavailable actions before spending a tool call:

```python
actions = extractor.extract_action_plan("https://example.com/settings")
for action in actions:
    if action["enabled"]:
        print(action["id"], action["cache_key"], action["role"], action["actions"])
```

Use `extract_action_plan_index()` when a Browser Use workflow needs to resolve
a cached action back to a current SOM target without scanning the menu:

```python
index = extractor.extract_action_plan_index("https://example.com/settings")
target = index["by_cache_key"]["plasmate-action:v1:..."]
print(target["id"], target.get("html_id"), target.get("test_id"))

# Or scope a Browser Use plan before replaying a cached target.
buttons = extractor.find_action_targets_by_role(
    "https://example.com/settings",
    "button",
)
enabled_clicks = extractor.find_action_targets_by_action(
    "https://example.com/settings",
    "click",
    enabled_only=True,
)

# Or let Plasmate resolve SOM id, cache key, HTML id, or test id automatically.
target = extractor.find_action_target(
    "https://example.com/settings",
    "plasmate-action:v1:...",
    enabled_only=True,
)
```

Browser Use page contexts are tested against the shared
`integrations/fixtures/action-availability.som.json` fixture so availability,
cache-key, required, readonly, inert, group, type, value, checked, expanded, pressed,
selected, current, controls, haspopup, name, accept, capture, multiple,
autocomplete, inputmode, enterkeyhint, autocapitalize, dirname, spellcheck,
link target/rel/download cues, form plus form submission context,
submitter override cues, list, popover/command relationships, accesskey, ARIA
placeholder/autocomplete/active-descendant/error-message state, keyshortcuts,
roledescription, live-region state, ARIA owns/flowto/details relationships,
ARIA readonly/multiline/multiselectable widget state, orientation/sort/value
state, ARIA set-position cues, validation constraints, range constraints, invalid state, and
description cues
stay aligned with other adapters.

### Markdown extraction

```python
md = extractor.extract_markdown(
    "https://example.com",
    selector="main",
)
print(md)
```

Pass any supported SOM selector to keep only the page region or action surface
needed by the agent before Markdown conversion.

### Async support

All methods have async variants:

```python
import asyncio

async def main():
    extractor = PlasmateExtractor()
    context = await extractor.get_page_context_async("https://example.com")
    som = await extractor.extract_async("https://example.com")
    md = await extractor.extract_markdown_async("https://example.com")

asyncio.run(main())
```

### Using with a Browser Use agent

```python
from browser_use import Agent
from plasmate_browser_use import PlasmateExtractor

extractor = PlasmateExtractor()

# Get compact page context instead of full DOM
context = extractor.get_page_context("https://example.com/products")

# Feed structured SOM context to your Browser Use agent
agent = Agent(task="Find the cheapest product", page_context=context)
result = await agent.run()
```

### Input-specific size estimate

```python
from plasmate_browser_use import PlasmateExtractor, token_count_comparison

extractor = PlasmateExtractor()
som = extractor.extract("https://news.ycombinator.com")
stats = token_count_comparison(som)

print(f"HTML tokens: ~{stats['html_tokens_est']:,}")
print(f"SOM tokens:  ~{stats['som_tokens_est']:,}")
print(f"Est. change: {stats['token_savings_pct']}%")
print(f"Estimated token ratio for this input: {stats['token_ratio']}x")
```

## Measure your workload

The helper uses an approximate byte-to-token conversion; it is not a named
model tokenizer or a benchmark result. For publishable token evidence, use the
target model's tokenizer and retain every attempted input.

| Input | HTML tokens | SOM tokens | Ratio |
|------|------------|------------|-----------|
| Your measured page | Measure | Measure | Compute for the selected tokenizer |

Results vary by page, configuration, serialization, selector, and tokenizer.

## How it works

1. Plasmate fetches the page and parses the HTML
2. The DOM is compiled into a Semantic Object Model (SOM) that preserves meaning while stripping layout noise
3. The SOM is serialized into a compact format with tagged interactive elements
4. Your LLM agent sees structured page context whose size and retained
   information you should evaluate on the target workflow

## Links

- [Plasmate](https://plasmate.app) -- the SOM engine
- [SOM Spec](https://plasmate.app/docs/som-spec) -- Semantic Object Model specification
- [Browser Use](https://github.com/browser-use/browser-use) -- AI agent browser framework
- [Claim and evidence registry](https://docs.plasmate.app/claims) -- allowed wording and retained evidence

## License

Apache-2.0
