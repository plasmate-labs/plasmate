# SOM (Semantic Object Model) Specification v1.0

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-21
**Authors:** Plasmate Contributors

## Abstract

The Semantic Object Model (SOM) is a structured JSON representation of web pages
designed for consumption by Large Language Models (LLMs) and autonomous agents.
SOM compiles raw HTML into a compact, semantically meaningful document that
preserves the intent-relevant structure of a page — interactive elements, content
hierarchy, navigation landmarks, and form controls — while stripping away
presentational markup, scripts, styles, and other noise that is irrelevant to
agent reasoning.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

## 1. Goals

1. **Token efficiency.** SOM output MUST be substantially smaller than the
   source HTML. The compilation process strips scripts, styles, comments,
   decorative elements, hidden elements, and layout-only markup.

2. **Semantic fidelity.** SOM MUST preserve all interactive elements (links,
   buttons, form controls) and content structure (headings, paragraphs, lists,
   tables) that an agent needs to understand and act on a page.

3. **Stable identifiers.** Every element and region in a SOM document MUST have
   a deterministic, stable ID derived from the element's properties and position,
   enabling agents to reference elements across snapshots and mutations.

4. **Machine readability.** SOM output MUST be valid JSON conforming to the
   schema defined in `som-schema.json`.

5. **Structured data preservation.** SOM SHOULD extract and surface structured
   data embedded in the page (JSON-LD, OpenGraph, Twitter Cards, HTML meta tags).

## 2. SOM Document Structure

A SOM document is a JSON object with the following top-level fields:

| Field             | Type                | Required | Description |
|-------------------|---------------------|----------|-------------|
| `som_version`     | string              | REQUIRED | SOM specification version (e.g., `"0.1"`). |
| `url`             | string              | REQUIRED | The URL of the page. |
| `title`           | string              | REQUIRED | The page title extracted from `<title>`. Empty string if absent. |
| `lang`            | string              | REQUIRED | The page language from `<html lang>`. Defaults to `"en"`. |
| `regions`         | array of Region     | REQUIRED | Semantic regions extracted from the page body. |
| `meta`            | SomMeta             | REQUIRED | Compilation metadata. |
| `structured_data` | StructuredData      | OPTIONAL | Structured data extracted from the page head. Omitted if empty. |

### 2.1 SomMeta

| Field              | Type    | Required | Description |
|--------------------|---------|----------|-------------|
| `html_bytes`       | integer | REQUIRED | Size of the source HTML in bytes. |
| `som_bytes`        | integer | REQUIRED | Size of the serialized SOM JSON in bytes. |
| `element_count`    | integer | REQUIRED | Total number of elements across all regions. |
| `interactive_count`| integer | REQUIRED | Number of interactive elements (links, buttons, form controls). |

### 2.2 StructuredData

The `structured_data` object surfaces metadata extracted from the page `<head>`.
It is OPTIONAL and MUST be omitted when all sub-fields are empty.

| Field         | Type                        | Description |
|---------------|-----------------------------|-------------|
| `json_ld`     | array of object             | JSON-LD blocks (`<script type="application/ld+json">`). |
| `open_graph`  | object (string -> string)   | OpenGraph metadata (`og:*` properties). |
| `twitter_card`| object (string -> string)   | Twitter/X Card metadata (`twitter:*` names). |
| `meta`        | object (string -> string)   | Standard HTML meta tags: `description`, `author`, `keywords`, `robots`, `viewport`, `generator`, `theme-color`, `charset`. |
| `links`       | array of LinkElement        | Semantically meaningful `<link>` elements. |

Each sub-field MUST be omitted (not present in the JSON) when it is empty.

#### 2.2.1 LinkElement

| Field      | Type   | Required | Description |
|------------|--------|----------|-------------|
| `rel`      | string | REQUIRED | The `rel` attribute value (lowercased). |
| `href`     | string | REQUIRED | The `href` attribute value. |
| `type`     | string | OPTIONAL | The `type` attribute value. |
| `hreflang` | string | OPTIONAL | The `hreflang` attribute value. |

