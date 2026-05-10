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

## Running State

### 2026-05-10 - Plasmate Improvements Automation

- Git sync: attempted `git fetch --prune origin` and `git pull --ff-only origin
  master` in the automation worktree, but shared worktree metadata is still
  blocked at `/Users/steve/Git/plasmate/.git/worktrees/plasmate12/FETCH_HEAD`.
  The primary checkout also cannot resolve `github.com`, so remote pull, push,
  PR creation, and merge remain blocked from this environment.
- Market direction: Playwright MCP and Cloudflare Browser Run keep validating
  structured accessibility snapshots, Stagehand continues to push
  `observe()`/action caching, Firecrawl is broadening hosted browser sessions,
  and Skyvern targets visual workflow completion. Plasmate should keep the
  local-first SOM wedge and prioritize actionability parity plus adapter
  tolerance over hosted browser-cloud features.
- Ecosystem state: the project still spans Rust CLI/daemon/MCP/CDP/AWP,
  Python/Node/Go SDKs, parser packages, Browser Use, LangChain, Vercel AI,
  generated docs, comparison pages, and marketing assets. Parser and schema
  conformance remain the highest-leverage way to keep this surface sticky.
- Code changes: Rust SOM compilation now resolves accessible labels from
  `aria-labelledby` and external `<label for="...">`; Python
  `from_plasmate()` now extracts SOM JSON from mixed CLI output; Node
  `fromPlasmate()` now accepts wrapped `{ som: ... }` payloads in clean and
  mixed output.
- Docs changes: updated PRD, roadmap, website doc sources, and generated docs
  with the 2026-05-10 market read, change rationale, completed improvements,
  and next steps around deeper accessible-name conformance.
- Verification: focused tests passed for `som_compiler_test` (43 tests),
  Python parser tests (59 tests), and Node parser tests (43 tests). Rust build,
  Node parser build, website doc generation, and `git diff --check` passed.

### 2026-05-09 - Plasmate Improvements Automation

- Git sync: `git fetch --prune origin` is still blocked in the automation
  worktree by shared worktree metadata permissions, and the primary checkout
  cannot resolve `github.com`. `gh auth status` still reports the configured
  `dbhurley` token is invalid, so PR creation and remote merge remain blocked
  from this environment.
- Market direction: Playwright MCP/Cloudflare structured snapshots,
  Stagehand `observe()` plus action caching, Firecrawl agent/browser-session
  APIs, Browser Use MCP surfaces, and Skyvern visual workflow reliability all
  point toward reusable action surfaces as the stickiness layer. Plasmate should
  continue to focus on local SOM action plans, conformance, and cache/diff
  observability rather than pivoting into hosted browser-cloud infrastructure.
- Ecosystem state: the repo surface remains broad across Rust core protocols,
  Python/Node/Go SDKs, parser packages, Browser Use, LangChain, Vercel AI,
  docs, generated website pages, benchmarks, and marketing. The main product
  risk is still contract drift across those libraries.
- Code changes: Python and Node parser packages now expose action lookup, hint
  lookup, and compact action-plan helpers so agents can query SOM action
  metadata without hand-walking the tree. Node compression-ratio handling now
  matches Python by returning infinity when `som_bytes` is zero.
- Docs changes: updated PRD, roadmap, and parser package READMEs with the
  2026-05-09 market read, action-plan rationale, completed improvements, and
  next steps for Go/integration parity.

### 2026-05-06 - Plasmate Improvements Automation

