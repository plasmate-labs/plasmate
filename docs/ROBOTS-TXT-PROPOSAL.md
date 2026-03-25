# Robots.txt for the Agentic Web: A Proposal

**Author:** David Hurley, Plasmate Labs
**Date:** March 25, 2026
**Status:** Draft Proposal
**Related:** [W3C Web Content Browser for AI Agents Community Group](https://www.w3.org/community/web-content-browser-ai/)

---

## The Problem

The robots.txt standard was designed in 1994 for a simple question: should this crawler index my page? Yes or no.

Thirty-two years later, we're using the same binary mechanism to manage a fundamentally different relationship. AI agents don't just index pages. They read, reason, extract, summarize, and act on web content. The current robots.txt conversation is stuck on a single axis: **block or allow**.

```
# The current state of robots.txt for AI
User-agent: GPTBot
Disallow: /

User-agent: ClaudeBot
Disallow: /

User-agent: PerplexityBot
Disallow: /
```

This is a blunt instrument for a nuanced problem. Website owners face an all-or-nothing choice:
- **Allow everything** and lose control over how content is consumed
- **Block everything** and become invisible to the fastest-growing discovery channel on the web

There's no way to say: "Yes, you can read my content, and here's a better way to do it."

## The Proposal: SOM Directives

We propose extending robots.txt with directives that let websites advertise **semantic representations** of their content - structured, token-efficient formats that AI agents can consume directly instead of parsing raw HTML.

### New Directives

```
# Standard robots.txt directives still apply
User-agent: *
Allow: /

# SOM (Semantic Object Model) directives
SOM-Endpoint: https://cache.example.com/v1/som
SOM-Format: SOM/1.0
SOM-Scope: full-page
SOM-Freshness: 3600
```

### Directive Reference

| Directive | Required | Description | Example |
|-----------|----------|-------------|---------|
| `SOM-Endpoint` | Yes | Base URL of the SOM service endpoint. Agents append `?url=` with the target page URL. | `https://cache.plasmate.app/v1/som` |
| `SOM-Format` | Yes | The format of the semantic representation. | `SOM/1.0`, `markdown`, `accessibility-tree` |
| `SOM-Scope` | No | What content the SOM covers. Default: `full-page`. | `full-page`, `main-content`, `article-body` |
| `SOM-Freshness` | No | Maximum age in seconds of an acceptable cached SOM. Default: `86400` (24h). | `3600` (1 hour) |
| `SOM-Token-Budget` | No | Suggested maximum token count for the SOM response. Helps agents estimate costs before fetching. | `2000` |
| `SOM-Auth` | No | Authentication method required to access the SOM endpoint. | `none`, `api-key`, `bearer` |

### How It Works

1. An AI agent wants to read `https://example.com/article/ai-future`
2. The agent first checks `https://example.com/robots.txt`
3. It finds the SOM directives indicating a semantic endpoint is available
4. Instead of fetching and parsing the full HTML (potentially 50,000+ tokens), the agent requests:
   ```
   GET https://cache.example.com/v1/som?url=https://example.com/article/ai-future
   ```
5. It receives a clean, structured SOM response (~3,000 tokens)
6. Both the website and the agent benefit: less bandwidth, fewer tokens, better extraction quality

### Full Example

```
# robots.txt for example.com

# Traditional crawlers
User-agent: Googlebot
Allow: /
Crawl-delay: 1

User-agent: Bingbot
Allow: /

# AI agent crawlers - allowed, with a better option
User-agent: GPTBot
Allow: /

User-agent: ClaudeBot
Allow: /

User-agent: PerplexityBot
Allow: /

# Semantic Object Model available
# Agents SHOULD prefer this over raw HTML when available
SOM-Endpoint: https://cache.example.com/v1/som
SOM-Format: SOM/1.0
SOM-Scope: main-content
SOM-Freshness: 3600
SOM-Token-Budget: 5000

# For pages behind authentication
SOM-Auth: none

Sitemap: https://example.com/sitemap.xml
```

## Why This Matters

### For Website Owners

**Control without blocking.** Instead of the binary choice between allowing or blocking AI agents, website owners can direct agents to a representation they control. The SOM endpoint can:
- Exclude content the owner doesn't want shared (ads, paywalls, internal navigation)
- Include structured metadata the owner wants highlighted
- Rate-limit access at the endpoint level
- Track which pages agents actually consume

**Reduced server load.** AI agents currently fetch full HTML pages, execute JavaScript, and often make multiple requests to render a single page. A pre-computed SOM endpoint serves a fraction of the data with zero rendering overhead.

**Better representation.** Raw HTML is a terrible format for AI consumption. It's full of presentational markup, tracking scripts, and structural noise. A SOM gives agents exactly what they need: the content, the structure, and the semantics.

### For AI Agent Developers

**Fewer tokens, lower costs.** The Semantic Object Model compresses web content by 10-16x compared to raw HTML. At GPT-4 rates ($30/1M tokens), processing 1M pages drops from ~$1M to ~$60K.

**Better extraction quality.** Instead of hoping your HTML parser correctly identifies the main content, you get a structured representation where the website owner has already made that determination.

**Faster processing.** No need to spin up a headless browser, wait for JavaScript execution, and parse the DOM. A SOM fetch is a single HTTP request returning clean JSON.

**Respect for publishers.** Agents that honor SOM directives demonstrate good citizenship. They consume content through the channel the publisher provides, rather than scraping the raw HTML.

### For the Web Ecosystem

**A path beyond the block-or-allow impasse.** The current conversation about AI and web content is adversarial: publishers block, agents circumvent. SOM directives create a cooperative alternative where both parties benefit.

**Standards-based approach.** This proposal is designed to be incubated through the W3C Web Content Browser for AI Agents Community Group and eventually formalized as an extension to the Robots Exclusion Protocol.

**Format-agnostic.** While this proposal uses Plasmate's SOM format as the reference implementation, the `SOM-Format` directive supports any structured representation: markdown, accessibility trees, custom JSON schemas. The framework is the contribution, not the specific format.

## Relationship to Existing Standards

### Robots Exclusion Protocol (REP)
SOM directives extend the existing robots.txt format. They are purely additive - existing `User-agent`, `Allow`, `Disallow`, and `Crawl-delay` directives continue to work exactly as specified in RFC 9309. Agents that don't understand SOM directives simply ignore them.

### Sitemaps
Sitemaps tell agents which pages exist. SOM directives tell agents how to best consume those pages. They are complementary.

### Schema.org
We have also proposed a `WebPageSemanticRepresentation` type for Schema.org ([issue #4786](https://github.com/schemaorg/schemaorg/issues/4786)) that serves a similar purpose within HTML pages. The robots.txt approach is site-wide; the Schema.org approach is per-page. Both can coexist.

### HTTP Link Headers
An alternative delivery mechanism would be HTTP `Link` headers with a `rel="semantic-representation"` type. This could complement the robots.txt approach for per-page signaling.

## Implementation Status

### Available Today
- **Plasmate SOM Cache** ([cache.plasmate.app](https://cache.plasmate.app)) can serve as the SOM endpoint for any website
- **Plasmate engine** ([plasmate.app](https://plasmate.app)) can generate SOM representations on demand
- The W3C Community Group is actively incubating the SOM and AWP specifications

### Adoption Path
1. **Phase 1:** Plasmate honors SOM directives in its own crawler. Websites that add SOM directives get better, more efficient agent access.
2. **Phase 2:** Publish this proposal through the W3C Community Group. Gather feedback from AI companies, publishers, and the web standards community.
3. **Phase 3:** Work with major AI agent platforms (OpenAI, Anthropic, Google, Perplexity) to recognize SOM directives in their crawlers.
4. **Phase 4:** Formalize as an extension to RFC 9309 (Robots Exclusion Protocol).

## FAQ

**Q: Doesn't this just create another standard to support?**
A: Yes, and that's the point. The current standard (block or allow) is insufficient for the agentic web. Adding SOM directives is a minimal extension that solves a real problem.

**Q: What if a site has SOM directives but the SOM endpoint is down?**
A: Agents should fall back to standard HTML fetching. SOM directives are a preference, not a requirement.

**Q: Can I use SOM directives to restrict AI access?**
A: SOM directives are additive. They don't replace `Disallow` rules. If a page is disallowed, the SOM endpoint should also refuse to serve it.

**Q: Does this only work with Plasmate's SOM format?**
A: No. The `SOM-Format` directive supports any format. We encourage the community to propose alternatives. The framework matters more than any single format.

## Get Involved

This proposal is being developed within the W3C Web Content Browser for AI Agents Community Group.

- **Join the group:** https://www.w3.org/community/web-content-browser-ai/
- **Plasmate engine:** https://plasmate.app
- **GitHub:** https://github.com/plasmate-labs/plasmate
- **Benchmarks:** https://github.com/plasmate-labs/plasmate-benchmarks

---

*Plasmate Labs. Apache 2.0.*