Implementations MUST only include link elements with the following `rel` values:
`canonical`, `icon`, `shortcut icon`, `apple-touch-icon`, `manifest`,
`alternate`, `amphtml`, `preconnect`, `dns-prefetch`, `author`, `license`,
`search`. Stylesheet links MUST be excluded.

## 3. Regions

A **Region** represents a semantic section of the page. Regions are the primary
organizational unit of a SOM document.

### 3.1 Region Object

| Field      | Type             | Required | Description |
|------------|------------------|----------|-------------|
| `id`       | string           | REQUIRED | Stable region identifier (see Section 7.2). |
| `role`     | RegionRole       | REQUIRED | The semantic role of this region. |
| `label`    | string           | OPTIONAL | Accessible label for the region (from `aria-label`, `title`). |
| `action`   | string           | OPTIONAL | Form action URL (only for `form` regions). |
| `method`   | string           | OPTIONAL | Form method, uppercased (only for `form` regions, e.g., `"GET"`, `"POST"`). |
| `elements` | array of Element | REQUIRED | The semantic elements within this region. |

Empty regions (no elements after compilation) MUST be excluded from the output.

### 3.2 RegionRole

The `role` field MUST be one of the following string values (serialized lowercase):

| Role          | Detection Source | Description |
|---------------|-----------------|-------------|
| `navigation`  | `<nav>`, `role="navigation"`, class/id heuristics, link-density heuristic | Site or page navigation. |
| `main`        | `<main>`, `role="main"`, class/id heuristics | Primary content area. |
| `aside`       | `<aside>`, `role="complementary"`, class/id heuristics | Sidebar or supplementary content. |
| `header`      | `<header>`, `role="banner"`, class/id heuristics | Page or site header. |
| `footer`      | `<footer>`, `role="contentinfo"`, class/id heuristics, content heuristics | Page or site footer. |
| `form`        | `<form>` | A form with interactive controls. |
| `dialog`      | `<dialog>`, `role="dialog"`, `role="alertdialog"` | A dialog or modal overlay. |
| `content`     | Fallback | Catch-all for elements not belonging to any landmark region. |

### 3.3 Region Detection Rules

Implementations MUST detect regions using the following precedence:

1. **ARIA roles** (highest priority): The `role` attribute on any element. ARIA
   role `"complementary"` maps to `aside`, `"banner"` maps to `header`,
   `"contentinfo"` maps to `footer`, `"dialog"` and `"alertdialog"` map to
   `dialog`. The `"search"` ARIA role maps to `navigation`.

2. **HTML5 landmark elements**: `<nav>`, `<main>`, `<aside>`, `<header>`,
   `<footer>`, `<dialog>`.

3. **Form elements**: `<form>` tags are promoted to `form` regions. The `action`
   and `method` attributes are captured. The label is resolved from
   `aria-label`, `title`, or the `name`/`id` attribute.

4. **Class/ID heuristics**: For `<div>`, `<section>`, `<article>`, `<ul>`, and
   `<ol>` elements, the class and id attributes are checked against patterns:
   - **Navigation**: contains `nav`, `menu`, or `navigation`
   - **Main**: contains `main-content`, `maincontent`, `primary-content`,
     `article-body`, or `main` (without `nav`)
   - **Aside**: contains `sidebar`, `side-bar`, `aside`, or `rail`
   - **Footer**: contains `footer`, `copyright`, `site-footer`, or `page-footer`
   - **Header**: contains `header`, `site-header`, `page-header`, or `masthead`

5. **Link-density heuristic**: A `<div>`, `<ul>`, or `<ol>` is promoted to
   `navigation` if it contains >= 3 links AND more than 50% of its direct
   children are links, AND it does not contain descendant `<main>` or
   `<article>` elements (checked to depth 6).

