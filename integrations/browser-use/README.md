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
  [e1] link "More information..." (click)

## Content
This domain is for use in illustrative examples in documents...

---
Compression: 15.2x (1256 HTML bytes -> 83 SOM bytes)
Elements: 5 (1 interactive)
```

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
