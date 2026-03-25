# Browser Use Integration

Use Plasmate as the browser backend for [Browser Use](https://github.com/browser-use/browser-use), replacing Chrome + Playwright with SOM output for **~10x fewer tokens**.

Browser Use is the most popular open-source AI browser agent framework. By default it feeds raw DOM trees to the LLM. Plasmate replaces this with the Semantic Object Model -  compact, structured page representations that preserve all interactive elements while stripping layout noise.

Source: [`integrations/browser-use/`](https://github.com/nicepkg/plasmate/tree/master/integrations/browser-use)

## Installation

```bash
pip install plasmate-browser-use
```

Requires the `plasmate` binary on your PATH:

```bash
curl -fsSL https://plasmate.app/install.sh | sh
```

## Quick Start

```python
import asyncio
from plasmate_browser_use import PlasmateBrowser

async def main():
    async with PlasmateBrowser() as browser:
        # Navigate and get SOM state
        state = await browser.navigate("https://news.ycombinator.com")
        print(state.text)  # What the LLM sees

        # Click an element by its index
        link = state.interactive_elements[0]
        state = await browser.click(link.index)

        # Type into a form field
        inputs = [el for el in state.interactive_elements if el.role == "text_input"]
        if inputs:
            state = await browser.type_text(inputs[0].index, "hello world")

asyncio.run(main())
```

## How It Works

`PlasmateBrowser` wraps a Plasmate MCP subprocess. When you call `navigate()`, `click()`, or `type_text()`, it sends AWP commands to Plasmate and returns SOM output formatted for Browser Use compatibility.

| | Browser Use (Playwright) | Browser Use (Plasmate) |
|---|---|---|
| **Backend** | Chrome via Playwright | Plasmate MCP subprocess |
| **Output to LLM** | Raw DOM tree | SOM (Semantic Object Model) |
| **Typical tokens** | ~20,000 per page | ~2,000 per page |
| **Interactive elements** | `[backend_node_id]<tag>` | `[N] role "label"` |
| **Dependencies** | Chrome, Playwright | `plasmate` binary only |
| **Startup time** | ~2-5s (browser launch) | ~50ms (subprocess) |

SOM replaces DOM screenshots with structured text. Instead of parsing a raw DOM tree with thousands of nodes, the LLM sees a compact summary with numbered interactive elements:

```
[Tab] Hacker News
[URL] https://news.ycombinator.com

--- navigation "Main menu" ---
  [1] link "Hacker News" -> /
  [2] link "new" -> /newest
  [3] link "past" -> /front

--- main ---
  [4] link "Show HN: Something Cool" -> https://example.com
  142 points by someone
  [5] link "89 comments" -> /item?id=12345678

[SOM] 87,234 -> 4,521 bytes (19.3x) | 156 elements, 89 interactive
```

## Token Savings

| Site | HTML tokens | SOM tokens | Savings |
|------|------------|------------|---------|
| Hacker News | ~22,000 | ~1,500 | **15x** |
| GitHub repo page | ~45,000 | ~3,500 | **13x** |
| Wikipedia article | ~60,000 | ~5,000 | **12x** |
| News article | ~35,000 | ~3,000 | **12x** |
| E-commerce product | ~40,000 | ~4,000 | **10x** |

Over a multi-step agent task (5-10 page loads), this translates to **50,000-150,000 fewer tokens**.

## API Reference

### `PlasmateBrowser`

```python
PlasmateBrowser(
    binary="plasmate",   # Path to plasmate binary
    timeout=30,          # Response timeout in seconds
    budget=None,         # Optional SOM token budget
)
```

| Method | Description | Returns |
|--------|-------------|---------|
| `navigate(url)` | Open a URL in a persistent session | `PageState` |
| `click(element_index)` | Click element by its `[N]` index | `PageState` |
| `type_text(element_index, text)` | Type into an input/textarea | `PageState` |
| `get_state()` | Get current page state as SOM | `PageState` |
| `screenshot()` | Returns `None` (no visual rendering) | `None` |
| `close()` | Close session and shut down process | -  |

### `PageState`

| Field | Type | Description |
|-------|------|-------------|
| `url` | `str` | Current page URL |
| `title` | `str` | Page title |
| `text` | `str` | SOM text for the LLM |
| `interactive_elements` | `list[InteractiveElement]` | All clickable/typeable elements |
| `selector_map` | `dict[int, InteractiveElement]` | Index -> element lookup |
| `som` | `dict` | Raw SOM dict |
| `som_tokens` | `int` | Estimated token count |
| `html_bytes` | `int` | Original HTML size |
| `som_bytes` | `int` | SOM output size |

### `InteractiveElement`

| Field | Type | Description |
|-------|------|-------------|
| `index` | `int` | Integer index (`[N]` in SOM text) |
| `som_id` | `str` | Original SOM element ID |
| `role` | `str` | `link`, `button`, `text_input`, etc. |
| `text` | `str` | Display text or label |
| `attrs` | `dict` | Role-specific attributes |

## Known Limitations

- **No screenshots** -  Plasmate has no visual rendering. Agents that rely on screenshots (CAPTCHA, visual layout) should use the Playwright backend.
- **No coordinate-based clicking** -  all interactions use SOM element IDs, not pixel coordinates.
- **No file uploads** -  the `upload_file` action is not supported.
- **No scroll position** -  Plasmate renders the full page. No concept of viewport or scroll.
- **Single tab** -  each `PlasmateBrowser` instance maintains one session. Create multiple instances for multi-tab workflows.