6. **Footer content heuristic**: A `<div>` or `<section>` is promoted to
   `footer` if its text content contains "copyright", "©", "privacy",
   "terms of", or "all rights reserved" (case-insensitive), AND it does not
   contain descendant `<main>` or `<article>` elements.

7. **Fallback `content` region**: Any elements not assigned to a landmark region
   are collected into a `content` region. If no landmarks are detected at all,
   a single `content` region wraps all body elements.

### 3.4 Region ID Generation

Region IDs follow the pattern `r_{role}` for the first region of each role, and
`r_{role}_{index}` for subsequent regions of the same role, where `index` starts
at 1 for the second occurrence.

Examples: `r_navigation`, `r_main`, `r_footer`, `r_navigation_1`, `r_form_2`.

## 4. Elements

An **Element** represents a single semantic unit within a region.

### 4.1 Element Object

| Field      | Type             | Required | Description |
|------------|------------------|----------|-------------|
| `id`       | string           | REQUIRED | Stable element identifier (see Section 7.1). |
| `role`     | ElementRole      | REQUIRED | The semantic role of this element. |
| `html_id`  | string           | OPTIONAL | The original HTML `id` attribute from the source element, when present and non-empty. Enables agents to resolve back to the live DOM for interaction (e.g. via `document.getElementById()` or CSS selector `#id`). |
| `text`     | string           | OPTIONAL | Visible text content of the element. |
| `label`    | string           | OPTIONAL | Accessible label (from `aria-label`, `title`, or `placeholder`), included only when different from `text`. |
| `actions`  | array of string  | OPTIONAL | Available actions for this element. Omitted for non-interactive elements. |
| `attrs`    | object           | OPTIONAL | Role-specific attributes (see Section 4.3). |
| `children` | array of Element | OPTIONAL | Child elements (reserved for future use). |
| `hints`    | array of string  | OPTIONAL | Semantic hints inferred from CSS classes (see Section 5). |

### 4.2 ElementRole

The `role` field MUST be one of the following string values (serialized in
`snake_case`):

| Role          | Interactive | Default Actions        | HTML Sources |
|---------------|-------------|------------------------|--------------|
| `link`        | Yes         | `["click"]`            | `<a href>`, `role="link"` |
| `button`      | Yes         | `["click"]`            | `<button>`, `<input type="submit\|button\|reset">`, `role="button"` |
| `text_input`  | Yes         | `["type", "clear"]`    | `<input type="text\|email\|password\|search\|tel\|url\|number\|date\|time\|datetime-local\|month\|week\|color">` |
| `textarea`    | Yes         | `["type", "clear"]`    | `<textarea>` |
| `select`      | Yes         | `["select"]`           | `<select>` |
| `checkbox`    | Yes         | `["toggle"]`           | `<input type="checkbox">`, `role="checkbox"` |
| `radio`       | Yes         | `["select"]`           | `<input type="radio">`, `role="radio"` |
| `heading`     | No          | (none)                 | `<h1>` through `<h6>` |
| `image`       | No          | (none)                 | `<img>`, `<picture>`, `role="img"` |
| `list`        | No          | (none)                 | `<ul>`, `<ol>` |
| `table`       | No          | (none)                 | `<table>` (data tables only) |
| `paragraph`   | No          | (none)                 | `<p>`, bare text nodes |
| `section`     | No          | (none)                 | `<section>`, `<article>` |
| `separator`   | No          | (none)                 | `<hr>` |
| `details`     | Yes         | `["toggle"]`           | `<details>` with `<summary>` (disclosure widget) |

For interactive elements, the `actions` array MUST be present and contain the
default actions listed above. For non-interactive elements, the `actions` field
MUST be omitted.

### 4.3 Role-Specific Attributes

The `attrs` object contains properties specific to each element role. The `attrs`
field MUST be omitted when the object would be empty.

#### `link`

| Attribute | Type   | Description |
|-----------|--------|-------------|
| `href`    | string | The link destination URL. |

