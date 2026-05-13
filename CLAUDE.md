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

### 2026-05-13T03:09:11Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted in the automation worktree, but
  shared worktree metadata writes are still denied at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate21/FETCH_HEAD`
  (`Operation not permitted`). Retried from the primary checkout, but
  `github.com` DNS resolution failed. Continued from the locally known
  `origin/master` at `9497429`.
- Market direction: current official docs continue to push browser-agent
  products toward structured action menus. Playwright MCP uses accessibility
  snapshots with refs as the interaction unit, Stagehand v3 `observe()` returns
  cacheable/validated actions, and Firecrawl plus Browser Use keep expanding
  managed browser sessions. The startup direction should remain local-first SOM
  portability, with safer reusable action plans before hosted browser scale.
- Code changes: Python SOM parser, Node SOM parser, and Go SDK compact
  action-plan helpers now expose explicit availability state. Enabled targets
  include `enabled: true`; disabled targets include `enabled: false` plus a
  disabled `blocked_reason` while preserving the existing `disabled` attr.
- Tests/docs changes: added parser and Go coverage for disabled action-plan
  targets, updated package/SDK READMEs, updated PRD and roadmap source docs,
  and rebuilt generated website PRD/roadmap pages. The next adapter step is to
  forward availability through Browser Use, LangChain, and Vercel AI.
- Verification: Python parser tests passed 66 tests; Node parser tests passed
  51 tests using the primary checkout's existing `vitest`; Node `tsc --noEmit`
  passed; Go SDK tests passed with `GOCACHE=/private/tmp/plasmate-go-cache`;
  `node website/build.mjs` rebuilt 39 pages through a temporary symlink to the
  primary checkout's existing website dependencies; `git diff --check` passed.
  A default cold `cargo build` failed only because `rusty_v8` could not
  download from GitHub in the sandbox, then
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings.
- Commit/push state: committed `6df8c78` as
  `chore: expose action plan availability`, pushed review branch
  `codex/plasmate-improvements-2026-05-13-action-availability`, and
  fast-forwarded remote `master` from `9497429` to `6df8c78`. A follow-up
  state-only commit records this final push state.

### 2026-05-13T02:08:23Z - Plasmate Improvements Automation

- Git sync: attempted the requested latest pull in the automation worktree
  first, but shared worktree metadata writes are still denied at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate20/FETCH_HEAD`
  (`Operation not permitted`). Retried from the primary checkout, but
  `git fetch --prune origin` failed because `github.com` could not resolve.
  Continued from the locally known `origin/master` at `74363bc`.
- Market direction: current 2026 competitor docs still validate local,
  structured action state as Plasmate's wedge. Playwright MCP snapshots make
  accessibility refs the deterministic interaction unit; Stagehand 3.3 adds
  verified agent identity, strict structured outputs, metrics, and clearer file
  upload/action state; Browserbase, Browser Use, Skyvern, Firecrawl, and
  Cloudflare Browser Run continue to compete on hosted sessions, profiles,
  traces, and managed scale. Plasmate should keep prioritizing portable
  SOM/action-state correctness and conformance before any hosted-browser pivot.
- Code changes: Rust SOM compilation now propagates disabled native
  `<fieldset>` state to descendant native controls, so radios, textareas,
  selects, and buttons inside locked field groups expose `attrs.disabled`
  directly instead of requiring agents to inspect parent group state.
- Conformance/docs changes: added shared `specs/conformance/015-action-state.*`
  fixture for disabled fieldset inheritance plus ARIA required/disabled
  promotion, updated the conformance index, and updated PRD/roadmap source plus
  generated website docs with the conformance rationale and next adapter-runner
  step.
- Verification: `rustfmt --check src/som/compiler.rs
  tests/som_compiler_test.rs` passed; focused disabled/ARIA and fieldset tests
  passed; `cargo test --test som_compiler_test -- --nocapture` passed 53 tests;
  `cargo build` passed with existing warnings; `node website/build.mjs` rebuilt
  39 pages; `git diff --check` passed. Full `cargo test` passed 245 lib tests
  and 5 main/MCP tests, then failed only in `tests/awp_integration_test.rs`
  because sandbox local socket setup is denied with `Operation not permitted`,
  matching prior automation runs.
- Commit/push state: committed `3099654` as
  `chore: add SOM action state conformance`, pushed branch
  `codex/plasmate-improvements-2026-05-13-action-state`, and fast-forwarded
  remote `master` from `74363bc` to `3099654`. A follow-up state-only commit
  records this final push state.

### 2026-05-13T00:47:00Z - Plasmate Improvements Automation

