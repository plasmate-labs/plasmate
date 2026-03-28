# SOM Specification

The Semantic Object Model (SOM) is a formal, open specification for representing web pages as structured JSON documents optimized for LLM and agent consumption. SOM compiles raw HTML into compact, semantically meaningful output that preserves interactive elements and content hierarchy while stripping presentational noise.

The full specification is at [`specs/SOM-SPEC-v1.0.md`](https://github.com/nicepkg/plasmate/blob/master/specs/SOM-SPEC-v1.0.md) in the repository.

## Document Structure

Every SOM document is a JSON object with these top-level fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `som_version` | string | Yes | Specification version (e.g., `"0.1"`) |
| `url` | string | Yes | Page URL |
| `title` | string | Yes | Page title from `<title>` |
| `lang` | string | Yes | Page language from `<html lang>`, defaults to `"en"` |
| `regions` | array | Yes | Semantic regions extracted from the page |
| `meta` | object | Yes | Compilation metadata (html_bytes, som_bytes, element_count, interactive_count) |
| `structured_data` | object | No | JSON-LD, OpenGraph, Twitter Cards, meta tags |

## Region Roles

Regions are the primary organizational unit. SOM detects them from HTML5 landmarks, ARIA roles, and class/ID heuristics.

| Role | Detection Sources |
|------|-------------------|
| `navigation` | `<nav>`, `role="navigation"`, link-density heuristic |
| `main` | `<main>`, `role="main"`, class/ID heuristics |
| `complementary` | `<aside>`, `role="complementary"` |
| `contentinfo` | `<footer>`, `role="contentinfo"` |
| `banner` | `<header>`, `role="banner"` |
| `form` | `<form>` elements with interactive controls |
| `search` | `role="search"` (maps to navigation) |
| `generic` | Fallback for unclassified content |

Detection precedence: ARIA roles > HTML5 landmarks > `<form>` elements > class/ID heuristics > link-density heuristic > fallback.

## Element Types

Each element has a `role`, stable `id`, optional `html_id` (the original HTML element ID for DOM resolution), and role-specific `attrs`.

| Role | Interactive | Default Actions | HTML Sources |
|------|-------------|-----------------|--------------|
| `link` | Yes | `["click"]` | `<a href>`, `role="link"` |
| `button` | Yes | `["click"]` | `<button>`, `<input type="submit">`, `role="button"` |
| `text_input` | Yes | `["type", "clear"]` | `<input type="text\|email\|password\|search\|...">` |
| `textarea` | Yes | `["type", "clear"]` | `<textarea>` |
| `select` | Yes | `["select"]` | `<select>` |
| `checkbox` | Yes | `["toggle"]` | `<input type="checkbox">`, `role="checkbox"` |
| `radio` | Yes | `["select"]` | `<input type="radio">`, `role="radio"` |
| `image` | No | -  | `<img>`, `<picture>`, `role="img"` |
| `heading` | No | -  | `<h1>` through `<h6>` |
| `paragraph` | No | -  | `<p>`, bare text nodes |
| `table` | No | -  | `<table>` (data tables only) |
| `list` | No | -  | `<ul>`, `<ol>` |
| `section` | No | -  | `<section>`, `<article>` |

Interactive elements are **never** dropped by budget limits.

## Stable Element IDs

Every element gets a deterministic ID derived from its semantic identity:

```
element_id = "e_" + hex(sha256(origin + "|" + role + "|" + name + "|" + dom_path))[0..12]
```

IDs survive page refreshes and minor layout changes. The same button with the same text on the same page always produces the same ID. Agents can reference elements across snapshots using these stable IDs.

Region IDs follow a simpler pattern: `r_{role}` for the first of each role, `r_{role}_{index}` for subsequent ones (e.g., `r_navigation`, `r_navigation_1`).

## Affordances

The `actions` array on interactive elements tells agents what operations are available:

| Action | Elements | Description |
|--------|----------|-------------|
| `click` | link, button | Navigate or activate |
| `type` | text_input, textarea | Enter text content |
| `clear` | text_input, textarea | Clear current value |
| `select` | select, radio | Choose an option |
| `toggle` | checkbox | Toggle checked state |

Non-interactive elements omit the `actions` field entirely.

## JSON Schema

The formal JSON Schema is at [`specs/som-schema.json`](https://github.com/nicepkg/plasmate/blob/master/specs/som-schema.json). Use it to validate SOM output from any producer:

```bash
# Using ajv-cli
npm install -g ajv-cli
ajv validate -s specs/som-schema.json -d output.json

# Using Python jsonschema
pip install jsonschema
python -c "
import json, jsonschema
schema = json.load(open('specs/som-schema.json'))
doc = json.load(open('output.json'))
jsonschema.validate(doc, schema)
print('Valid SOM document')
"
```

## Conformance Tests

The conformance test suite at [`specs/conformance/`](https://github.com/nicepkg/plasmate/tree/master/specs/conformance) contains HTML inputs paired with expected SOM outputs. Use these to verify a SOM producer handles all element types, region detection, ID generation, and budget enforcement correctly.

## What's Next

- Read the [SOM Reference](som) for implementation details on budgets, hints, and compression
- Read the [AWP Protocol](awp) for the WebSocket API that serves SOM
- Check the [Quick Start](quickstart) to produce your first SOM output
