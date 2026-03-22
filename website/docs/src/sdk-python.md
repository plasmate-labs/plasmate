# Python SDK

The official Python SDK for Plasmate, with Pydantic models and SOM query helpers.

## Installation

```sh
pip install plasmate
```

Requires Python 3.9+, Pydantic 2.0+, and the `plasmate` binary on your PATH.

## Quick Start

```python
from plasmate import Plasmate

with Plasmate() as client:
    som = client.fetch_page("https://example.com")
    print(som["title"])
    print(som["meta"])
```

## Pydantic Models

All SOM types are fully typed Pydantic models, making them easy to validate and serialize.

### `Som`

```python
class Som(BaseModel):
    som_version: str
    url: str
    title: str
    lang: str
    regions: List[SomRegion]
    meta: SomMeta
    structured_data: Optional[StructuredData] = None
```

### `SomRegion`

```python
class SomRegion(BaseModel):
    id: str
    role: RegionRole   # navigation, main, aside, header, footer, form, dialog, content
    label: Optional[str] = None
    action: Optional[str] = None
    method: Optional[str] = None
    elements: List[SomElement]
```

### `SomElement`

```python
class SomElement(BaseModel):
    id: str
    role: ElementRole  # link, button, text_input, textarea, select, checkbox, radio, heading, image, list, table, paragraph, section, separator
    text: Optional[str] = None
    label: Optional[str] = None
    actions: Optional[List[str]] = None
    attrs: Optional[ElementAttrs] = None
    children: Optional[List[SomElement]] = None
    hints: Optional[List[SemanticHint]] = None
```

### `SomMeta`

```python
class SomMeta(BaseModel):
    html_bytes: int
    som_bytes: int
    element_count: int
    interactive_count: int
```

## Query Helpers

### `find_by_role(som, role)`

Find all regions matching a given role.

```python
from plasmate.query import find_by_role

nav_regions = find_by_role(som, "navigation")
```

### `find_by_id(som, element_id)`

Find a single element by its stable ID.

```python
from plasmate.query import find_by_id

el = find_by_id(som, "login-btn")
if el:
    print(el.text)
```

### `find_by_tag(som, tag)`

Find elements matching a tag/role string.

```python
from plasmate.query import find_by_tag

links = find_by_tag(som, "link")
```

### `find_interactive(som)`

Return all interactive elements.

```python
from plasmate.query import find_interactive

interactive = find_interactive(som)
print(f"{len(interactive)} interactive elements")
```

### `find_by_text(som, text)`

Find elements whose text content contains the given string (case-insensitive).

```python
from plasmate.query import find_by_text

matches = find_by_text(som, "Sign in")
```

### `flat_elements(som)`

Flatten all elements across all regions into a single list.

```python
from plasmate.query import flat_elements

all_elements = flat_elements(som)
```

### `get_token_estimate(som)`

Estimate the LLM token count for the SOM (heuristic: ~4 characters per token).

```python
from plasmate.query import get_token_estimate

tokens = get_token_estimate(som)
print(f"~{tokens} tokens")
```

## Client API

### Synchronous Client

```python
from plasmate import Plasmate

client = Plasmate(
    binary="plasmate",  # path to plasmate binary (default: "plasmate")
    timeout=30,         # timeout in seconds (default: 30)
)

# Fetch a page and return SOM
som = client.fetch_page("https://example.com")

# Extract plain text
text = client.extract_text("https://example.com")

# Stateful sessions
session = client.open_page("https://example.com")
session_id = session["session_id"]

result = client.evaluate(session_id, "document.title")
updated = client.click(session_id, "login-btn")
client.close_page(session_id)

client.close()
```

Supports context manager:

```python
with Plasmate() as client:
    som = client.fetch_page("https://example.com")
```

## Async Support

The `AsyncPlasmate` client provides the same API with `async`/`await`.

```python
from plasmate import AsyncPlasmate

async with AsyncPlasmate() as client:
    som = await client.fetch_page("https://example.com")
    text = await client.extract_text("https://example.com")

    session = await client.open_page("https://example.com")
    session_id = session["session_id"]

    result = await client.evaluate(session_id, "document.title")
    updated = await client.click(session_id, "login-btn")
    await client.close_page(session_id)
```

Both clients communicate with the `plasmate mcp` subprocess over JSON-RPC 2.0 on stdio.
