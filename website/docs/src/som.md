# Semantic Object Model (SOM) Reference

## Overview

The Semantic Object Model is Plasmate's core output format. It replaces the DOM as the interface between browser and agent.

Where the DOM is a faithful representation of every tag, attribute, and style in an HTML document, SOM is a compressed, budgeted, semantic representation designed for LLM consumption.

## Design Principles

1. **Token efficiency over fidelity.** SOM drops anything an agent does not need: inline styles, class names (except for hint inference), script tags, empty elements, decorative markup.

2. **Deterministic IDs.** Every element gets a stable ID derived from its semantic identity, not its position in a mutable tree. The same button on the same page always has the same ID.

3. **Budget enforcement.** SOM has hard limits on output size. Content is summarized, lists are collapsed, duplicate links are removed. The model does not grow unbounded.

4. **Interactive elements are sacred.** Links, buttons, inputs, selects, and textareas are never dropped by budget limits. An agent must always be able to act.

## Structure

```json
{
  "version": "0.1",
  "url": "https://example.com",
  "title": "Page Title",
  "meta": {
    "html_bytes": 589546,
    "som_bytes": 56625,
    "element_count": 325,
    "compression_ratio": 10.4
  },
  "structured_data": {
    "json_ld": [...],
    "opengraph": {...},
    "twitter_cards": {...},
    "meta_tags": [...]
  },
  "regions": [
    {
      "id": "r_navigation",
      "role": "Navigation",
      "elements": [...]
    },
    {
      "id": "r_main",
      "role": "Main",
      "elements": [...]
    }
  ]
}
```

## Regions

SOM organizes page content into semantic regions. Region detection uses HTML5 landmark elements and heuristic analysis of class names, IDs, and DOM structure.

| Role | Detection |
|------|-----------|
| **Navigation** | `<nav>`, class/ID containing "nav", "menu", "header-links" |
| **Main** | `<main>`, `<article>`, class/ID containing "content", "main", "post" |
| **Header** | `<header>`, class/ID containing "header", "masthead", "banner" |
| **Footer** | `<footer>`, class/ID containing "footer", "colophon" |
| **Aside** | `<aside>`, class/ID containing "sidebar", "aside", "widget" |
| **Form** | `<form>`, class/ID containing "form", "login", "signup" |
| **Dialog** | `<dialog>`, role="dialog", class containing "modal", "popup" |
| **Content** | Fallback for regions that do not match other roles |

### Region ID Format

Region IDs use the format `r_` + a descriptor derived from the region's role and position:

- `r_navigation` (primary nav)
- `r_main` (primary content)
- `r_footer` (page footer)
- `r_aside_0`, `r_aside_1` (multiple sidebars)

## Elements

Each element in a region has:

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Deterministic ID: `e_` + hex(sha256(origin\|role\|name\|dom_path))[0..12] |
| `role` | string | Semantic role: link, button, input, select, textarea, heading, paragraph, image, list, table |
| `name` | string | Accessible name from aria-label, label[for], placeholder, alt, or visible text |
| `text` | string | Summarized text content (budget-limited) |
| `href` | string | For links: the target URL |
| `type` | string | For inputs: the input type (text, email, password, etc.) |
| `value` | string | Current value for form elements |
| `hints` | array | CSS class-inferred hints: primary, secondary, danger, disabled, active, etc. |

### Element ID Stability

Element IDs are computed as:

```
sha256(normalized_origin + "|" + role + "|" + accessible_name + "|" + dom_path_indices)
```

The first 12 hex characters are used, prefixed with `e_`.

This means:
- The same link to "About" on example.com always gets the same ID
- IDs survive page refreshes and minor layout changes
- IDs change if the element's semantic identity changes (different text, different role)

## Content Budgets