- Git sync: attempted the requested latest pull first with `git fetch --prune
  origin`, but this sandbox cannot write the parent repository worktree
  metadata at `/Users/steve/Git/plasmate/.git/worktrees/plasmate19/FETCH_HEAD`
  (`Operation not permitted`). Continued from the locally known `origin/master`
  at `d0caf2c`.
- Market direction: 2026 browser-agent research still points to hybrid
  deterministic execution plus selective AI planning. Playwright/Playwright MCP
  anchor structured snapshots and stable execution, Stagehand-style
  `observe()` APIs turn ambiguous state into cacheable actions, Browserbase,
  Browser Use, Skyvern, and Firecrawl compete on managed sessions, profiles,
  proxies/CAPTCHA, and traces, and WebMCP remains a standards watch item. The
  Plasmate direction should remain local-first SOM/action state fidelity rather
  than hosted browser infrastructure.
- Code changes: Rust SOM compilation now preserves `attrs.disabled` for native
  disabled `<textarea>` and `<select>` controls, promotes
  `aria-required="true"` to top-level `attrs.required`, and promotes
  `aria-disabled="true"` to top-level `attrs.disabled` while retaining nested
  `attrs.aria.disabled`.
- Tests/docs changes: added compiler coverage for disabled textarea/select
  controls and ARIA required/disabled custom controls. Updated PRD, roadmap,
  website docs source, and generated website docs with the state-fidelity
  market read and next conformance priorities.
- Verification: targeted `rustfmt --check src/som/compiler.rs
  tests/som_compiler_test.rs` passed; `cargo build` passed with existing
  warnings; focused state test passed; full `cargo test --test
  som_compiler_test -- --nocapture` passed 53 tests; `node website/build.mjs`
  rebuilt 39 pages after temporarily pointing this worktree at the primary
  checkout's existing `website/node_modules`; and `git diff --check` passed.
  Full `cargo test` passed 245 lib tests and 5 main/MCP tests, then failed only
  in `tests/awp_integration_test.rs` because sandbox local socket setup is
  denied with `Operation not permitted`.
- Commit/push state: committed `30787d3` as
  `chore: improve SOM action state fidelity`, pushed review branch
  `codex/plasmate-improvements-2026-05-13-state-fidelity`, and
  fast-forwarded remote `master` from `d0caf2c` to `30787d3`. A follow-up
  state-only commit records this final push state.

### 2026-05-12T23:41:02Z - Plasmate Improvements Automation

- Git sync: attempted the requested latest pull first with `git fetch --prune
  origin`, but this sandbox cannot write the parent repository worktree
  metadata at `/Users/steve/Git/plasmate/.git/worktrees/plasmate17/FETCH_HEAD`
  (`Operation not permitted`). Continued from the locally known `origin/master`
  at `d281213`.
- Market direction: current Playwright MCP docs make accessibility snapshots
  with interaction refs table stakes; Stagehand continues to push `observe()`
  and local/managed action caching; Firecrawl's MCP surface now spans
  scrape/search/extract plus browser interaction; Cloudflare Browser Run is
  adding CDP/MCP/WebMCP around hosted sessions; and Browser Use Cloud continues
  to package agents, profiles, CDP sessions, and managed scale. The startup
  direction should remain local-first, with richer portable SOM action plans
  before any hosted-browser pivot.
- Code changes: ARIA landmark role parsing is now case-insensitive for
  uppercase production markup, and declarative shadow DOM extraction now
  recurses through wrapper containers so nested web-component controls are not
  dropped.
- SDK/parser changes: Python and Node parser packages plus the Go SDK now add
  placeholder, description, required, disabled, and group metadata to compact
  action-plan helpers.
- Docs changes: updated PRD, roadmap, website docs source, generated website
  docs, and this running state with the action-plan/WebMCP market read and next
  steps around cross-adapter fixtures.
- Verification: `cargo build` passed with existing warnings; focused
  `cargo test --test som_compiler_test -- --nocapture` passed 52 tests; Python
  parser tests passed 65 tests; Node parser tests passed 50 tests; Node parser
  build passed; Go SDK tests passed; `node website/build.mjs` rebuilt 39 pages;
  and `git diff --check` passed. Full `cargo test` passed 245 lib tests and 5
  main/MCP tests, then failed only in `tests/awp_integration_test.rs` because
  sandbox local socket setup is denied with `Operation not permitted`.
- Commit/push state: committed `515cb76` as
  `chore: enrich SOM action planning metadata`, pushed review branch
  `codex/plasmate-improvements-2026-05-12-action-plans`, fast-forwarded
  remote `master` from `d281213`, and pushed a follow-up state-only
  `CLAUDE.md` commit to both the review branch and `master`.

### 2026-05-12T08:50:29Z - Plasmate Improvements Automation

