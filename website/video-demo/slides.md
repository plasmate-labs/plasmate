# Plasmate Video Slides

Use these slides as visual breaks or overlays during the video.

---

## Slide 1: The Problem

**Title:** Your AI is drowning in HTML noise

**Stats:**
- Average webpage: **2-5 MB** of HTML
- That's **500,000+ tokens** per page
- At GPT-4 pricing: **$0.50-1.00** just to read ONE page
- 90% of tokens wasted on styling, scripts, and tracking

**Visual:** Wall of HTML tags fading to gray noise

---

## Slide 2: The Solution

**Title:** Plasmate: Semantic Object Model

**What it does:**
- Extracts only the **semantic content**
- Structures it as **clean JSON**
- Preserves navigation, headings, links, forms
- Strips CSS, scripts, tracking, decorative markup

**Visual:** HTML chaos transforming into organized JSON tree

---

## Slide 3: Real Results

**Title:** 105x compression on Linear.app

| Metric | Raw HTML | Plasmate SOM |
|--------|----------|--------------|
| Size | 2.2 MB | 21 KB |
| Tokens | ~500K | ~5K |
| Cost | $0.75 | $0.007 |
| Context used | 100% | 1% |

**More benchmarks:**
- NYT homepage: **87x** compression
- Wikipedia article: **42x** compression
- HackerNews: **31x** compression

**Visual:** Bar chart showing compression ratios

---

## Slide 4: Get Started

**Title:** Ready in 30 seconds

**Install:**
```bash
# Python
pip install plasmate

# Rust
cargo install plasmate
```

**Use with AI tools:**
```bash
# MCP server for Claude, Cursor, etc.
plasmate mcp

# Direct fetch
plasmate fetch https://example.com
```

**Learn more:**
- **Website:** plasmate.app
- **GitHub:** github.com/anthropics/plasmate
- **Docs:** docs.plasmate.app

**Visual:** Terminal with commands, logo animation

---

## Design Notes

- **Colors:** Dark theme (#0a0a0a background)
- **Accent:** Green (#4ade80) for positive stats, Red (#f87171) for problems
- **Font:** Inter or SF Pro for slides, SF Mono for code
- **Animation:** Minimal, professional - no flashy transitions
- **Logo:** Plasmate wordmark + icon in bottom right corner
