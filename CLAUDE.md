# Plasmate

The browser engine for AI agents. HTML in, Semantic Object Model out.

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/plasmate fetch https://example.com

# MCP mode (for Claude Code, Cursor, etc.)
./target/release/plasmate mcp
```

## Architecture

- `src/main.rs` - CLI entry point (Clap)
- `src/network/` - HTTP fetch with H1/H2 fallback, custom headers
- `src/som/` - Semantic Object Model compiler (HTML to structured JSON)
- `src/cdp/` - Chrome DevTools Protocol server
- `src/awp/` - Agent Web Protocol handler
- `src/bench/` - Benchmarking runner
- `src/js/` - V8 JavaScript execution engine
- `website/` - plasmate.app homepage
- `integrations/` - Framework adapters (Vercel AI, LangChain, browser-use)

## Key Commands

- `plasmate fetch <url>` - Fetch URL, output SOM JSON
- `plasmate fetch <url> --text` - Extract readable text only
- `plasmate fetch <url> -H "Authorization: Bearer ..."` - Fetch with custom headers
- `plasmate serve` - Start CDP server
- `plasmate mcp` - Start MCP server (stdio)
- `plasmate bench <url>` - Run benchmarks

## Testing

```bash
cargo test
cargo check
```

## Version

Version is derived from `Cargo.toml` via `env!("CARGO_PKG_VERSION")`. Do not hardcode version strings.

## Automation State

Last updated: 2026-04-28 by Plasmate Improvements automation.

- Shell Git fetch/push is available again. The automation branch is `codex/plasmate-improvements-2026-04-28`, based on `origin/master`.
- Current strategic direction: keep Plasmate local-first and SOM-centered, then increase stickiness through compatibility bridges into Browser Use, Playwright MCP-style snapshots, Stagehand/Browserbase observe/extract flows, and Firecrawl-style structured extraction.
- Current code focus: maintain schema parity across Rust, Python, Node, Go, and parser packages so compiler output is accepted and queryable everywhere.
- 2026-04-28 changes verified: SDK/parser support for `html_id`, `details`, `iframe`, iframe attrs, and declarative `shadow`; Python/Node query helpers traverse shadow roots; SOM selectors trim whitespace and accept `article` as a `content` alias; product docs now record market-informed roadmap updates.
- Build/checks passed: `cargo build`, `cargo test som::filter -- --nocapture`, Python SDK tests, Python parser tests, Node SDK build, Node parser build/test, JSON schema parse, and `git diff --check`.

---

## Claude Code Guidelines

Behavioral guidelines to reduce common LLM coding mistakes. Bias toward caution over speed.

### Think Before Coding
- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — do not pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what is confusing. Ask.

### Simplicity First
- No features beyond what was asked.
- No abstractions for single-use code.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

### Surgical Changes
- Do not "improve" adjacent code, comments, or formatting.
- Do not refactor things that are not broken.
- Match existing style, even if you would do it differently.
- Every changed line should trace directly to the user's request.

### Goal-Driven Execution
- Transform tasks into verifiable goals with success criteria.
- For multi-step tasks, state a brief plan with verification checkpoints.
- Strong success criteria enable independent work. Weak criteria require constant clarification.
