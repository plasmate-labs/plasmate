# som-parser

Parse and query [SOM (Semantic Object Model)](https://plasmate.app/docs/som-spec) output in Python. SOM is a structured JSON format that represents web pages as semantic regions and elements, designed for AI agents, browser automation, and web scraping. This library provides Pydantic v2 models for type-safe parsing, validation, and a rich set of query utilities to extract exactly what you need.

## Install

```bash
pip install som-parser
```

## Quick Start

### Parse Plasmate output

```python
import subprocess
from som_parser import parse_som, from_plasmate

# Parse a SOM JSON string or dict
som = parse_som('{"som_version": "0.1", ...}')

# Or parse raw Plasmate CLI output directly
result = subprocess.run(["plasmate", "https://example.com"], capture_output=True, text=True)
som = from_plasmate(result.stdout)

print(som.title)       # "Example Domain"
print(som.url)         # "https://example.com/"
print(som.som_version) # "0.1"
```

### Find links

```python
from som_parser import parse_som, get_links, find_by_role

som = parse_som(data)

# Get all links as simple dicts
for link in get_links(som):
    print(f"{link['text']} -> {link['href']}")

# Or find by role for full SomElement objects
for el in find_by_role(som, "link"):
    print(el.id, el.text, el.attrs.href)
```

### Plan agent actions

```python
from som_parser import parse_som, find_by_action, get_action_plan

som = parse_som(data)
for item in get_action_plan(som):
    if item["enabled"]:
        print(item["id"], item["cache_key"], item["role"], item["actions"], item.get("label"))
    else:
        print(item["id"], "blocked:", item.get("blocked_reason"))

for button in find_by_action(som, "click"):
    print(button.id, button.text or button.label)
```

### Convert to markdown

```python
from som_parser import parse_som, to_markdown

som = parse_som(data)
print(to_markdown(som))
```

### Use Pydantic models directly

```python
from som_parser import Som, SomElement, ElementRole

# Validate and construct from a dict
som = Som.model_validate(my_dict)

# Access typed fields
for region in som.regions:
    for element in region.elements:
        if element.role == ElementRole.LINK:
            print(element.attrs.href)

# Serialize back to JSON
print(som.model_dump_json(indent=2))
```

## API Reference

### Parser

| Function | Description |
|----------|-------------|
| `parse_som(input: str \| dict) -> Som` | Parse JSON string or dict into a validated Som object |
| `is_valid_som(input) -> bool` | Check if input conforms to the SOM schema |
| `from_plasmate(json_output: str) -> Som` | Parse raw Plasmate CLI JSON output |

### Query Utilities

| Function | Description |
|----------|-------------|
| `get_all_elements(som) -> list[SomElement]` | Flatten all elements from all regions |
| `find_by_role(som, role) -> list[SomElement]` | Find elements by role (enum or string) |
| `find_by_id(som, id) -> SomElement \| None` | Find a single element by its SOM id |
| `find_by_text(som, text, exact=False) -> list[SomElement]` | Search elements by text content |
| `find_by_action(som, action) -> list[SomElement]` | Find elements that expose a specific action |
| `find_by_hint(som, hint) -> list[SomElement]` | Find elements tagged with a semantic hint |
| `get_action_plan(som) -> list[dict]` | Return compact `{id, cache_key, role, actions, enabled, label}` action targets with availability, form/list and popover/command relation cues, input-affordance cues, validation constraints, ARIA live-region cues, and ARIA owns/flowto/details relationships |
| `get_action_plan_cache_key(item) -> str` | Return a deterministic key for caching or comparing an action target |
| `get_interactive_elements(som) -> list[SomElement]` | Get elements that have actions |
| `get_links(som) -> list[dict]` | Extract all links as `{text, href, id}` dicts |
| `get_forms(som) -> list[SomRegion]` | Get all form regions |
| `get_inputs(som) -> list[SomElement]` | Get all input elements |
| `get_headings(som) -> list[dict]` | Extract heading hierarchy as `{level, text, id}` |
| `get_text(som) -> str` | Extract all visible text content |
| `get_text_by_region(som) -> list[dict]` | Extract text grouped by region |
| `get_compression_ratio(som) -> float` | Return `html_bytes / som_bytes` |
| `to_markdown(som) -> str` | Convert SOM to readable markdown |
| `filter_elements(som, predicate) -> list[SomElement]` | Generic filter with a callable |

### Types

All Pydantic v2 models are exported from the top level:

- `Som`, `SomRegion`, `SomElement`, `SomElementAttrs`, `SomMeta`
- `StructuredData`, `LinkElement`, `SelectOption`, `ListItem`
- `RegionRole`, `ElementRole`, `ElementAction`, `SemanticHint` (enums)

## Links

- [SOM Spec](https://plasmate.app/docs/som-spec)
- [Plasmate](https://plasmate.app)
- [GitHub Repository](https://github.com/plasmate-labs/plasmate)

## License

Apache-2.0