SOM enforces output budgets to keep token costs predictable:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `first_para_max` | 200 chars | First paragraph of a content block gets more space |
| `subsequent_para_max` | 80 chars | Later paragraphs are aggressively summarized |
| `max_paragraphs` | 10 | Content blocks collapse after this many paragraphs |
| `max_list_items` | 5 | Lists show first 5 items, then collapse |
| `max_links` | 200 | Total links before excess are dropped |
| `max_navigation_links` | 80 | Navigation links before excess are dropped |
| `max_elements` | 400 | Total elements before budget trimming |
| `max_table_cell_chars` | 80 | Table cell content truncation |

### Budget Priority

When element limits are reached, SOM drops in this order:

1. Duplicate links (same href, already seen)
2. Navigation links beyond budget
3. Paragraph elements beyond budget
4. Non-interactive elements (decorative, redundant)

Interactive elements (links, buttons, inputs, selects, textareas) are **never** dropped.

## Smart Text Truncation

When text exceeds its budget, SOM prefers to break at sentence boundaries:

- Sentence boundary within 40%-100% of budget: break there
- No sentence boundary: break at word boundary
- Adds "..." suffix when truncated

## CSS Class Hint Inference

SOM extracts semantic hints from CSS class names. This gives agents context about element purpose without exposing raw class strings.

22 recognized hint categories:

`primary`, `secondary`, `danger`, `warning`, `success`, `error`, `disabled`, `active`, `selected`, `hidden`, `loading`, `collapsed`, `expanded`, `large`, `small`, `card`, `hero`, `modal`, `notification`, `badge`, `sticky`, `required`

Example: A button with class `btn btn-primary btn-lg disabled` produces hints `["primary", "large", "disabled"]`.

## Heading Hierarchy

Headings (`h1` through `h6`) are always preserved regardless of budget limits. They provide the structural outline of the page that agents use for navigation and context.

## Layout Table Detection

SOM distinguishes layout tables from data tables using heuristics:

- Tables with `role="presentation"` or `role="layout"` are layout tables
- Tables nested inside other tables are likely layout tables
- Tables with a high ratio of links to cells are likely layout tables (e.g., HN)
- Tables with proper `<thead>`, `<th>` elements are data tables

Layout tables are decomposed into their semantic child elements rather than preserved as table structures.

## Link Deduplication

SOM normalizes URLs and deduplicates links across the entire page. On Wikipedia, this removes ~839 duplicate links, significantly improving token efficiency.

Normalization: lowercase scheme and host, remove trailing slashes, remove default ports, sort query parameters.

## JS DOM Mutations

When JavaScript modifies the DOM (via `document.write()`, `appendChild`, etc.), Plasmate's V8 runtime captures these mutations and feeds them back into the SOM compiler. The final SOM reflects the page as it appears after script execution.

## Structured Data Extraction

SOM extracts structured metadata from:

- **JSON-LD** (`<script type="application/ld+json">`)
- **OpenGraph** (`<meta property="og:*">`)
- **Twitter Cards** (`<meta name="twitter:*">`)
- **Standard meta tags** (description, author, keywords)
- **Link elements** (canonical, alternate, stylesheet counts)

## CSS Visibility

SOM respects CSS visibility rules from inline `<style>` blocks:

- `display: none` - element excluded from SOM
- `visibility: hidden` - element excluded from SOM
- `opacity: 0` - element excluded from SOM

This prevents hidden elements (dropdowns, modals, off-screen content) from polluting the agent's view.

## Benchmarks

| Site | HTML | SOM | Compression |
|------|------|-----|-------------|
| Wikipedia | 589 KB | 56 KB | 10.4x |
| BBC News | ~180 KB | ~12 KB | 15x |
| GitHub | ~120 KB | ~12 KB | 10.1x |
| Hacker News | 34 KB | ~26 KB | 1.3x |
| x.com | ~15 KB | ~0.1 KB | 159x |

Median across 38 URLs: **9.4x compression**.

Note: Link-dense pages like HN compress less because links are interactive and cannot be dropped.
