# AGENTS.md — Plasmate Codebase Guide

This file is for AI coding agents (Cursor, Devin, Claude Code, Copilot, etc.). It tells you what the codebase does, how it is structured, and how to make changes safely.

## What Plasmate is

Plasmate is a headless browser engine that compiles web pages into a **Semantic Object Model (SOM)** — structured JSON optimised for LLM consumption — instead of returning raw HTML. 17x average token reduction. No API key, no cloud.

It runs as a CLI, a persistent daemon, an MCP server, and a CDP server.

## Build

```bash
~/.cargo/bin/cargo build              # debug
~/.cargo/bin/cargo build --release    # release
```

Requires Rust stable (1.77+). No system dependencies beyond a C linker.

## Test

```bash
~/.cargo/bin/cargo test               # all tests
~/.cargo/bin/cargo test som::         # SOM tests only
~/.cargo/bin/cargo test mcp::         # MCP tests only
RUST_LOG=debug ~/.cargo/bin/cargo test -- --nocapture  # with logging
```

There are 224+ tests. All must pass before a PR.

## Key directories

```
src/
  main.rs          CLI entry point (fetch, compile, diff, mcp, serve, daemon, screenshot)
  mcp/
    mod.rs         MCP server, JSON-RPC router, session manager
    tools.rs       ALL MCP tool definitions + handlers (add new tools here)
    sessions.rs    Persistent browser session state
  som/
    mod.rs         SOM data structures and serialisation
    filter.rs      apply_selector() — shared between CLI and MCP
    compiler.rs    HTML → SOM compiler (the core algorithm)
  js/
    runtime.rs     V8-backed JS execution
    pipeline.rs    Full fetch+JS+compile pipeline
  network/
    fetch.rs       HTTP client (reqwest)
sdk/python/        Python SDK (MCP client)
sdk/node/          Node.js SDK (MCP client)
integrations/      LangChain, LlamaIndex, Browser Use, etc.
packages/          som-parser-python, som-parser-node
```

## How to add an MCP tool

1. Add a `struct YourToolParams` with `#[derive(Deserialize)]` in `src/mcp/tools.rs`
2. Write `pub fn your_tool_definition() -> ToolDefinition` with name, description, and input_schema
3. Write `pub async fn handle_your_tool(arguments: &Value, ...) -> Value` handler
4. Register both in `src/mcp/mod.rs` — add to `list_tools()` and to the match in `call_tool()`
5. Add tests in `src/mcp/tools.rs` under `#[cfg(test)]`

Look at `extract_links_definition()` and `handle_extract_links()` for a clean minimal example.

## MCP tool description guidelines

Tool descriptions are read by LLMs (Claude, GPT-4, etc.) to decide which tool to call. Write them as action-oriented instructions, not feature lists:

- State WHAT it returns concretely
- State WHEN to use it vs alternatives
- Include any token-saving tips (`selector='main'`)
- Avoid vague phrases like "token-efficient" without numbers

## SOM selector syntax

`apply_selector(som, sel)` in `src/som/filter.rs` — supported values:

| Selector | Matches |
|----------|---------|
| `main` | `<main>` and `role=main` regions |
| `nav` | Navigation regions |
| `header` / `footer` | Header / footer regions |
| `aside` | Sidebar regions |
| `content` | Article / content regions |
| `form` | Form regions |
| `dialog` | Dialog/modal regions |
| `#foo` | Region with id `foo` |

Returns full SOM if selector matches nothing (graceful fallback).

## Python SDK

Located in `sdk/python/`. Run tests with:

```bash
cd sdk/python && PYTHONPATH=src python3 -m pytest tests/ -v
```

The key helper to know: `_extract_last_json(text)` in `client.py` — hardened JSON parser used by both sync and async `_call_tool`. It handles mixed output (progress lines before JSON, embedded JSON in log messages).

## Common patterns

**Error responses (Rust MCP handlers):**
```rust
return error_response("descriptive message here");
```

**Returning SOM as MCP content:**
```rust
return tool_response(serde_json::to_string(&result).unwrap_or_default());
```

**Applying selector before responding:**
```rust
let effective_som = if let Some(ref sel) = params.selector {
    crate::som::filter::apply_selector(&page_result.som, sel)
} else {
    page_result.som.clone()
};
```

## What NOT to do

- Do not call `reqwest::blocking` from inside a V8 callback or Tokio async context — use `std::thread::spawn` + `mpsc::channel` to escape (see PR #27 for the pattern)
- Do not add `unwrap()` on network operations — always handle errors and return `error_response()`
- Do not break the `apply_selector()` contract — it must return full SOM on no-match, never panic
- Do not change the `--format` or `--selector` CLI flags without updating both `main.rs` and `src/mcp/tools.rs`

## CI

GitHub Actions runs `cargo test` and `cargo clippy` on every PR. Both must pass.
