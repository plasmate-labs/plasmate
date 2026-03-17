# Plasmate Benchmark Results (March 2026)

## CDP Parity Benchmark (Lightpanda campfire-commerce)

**Test:** Puppeteer script navigates to a local product page, waits for JS-rendered content (product price, reviews from XHR), extracts structured data.

**Result:** PASS - All assertions pass consistently

| Metric | Value |
|--------|-------|
| Runs | 10 |
| Total duration | 3,165ms |
| Avg run duration | 308ms |
| Min run duration | 275ms |
| Max run duration | 348ms |

**What this validates:**
- CDP WebSocket server compatibility with Puppeteer
- `page.goto()` with full JS pipeline execution
- `page.waitForFunction()` polling
- `page.evaluate()` for DOM queries
- XHR/fetch requests from V8 (blocking bridge)
- DOM serialization with JS-mutated content

---

## Major Sites Speed Test

Engine: plasmate v0.1.0
URLs tested: 38
Successful: 36 (95%)

### Summary

| Metric | Mean | Median | P95 |
|--------|------|--------|-----|
| HTML bytes | 379,656 | 193,796 | 1,214,266 |
| SOM bytes | 22,759 | 22,034 | 56,625 |
| **Compression ratio** | **45.4x** | **10.2x** | 105.1x |
| HTML tokens (est) | 94,914 | 48,449 | 303,566 |
| SOM tokens (est) | 5,689 | 5,508 | 14,156 |
| Fetch time (ms) | 256 | 172 | 629 |
| Parse+SOM time (ms) | 20 | 16 | 61 |

### Grade Distribution

| Grade | Count | Criteria |
|-------|-------|----------|
| A | 12 | >15x compression |
| B | 8 | 8-15x compression |
| C | 9 | 3-8x compression |
| D | 4 | 1-3x compression |
| F | 3 | <1x or error |

### Highlights

| Site | HTML | SOM | Compression |
|------|------|-----|-------------|
| accounts.google.com | 1.2MB | 1.4KB | **864x** |
| x.com (Twitter) | 239KB | 1.5KB | **159x** |
| linear.app | 2.2MB | 21KB | **105x** |
| bing.com | 157KB | 1.7KB | **93x** |
| google.com | 194KB | 2.6KB | **74x** |
| vercel.com | 941KB | 22KB | **43x** |
| steampowered.com | 842KB | 23KB | **36x** |
| duckduckgo.com | 391KB | 12KB | **33x** |
| ebay.com | 831KB | 33KB | **25x** |
| Wikipedia (USA) | 1.7MB | 70KB | **25x** |

### Performance Notes

- **Parse+SOM time**: Median 16ms for full HTML->SOM compilation
- **Heavy JS sites**: Excellent compression (login pages, SPAs) due to stripping non-essential markup
- **Content-rich sites**: Good compression (Wikipedia 10-25x, news sites 11-15x)
- **Text-minimal sites**: Lower ratios expected (text.npr.org, example.com)

### Test Environment

- macOS, Apple Silicon
- Local network fetch (varies by site latency)
- No caching between runs