- Git sync: attempted the requested latest pull first with `git fetch --prune
  origin`, but the automation worktree still cannot write shared worktree
  metadata at `/Users/steve/Git/plasmate/.git/worktrees/plasmate16/FETCH_HEAD`
  (`Operation not permitted`). Local `HEAD` matched the locally known
  `origin/master` at `34ed815` before changes.
- Market direction: current Playwright MCP docs still center structured
  accessibility snapshots with refs; Stagehand foregrounds `observe()` plus
  local/managed action caching; Cloudflare Browser Run and Browser Use Cloud
  continue to make hosted browser sessions, profiles, recordings, proxies, and
  managed scale easier to buy. Plasmate should keep the local-first wedge and
  improve portable SOM action semantics for repeated SaaS form workflows rather
  than pivoting into hosted browser infrastructure.
- Ecosystem state: the project remains broad across Rust CLI/daemon/MCP/CDP/AWP,
  Python/Node/Go SDKs, parser packages, Browser Use, LangChain, Vercel AI,
  generated docs, comparison pages, benchmarks, and marketing. Contract drift
  remains the main retention risk, so this run changed schema/spec, SDKs,
  parser packages, CDP mappings, docs, and conformance together.
- Code changes: Rust SOM compilation now emits labelled `group` elements for
  native `<fieldset>` controls and ARIA `group`/`radiogroup`; fieldset groups
  derive labels plus `attrs.legend` from the first `<legend>` and preserve
  `attrs.disabled`; CDP accessibility/DOM mappings understand `group`.
- SDK/schema changes: SOM schema/spec, Python and Node SDK types, Python and
  Node parser types, and Go SDK attrs now accept `group` and `attrs.legend`.
  Parser and Go tests cover group/legend payload compatibility.
- Docs changes: updated PRD, roadmap, website docs sources, generated docs, and
  conformance docs with the 2026-05-12 market read and fieldset/radiogroup
  rationale. Added `specs/conformance/014-fieldset-groups.*`.
- Verification: `cargo test --test som_compiler_test -- --nocapture` passed 50
  tests; Python parser tests passed 65 tests; Node parser tests passed 50
  tests after `npm ci --ignore-scripts`; Node parser build passed; Go SDK tests
  passed; `cargo build` passed; `node website/build.mjs` rebuilt 39 pages; and
  `git diff --check` passed. Full `cargo test` passed 245 lib tests and 5
  main/MCP tests, then failed only in `tests/awp_integration_test.rs` because
  sandbox local socket setup is denied with `Operation not permitted`.
- Commit/push state: automation worktree commit was blocked by shared worktree
  `index.lock` permissions, so the reviewed patch was applied to the primary
  checkout and committed with message `chore: improve SOM fieldset group
  semantics`. Pushed branch `codex/plasmate-improvements-2026-05-12` and
  fast-forwarded remote `master` from `34ed815`. Unrelated
  untracked `.agents/` in the primary checkout remains untouched.

### 2026-05-11T13:55:05Z - Plasmate Improvements Automation

- Git sync: attempted the requested latest pull first. The automation worktree
  still cannot write shared metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate15/FETCH_HEAD`
  (`Operation not permitted`). The primary checkout fetch also failed because
  `github.com` could not resolve, so this run continued from the locally
  current `origin/master`/automation branch state at `16bf9d7`.
- Market direction: Playwright MCP and Stagehand still validate structured
  snapshots plus reusable action planning; Firecrawl continues to broaden
  search/scrape/browser-session surfaces; Cloudflare's Browser Run launch adds
  hosted browser sessions with Live View, recordings, human-in-loop, MCP/CDP,
  and structured extraction. The startup direction should stay local-first and
  deepen portable SOM correctness, traceability, and cache reuse rather than
  pivoting into hosted browser infrastructure.
- Ecosystem state: the repo remains broad across Rust CLI/daemon/MCP/CDP/AWP,
  Python/Node/Go SDKs, parser packages, Browser Use, LangChain, Vercel AI,
  generated website docs, benchmarks, and marketing. Contract drift across
  these surfaces is still the main retention risk.
- Code changes: Rust SOM compilation now resolves wrapped `<label>` controls
  for checkboxes/selects without leaking nested option text; landmark and form
  region labels now resolve `aria-labelledby`; input buttons now expose
  value-derived labels and normalized `attrs.input_type` for `submit`,
  `button`, and `reset`.
- Docs changes: updated PRD, roadmap, website doc sources, and generated docs
  with the Browser Run market read, completed accessibility-name improvements,
  and next steps around fieldsets/legends, selector-aware cache, trace export,
  and SaaS form conformance fixtures.
- Verification: `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test
  --test som_compiler_test -- --nocapture` passed 49 tests.
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed.
  `node website/build.mjs` rebuilt 39 pages. `git diff --check` passed. Full
  `cargo test` passed 245 lib tests and 5 main/MCP tests, then failed only in
  `tests/awp_integration_test.rs` because the sandbox denied local socket setup
  with `Operation not permitted`.

### 2026-05-11T13:28:00Z - Plasmate Improvements Automation

- Git sync: attempted the requested latest pull first. The automation worktree
  still cannot write shared metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate14/FETCH_HEAD`
  (`Operation not permitted`), the primary checkout cannot resolve
  `github.com`, and `gh auth status` still reports the configured `dbhurley`
  token is invalid. Remote merge to `master` remains blocked by network/auth.
