# Browser Use Integration

Use Plasmate's shipped extractor to give [Browser Use](https://github.com/browser-use/browser-use) agents structured SOM page context instead of raw HTML. Output size and token use depend on the page and model tokenizer.

This repository ships `plasmate_browser_use.PlasmateExtractor`. It does not ship `PlasmateBrowser`, session `navigate()` / `click()` / `type_text()`, or a Playwright replacement.

Source: [`integrations/browser-use/`](https://github.com/plasmate-labs/plasmate/tree/master/integrations/browser-use)

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
from plasmate_browser_use import PlasmateExtractor

extractor = PlasmateExtractor()
context = extractor.get_page_context("https://example.com")
print(context)

som = extractor.extract("https://example.com")
md = extractor.extract_markdown("https://example.com", selector="main")
```

Feed `get_page_context()` into your Browser Use agent as page context. Persistent click/type still uses Browser Use's own browser backend or the [Python SDK](sdk-python) `open_page` tools.

## How It Works

`PlasmateExtractor` runs `plasmate fetch` and returns a SOM dict, markdown, or formatted page context. It does not wrap an MCP subprocess or hold a browser session.

| | Browser Use default | PlasmateExtractor |
|---|---|---|
| **Role** | Browser agent runtime | Read-only SOM page context |
| **Output to LLM** | DOM-derived page text | Structured SOM text |
| **Interactive elements** | Backend node ids | SOM roles, ids, and actions |
| **Dependencies** | Chrome, Playwright | `plasmate` binary |
| **Click / type** | Playwright session | Not included; use Browser Use or the Python SDK |

The lightweight extractor also exposes `extract_action_plan()` and
`extract_action_plan_async()` for agents that want only reusable action
targets. Those targets carry `enabled`, disabled/inert `blocked_reason`, `required`,
`description`, `placeholder`, `group`, `current`, `controls`, and `haspopup`
context when Plasmate emits it, so Browser Use agents can skip unavailable
controls and understand popup/controlled-panel targets before spending a
browser action.

For repetitive workflows, scope targets before prompting:

```python
from plasmate_browser_use import PlasmateExtractor

extractor = PlasmateExtractor()
plan = extractor.extract_action_plan("https://example.com/settings")
buttons = extractor.find_action_targets_by_role(
    "https://example.com/settings",
    "button",
    enabled_only=True,
)
clicks = extractor.find_action_targets_by_action(
    "https://example.com/settings",
    "click",
    enabled_only=True,
)
```

## Output-size evidence

In the v0.5.1 observational benchmark snapshots, serialized SOM was smaller than
raw HTML by a median 9.98x across 83 successful non-JavaScript inputs out of 98
attempted, and by a median 9.32x across 82 successful JavaScript inputs out of
98 attempted. Results vary by page. These are serialized-byte ratios, not
universal token, cost, latency, or task-success guarantees; measure the complete
Browser Use workflow with your target pages and model.

## API Reference

### `PlasmateExtractor`

```python
PlasmateExtractor(
    plasmate_bin="plasmate",  # Path to plasmate binary
)
```

| Method | Description | Returns |
|--------|-------------|---------|
| `extract(url, selector=None, javascript=True)` | Fetch a URL as parsed SOM | `dict` |
| `get_page_context(url, selector=None, javascript=True)` | Formatted SOM text for an LLM | `str` |
| `extract_markdown(url, selector=None, javascript=True)` | SOM content as markdown | `str` |
| `extract_action_plan(url, javascript=True)` | Compact reusable action targets | `list[dict]` |
| `extract_action_plan_index(url, enabled_only=False, javascript=True)` | Action targets indexed for replay | `dict` |
| `find_action_target(url, value, ...)` | Resolve one target by SOM id, cache key, HTML id, or test id | `dict` or `None` |
| `find_action_targets_by_role(url, role, ...)` | Action targets with a SOM role | `list[dict]` |
| `find_action_targets_by_action(url, action, ...)` | Action targets exposing an action | `list[dict]` |

Async variants exist for each method (`extract_async`, `get_page_context_async`, and so on).

## Known Limitations

- **Read-only extractor** - `PlasmateExtractor` does not click, type, navigate a session, or replace Playwright.
- **No screenshots** - agents that need pixels should keep Browser Use's Playwright backend or use Plasmate `inspect_page`.
- **No file uploads** - the extractor has no upload API.