#### `button`

| Attribute  | Type    | Description |
|------------|---------|-------------|
| `disabled` | boolean | Present and `true` when the button is disabled. |

#### `text_input`

| Attribute    | Type    | Description |
|--------------|---------|-------------|
| `input_type` | string  | The input type (e.g., `"email"`, `"password"`). Only included for specific types: `text`, `email`, `password`, `search`, `tel`, `url`, `number`. |
| `value`      | string  | Current input value. |
| `placeholder`| string  | Placeholder text. |
| `required`   | boolean | Present and `true` when the field is required. |
| `disabled`   | boolean | Present and `true` when the field is disabled. |
| `checked`    | boolean | For checkbox/radio: present and `true` when checked. |
| `group`      | string  | For radio inputs: the `name` attribute identifying the radio group. |

#### `textarea`

| Attribute    | Type    | Description |
|--------------|---------|-------------|
| `placeholder`| string  | Placeholder text. |
| `required`   | boolean | Present and `true` when required. |

#### `select`

| Attribute  | Type             | Description |
|------------|------------------|-------------|
| `options`  | array of Option  | The available options. |
| `multiple` | boolean          | Present and `true` when multiple selection is allowed. |
| `required` | boolean          | Present and `true` when required. |

Each **Option** object:

| Field      | Type    | Required | Description |
|------------|---------|----------|-------------|
| `value`    | string  | REQUIRED | The option value. |
| `text`     | string  | REQUIRED | The option display text. |
| `selected` | boolean | OPTIONAL | Present and `true` when this option is selected. |

#### `heading`

| Attribute | Type    | Description |
|-----------|---------|-------------|
| `level`   | integer | Heading level (1-6). |

#### `image`

| Attribute | Type   | Description |
|-----------|--------|-------------|
| `alt`     | string | Alternative text. |
| `src`     | string | Image source URL. |

#### `list`

| Attribute | Type             | Description |
|-----------|------------------|-------------|
| `ordered` | boolean          | `true` for `<ol>`, `false` for `<ul>`. |
| `items`   | array of object  | List items, each with a `text` field. Truncated to `max_list_items` (default 5) with a summary item appended. |

#### `table`

| Attribute | Type                    | Description |
|-----------|-------------------------|-------------|
| `headers` | array of string         | Column header texts (from `<th>` elements). |
| `rows`    | array of array of string| Row data (from `<td>` elements). Limited to 30 rows, 12 columns. |
| `caption` | string                  | Table caption from `<caption>` element, when present. |

#### `section`

| Attribute      | Type   | Description |
|----------------|--------|-------------|
| `section_label`| string | The `aria-label` of the section, if present. |

#### `details`

| Attribute | Type    | Description |
|-----------|---------|-------------|
| `open`    | boolean | `true` when the `<details>` element has the `open` attribute (expanded state). |
| `summary` | string  | The visible text from the `<summary>` child element. |

### 4.4 ARIA State Preservation

Implementations SHOULD capture common ARIA state attributes on any element
and surface them in an `aria` sub-object within `attrs`. The following ARIA
attributes SHOULD be preserved when present:

| HTML Attribute   | `aria` Key  | Type            |
|------------------|-------------|-----------------|
| `aria-expanded`  | `expanded`  | boolean         |
| `aria-selected`  | `selected`  | boolean         |
| `aria-checked`   | `checked`   | boolean/string  |
| `aria-disabled`  | `disabled`  | boolean         |
| `aria-current`   | `current`   | boolean/string  |
| `aria-pressed`   | `pressed`   | boolean         |
| `aria-hidden`    | `hidden`    | boolean         |

Values `"true"` and `"false"` MUST be normalized to boolean. Other string
values (e.g. `aria-current="page"`, `aria-checked="mixed"`) MUST be preserved
as strings.

### 4.5 Element Detection from HTML

Implementations MUST map HTML elements to SOM element roles as follows:

