# plasmate-browser-use

[Browser Use](https://github.com/browser-use/browser-use) integration for [Plasmate](https://github.com/nicepkg/plasmate) — replace Playwright + Chrome with Plasmate's SOM output for **~10x fewer tokens**.

Browser Use is the most popular open-source AI browser agent framework. By default it uses Playwright to render pages in Chrome and feeds the raw DOM tree to the LLM. Plasmate replaces this with its Semantic Object Model (SOM), which compiles HTML into compact, structured representations that preserve all interactive elements while stripping layout noise.

## Why Plasmate?

| | Browser Use (Playwright) | Browser Use (Plasmate) |
|---|---|---|
| **Backend** | Chrome via Playwright | Plasmate MCP subprocess |
| **Output to LLM** | Raw DOM tree | SOM (Semantic Object Model) |
| **Typical tokens** | ~20,000 per page | ~2,000 per page |
| **Screenshots** | Yes | No (headless, no rendering) |
| **Interactive elements** | `[backend_node_id]<tag>` | `[N] role "label"` |
| **Dependencies** | Chrome, Playwright | `plasmate` binary only |
| **Startup time** | ~2-5s (browser launch) | ~50ms (subprocess) |

## Installation

```bash
pip install plasmate-browser-use
```

Requires the `plasmate` binary on your PATH:

```bash
# Via npm
npm install -g plasmate

# Via cargo
cargo install plasmate

# Via install script
curl -fsSL https://raw.githubusercontent.com/nicepkg/plasmate/master/install.sh | bash
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

## What the LLM Sees

### Browser Use (Playwright) — ~20,000 tokens

```
[1234]<div class="athing" id="12345678" />
  [1235]<td class="title" />
    [1236]<span class="titleline" />
      [1237]<a href="https://example.com" />
        Show HN: Something Cool
      [1238]<span class="sitebit comhead" />
        (<a href="from?site=example.com" />
          [1239]example.com
        )
  [1240]<td class="subtext" />
    [1241]<span class="score" id="score_12345678" />
      142 points
    [1242]<a href="user?id=someone" />
      someone
    [1243]<a href="item?id=12345678" />
      89 comments
...
```

### Plasmate (SOM) — ~2,000 tokens

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
...

[SOM] 87,234 -> 4,521 bytes (19.3x) | 156 elements, 89 interactive
```

Same information, 10x fewer tokens.

## API Reference

### `PlasmateBrowser`

The main class. Wraps a Plasmate MCP subprocess and provides Browser Use-compatible methods.

```python
PlasmateBrowser(
    binary="plasmate",   # Path to plasmate binary
    timeout=30,          # Response timeout in seconds
    budget=None,         # Optional SOM token budget
)
```

#### Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `navigate(url)` | Open a URL in a persistent session | `PageState` |
| `click(element_index)` | Click element by its `[N]` index | `PageState` |
| `type_text(element_index, text)` | Type into an input/textarea | `PageState` |
| `get_state()` | Get current page state as SOM | `PageState` |
| `screenshot()` | Returns `None` (no visual rendering) | `None` |
| `close()` | Close session and shut down process | — |

### `PageState`

Returned by all navigation/interaction methods.

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

### `som_to_browser_use_state(som, index_map=None)`

Convert a raw SOM dict to Browser Use-compatible text format.

### `token_count_comparison(som, som_text=None)`

Returns a dict comparing HTML vs SOM token counts:

```python
{
    "html_bytes": 87234,
    "som_bytes": 4521,
    "html_tokens_est": 21808,
    "som_tokens_est": 1130,
    "byte_ratio": 19.3,
    "token_ratio": 19.3,
    "token_savings_pct": 94.8,
}
```

## Token Savings

Benchmarked across common websites:

| Site | HTML tokens | SOM tokens | Savings |
|------|------------|------------|---------|
| Hacker News | ~22,000 | ~1,500 | **15x** |
| GitHub repo page | ~45,000 | ~3,500 | **13x** |
| Wikipedia article | ~60,000 | ~5,000 | **12x** |
| News article | ~35,000 | ~3,000 | **12x** |
| Simple landing page | ~8,000 | ~500 | **16x** |
| E-commerce product | ~40,000 | ~4,000 | **10x** |

Average: **~10-15x fewer tokens per page**.

Over a multi-step agent task (5-10 page loads), this translates to **50,000-150,000 fewer tokens** — significant cost savings and faster LLM responses.

## Known Limitations

- **No screenshots**: Plasmate has no visual rendering pipeline. Agents that rely on screenshots (e.g., for CAPTCHA solving or visual layout understanding) should use the default Playwright backend.
- **No coordinate-based clicking**: Plasmate operates at the DOM level. All interactions use SOM element IDs, not pixel coordinates.
- **No file uploads**: The `upload_file` action is not supported.
- **No scroll position**: Plasmate renders the full page, not a viewport. There is no concept of "pixels above/below."
- **Single tab**: Each `PlasmateBrowser` instance maintains one session. For multi-tab workflows, create multiple instances.
- **Not yet upstream**: This is a standalone integration package. The goal is to prove it works here, then submit as a PR to the Browser Use repo.

## Example

```bash
python example.py
```

See [example.py](example.py) for a full working example that navigates to Hacker News, finds the top story, clicks it, and compares token usage.

## License

MIT
