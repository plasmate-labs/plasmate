# SOM Conformance Test Suite

This directory contains conformance test cases for the SOM Specification v1.0.

## Structure

Each test case consists of a pair of files:

- `NNN-name.html` — Input HTML document
- `NNN-name.expected.json` — Expected SOM output (partial)

## How to Use

The expected JSON files are **partial match** documents, not exact output. A
conformant implementation should produce SOM output where:

1. Each region in the expected output exists in the actual output with matching
   `id` and `role`.
2. Each element in the expected output exists in the corresponding region with
   matching `role` and (where specified) `text`, `label`, `actions`, `attrs`,
   and `hints`.
3. The `meta.interactive_count` matches the expected value.
4. Fields not present in the expected output (like exact element `id` values)
   are not checked — they are implementation-dependent due to the hash-based ID
   algorithm.

The `$description` field in each expected JSON explains what the test case
validates.

## Test Cases

| # | Name | Tests |
|---|------|-------|
| 001 | static-page | Basic landmarks (header, main, footer), headings, paragraphs, separator |
| 002 | navigation | `<nav>` with aria-label, link elements with href and actions |
| 003 | login-form | Form region with action/method, text inputs, checkbox, button, hints |
| 004 | data-table | Data table with headers and rows |
| 005 | product-page | Structured data (JSON-LD, OG, meta), select options, semantic hints |
| 006 | article | Article layout, nested headings, lists, images, aside |
| 007 | hidden-elements | Stripping hidden/aria-hidden/display:none/CSS-hidden/decorative/scripts |
| 008 | search-form | Search input type, GET form method, ordered list |
| 009 | aria-roles | ARIA role-based region detection and element roles |
| 010 | dialog | Dialog region, danger/secondary hints |
| 011 | radio-checkbox | Radio groups, checked state, textarea, hidden input exclusion |
| 012 | class-heuristics | Class/ID-based region detection (masthead, sidebar, nav-menu, etc.) |
| 013 | no-landmarks | Fallback content region when no landmarks exist |

## Running Tests

To validate a SOM implementation against these test cases:

```bash
# Pseudocode for a test runner
for each test case (HTML, expected JSON):
    som_output = your_compiler.compile(html, "https://example.com/...")
    assert_partial_match(som_output, expected_json)
```

The test runner should:
1. Compile each HTML file to SOM using the URL from the expected JSON's `url` field.
2. Verify that the output structurally matches the expected JSON.
3. Allow extra elements/regions in the actual output (partial matching).
4. Validate the output against `../som-schema.json`.