1. **ARIA `role` attribute** takes precedence: `role="button"` -> `button`,
   `role="link"` -> `link`, `role="checkbox"` -> `checkbox`,
   `role="radio"` -> `radio`, `role="img"` -> `image`.

2. **`<a>` elements** MUST only become `link` elements if they have an `href`
   attribute. Anchor elements without `href` are ignored.

3. **`<input>` elements** are mapped by their `type` attribute:
   - `submit`, `button`, `reset` -> `button`
   - `checkbox` -> `checkbox`
   - `radio` -> `radio`
   - `hidden` -> excluded entirely
   - All other types -> `text_input`

4. **`<table>` elements** MUST be checked for layout usage before being treated
   as data tables (see Section 6).

### 4.5 Label Resolution

The `label` field provides an accessible name for the element. Implementations
MUST resolve labels in this priority order:

1. `aria-label` attribute
2. `title` attribute
3. `placeholder` attribute (for form controls)

The label MUST be omitted if it is identical to the `text` content of the
element (to avoid redundancy). All label values MUST be normalized: whitespace
collapsed to single spaces and trimmed.

### 4.6 Text Normalization

All text content MUST be normalized:

1. Collapse all whitespace (spaces, tabs, newlines) to single spaces.
2. Trim leading and trailing whitespace.
3. Truncate to 200 characters maximum using smart truncation:
   - Prefer breaking at sentence boundaries (`.`, `!`, `?`) if the boundary
     captures at least 40% of the budget.
   - Fall back to word boundaries with `...` appended.

## 5. Semantic Hints

The `hints` array provides semantic signals inferred from CSS class names.
These help agents understand element importance and state without seeing raw CSS.

Implementations SHOULD infer the following hint categories from class names:

### Importance / Variant
| Hint        | Triggers (class contains) |
|-------------|--------------------------|
| `primary`   | `primary`, `cta` |
| `secondary` | `secondary` |
| `danger`    | `danger`, `destructive`, `delete` |
| `warning`   | `warning`, `caution` |
| `success`   | `success` |
| `error`     | `error`, `invalid` |

### State
| Hint        | Triggers (class contains) |
|-------------|--------------------------|
| `disabled`  | `disabled`, `is-disabled` |
| `active`    | `active`, `is-active`, `current` |
| `selected`  | `selected`, `is-selected`, `checked` |
| `hidden`    | `hidden`, `sr-only`, `visually-hidden` |
| `loading`   | `loading`, `spinner`, `skeleton` |
| `collapsed` | `collapsed`, `is-closed` |
| `expanded`  | `expanded`, `is-open`, `show` |

### Size
| Hint    | Triggers (class contains) |
|---------|--------------------------|
| `large` | `lg`, `large`, `xl` |
| `small` | `sm`, `small`, `xs`, `mini` |

### Layout / Grouping
| Hint           | Triggers (class contains) |
|----------------|--------------------------|
| `card`         | `card` (not `discard`) |
| `hero`         | `hero`, `jumbotron`, `banner` |
| `modal`        | `modal`, `dialog`, `popup`, `overlay` |
| `notification` | `toast`, `snackbar`, `notification`, `alert` |
| `badge`        | `badge`, `chip`, `tag`, `pill` |
| `sticky`       | `sticky`, `fixed`, `pinned` |
| `required`     | `required`, `mandatory` |

Hints MUST be sorted alphabetically and deduplicated. The `hints` field MUST be
omitted when no hints are detected.

## 6. Stripping and Filtering Rules

### 6.1 Elements That MUST Be Stripped

The following elements MUST be completely removed from SOM output:

- `<script>`, `<style>`, `<noscript>`, `<template>`, `<meta>`, `<link>`
- HTML comments
- Elements with the `hidden` attribute
- Elements with `aria-hidden="true"`
- Elements with inline `style` containing `display:none` or `visibility:hidden`
- Decorative images: `<img>` with empty `alt=""` or `role="presentation"`
- `<svg>` elements, UNLESS they have both `role="img"` AND an accessible name
  (`aria-label` or `title`)

