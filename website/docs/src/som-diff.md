# SOM Diff

Compare two SOM snapshots and get a structured report of what changed. Useful for competitive monitoring, deploy regression testing, content tracking, and compliance auditing.

## Quick Start

```bash
# Capture a baseline
plasmate fetch https://example.com/pricing > baseline.json

# ... time passes, the page changes ...

# Capture again
plasmate fetch https://example.com/pricing > current.json

# See what changed
plasmate diff baseline.json current.json
```

## Output Formats

### JSON (default)

Full structured diff, machine-readable. Pipe to `jq` or feed to another tool.

```bash
plasmate diff old.json new.json
```

```json
{
  "page": {
    "title_change": { "old": "Pricing - Acme", "new": "Pricing - Acme Corp" }
  },
  "regions": [...],
  "elements": [
    {
      "id": "e_a3f2b1",
      "change_type": "modified",
      "role": null,
      "text_change": { "old": "$49.99/mo", "new": "$59.99/mo" }
    },
    {
      "id": "e_b7c4d2",
      "change_type": "added",
      "role": "button",
      "text": "Start Free Trial"
    }
  ],
  "summary": {
    "total_changes": 3,
    "elements_added": 1,
    "elements_modified": 1,
    "has_price_changes": true
  }
}
```

### Human-readable text

Colored diff output with content previews.

```bash
plasmate diff old.json new.json --format text
```

```
Page changes:
  Title: "Pricing - Acme" → "Pricing - Acme Corp"

[~] Region r_main
  [~] Element e_a3f2b1
      Text: "$49.99/mo" → "$59.99/mo"
  [+] button e_b7c4d2 "Start Free Trial"

1 added, 0 removed, 1 modified, price change detected
```

### One-line summary

For scripts, cron output, or Slack alerts.

```bash
plasmate diff old.json new.json --format summary
```

```
1 added, 0 removed, 1 modified, price change detected
```

## Options

| Flag | Description |
|---|---|
| `--format json` | Full structured JSON (default) |
| `--format text` | Human-readable diff |
| `--format summary` | One-line change summary |
| `--ignore-meta` | Skip SomMeta changes (html_bytes, som_bytes, element counts) |
| `--output file` | Write to file instead of stdout |

## What It Detects

### Element-level changes

| Change | Detected | Example |
|---|---|---|
| Text modified | Yes | Price changed from $49.99 to $59.99 |
| Element added | Yes | New "Free Trial" button appeared |
| Element removed | Yes | "Sale" banner was removed |
| Role changed | Yes | Link became a button |
| Attributes changed | Yes | href updated, new data attributes |
| Actions changed | Yes | Element gained "type" action |
| Hints changed | Yes | "primary" hint added |
| Nested children | Yes | List items added/removed within a section |

### Region-level changes

| Change | Detected |
|---|---|
| Region added | Yes |
| Region removed | Yes |
| Region role changed | Yes |
| Region label changed | Yes |

### Page-level changes

| Change | Detected |
|---|---|
| Title changed | Yes |
| URL changed | Yes |
| Language changed | Yes |
| Element count delta | Yes |
| Interactive count delta | Yes |

### Semantic detection

| Signal | How |
|---|---|
| Price changes | Regex matching $, EUR, GBP, JPY currency patterns in modified text |
| Content changes | Text modifications in `main` regions |
| Structural changes | Added/removed regions or elements |

## Use Cases

### Competitive price monitoring

```bash
#!/bin/bash
# Run daily via cron
plasmate fetch https://competitor.com/pricing > /tmp/today.json

if [ -f /var/data/yesterday.json ]; then
  SUMMARY=$(plasmate diff /var/data/yesterday.json /tmp/today.json --format summary)
  if echo "$SUMMARY" | grep -q "price change"; then
    echo "ALERT: $SUMMARY" | mail -s "Competitor price change" team@company.com
  fi
fi

cp /tmp/today.json /var/data/yesterday.json
```

### Deploy regression testing

```bash
# Before deploy: capture baseline
plasmate fetch https://staging.example.com > pre-deploy.json

# Deploy happens...

# After deploy: compare
DIFF=$(plasmate diff pre-deploy.json <(plasmate fetch https://staging.example.com) --format summary)
if echo "$DIFF" | grep -q "removed"; then
  echo "WARNING: Content removed after deploy: $DIFF"
  exit 1
fi
```

### Terms of service tracking

```bash
# Weekly cron
plasmate fetch https://service.com/terms > /var/data/tos-$(date +%Y%m%d).json
PREV=$(ls -t /var/data/tos-*.json | sed -n '2p')
if [ -n "$PREV" ]; then
  plasmate diff "$PREV" /var/data/tos-$(date +%Y%m%d).json --format text
fi
```

## Algorithm

The diff uses O(n) HashMap lookups by stable element ID. Element IDs in SOM are SHA-256 derived from the element's origin, role, accessible name, and DOM path, so the same element on the same page always has the same ID across snapshots.

This means the diff correctly handles:
- Elements that moved between regions (tracked by ID, not position)
- Pages that reorder content (same IDs, different positions)
- Minor markup changes that do not affect semantic content

## Programmatic Usage (Rust)

```rust
use plasmate::som::diff::{diff_soms, render_text, render_summary};
use plasmate::som::types::Som;

let old: Som = serde_json::from_str(&old_json)?;
let new: Som = serde_json::from_str(&new_json)?;

let diff = diff_soms(&old, &new, false);

// Check for price changes
if diff.summary.has_price_changes {
    alert("Price change detected!");
}

// Get human-readable output
println!("{}", render_text(&diff));

// One-liner
println!("{}", render_summary(&diff.summary));
```

## jq Recipes

```bash
# Count changes by type
plasmate diff old.json new.json | jq '.summary'

# List only price-related element changes
plasmate diff old.json new.json | jq '[.elements[] | select(.text_change) | select(.text_change.old | test("[$€£¥]"))]'

# Get all added elements with their content
plasmate diff old.json new.json | jq '[.elements[] | select(.change_type == "added") | {id, role, text}]'

# Check if anything changed (exit code)
plasmate diff old.json new.json | jq -e '.summary.total_changes > 0' > /dev/null
```