- Market direction: current Playwright MCP docs reinforce accessibility
  snapshots with refs, Browserbase/Stagehand is marketing `observe()` plus
  local/managed action caching, and Firecrawl continues to package managed
  browser sessions. The startup direction should stay local-first and increase
  stickiness through cross-language SOM action contracts rather than a hosted
  browser-cloud pivot.
- Ecosystem state: the project spans Rust CLI/daemon/MCP/CDP/AWP, Python,
  Node, and Go SDKs, Browser Use, LangChain, Vercel AI, SOM parser packages,
  generated website docs, comparison pages, benchmarks, and marketing assets.
  The most important product risk is contract drift across those repositories
  and libraries.
- Code changes: Go SDK types now parse `shadow`, accessible descriptions,
  `name`, `autocomplete`, ARIA state, details attrs, and iframe attrs; Go query
  helpers now traverse shadow roots for id, role, text, interactivity, and
  flattened element queries; Go now exposes `FindByAction`, `FindByHint`, and
  `GetActionPlan` for compact agent planning parity with Python/Node parser
  packages.
- Docs changes: updated the Go SDK README, PRD, roadmap, website docs source,
  and generated website docs with the Go parity rationale, completed
  improvements, and next steps around shared conformance fixtures and framework
  integration parity.
- Verification: `GOCACHE=/Users/steve/Git/plasmate/target/go-build go test
  ./...` passed in `sdk/go`; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target
  cargo build` passed; `node website/build.mjs` rebuilt 39 docs pages;
  `git diff --check` passed. Full `cargo test` passed 245 lib tests and 5
  main/MCP tests, then failed only in `tests/awp_integration_test.rs` because
  the sandbox denied local socket setup with `Operation not permitted`.

### 2026-05-11 - Plasmate Improvements Automation

- Git sync: attempted `git fetch origin --prune` in the automation worktree,
  but shared worktree metadata is still blocked at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate13/FETCH_HEAD` (`Operation
  not permitted`). The primary checkout was on the latest locally available
  automation branch (`codex/plasmate-improvements-2026-05-09` at `1f63a47`),
  so this run continued from that state. Remote push/merge remains blocked
  because `gh auth status` reports the configured `dbhurley` token is invalid.
- Market direction: Playwright MCP now documents accessibility snapshots with
  stable refs as the primary interaction contract; Stagehand positions
  `observe()` as planning, validation, and caching for repeated actions across
  iframes and shadow DOM; Firecrawl and Browser Use keep pushing managed
  browser sessions and persistent profiles. Plasmate should keep the local-first
  wedge and improve SOM fidelity, accessible names/descriptions, full-tree
  counts, and conformance fixtures rather than pivoting to hosted browser
  infrastructure.
- Ecosystem state: the project still spans Rust CLI/daemon/MCP/CDP/AWP, Python,
  Node, and Go SDKs, Browser Use, LangChain, Vercel AI, SOM parser packages,
  website docs, comparison pages, and marketing assets. The roadmap risk remains
  contract drift across these many surfaces.
- Code changes: SOM metadata now counts shadow-root elements and controls;
  `aria-labelledby` now takes precedence over `aria-label`; SOM attrs now expose
  descriptions from `aria-describedby` and `aria-description`; schema and
  Python/Node types now accept `attrs.description`. Compiler tests cover label
  precedence, accessible descriptions, and shadow-root meta counts.
- Docs changes: updated PRD, roadmap, SOM spec/schema, website docs source, and
  generated docs with the 2026-05-11 market read, change rationale, completed
  improvements, and next steps around wrapped labels, fieldsets/legends,
  description fixtures, selector-aware cache, trace export, and shared
  conformance fixtures.
- Verification: `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test
  --test som_compiler_test -- --nocapture` passed 46 tests.
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed.
  Full `cargo test` passed 245 lib tests and 5 main/MCP tests, then failed only
  in `tests/awp_integration_test.rs` because the sandbox denied local socket
  creation with `Operation not permitted`.

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