- Git sync: attempted `git fetch --prune origin` in the automation worktree,
  but shared worktree metadata is still blocked at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate8/FETCH_HEAD`. The primary
  checkout also cannot resolve `github.com`, and `gh auth status` reports the
  configured `dbhurley` token is invalid, so push/merge cannot complete from
  this environment.
- Market direction: Playwright MCP structured snapshots, Stagehand
  `observe()`/action caching, Firecrawl MCP browser interaction, and Skyvern
  visual workflows all point toward agent-ready page state as the sticky layer.
  Plasmate should keep the local-first wedge and improve SOM actionability,
  conformance, and deterministic cache/diff behavior rather than pivoting into
  hosted browser-cloud infrastructure.
- Ecosystem state: the project spans Rust CLI/daemon/MCP/CDP/AWP, Python/Node/Go
  SDKs, Browser Use, LangChain, Vercel AI, SOM parser packages, plugins,
  generated docs, comparison pages, and marketing assets. Contract drift across
  these surfaces remains the main roadmap risk.
- Code changes: SOM link deduplication now preserves case-sensitive paths;
  input type and ARIA role parsing tolerates real-world casing; custom controls
  retain `contenteditable`, `tabindex`, `name`, and `autocomplete` attrs; MCP
  `extract_text` truncation is UTF-8 safe.
- Docs changes: updated PRD, roadmap, and website docs source with the
  2026-05-06 market read, rationale, completed improvements, and next steps
  around accessible-name conformance and actionability fixtures.
- Verification: `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test --test som_compiler_test -- --nocapture`
  passed 42 tests; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test mcp::tools::tests::test_truncate_text_to_chars_preserves_utf8_boundaries -- --nocapture`
  passed. `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed. Full `cargo test` passed 245 lib tests and 5 main/MCP tests, then
  failed only in `tests/awp_integration_test.rs` because the sandbox denied
  local socket binding with `Operation not permitted`.

### 2026-05-05 - Plasmate Improvements Automation

- Git sync: attempted `git fetch --prune origin`, but the sandbox still cannot
  write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate7/FETCH_HEAD`. `gh` is
  installed, but the configured token for `dbhurley` is invalid, so push/merge
  cannot complete from this environment.
- Market direction: Playwright MCP structured accessibility snapshots,
  Browserbase/Stagehand action caching, and Firecrawl MCP/browser sessions
  continue to validate Plasmate's sticky wedge: local, reusable, full-tree SOM
  snapshots with stable agent-facing contracts.
- Ecosystem state: the project spans Rust core protocols, Python/Node/Go SDKs,
  Browser Use, LangChain, Vercel AI, SOM parser packages, plugin examples,
  smoke tests, generated website docs, comparison pages, and marketing assets.
  This is a distribution advantage, but schema and fixture conformance now need
  to be treated as roadmap work.
- Code changes: cache prefetch extraction now walks nested and shadow-root SOM
  elements, dedupes HTTP(S) URLs, and ignores non-navigation schemes; cache URL
  normalization preserves case-sensitive paths; MCP text/link extraction now
  includes shadow-root content.
- Docs changes: updated the PRD and roadmap with the current market read,
  ecosystem surface risk, reasons for the changes, and next steps around
  selector-aware cache, trace export, ARIA/SaaS parity, and shadow-DOM
  conformance.
- Verification: focused tests passed with the shared target cache:
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test cache::store -- --nocapture`
  and
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test mcp::tools::tests -- --nocapture`.
  Direct tests in the worktree target failed because `rusty_v8` attempted to
  download its prebuilt archive and DNS/network is blocked.

### 2026-05-04 - Plasmate Improvements Automation

- Git sync: attempted `git fetch --prune origin`, but the sandbox could not write shared worktree metadata at `/Users/steve/Git/plasmate/.git/worktrees/plasmate6/FETCH_HEAD`. Local detached HEAD matched the tracked `origin/master` snapshot available in this worktree.
- Market direction: Playwright MCP is making structured accessibility snapshots a baseline, Firecrawl is expanding MCP scraping/browser sessions, and Browserbase/Stagehand is using action caching and observability as retention hooks. Plasmate should lean into local SOM snapshots, selector-aware caching, and ecosystem conformance.
- Code changes: selector filtering now trims whitespace and supports documented `#region-id` selection; common ARIA widgets map to actionable SOM roles; hidden inline-style stripping handles casing and whitespace variants.
- Docs changes: added PRD direction, updated roadmap priorities, and corrected README tool count/Rust version drift.
- Next focus: selector-aware SOM cache entries, MCP/AWP trace export, and ARIA-heavy SaaS conformance fixtures.

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