### 6.2 CSS Visibility Filtering

Implementations SHOULD parse `<style>` blocks to identify CSS rules that hide
elements. The following CSS patterns indicate hidden elements:

- `display: none`
- `visibility: hidden`
- `opacity: 0`
- `clip: rect(0...)` combined with `position: absolute`
- `height: 0` or `width: 0` or `max-height: 0` combined with `overflow: hidden`
- `font-size: 0`

CSS classes, IDs, and tag selectors matching these patterns SHOULD be tracked.
Elements matching hidden selectors MUST be excluded, UNLESS a corresponding
visibility override (`display: block/flex/grid/inline`, `visibility: visible`)
exists for the same class.

### 6.3 Layout Table Decomposition

Tables used for layout purposes MUST NOT be emitted as `table` elements.
Instead, their contents MUST be extracted and processed as if the table
structure were not present.

A table is considered a **layout table** if ANY of the following conditions
are true:

1. It contains no `<th>` elements AND no `<caption>` element, AND any of:
   - Contains nested `<table>` elements
   - Contains layout children (`<nav>`, `<form>`, `<header>`, `<footer>`,
     `<aside>`, `<main>`, `<article>`, `<section>`, or `<div>` with >= 2
     descendant links)
   - Has layout-specific HTML attributes: `cellpadding`, `cellspacing`,
     `border`, `width`, `bgcolor`
   - Contains >= 20 descendant links
   - Has <= 2 `<td>` cells total

Tables with `<th>` or `<caption>` elements are treated as data tables regardless
of other signals.

### 6.4 Collapsible Wrappers

Wrapper `<div>` and `<span>` elements MUST be collapsed (their children
promoted up) when they contain:

- Exactly one element child, OR
- Only text content (no element children)

This prevents deep nesting of semantically meaningless containers.

## 7. Stable ID Generation

### 7.1 Element IDs

Element IDs MUST be generated deterministically using the following algorithm:

```
element_id = "e_" + hex(sha256(origin + "|" + role + "|" + normalized_name + "|" + dom_path))[0..12]
```

Where:
- `origin`: The origin (scheme + host + port) of the page URL.
- `role`: The element role string (e.g., `"button"`, `"link"`).
- `normalized_name`: The accessible name (label or text), lowercased, trimmed,
  truncated to 100 characters.
- `dom_path`: A `/`-separated path of child indices from the body root
  (e.g., `"0/3/1/0"`).

The resulting ID is `"e_"` followed by the first 12 hexadecimal characters of
the SHA-256 hash, giving 14 characters total (e.g., `"e_a1b2c3d4e5f6"`).

### 7.2 Region IDs

Region IDs follow a simpler pattern:

- First region of a role: `r_{role}` (e.g., `r_navigation`, `r_main`)
- Subsequent regions of the same role: `r_{role}_{index}` where index starts
  at 1 for the second occurrence (e.g., `r_navigation_1`, `r_form_2`)

### 7.3 Collision Handling

Implementations MUST track element IDs and handle collisions by appending a
counter suffix. If an ID `e_abc123def456` is generated twice, the second
occurrence becomes `e_abc123def456_2`, the third becomes `e_abc123def456_3`,
and so on.

## 8. Content Summarization

SOM implementations MUST enforce per-region budgets to maintain token efficiency.

### 8.1 Default Budget Configuration

| Parameter                | Default | Description |
|--------------------------|---------|-------------|
| `first_para_max`         | 200     | Max characters for the first paragraph in main content. |
| `subsequent_para_max`    | 80      | Max characters for subsequent paragraphs in main content. |
| `max_paragraphs`         | 10      | Max paragraph elements per region. |
| `max_list_items`         | 5       | Max list items before collapsing with summary. |
| `max_links`              | 200     | Max link elements per non-navigation region. |
| `max_navigation_links`   | 80      | Max link elements per navigation region. |
| `max_elements`           | 400     | Max total elements per region. |
| `max_table_cell_chars`   | 80      | Max characters per table cell. |
| `max_table_rows`         | 20      | Max data rows per table. |
| `max_table_columns`      | 8       | Max columns per table. |

