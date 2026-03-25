# Robots.txt for the Agentic Web

The robots.txt standard was designed in 1994 for a simple question: should this crawler index my page?

Thirty-two years later, we're using the same binary mechanism to manage a fundamentally different relationship. AI agents don't just index pages - they read, reason, extract, and act on web content. The conversation is stuck on a single axis: **block or allow**.

## The Current State

```
User-agent: GPTBot
Disallow: /

User-agent: ClaudeBot
Disallow: /
```

Website owners face an all-or-nothing choice. Allow everything and lose control, or block everything and become invisible to the fastest-growing discovery channel on the web.

There's no way to say: "Yes, you can read my content, and here's a better way to do it."

## The Proposal: SOM Directives

We propose extending robots.txt with directives that let websites advertise semantic representations of their content.

```
User-agent: *
Allow: /

# Semantic Object Model available
SOM-Endpoint: https://cache.example.com/v1/som
SOM-Format: SOM/1.0
SOM-Scope: main-content
SOM-Freshness: 3600
```

When an AI agent sees these directives, instead of fetching the full HTML page (50,000+ tokens), it can request the SOM endpoint and get a clean, structured representation (~3,000 tokens).

## New Directives

| Directive | Description |
|-----------|-------------|
| `SOM-Endpoint` | Base URL of the SOM service. Agents append `?url=` with the target page. |
| `SOM-Format` | Format of the representation: `SOM/1.0`, `markdown`, `accessibility-tree` |
| `SOM-Scope` | Content coverage: `full-page`, `main-content`, `article-body` |
| `SOM-Freshness` | Max age in seconds of a cached SOM (default: 86400) |
| `SOM-Token-Budget` | Suggested max tokens, helping agents estimate costs before fetching |

## Why This Matters

**For website owners:** Control without blocking. Direct agents to a representation you control - exclude ads, paywalls, and noise. Include what you want highlighted.

**For agent developers:** 10-16x fewer tokens, better extraction quality, no headless browser needed. A single HTTP request instead of Chrome + JS execution + DOM parsing.

**For the web ecosystem:** A cooperative alternative to the current adversarial dynamic where publishers block and agents circumvent.

## Relationship to Existing Standards

SOM directives are purely additive to RFC 9309 (Robots Exclusion Protocol). Existing `User-agent`, `Allow`, `Disallow`, and `Crawl-delay` rules continue to work unchanged. Agents that don't understand SOM directives simply ignore them.

This complements our [Schema.org proposal](https://github.com/schemaorg/schemaorg/issues/4786) for `WebPageSemanticRepresentation` (per-page signaling) and the [SOM specification](/som-spec) being incubated at the W3C.

## Get Involved

This proposal is being developed within the [W3C Web Content Browser for AI Agents Community Group](https://www.w3.org/community/web-content-browser-ai/).

- [Full proposal](https://github.com/plasmate-labs/plasmate/blob/master/docs/ROBOTS-TXT-PROPOSAL.md)
- [Join the W3C group](https://www.w3.org/community/web-content-browser-ai/)
- [SOM Specification](/som-spec)
