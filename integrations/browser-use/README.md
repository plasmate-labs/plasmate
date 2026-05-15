# plasmate-browser-use

SOM-based content extraction for [Browser Use](https://github.com/browser-use/browser-use). Drop-in alternative to Browser Use's default DOM serializer that uses Plasmate's Semantic Object Model (SOM) to reduce token costs by 10x or more.

Instead of sending the full DOM tree to your LLM, Plasmate compresses web pages into a compact semantic representation. Same information, 90% fewer tokens, lower costs, faster responses.

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

ready_actions = extractor.extract_enabled_action_plan("https://example.com/settings")
replay_index = extractor.extract_action_plan_index(
    "https://example.com/settings",
    enabled_only=True,
)
cached = replay_index["by_cache_key"].get("plasmate-action:v1:...")

summary = extractor.extract_action_plan_summary("https://example.com/settings")
fingerprint = extractor.extract_action_plan_fingerprint(
    "https://example.com/settings",
    enabled_only=True,
)
print(summary["enabled"], summary["unique_cache_keys"], summary["duplicate_cache_keys"], fingerprint)
```

Use the summary and fingerprint helpers before replaying cached Browser Use
actions: they reveal whether the current page still has the same compact action
menu, whether replay lookup is complete or ambiguous, and whether drift came
from missing targets, disabled/read-only controls, or a changed role mix.

Browser Use page contexts are tested against the shared
`integrations/fixtures/action-availability.som.json` fixture so availability,
cache-key, required, readonly, inert, group, type, value, checked, expanded, pressed,
selected, current, controls, haspopup, name, accept, capture, multiple,
autocomplete, inputmode, enterkeyhint, autocapitalize, dirname, dir, lang, spellcheck,
link target/rel/download cues, graphical submitter alt/src cues, form plus
form submission context, submitter override cues, select selected_values/size
context, original `html_id` bridge cues, list, popover/command relationships, accesskey, ARIA
placeholder/autocomplete/active-descendant/error-message state, title,
aria_label/aria_description source text, labelledby/describedby relationship refs, keyshortcuts, roledescription,
live-region state, ARIA owns/flowto/details relationships,
ARIA readonly/multiline/multiselectable widget state, orientation/sort/value
state, ARIA set-position cues, validation constraints, range constraints, invalid state, and
description cues
stay aligned with other adapters.

### Markdown extraction

```python
md = extractor.extract_markdown("https://example.com")
print(md)
```

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

# Feed to your Browser Use agent with 10x fewer tokens
agent = Agent(task="Find the cheapest product", page_context=context)
result = await agent.run()
```

### Token savings comparison

```python
from plasmate_browser_use import PlasmateExtractor, token_count_comparison

extractor = PlasmateExtractor()
som = extractor.extract("https://news.ycombinator.com")
stats = token_count_comparison(som)

print(f"HTML tokens: ~{stats['html_tokens_est']:,}")
print(f"SOM tokens:  ~{stats['som_tokens_est']:,}")
print(f"Savings:     {stats['token_savings_pct']}%")
print(f"Ratio:       {stats['token_ratio']}x fewer tokens")
```

## Typical token savings

| Site | HTML tokens | SOM tokens | Reduction |
|------|------------|------------|-----------|
| Hacker News | ~22,000 | ~1,200 | 18x |
| Wikipedia article | ~85,000 | ~8,500 | 10x |
| Amazon product page | ~120,000 | ~6,000 | 20x |
| Google search results | ~45,000 | ~3,500 | 13x |

Numbers vary by page. The more complex the page (ads, trackers, layout noise), the bigger the savings.

## How it works

1. Plasmate fetches the page and parses the HTML
2. The DOM is compiled into a Semantic Object Model (SOM) that preserves meaning while stripping layout noise
3. The SOM is serialized into a compact format with tagged interactive elements
4. Your LLM agent sees the same page information in 10x fewer tokens

## Links

- [Plasmate](https://plasmate.app) -- the SOM engine
- [SOM Spec](https://plasmate.app/docs/som-spec) -- Semantic Object Model specification
- [Browser Use](https://github.com/browser-use/browser-use) -- AI agent browser framework
- [Token cost analysis](https://plasmate.app/docs/cost-analysis) -- detailed benchmarks

## License

Apache-2.0