### 8.2 Summarization Rules

1. **Link deduplication**: Links with the same normalized `href` (fragment
   stripped, trailing slash stripped, lowercased) MUST be deduplicated. Only the
   first occurrence is kept.

2. **Paragraph truncation in main regions**: In `main` regions, the first
   paragraph is truncated to `first_para_max` characters. Subsequent paragraphs
   are truncated to `subsequent_para_max` characters.

3. **Budget enforcement priority**: When the element budget is reached,
   implementations MUST continue to include:
   - All form controls (text_input, textarea, select, checkbox, radio, button)
   - All headings (for structural navigation)

4. **Summary element**: When elements are dropped, a summary `paragraph` element
   MUST be appended describing what was omitted:
   ```
   [N duplicate links, N more links, N more paragraphs, N more elements dropped, ~N chars]
   ```

5. **List truncation**: Lists exceeding `max_list_items` MUST include a summary
   item: `"[N more items]"`.

6. **Table data**: Tables MUST NOT include `text` content (to avoid duplicating
   cell data). Table data is in `attrs.headers` and `attrs.rows`. Similarly,
   lists MUST NOT include `text` content.

## 9. SOM Mutations (JSON Patch)

SOM mutations describe changes between two SOM snapshots. Implementations
SHOULD support expressing mutations as [RFC 6902 JSON Patch](https://www.rfc-editor.org/rfc/rfc6902) operations.

A SOM mutation document is a JSON array of patch operations:

```json
[
  { "op": "replace", "path": "/regions/0/elements/2/text", "value": "New text" },
  { "op": "add", "path": "/regions/1/elements/-", "value": { "id": "e_abc123", "role": "button", "text": "Submit" } },
  { "op": "remove", "path": "/regions/0/elements/5" }
]
```

Supported operations:

| Operation | Description |
|-----------|-------------|
| `add`     | Add a new element or region. |
| `remove`  | Remove an element or region. |
| `replace` | Replace a value (text, label, attrs, etc.). |

Paths use JSON Pointer syntax (RFC 6901) relative to the SOM document root.

## 10. Token Estimation Methodology

SOM provides byte-level compression metrics via `meta.html_bytes` and
`meta.som_bytes`. To estimate token usage for LLM consumption:

1. **Compression ratio**: `som_bytes / html_bytes` gives the byte-level
   compression ratio. Typical SOM output achieves 5-20x compression over raw
   HTML.

2. **Token estimation**: For JSON content consumed by LLMs, a reasonable
   approximation is ~4 characters per token. Thus:
   `estimated_tokens ≈ som_bytes / 4`

3. **Interactive density**: `meta.interactive_count / meta.element_count` gives
   the ratio of actionable elements, which correlates with agent task complexity.

## 11. Versioning

The `som_version` field in the SOM document identifies which version of this
specification the output conforms to.

- The current version is `"0.1"`.
- Implementations MUST include the `som_version` field.
- Consumers SHOULD check `som_version` before processing and SHOULD reject
  documents with unrecognized versions.
- Minor version increments (e.g., `0.1` -> `0.2`) MAY add new optional fields
  but MUST NOT remove or change the semantics of existing fields.
- Major version increments indicate breaking changes.

## 12. Conformance Requirements

### 12.1 Producers

A conformant SOM producer (compiler):

1. MUST output valid JSON conforming to `som-schema.json`.
2. MUST generate deterministic, stable element and region IDs per Section 7.
3. MUST strip all elements listed in Section 6.1.
4. MUST detect regions using the precedence rules in Section 3.3.
5. MUST map HTML elements to SOM roles per Section 4.4.
6. MUST enforce content summarization budgets per Section 8.
7. MUST exclude empty regions.
8. MUST include the `som_version` field.
9. MUST populate the `meta` object with accurate compilation statistics.
10. SHOULD parse CSS for visibility filtering per Section 6.2.
11. SHOULD detect layout tables per Section 6.3.
12. SHOULD extract structured data per Section 2.2.
13. SHOULD infer semantic hints from CSS classes per Section 5.
14. MAY support custom content budget configuration.

### 12.2 Consumers

A conformant SOM consumer (agent, SDK, tool):

1. MUST be able to parse any valid SOM document.
2. MUST use element `id` fields to reference elements for actions.
3. SHOULD check `som_version` before processing.
4. SHOULD handle unknown element roles and region roles gracefully (by ignoring
   them rather than failing).
5. MAY use the `hints` array to inform decision-making.
6. MAY use `structured_data` to enrich understanding of the page.

## Appendix A: Complete Example

```json
{
  "som_version": "0.1",
  "url": "https://example.com/products/widget",
  "title": "Widget - Example Store",
  "lang": "en",
  "regions": [
    {
      "id": "r_navigation",
      "role": "navigation",
      "label": "Main menu",
      "elements": [
        {
          "id": "e_a1b2c3d4e5f6",
          "role": "link",
          "text": "Home",
          "actions": ["click"],
          "attrs": { "href": "/" }
        },
        {
          "id": "e_b2c3d4e5f6a7",
          "role": "link",
          "text": "Products",
          "actions": ["click"],
          "attrs": { "href": "/products" },
          "hints": ["active"]
        }
      ]
    },
    {
      "id": "r_main",
      "role": "main",
      "elements": [
        {
          "id": "e_c3d4e5f6a7b8",
          "role": "heading",
          "text": "Widget",
          "attrs": { "level": 1 }
        },
        {
          "id": "e_d4e5f6a7b8c9",
          "role": "image",
          "attrs": { "alt": "Widget product photo", "src": "/images/widget.jpg" }
        },
        {
          "id": "e_e5f6a7b8c9d0",
          "role": "paragraph",
          "text": "The Widget is our most popular product. Built with premium materials."
        },
        {
          "id": "e_f6a7b8c9d0e1",
          "role": "button",
          "text": "Add to Cart",
          "actions": ["click"],
          "hints": ["primary"]
        }
      ]
    },
    {
      "id": "r_footer",
      "role": "footer",
      "elements": [
        {
          "id": "e_a7b8c9d0e1f2",
          "role": "paragraph",
          "text": "© 2026 Example Store. All rights reserved."
        }
      ]
    }
  ],
  "meta": {
    "html_bytes": 15234,
    "som_bytes": 1847,
    "element_count": 7,
    "interactive_count": 3
  },
  "structured_data": {
    "json_ld": [
      {
        "@context": "https://schema.org",
        "@type": "Product",
        "name": "Widget",
        "offers": {
          "@type": "Offer",
          "price": "29.99",
          "priceCurrency": "USD"
        }
      }
    ],
    "open_graph": {
      "og:title": "Widget - Example Store",
      "og:type": "product"
    },
    "meta": {
      "description": "Buy the Widget - our most popular product"
    }
  }
}
```

## Appendix B: References

- [RFC 2119 - Key words for use in RFCs](https://www.ietf.org/rfc/rfc2119.txt)
- [RFC 6901 - JavaScript Object Notation (JSON) Pointer](https://www.rfc-editor.org/rfc/rfc6901)
- [RFC 6902 - JavaScript Object Notation (JSON) Patch](https://www.rfc-editor.org/rfc/rfc6902)
- [WAI-ARIA 1.2 - Landmark Roles](https://www.w3.org/TR/wai-aria-1.2/#landmark_roles)
- [HTML5 - Sections](https://html.spec.whatwg.org/multipage/sections.html)
