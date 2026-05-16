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

### 2026-05-16T02:05:58Z - Plasmate Improvements Automation

- Git sync: required latest pull was retried. The automation worktree still
  cannot open the linked checkout `FETCH_HEAD`, and the primary checkout
  still cannot resolve `github.com`. Checkout to the newest local branch also
  failed on the linked worktree `index.lock`, so the newest locally available
  diff (`origin/codex/link-navigation-replay-cues`) was applied directly
  before this run's ARIA naming-provenance work.
- Market direction: current docs continue to validate Plasmate's local-first
  replay-validation wedge. Playwright MCP uses snapshot-scoped accessibility
  refs, Browserbase/Stagehand positions action caching around DOM validation,
  Firecrawl keeps broadening MCP scrape/extract/browser sessions, and Browser
  Use/AWS AgentCore Browser emphasize profiles for repeated authenticated
  workflows. The roadmap should keep improving portable SOM/action target
  fidelity rather than pivoting into hosted browser infrastructure.
- Code changes: Rust SOM attrs and JSON Schema now preserve three ARIA naming
  provenance cues: raw `aria-label`, raw `aria-labelledby`, and raw
  `aria-describedby`. Python/Node parser packages, Python/Node/Go SDKs,
  Browser Use, LangChain, and Vercel AI action-plan surfaces now carry/render
  these as `aria_label`, `labelledby`, and `describedby` without changing
  deterministic action `cache_key` values.
- Fixture/docs changes: the shared action-availability fixture now asserts
  ARIA naming provenance beside resolved `description` text. PRD and roadmap
  docs now record the 2026-05-16 ARIA naming-provenance rationale and next
  conformance step.
- Verification passed: `rustfmt --check src/som/compiler.rs
  tests/som_compiler_test.rs`, JSON fixture/schema parse checks, touched
  Python syntax compile, focused Rust ARIA compiler tests, `cargo build`,
  `cargo test --lib --quiet` (258), Python parser tests (72), Python SDK
  query tests (40), Browser Use fixture test, LangChain fixture test, Go SDK
  tests, and `git diff --check`.
- Verification gaps: Node parser and SDK tests remain blocked by missing local
  dev dependencies (`vitest`, `tsc`); Vercel AI tests remain blocked by
  missing `tsup`; website docs generation remains blocked by missing `marked`.
  A mistyped focused cargo command rejected multiple test-name arguments before
  the corrected Rust test run passed.
- Commit/push state: created alternate-index commit `98613ae` (`chore: expose
  aria naming provenance`), fast-forwarded local `master` to it, and pushed
  review branch `codex/aria-naming-provenance`. Direct `origin/master` push
  reached GitHub but was rejected with `fetch first`; retrying `git fetch
  origin master` from the primary checkout failed DNS for `github.com`, and
  `gh pr create` failed to reach `api.github.com`. Remote merge remains
  blocked until fetch/API access works.

### 2026-05-16T01:13:12Z - Plasmate Improvements Automation

- Git sync: required latest pull was retried. The automation worktree still
  cannot open the linked checkout `FETCH_HEAD`; switching to the newest local
  remote-tracking branch also failed on the linked worktree `index.lock`. The
  tree was clean, so the newest available local remote-tracking diff
  (`origin/codex/drag-replay-cues`) was applied directly before this run's
  link-replay work.
- Market direction: current docs continue to validate Plasmate's local-first
  action-surface wedge. Playwright MCP uses accessibility snapshots with refs
  scoped to the current snapshot; Browserbase/Stagehand positions observed
  actions and action caching around DOM validation; Browser Use and Firecrawl
  keep broadening hosted sessions, profiles, and skills. The roadmap should
  keep making local SOM action targets richer and easier to validate rather
  than pivoting into hosted browser infrastructure.
- Code changes: Rust SOM attrs and JSON Schema now preserve three link replay
  validation cues: `hreflang`, link MIME `type`, and `referrerpolicy`.
  Python/Node parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan surfaces now carry/render those fields without
  changing deterministic action `cache_key` values.
- Fixture/docs changes: the shared action-availability fixture now asserts
  link locale, resource type, and referrer-policy context. PRD and roadmap
  docs now record the 2026-05-16 link replay rationale and next conformance
  step.
- Verification passed: `rustfmt --check src/som/compiler.rs
  tests/som_compiler_test.rs`, JSON fixture/schema parse checks, touched
  Python syntax compile, focused Rust link-metadata test, `cargo build`,
  `cargo test --lib --quiet` (258), Python parser tests (72), Python SDK query
  tests (40), Browser Use fixture test, LangChain fixture test, Go SDK tests,
  and `git diff --check`.
- Verification gaps: whole-worktree cargo tests are still constrained by the
  sandboxed AWP listener issue noted in prior runs; Node parser, Node SDK, and
  Vercel AI tests remain blocked by missing local dev dependencies (`vitest`,
  `tsc`, `tsup`); website docs generation remains blocked by missing `marked`.
- Commit/push state: created alternate-index code/docs commit `92a1206`
  (`chore: expose link replay cues`) plus running-state follow-ups and pushed
  review branch `codex/link-navigation-replay-cues`. Direct `master` push was
  rejected with `fetch first`; retrying `git fetch origin master` from the
  primary checkout failed DNS for `github.com`, and `gh pr create` failed to
  reach `api.github.com`. Remote merge remains blocked until fetch/API access
  works.

### 2026-05-16T00:08:51Z - Plasmate Improvements Automation

- Git sync: required GitHub refresh was retried. The automation worktree still
  cannot write its linked `FETCH_HEAD`, and the primary checkout still cannot
  resolve `github.com`, so work continued by applying the newest available
  local remote-tracking diff (`codex/action-target-replay-lookup`) directly to
  the worktree before adding this run's changes.
- Market direction: current docs continue to favor reusable structured action
  surfaces over another hosted browser fleet. Playwright MCP keeps refs scoped
  to fresh accessibility snapshots; Browserbase/Stagehand markets validated
  action caching and DOM fingerprints; Browser Use and Firecrawl package
  hosted sessions, profiles, skills, and browser sandboxes. Plasmate should
  keep improving local replay validation context across its broad SDK and
  adapter surface.
- Code changes: Rust SOM attrs and schema now preserve drag/drop replay cues
  with native `draggable`, ARIA `grabbed`, and ARIA `dropeffect`. Python/Node
  parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel AI
  action-plan surfaces now expose those cues without changing deterministic
  action `cache_key` values.
- Fixture/docs changes: the shared action-availability fixture now asserts
  drag/drop replay context, and PRD/roadmap docs record the reason: kanban
  boards, upload builders, scheduling grids, and workflow canvases need more
  than cached click identity to validate local replay.
- Verification passed: `rustfmt --check src/som/compiler.rs`, JSON fixture and
  schema parse checks, touched Python syntax compile, `git diff --check`,
  focused Rust drag-cue test, `cargo build`, `cargo test --lib --quiet` (258),
  Python parser tests (72), Python SDK query tests (40), Browser Use fixture
  test, LangChain fixture test, and Go SDK tests.
- Verification gaps: full `cargo test --quiet` still fails in sandboxed AWP
  integration tests because `TcpListener::bind("127.0.0.1:0")` returns
  `Operation not permitted`; Node parser, Node SDK, and Vercel AI tests remain
  blocked by missing local dev dependencies (`vitest`, `tsc`, `tsup`); website
  docs generation remains blocked by missing `marked`.
- Commit/push state: created alternate-index code/docs commit `ccab552`
  (`chore: carry drag replay cues`) plus running-state commit `0314aac`, then
  pushed review branch `codex/drag-replay-cues`. Direct `master` push was
  rejected with `fetch first`; `git fetch origin master` and `git ls-remote`
  failed DNS for `github.com`, and `gh pr create` failed to reach
  `api.github.com`. Remote merge remains blocked until fetch/API access works.

### 2026-05-15T23:12:05Z - Plasmate Improvements Automation

- Git sync: latest pull was retried from the automation worktree and failed
  before network access because the linked worktree cannot write
  `FETCH_HEAD`. A fast-forward to the known remote-tracking
  `origin/codex/locator-provenance-cues` ref also failed on `ORIG_HEAD`, so
  that remote-tracking diff was applied directly before this run's new work.
- Market direction: current docs still favor the local-first reusable action
  surface. Playwright MCP keeps snapshot-scoped accessibility refs, Stagehand
  documents `observe()` plus local/server action caching, Firecrawl and
  Browser Use keep broadening hosted browser/session products, and Cloudflare
  Browser Run/WebMCP points toward typed action discovery. Plasmate should
  keep deepening portable local SOM/action replay rather than pivoting into
  hosted browser infrastructure.
- Code changes: carried forward locator provenance (`title`, `source_role`,
  `test_id`) from the known remote-tracking branch, then added compact action
  target replay helpers across Python/Node parser packages, Python/Node SDKs,
  and Go SDK. New helpers return enabled-only action plans, build indexes by
  SOM id, `cache_key`, `html_id`, and `test_id`, and expose direct lookup
  helpers for those keys.
- Docs changes: PRD and roadmap docs now record action target lookup as the
  next stickiness layer after locator provenance, with a follow-up to promote
  the lookup/index contract into Browser Use, LangChain, and Vercel AI.
- Verification: `rustfmt --check src/som/compiler.rs`, `cargo build`,
  `cargo test --lib --quiet` (257), Python parser tests (72), Python SDK query
  tests (40), Go SDK tests, Python syntax compile, JSON fixture/schema parse,
  and `git diff --check` passed.
- Verification gaps: Node SDK and Node parser tests remain blocked because
  local dev dependencies are absent (`tsc` and `vitest` not found).
- Commit/push state: created code commit `39ad726` (`chore: add action target
  replay lookup`) plus running-state follow-ups; branch
  `codex/action-target-replay-lookup` was pushed to GitHub. Direct push to
  `master` was rejected with `fetch first`; retrying
  `git fetch origin master` from `/Users/steve/Git/plasmate` failed DNS for
  `github.com`, and `gh pr create` failed to reach `api.github.com`. Remote
  merge remains blocked until fetch/API access works.

### 2026-05-15T22:11:34Z - Plasmate Improvements Automation

- Git sync: latest pull was retried from the automation worktree and failed
  because the linked worktree cannot open `FETCH_HEAD`. The review branch was
  pushed successfully, but a direct `master` push was rejected with
  `fetch first`; retrying `git fetch origin master` from
  `/Users/steve/Git/plasmate` failed DNS for `github.com`. Remote merge
  remains blocked until fetch/API access works.
- Market direction: current docs still favor Plasmate's local-first
  structured action surface. Playwright MCP uses snapshot-scoped refs,
  Stagehand documents local/server action caching, Firecrawl Interact resumes
  scrape sessions for prompt/code actions, and Browser Use Cloud emphasizes
  CDP sessions plus profiles. The roadmap should continue improving portable
  local action provenance instead of pivoting into hosted browser fleets.
- Code changes: Rust SOM attrs and schema now preserve `title`,
  `source_role`, and normalized `test_id` values from `data-testid`,
  `data-test`, or `data-qa`. Python/Node parser packages, Python/Node/Go SDK
  action plans, Browser Use, LangChain, and Vercel AI now carry/render those
  fields without changing deterministic action `cache_key` values.
- Fixture/docs changes: the shared action-availability fixture now asserts
  locator provenance on an action target. PRD, roadmap, website doc sources,
  SOM spec, and this running state now record the locator-provenance rationale
  and next conformance step.
- Verification: `rustfmt --check src/som/compiler.rs` passed; focused Rust
  compiler tests for keyboard/custom-role and test-id fallback attrs passed;
  `cargo build` passed; `cargo test --lib --quiet` passed 257 tests; Python
  parser tests passed 70 tests; Python SDK query tests passed 38 tests; Go SDK
  tests passed; Browser Use and LangChain fixture tests passed; JSON fixture
  parse, Python syntax compile, and `git diff --check` passed.
- Verification gaps: Node parser, Node SDK, and Vercel AI tests remain blocked
  because local `node_modules` are absent (`vitest`, `tsc`, `tsup` not found).
  `node website/build.mjs` remains blocked because `marked` is not installed.
- Commit/push state: worktree refs are still locked by sandbox permissions, so
  the patch was committed with an alternate index. Review branch
  `codex/locator-provenance-cues` was pushed with code commit `538404e` plus
  running-state follow-up commits; follow-up PR creation failed because
  `api.github.com` was unreachable.

### 2026-05-15T21:15:26Z - Plasmate Improvements Automation

- Git sync: latest pull was retried from the automation worktree and failed
  because the linked worktree cannot open `FETCH_HEAD`. Retrying from
  `/Users/steve/Git/plasmate` failed DNS for `github.com`, so work continued
  from local `HEAD` `4fabe19`/primary branch state `895d5b6`.
- Market direction: current competitor/product motion still favors
  Plasmate's local-first SOM/action-state wedge. Playwright MCP keeps
  structured snapshots and refs central, Stagehand-style workflows emphasize
  cacheable observed actions, and browser-cloud MCP/CDP surfaces keep
  broadening. The roadmap should continue increasing stickiness through
  precise portable action provenance across SDKs and integrations instead of
  pivoting into hosted browser infrastructure.
- Code changes: Rust already emitted `html_id`; this run completed downstream
  parity by adding `html_id` parsing/query/action-plan support across the
  Python parser package, Python SDK, Node parser package, Node SDK, and Go SDK.
  Browser Use, LangChain, and Vercel AI renderers now surface HTML ids beside
  cache keys so agents can choose stable DOM anchors when refs are ephemeral.
- Fixture/docs changes: shared action-availability fixtures now assert
  `html_id` on representative form controls, links, buttons, selects,
  checkboxes, sliders, and toggles. PRD, roadmap, and website doc sources now
  record HTML-id provenance as the next cross-adapter conformance focus.
- Verification: Python parser tests passed 70 tests; Python SDK tests passed
  52 tests; Go SDK tests passed with a local `GOCACHE`; Browser Use and
  LangChain integration tests passed; JSON fixtures parsed; `git diff --check`
  passed; `cargo build` passed; and `cargo test --lib --quiet` passed 256
  tests.
- Verification gaps: Node package tests were blocked because local
  `node_modules` are absent (`vitest`/`tsc` not found), and
  `node website/build.mjs` remains blocked because `marked` is not installed.
- Commit/push state: the linked worktree still cannot update its own index, so
  the patch was committed with an alternate index and local `master` was
  advanced to `d626def` (`chore: expose html id action provenance`). Direct
  `origin/master` push was rejected with `fetch first`; follow-up fetch and
  `gh pr create` failed DNS/API access. Review branch
  `codex/html-id-action-provenance` was pushed successfully and is available
  at `https://github.com/plasmate-labs/plasmate/pull/new/codex/html-id-action-provenance`.
  Remote merge remains blocked until fetch/API access works.

### 2026-05-14T18:08:19Z - Plasmate Improvements Automation

- Git sync: latest pull was retried from the automation worktree and failed
  because the linked worktree cannot open `FETCH_HEAD`. Retrying from
  `/Users/steve/Git/plasmate` failed DNS for `github.com`, so work continued
  from local `HEAD` `3227d52`.
- Market direction: current official docs still validate Plasmate's
  local-first action-state wedge. Playwright MCP keeps structured
  accessibility snapshots/ref selection central, Browserbase/Stagehand keeps
  pushing validated cached actions, and Cloudflare Browser Run/WebMCP keeps
  broadening hosted browser interaction surfaces. The roadmap should continue
  improving portable SOM/action fidelity across the existing repo surface
  before pursuing hosted browser infrastructure.
- Code changes: Rust SOM select extraction now marks the browser-default first
  option as selected for single-select controls without explicit `selected`,
  propagates disabled `<optgroup>` state to child option summaries, and
  preserves explicit select `size`.
- Contract changes: JSON Schema/SOM spec, Python/Node parser package types,
  Python/Node/Go SDK types, action-plan helpers, Browser Use, LangChain, and
  Vercel AI prompt renderers now carry select `selected_values` and `size`;
  option summaries now validate `disabled` and optgroup `group`.
- Docs changes: PRD, roadmap, website doc sources, and this running state now
  record the select-parity rationale and the next step to promote
  select-option parser/SDK/adapter parity into shared conformance fixtures.
- Verification: touched-file `rustfmt --check` passed; focused
  `test_select_option_state_and_groups_are_preserved` passed; full
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test --test
  som_compiler_test -- --test-threads=1` passed 66 tests; `cargo build` with
  the shared target passed; Python parser tests passed 68 tests; Python SDK
  query tests passed 36 tests; Go SDK tests passed; JSON schema parse,
  touched Python syntax compile, and `git diff --check` passed.
- Verification gaps: whole-crate `cargo fmt --check` still reports unrelated
  pre-existing formatting drift outside the touched files; Node package tests
  were unavailable because local `node_modules` directories are absent; and
  `node website/build.mjs` remains blocked because `marked` is not installed.
- Commit/push state: worktree commit was blocked by the linked-worktree
  `index.lock`, so the exact patch was applied and committed from
  `/Users/steve/Git/plasmate` as `def6534` (`chore: tighten select action
  parity`). Direct `origin/master` push was rejected with `fetch first`;
  `git fetch origin master` failed DNS for `github.com`; review branch
  `codex/select-action-parity` was pushed successfully; `gh pr create` failed
  because `api.github.com` is unreachable. Remote merge remains blocked until
  fetch/API access works.

### 2026-05-14T17:06:29Z - Plasmate Improvements Automation

- Git sync: latest pull was retried from the automation worktree and failed
  because the sandbox cannot open the linked primary checkout `FETCH_HEAD`.
  Retrying `git fetch --all --prune` from `/Users/steve/Git/plasmate` failed
  DNS for `github.com` (`ssh: Could not resolve hostname github.com: -65563`),
  so work continued from local `HEAD` `8b78d48`.
- Market direction: current docs and trend research still favor Plasmate's
  local-first action-state wedge. Playwright MCP uses structured
  accessibility snapshots with snapshot-scoped refs, Browserbase/Stagehand is
  pushing validated action caching, and Cloudflare Browser Run/WebMCP is
  widening hosted MCP/CDP/browser-native interaction. The roadmap should keep
  improving compact local SOM/action fidelity before hosted browser scale.
- Code changes: Rust SOM select extraction now follows browser option-value
  defaults when an `<option>` omits `value`, preserves disabled option state,
  preserves optgroup labels on option summaries, and exposes
  `attrs.selected_values` for multi-select controls while keeping the existing
  first selected `attrs.value`.
- Docs changes: PRD, roadmap, and website doc sources now record the
  select-option state rationale and next step to promote option value fallback,
  option disabled state, optgroup grouping, and multi-select values into shared
  Rust/parser/SDK and adapter fixtures.
- Verification: `rustfmt --check src/som/compiler.rs tests/som_compiler_test.rs`
  passed; focused `test_select_option_state_and_groups_are_preserved` passed;
  full `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test --test
  som_compiler_test -- --test-threads=1` passed 66 tests; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; and `git diff --check` passed.
- Verification gap: `node website/build.mjs` is still blocked because this
  worktree lacks the `marked` package (`ERR_MODULE_NOT_FOUND`), so generated
  website HTML was not refreshed.
- Commit/push state: implementation commit `595454b` was created from the
  primary checkout and pushed to
  `origin/codex/select-option-action-state`; follow-up documentation commits
  were pushed to the same branch. Direct `origin/master` push was rejected with
  `fetch first`; follow-up `git fetch origin master` failed DNS for
  `github.com`, and `gh pr create` could not reach `api.github.com`. Remote
  merge remains blocked until fetch/API access works.

### 2026-05-14T16:04:26Z - Plasmate Improvements Automation

- Git sync: latest pull was retried from the automation worktree and still
  failed because the sandbox cannot open the linked primary checkout
  `FETCH_HEAD`. Retrying from `/Users/steve/Git/plasmate` failed DNS for
  `github.com`, so work continued from local `HEAD` `0413345`.
- Market direction: Playwright MCP still centers structured accessibility
  snapshots with snapshot-scoped refs, Browserbase/Stagehand is emphasizing
  validated action caching, and Cloudflare Browser Run/WebMCP is widening
  hosted MCP/CDP surfaces. Plasmate should keep deepening local SOM fidelity
  rather than pivoting into hosted browser infrastructure.
- Code changes: Rust SOM text extraction is now stylesheet-visibility aware
  for visible parent text, interactive names, accessible label indexing,
  select options, list items, table captions, and table cells. This closes the
  hidden-descendant follow-up from the prior run.
- Docs changes: PRD, roadmap, and website doc sources now record the
  hidden-descendant rationale and the next conformance step: promote these
  cases into parser/SDK and adapter fixtures so hidden text filtering stays
  synchronized outside Rust.
- Verification: `rustfmt --check src/som/compiler.rs tests/som_compiler_test.rs`
  passed; focused hidden-descendant SOM test passed; full
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test --test
  som_compiler_test -- --test-threads=1` passed 65 tests; and
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings.
- Verification gap: whole-crate `cargo fmt --check` still reports unrelated
  pre-existing formatting drift in AWP/proxy/main/MCP files, matching earlier
  automation runs.

### 2026-05-14T15:05:46Z - Plasmate Improvements Automation

- Git sync: latest pull was retried from the automation worktree and failed
  before network access because the sandbox cannot open the primary checkout
  worktree `FETCH_HEAD`. Work continued from local `HEAD` `86df582`.
- Market direction: current docs still favor reusable structured action menus
  over a hosted-browser pivot. Playwright MCP uses accessibility snapshots
  with snapshot-scoped refs, Stagehand/Browserbase emphasize validated cached
  actions, and Cloudflare Browser Run/WebMCP is expanding hosted MCP/CDP
  browser surfaces. Plasmate's sticky wedge remains precise local SOM action
  fidelity.
- Code changes: Rust SOM now preserves native `<button value>` for submitter
  identity, normalizes invalid native `<button type>` values to browser-default
  `submit`, prevents stylesheet-hidden nested controls from entering the
  interactive child action surface, and propagates inert host state into
  shadow-root actions.
- Docs changes: PRD and roadmap now record the submitter-fidelity/hidden-action
  rationale plus the next conformance step. A follow-up is documented to apply
  stylesheet visibility filtering to descendant text extraction, because this
  run found hidden descendant text can still appear inside parent paragraph
  text even though the hidden control is no longer actionable.
- Verification: `rustfmt --check src/som/compiler.rs tests/som_compiler_test.rs`
  passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test
  --test som_compiler_test -- --test-threads=1` passed 64 tests with existing
  warnings; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; and `git diff --check` passed.
- Verification gap: `node website/build.mjs` could not run because this
  worktree lacks the `marked` package (`ERR_MODULE_NOT_FOUND`), so generated
  website docs were not refreshed in this run.

### 2026-05-14T14:05:23Z - Plasmate Improvements Automation

- Git sync: latest pull was retried from the automation worktree and failed
  because the sandbox cannot open the primary checkout worktree `FETCH_HEAD`.
  Retrying from the primary checkout reached SSH DNS and still failed with
  `ssh: Could not resolve hostname github.com: -65563`. Local `master`
  remains ahead of stale `origin/master`; direct `origin/master` push reached
  GitHub but was rejected with `fetch first`. Review branch
  `codex/graphical-submitter-action-cues` was pushed successfully at
  `ec77fd4`; `gh pr create` failed because `api.github.com` is unreachable.
  Remote merge remains blocked until GitHub fetch/API access works.
- Market direction: current docs still favor local-first action-state fidelity
  over a hosted browser-cloud pivot. Playwright MCP uses structured
  accessibility snapshots with snapshot-scoped refs, Stagehand/Browserbase is
  marketing cached action validation for repeated workflows, and Cloudflare
  Browser Run/WebMCP is widening hosted MCP/CDP browser access. The sticky
  Plasmate move remains exact, portable SOM action menus.
- Code changes: Rust SOM now maps `input type="image"` to an actionable
  button instead of a text input, adds `button_type` to input-backed
  submitters (`submit`, `button`, `reset`, and `image`), resolves graphical
  submitter labels from `alt`, and preserves `alt`/`src` context for
  icon-only form buttons.
- Docs changes: PRD and roadmap docs now record the graphical submitter
  rationale and next step to promote these cases into shared manifest and
  adapter conformance. Website doc sources were updated alongside root docs.
- Verification: `rustfmt --check src/som/compiler.rs tests/som_compiler_test.rs`
  passed; focused graphical submitter test passed; full `som_compiler_test`
  suite passed 61 tests; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `node website/build.mjs` rebuilt 39 docs
  pages; `git diff --check` passed. Whole-crate `cargo fmt --check` still
  reports unrelated pre-existing formatting drift in AWP/proxy/main files.

### 2026-05-14T13:16:38Z - Plasmate Improvements Automation

- Git sync: requested latest pull was retried from the automation worktree and
  still failed because the sandbox cannot open the primary checkout worktree
  `FETCH_HEAD`. `gh` is installed, but the configured token is invalid, so PR
  creation and remote merge remain blocked until GitHub auth/network access is
  repaired.
- Market direction: current docs still validate local action-state fidelity as
  the sticky wedge. Playwright MCP uses accessibility snapshots with
  snapshot-scoped refs, Stagehand `observe()` returns cacheable actions, and
  Browserbase/Cloudflare Browser Run/WebMCP continue pushing validated action
  replay and typed browser interaction. Plasmate should keep making local SOM
  action menus safer to reuse instead of pivoting into hosted browser fleets.
- Code changes: Rust SOM now preserves native `inert` state and inherited
  inert context for nested interactive controls. Python/Node parser packages,
  Python/Node/Go SDKs, Browser Use, LangChain, and Vercel AI action-plan
  surfaces expose `inert`, mark inert targets unavailable with
  `blocked_reason="inert"`, and keep deterministic `cache_key` values stable.
- Fixture and docs changes: `015-action-state` and the shared
  action-availability manifest now assert inert gating. SOM schema/spec,
  integration docs, PRD, roadmap, website docs, and this running state were
  updated with the rationale and next step to promote inert cases into broader
  conformance.
- Verification: JSON validation passed; Python parser, Python SDK, Browser
  Use, LangChain, and Go SDK focused manifest tests passed; `rustfmt --check`
  on the touched Rust files passed; focused SOM compiler test passed;
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed; and
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test --lib` passed
  256 tests.
- Verification gaps: the full action-manifest script stopped at Node parser
  setup because `vitest` is not installed and local Node `node_modules`
  directories are absent. `node website/build.mjs` also failed because
  `marked` is not installed. A first cargo run without the shared target failed
  trying to download `rusty_v8` while DNS/network access was unavailable.

### 2026-05-14T12:08:02Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted from the automation worktree
  and failed before network access because the sandbox could not open the
  worktree `FETCH_HEAD` under the primary checkout. Retrying from the primary
  checkout reached SSH resolution and failed with `ssh: Could not resolve
  hostname github.com: -65563`. The repo still has no `main` branch; local
  `master` remains ahead of stale `origin/master`, and remote merge is blocked
  until GitHub fetch/push works.
- Market direction: current official docs continue to reinforce local
  action-menu fidelity as the sticky wedge. Playwright MCP uses accessibility
  snapshots and snapshot-scoped refs; Stagehand v3 documents `observe()` for
  discovering, validating, and caching executable actions; Browser Use Cloud
  and AWS AgentCore emphasize managed sessions, profiles, proxies, and replay.
  Plasmate should keep deepening portable local SOM/action state rather than
  pivoting into hosted browser infrastructure.
- Code changes: Rust SOM now maps ARIA `slider` and `spinbutton` roles to
  actionable `text_input` targets and ARIA `option` to an actionable `button`
  target, closing three custom-control gaps for SaaS numeric settings and
  listbox choices.
- Fixture and docs changes: `016-action-semantics` now asserts slider,
  spinbutton, and option action-role coverage with ARIA value/selected state.
  SOM spec, generated website docs, PRD, roadmap, and this running state were
  updated with the rationale and next conformance step.
- Verification: `rustfmt --check src/som/compiler.rs tests/som_compiler_test.rs`
  passed; focused SOM role tests passed; `cargo test --test som_compiler_test
  -- --test-threads=1` passed 61 tests; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 256 tests; `./scripts/action-manifest-conformance.sh --quick` and
  `--full` passed; `node website/build.mjs` rebuilt 39 pages; `git diff
  --check` passed.
- Verification gap: full `cargo test -- --test-threads=1` again reached
  `tests/awp_integration_test.rs` and failed because this sandbox cannot spawn
  the test subprocess (`Operation not permitted`). The failure matches the
  known environment restriction from earlier runs; the changed SOM/compiler and
  shared adapter surfaces passed.
- Commit/push state: local commit `b1e109b` (`chore: expand aria action role
  coverage`) was created on `master` and pushed to
  `origin/codex/aria-action-role-coverage`. Direct push to `origin/master` was
  rejected with `fetch first`, and follow-up `git fetch` plus `gh pr create`
  were blocked by DNS/API connectivity. Merge to the remote default branch
  remains blocked until the remote can be fetched/rebased or a PR can be opened
  from the pushed branch.

### 2026-05-14T11:07:00Z - Plasmate Improvements Automation

- Git sync: requested latest pull was retried from the primary checkout, but
  SSH DNS again could not resolve `github.com` (`ssh: Could not resolve
  hostname github.com: -65563`). A direct `git push origin master` reached
  GitHub and was rejected with `fetch first`, confirming remote `master` has
  work not available locally. This repo still has no `main` branch; local
  `master` is ahead of stale `origin/master` by the automation commits, and
  remote push/merge remains blocked until `git fetch` can complete.
- Local continuation: after the upload-affordance branch push, additional
  submit-button override edits were present in the working tree. They align
  with the existing form-submission-context direction, so they were preserved
  and carried forward rather than reverted.
- Code changes: Rust SOM now preserves submit-button override cues:
  normalized `button_type`, `formaction`, `formmethod`, `formenctype`,
  `formtarget`, and boolean `formnovalidate`.
- Parser/SDK/adapter changes: Python/Node parser packages, Python/Node/Go
  SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces expose the
  submit override cues so cached submit actions can validate endpoint, method,
  encoding, target, and validation mode before replay.
- Fixture and docs changes: the shared action-availability SOM/expected
  manifest now asserts submit-button override context. PRD, roadmap, generated
  website docs, and this running state were updated with the next broader
  conformance step.
- Verification: JSON validation passed; `rustfmt --check src/som/compiler.rs
  tests/som_compiler_test.rs` passed; `gofmt` was applied to touched Go files;
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings; `cargo test --test som_compiler_test -- --test-threads=1`
  passed 61 tests; `./scripts/action-manifest-conformance.sh --quick` and
  `--full` passed; Node parser, Node SDK, and Vercel AI builds passed; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed.
- Commit/push state: local commits through `1641d0b` (`docs: record submit
  override push block`) are present on `master` and were pushed to remote
  branch `codex/upload-action-cues`. Direct push to `master` is still rejected
  with `fetch first`; GitHub connector PR creation returned 403 (`Resource not
  accessible by integration`), and `gh pr create` could not reach
  `api.github.com`. Checkout is clean except pre-existing untracked `.agents/`.

### 2026-05-14T10:13:09Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted from the primary checkout, but
  SSH DNS still could not resolve `github.com` (`ssh: Could not resolve
  hostname github.com: -65563`). This run continued from local `master`, which
  was already ahead of `origin/master` by the prior upload-affordance commit.
  The repo still has no `main` branch; `origin/HEAD` points at
  `origin/master`.
- Market direction: current Playwright MCP docs keep structured accessibility
  snapshots and snapshot-scoped refs as the action unit, while Stagehand and
  Browserbase docs emphasize `observe()` planning, local/server action
  caching, DOM-hash validation, and session replay. The startup direction
  remains local-first SOM/action-state fidelity rather than a hosted browser
  infrastructure pivot. The new sticky gap is submission-context validation:
  cached SaaS actions need to know whether a target still belongs to the same
  checkout, upload, or settings form before replay.
- Code changes: Rust SOM form regions now preserve submission context:
  `target`, `enctype`, `novalidate`, `accept-charset`, and form-level
  `autocomplete`, alongside existing `action` and `method`. JSON Schema and
  the SOM spec accept those fields.
- Parser/SDK/adapter changes: Python/Node parser packages, Python/Node/Go
  SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose
  `form_action`, `form_method`, `form_target`, `form_enctype`,
  `form_novalidate`, `form_accept_charset`, and `form_autocomplete` without
  changing deterministic `cache_key` inputs.
- Fixture and docs changes: the shared action-availability SOM/expected
  manifest now asserts form submission context across parser, SDK, and
  framework outputs. PRD, roadmap, SOM spec/schema, adapter docs, SDK/parser
  docs, generated website docs, and this running state were updated with the
  rationale and next conformance step.
- Verification: JSON validation passed through the conformance tests;
  `rustfmt --check src/som/compiler.rs src/som/types.rs src/som/filter.rs
  src/som/diff_tests.rs tests/som_compiler_test.rs` passed; `gofmt` was
  applied to touched Go files; Python syntax checks passed; focused Rust form
  submission test passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 256 tests; `cargo test --test som_compiler_test -- --test-threads=1`
  passed 61 tests; `./scripts/action-manifest-conformance.sh --quick` and
  `--full` passed; `node website/build.mjs` rebuilt 39 pages; `git diff
  --check` passed.
- Verification gap: full `cargo test -- --test-threads=1` again reached the
  AWP integration suite and failed because this sandbox cannot spawn the test
  subprocess (`Operation not permitted`) in `tests/awp_integration_test.rs`.
  The library, SOM compiler, build, and cross-adapter manifest suites passed.
- Commit/push state: local implementation/docs commit was created from the
  primary checkout. A push attempt reached GitHub but was rejected because the
  remote contains work not yet fetched (`fetch first`); follow-up fetch/pull
  attempts were blocked by intermittent `github.com` DNS resolution. Push and
  merge to the remote default branch remain blocked until the remote can be
  fetched and rebased.

### 2026-05-14T09:13:38Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted from the primary checkout, but
  SSH DNS still could not resolve `github.com` (`ssh: Could not resolve
  hostname github.com: -65563`). This run continued from local `master`
  commit `f7af658`; the repo has no `main` branch and `origin/HEAD` points at
  `origin/master`.
- Market direction: current Playwright MCP, Browserbase/Stagehand, and Browser
  Use direction still favors compact, fresh, replay-validated action targets.
  Production SaaS stickiness increasingly depends on forms that include file
  evidence, screenshots, resumes, and media uploads, so Plasmate should deepen
  portable local action metadata rather than pivoting into hosted browser
  infrastructure.
- Code changes: Rust SOM now preserves upload action cues: native `accept`,
  `capture`, and input `multiple`. Empty `capture` compiles as `true`, and
  JSON Schema/SOM spec docs accept `attrs.accept`, `attrs.capture`, and
  `attrs.multiple`.
- Parser/SDK/adapter changes: Python/Node parser packages, Python/Node/Go
  SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose
  `name`, `accept`, `capture`, and `multiple`. Field `name` remains part of
  deterministic `cache_key` inputs so similarly labelled upload controls do
  not collapse into the same cached action target.
- Fixture and docs changes: the shared action-availability SOM/expected
  manifest now asserts upload constraints and native multiple-selection state.
  PRD, roadmap, SOM spec/schema, adapter docs, generated website docs, and
  this running state were updated with rationale and next conformance steps.
- Verification: JSON validation passed; `rustfmt --check src/som/compiler.rs
  tests/som_compiler_test.rs` passed; `gofmt` was applied to touched Go files;
  focused Rust upload test passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 256 tests; `cargo test --test som_compiler_test -- --test-threads=1`
  passed 60 tests; `./scripts/action-manifest-conformance.sh --quick` and
  `--full` passed; Node parser, Node SDK, and Vercel AI builds passed; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed.
- Verification gap: full `cargo test -- --test-threads=1` reached the AWP
  integration suite and failed because this sandbox cannot spawn the test
  subprocess (`Operation not permitted`) in `tests/awp_integration_test.rs`.
  The library and SOM compiler suites passed before/after that sandbox-only
  failure.
- Commit/push state: local commit `9457ab2` (`chore: expose upload action
  cues`) was created and pushed to branch `codex/upload-action-cues`. Direct
  push to `origin/master` was rejected because remote `master` advanced by one
  coverage scorecard commit (`a5a44a9`), and local `git fetch`/`pull --rebase`
  remained blocked by DNS resolution for `github.com`. GitHub connector compare
  confirmed the branches only diverge by that coverage-file commit plus this
  upload-affordance commit. PR creation through the connector failed with
  `403 Resource not accessible by integration`, and `gh pr create` could not
  reach `api.github.com`, so merge to `master` remains blocked outside the
  pushed branch.

### 2026-05-14T08:14:37Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted from the primary checkout, but
  SSH DNS still could not resolve `github.com` (`ssh: Could not resolve
  hostname github.com: -65563`). This run continued from local `master`
  commit `f518449`; the repo has no `main` branch and `origin/HEAD` points at
  `origin/master`.
- Market direction: current Playwright MCP docs still center fresh structured
  accessibility snapshots and snapshot-scoped refs, while Browserbase/
  Stagehand emphasize `observe()` plus cached repeatable actions. Recent
  browser-agent commentary also continues to favor compact DOM/accessibility
  action menus over full-DOM prompts. Plasmate should keep deepening portable,
  local SOM action state rather than pivoting into hosted browser
  infrastructure.
- Code changes: Rust SOM now preserves text-entry affordance cues:
  `spellcheck`, `autocapitalize`, `dirname`, and ARIA `aria-placeholder`.
  JSON Schema and SOM spec docs accept the corresponding `attrs.spellcheck`,
  `attrs.autocapitalize`, `attrs.dirname`, and `attrs.aria.placeholder`
  fields.
- Parser/SDK/adapter changes: Python/Node parser packages, Python/Node/Go
  SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose
  `spellcheck`, `autocapitalize`, `dirname`, and `aria_placeholder` without
  changing deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability manifest and
  `016-action-semantics` conformance fixture now assert text-entry affordance
  cues. PRD, roadmap, adapter docs, SDK/parser docs, generated website docs,
  and this running state were updated with rationale and next conformance
  steps.
- Verification: JSON validation passed; `rustfmt --check src/som/compiler.rs
  tests/som_compiler_test.rs` passed; focused Rust text-entry and
  action-semantics tests passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 256 tests; `./scripts/action-manifest-conformance.sh --quick` and
  `--full` passed; `node website/build.mjs` rebuilt 39 pages; `git diff
  --check` passed. Temporary Node dependency symlinks to the primary checkout
  were removed after verification.
- Commit/push state: implementation/docs commit `72b0ff5`
  (`chore: expose text entry action cues`) was created from the primary
  checkout and pushed to `origin/master` after verification. There is no
  `main` branch in this repo; `origin/HEAD` points at `origin/master`.

### 2026-05-14T07:12:56Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted from the primary checkout, but
  SSH DNS could not resolve `github.com` (`ssh: Could not resolve hostname
  github.com: -65563`). This run continued from the locally available shared
  `master` / `origin/master` / detached automation HEAD state `ce747ce`.
- Market direction: current Playwright MCP docs still make structured
  accessibility snapshots with snapshot-scoped refs the interaction unit, while
  Stagehand/Browserbase emphasize `observe()` plus local/server action caching
  and session observability. Plasmate should keep deepening the local
  SOM/action-menu contract rather than pivoting into hosted browser
  infrastructure.
- Code changes: Rust SOM now preserves ARIA set-position cues:
  `aria-level`, `aria-posinset`, and `aria-setsize`; JSON Schema and SOM spec
  docs accept the corresponding `attrs.aria.level`, `posinset`, and `setsize`
  keys.
- Parser/SDK/adapter changes: Python/Node parser packages, Python/Node/Go
  SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose
  `level`, `posinset`, and `setsize` without changing deterministic
  `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert set-position cues. PRD, roadmap, SDK/adapter docs,
  generated website docs, and this running state were updated with rationale
  and next conformance steps.
- Verification: JSON validation passed; `rustfmt --check src/som/compiler.rs`
  passed; focused Rust set-position test passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 255 tests; `./scripts/action-manifest-conformance.sh --quick`
  passed; `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed. Temporary
  Node dependency symlinks to the primary checkout were removed after
  verification.
- Commit/push state: implementation/docs commit `0b06233`
  (`chore: expose aria set position cues`), running-state commit `1b9cb2f`
  (`docs: record aria set position state`), and this push-block note commit
  were created locally in the primary checkout. Push to `origin/master` was
  attempted four times and is blocked by DNS resolution for `github.com`.
  There is no `main` branch in this repo; `origin/HEAD` points at
  `origin/master`.

### 2026-05-14T06:14:32Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first from the automation
  worktree, but the sandbox could not write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate48/FETCH_HEAD`
  (`Operation not permitted`). A primary-checkout fetch also intermittently
  failed DNS for `github.com` over SSH, so this run continued from local
  `master` / `origin/master` state `1bf2ca7`. The final push to remote
  `master` succeeded.
- Market direction: current browser-agent positioning still validates
  Plasmate's local-first action-menu wedge. Playwright MCP-style structured
  snapshots, Stagehand/Browserbase cached actions, and Firecrawl/Browser Use
  managed sessions all reward compact state that can be validated before
  replay. The product should keep deepening portable SOM/action state rather
  than pivoting into hosted browser infrastructure.
- Code changes: Rust SOM now preserves ARIA widget affordance cues:
  `aria-readonly`, `aria-multiline`, and `aria-multiselectable`. ARIA
  read-only also promotes top-level `attrs.readonly` so custom read-only
  textboxes are treated like native read-only controls.
- Parser/SDK/adapter changes: Python/Node parser packages, Python/Node/Go
  SDKs, Browser Use, LangChain, and Vercel AI action-plan surfaces now expose
  `readonly`, `multiline`, and `multiselectable`; ARIA read-only targets are
  unavailable with `blocked_reason="readonly"` while deterministic action
  `cache_key` values stay stable.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert ARIA read-only gating, multiline text entry, and
  multiselectable widget cues. PRD, roadmap, schema/spec docs, adapter docs,
  SDK docs, generated website docs, and this running state were updated with
  rationale and next conformance steps.
- Verification: JSON validation passed; focused Rust widget-state test passed;
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings; `cargo test --lib -- --test-threads=1` passed 254 tests;
  `./scripts/action-manifest-conformance.sh --quick` passed;
  `./scripts/action-manifest-conformance.sh --full` passed; Vercel AI
  `npm test` passed; `node website/build.mjs` rebuilt 39 pages; `git diff
  --check` passed.
- Commit/push state: commit `8aae908` (`chore: expose aria widget action
  cues`) was pushed to remote `master`. There is no `main` branch in this
  repo; `origin/HEAD` points at `origin/master`. The unrelated untracked
  `.agents/` directory in the primary checkout was left untouched.

### 2026-05-14T05:13:35Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The detached automation
  worktree could not write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate47/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout failed DNS
  for `github.com` over SSH (`ssh: Could not resolve hostname github.com:
  -65563`), so this run continued from locally available `master` /
  `origin/master` state `201bc2e`.
- Market direction: current competitor docs continue to validate Plasmate's
  local-first action-menu wedge. Playwright MCP centers fresh structured
  accessibility snapshots with refs, Stagehand/Browserbase keep emphasizing
  observable cached actions, and Firecrawl/Browser Use keep selling managed
  browser sessions and profiles. The product should deepen portable local SOM
  action state rather than pivot into hosted browser infrastructure.
- Code changes: Rust SOM now preserves native range/value constraints
  (`min`, `max`, `step`) plus ARIA range/orientation/sort cues
  (`aria-valuemin`, `aria-valuemax`, `aria-valuenow`, `aria-valuetext`,
  `aria-orientation`, `aria-sort`). Parser packages, Python/Node/Go SDKs,
  Browser Use, LangChain, and Vercel AI action-plan surfaces expose `min`,
  `max`, `step`, `orientation`, `sort`, `valuemin`, `valuemax`, `valuenow`,
  and `valuetext` without changing deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert a range target with bounds and ARIA value state. PRD,
  roadmap, SOM schema/spec, SDK docs, adapter docs, generated website docs,
  and this running state were updated with rationale and next conformance
  steps.
- Verification: JSON validation for schema and shared fixtures passed;
  `rustfmt --check src/som/compiler.rs` passed; focused Rust range/sort test
  passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 253 tests; Python parser tests passed 68 tests; Python SDK query
  tests passed 36 tests; `./scripts/action-manifest-conformance.sh --quick`
  passed; `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed.
- Verification gaps: full `cargo fmt --check` still reports pre-existing
  formatting drift in unrelated Rust files (`src/awp/handler.rs`,
  `src/bench/runner.rs`, `src/network/proxy.rs`, `src/main.rs`, and
  `src/mcp/tools.rs`). Temporary Node dependency symlinks to the primary
  checkout were removed after verification.

### 2026-05-14T04:10:30Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  could not write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate46/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout could not
  resolve `github.com` over SSH (`ssh: Could not resolve hostname github.com:
  -65563`), so this run continued from locally available `master` /
  `origin/master` state `85792a3` plus local unpushed master commits through
  `2e08931`.
- Market direction: current official docs continue to validate Plasmate's
  local-first action-menu wedge. Playwright MCP centers structured
  accessibility snapshots with snapshot-scoped refs, Stagehand v3 documents
  `observe()` planning plus local/server action caching, Firecrawl packages
  managed browser sandboxes through API/CLI/SDK/MCP, and Browser Use Cloud
  sells profiles plus direct CDP sessions. The product should keep avoiding a
  hosted-browser pivot and make local compact targets richer and easier to
  validate.
- Code changes: Rust SOM now preserves link navigation cues: `target`, `rel`,
  and `download`. Parser packages, Python/Node/Go SDKs, Browser Use,
  LangChain, and Vercel AI action-plan surfaces expose those fields without
  changing deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert link target/rel/download cues across parser, SDK, and
  framework outputs. PRD, roadmap, SOM schema/spec, adapter docs, SDK docs,
  and this running state were updated with rationale and next conformance
  steps.
- Verification: JSON validation for the schema and shared fixtures passed;
  `rustfmt --check src/som/compiler.rs` passed; focused Rust link navigation
  test passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 252 tests; `./scripts/action-manifest-conformance.sh --quick`
  passed; `./scripts/action-manifest-conformance.sh --full` passed; `git diff
  --check` passed. Temporary Node dependency symlinks to the primary checkout
  were removed after verification.
- Commit/push state: implementation commit `959a9f0`
  (`chore: expose link navigation action cues`) was pushed to remote `master`.
  There is no `main` branch in this repo; `origin/HEAD` points at
  `origin/master`.

### 2026-05-14T03:08:42Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  could not write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate45/FETCH_HEAD`
  (`Operation not permitted`), so this run continued from local
  `master` / `origin/master` state `85792a3`. The linked automation worktree
  also could not create its git `index.lock`, so the patch was mirrored into
  the primary checkout for committing.
- Market direction: current official docs still validate the local action-menu
  wedge. Playwright MCP keeps structured accessibility snapshots and
  snapshot-scoped refs as the interaction unit, Stagehand v3 documents
  `observe()` actions plus local/server caching for repeated workflows,
  Firecrawl packages managed browser sandbox/session execution through API,
  CLI, SDKs, and MCP, and Browser Use Cloud sells persistent profiles plus
  direct CDP sessions. Plasmate should keep avoiding a hosted-browser pivot
  and deepen portable local SOM relationship context.
- Code changes: Rust SOM now preserves ARIA relationship cues:
  `aria-owns`, `aria-flowto`, and `aria-details`. Parser packages,
  Python/Node/Go SDKs, Browser Use, LangChain, and Vercel AI action-plan
  surfaces expose `owns`, `flowto`, and `details` without changing
  deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert owns/flowto/details cues across parser, SDK, and
  framework outputs. PRD, roadmap, SOM schema/spec, adapter docs, SDK docs,
  generated website docs, and this running state were updated with rationale
  and next conformance steps.
- Verification: `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo
  build` passed with existing warnings; focused Rust ARIA relationship test
  passed; `cargo test --lib -- --test-threads=1` passed 251 tests;
  `./scripts/action-manifest-conformance.sh --quick` passed;
  `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; JSON parsing for the schema and shared
  fixtures passed; `rustfmt --check src/som/compiler.rs` passed; `git diff
  --check` passed. Temporary Node dependency symlinks to the primary checkout
  were removed after verification.
- Verification gaps: direct Rust test/build without the shared target cache
  failed because sandboxed DNS could not download the `rusty_v8` archive. Full
  `cargo test -- --test-threads=1` still fails only in
  `tests/awp_integration_test.rs` because sandboxed local listener setup
  returns `Operation not permitted`, matching prior runs. Full `cargo fmt
  --check` still reports pre-existing formatting drift in unrelated Rust files.
- Commit/push state: implementation commit `d9f66a2`
  (`chore: expose aria relationship action cues`) was created locally in
  `/Users/steve/Git/plasmate`. Push to `origin/master` failed because SSH could
  not resolve `github.com` (`-65563`). There is no `main` branch in this repo;
  `origin/HEAD` points at `origin/master`.

### 2026-05-14T02:16:00Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate44/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout could not
  resolve `github.com` over SSH (`ssh: Could not resolve hostname github.com:
  -65563`), so this run continued from locally available `master` /
  `origin/master` state `c39fdf9`.
- Market direction: current docs and product launches still validate the
  local-first action-menu wedge. Playwright MCP exposes structured
  accessibility snapshots with snapshot-scoped refs, Firecrawl Interact turns
  scrape sessions into stateful browser interaction, Browser Use/AWS browser
  profiles make persistent hosted sessions easier to buy, and MDN/Chrome now
  document native Popover API and `commandfor`/`command` relationships. The
  sticky Plasmate move remains richer portable SOM action state, not a hosted
  browser pivot.
- Code changes: Rust SOM now preserves native popover and declarative command
  cues: `popovertarget`, `popovertargetaction`, `commandfor`, `command`, and
  `popover`. Parser packages, Python/Node/Go SDKs, Browser Use, LangChain, and
  Vercel AI action-plan surfaces expose `popovertarget`,
  `popovertargetaction`, `commandfor`, and `command` without changing
  deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert popover/command cues across parser, SDK, and framework
  outputs. PRD, roadmap, SOM schema/spec, adapter docs, SDK docs, generated
  website docs, and this running state were updated with rationale and next
  conformance steps.
- Verification: `cargo build` passed with existing warnings; focused Rust
  popover/command test passed; JSON/schema validation passed; `rustfmt --check
  src/som/compiler.rs` passed; `cargo test --lib -- --test-threads=1` passed
  250 tests; `./scripts/action-manifest-conformance.sh --quick` passed;
  `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed.
- Verification gap: full `cargo test -- --test-threads=1` still fails only in
  `tests/awp_integration_test.rs` because sandboxed local listener setup
  returns `Operation not permitted`, matching the existing environment
  limitation from prior runs.
- Commit/push state: implementation commit `d91d6b4`
  (`chore: expose popover command action cues`) was pushed to remote `master`.
  There is no `main` branch in this repo; `origin/HEAD` points at
  `origin/master`.

### 2026-05-14T01:11:14Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate43/FETCH_HEAD`
  (`Operation not permitted`), so this run continued from local
  `master`/`origin/master` state `9a04205`. Direct network-backed Cargo builds
  also could not resolve GitHub for the `rusty_v8` archive, so verification
  used the shared primary checkout target cache.
- Market direction: current docs still validate Plasmate's local-first action
  contract rather than a hosted-browser pivot. Playwright MCP keeps
  accessibility refs scoped to fresh snapshots, Stagehand emphasizes
  `observe()` plus local/server action caching, and Firecrawl/Browser Use keep
  selling persistent browser sessions and CDP/profile state. The sticky move
  remains richer portable SOM action state across SDKs and adapters.
- Code changes: Rust SOM now preserves ARIA live-region cues:
  `aria-busy`, `aria-live`, `aria-atomic`, and `aria-relevant`. Parser
  packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel AI
  action-plan surfaces expose `busy`, `live`, `atomic`, and `relevant` without
  changing deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert live-region cues across parser, SDK, and framework
  outputs. PRD, roadmap, SOM schema/spec, adapter docs, SDK docs, generated
  website docs, and this running state were updated with rationale and next
  conformance steps.
- Verification: `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; focused
  `cargo test som::compiler::tests::test_live_region_action_cues_are_preserved`
  passed; `cargo test --lib -- --test-threads=1` passed 249 tests;
  `./scripts/action-manifest-conformance.sh --quick` passed;
  `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; `rustfmt --check src/som/compiler.rs`
  passed; `git diff --check` passed. Full `cargo fmt --check` still reports
  pre-existing formatting drift in unrelated Rust files.
- Commit/push state: implementation commit `98acb34`
  (`chore: expose live region action cues`) was pushed to remote `master`.
  There is no `main` branch in this repo; `origin/HEAD` points at
  `origin/master`.

### 2026-05-14T00:11:36Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate42/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout could not
  resolve `github.com` over SSH, so this run continued from local
  `master`/`origin/master` state `4947076`; the final push later succeeded.
- Market direction: current docs still validate Plasmate's local-first action
  contract rather than a hosted-browser pivot. Playwright MCP keeps
  snapshot-scoped accessibility refs, Stagehand and Browserbase emphasize
  `observe()` plus action caching, and Firecrawl/Browser Use keep packaging
  persistent browser sessions. The sticky move remains richer portable SOM
  action state across SDKs and adapters.
- Code changes: Rust SOM now preserves `form`, `list`, and
  `aria-errormessage` as form ownership, datalist, and error-message
  relationship cues. Parser packages, Python/Node/Go SDKs, Browser Use,
  LangChain, and Vercel AI action-plan surfaces expose `form`, `list`, and
  `errormessage` without changing deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert form-relation cues across parser, SDK, and framework
  outputs. PRD, roadmap, SOM schema/spec, adapter docs, SDK docs, generated
  website docs, and this running state were updated with rationale and next
  conformance steps.
- Verification: `cargo build` passed; focused Rust compiler test passed;
  `cargo test --lib -- --test-threads=1` passed 248 tests; Python parser,
  Python SDK, Browser Use, LangChain, Go SDK, Node parser, Node SDK, Vercel AI,
  website docs build, `./scripts/action-manifest-conformance.sh --quick`, and
  `git diff --check` passed. Node/website checks used temporary symlinks to
  dependency installs in `/Users/steve/Git/plasmate`, then the symlinks were
  removed.
- Commit/push state: implementation commit `d7fddf2`
  (`chore: expose form relation action cues`) was pushed to remote `master`.
  There is no `main` branch in this repo; `origin/HEAD` points at
  `origin/master`.

### 2026-05-13T23:10:41Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate41/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout failed to
  resolve `github.com` over SSH (`ssh: Could not resolve hostname github.com:
  -65563`), so this run continued from locally available `master` /
  `origin/master` state `08e3a0d`.
- Market direction: current official docs and product pages still validate
  Plasmate's local-first action-menu wedge. Playwright MCP keeps structured
  accessibility snapshots and snapshot-scoped refs at the center of
  interaction, Stagehand/Browserbase emphasize `observe()` plus local/server
  action caching, and Firecrawl/Browser Use continue selling stateful browser
  sessions. The product response remains richer portable SOM action state
  across SDKs and adapters, not a hosted-browser pivot.
- Code changes: Rust SOM now preserves native `accesskey` plus ARIA
  `keyshortcuts` and `roledescription`. Parser packages, Python/Node/Go SDKs,
  Browser Use, LangChain, and Vercel AI action-plan surfaces expose
  `accesskey`, `keyshortcuts`, and `roledescription` without changing
  deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert keyboard/custom-role cues across parser, SDK, and
  framework outputs. PRD, roadmap, SOM schema/spec, adapter docs, SDK docs, and
  this running state were updated with the rationale and next conformance step.
- Verification: `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; focused
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo test
  som::compiler::tests::test_keyboard_and_custom_role_action_cues_are_preserved
  -- --nocapture` passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target
  cargo test --lib -- --test-threads=1` passed 247 tests; Python parser,
  Python SDK, Go SDK, Browser Use, and LangChain shared-manifest checks passed.
  `git diff --check` passed.
- Verification gaps: the full/quick action-manifest gate could not complete
  because Node parser tests require missing local `vitest`; `node
  website/build.mjs` could not regenerate website docs because the local
  `marked` package is missing. A direct worktree Rust build without the shared
  target cache also failed because blocked DNS prevented `rusty_v8` from
  downloading its prebuilt archive. `cargo fmt --check` reports unrelated
  pre-existing formatting drift in Rust files outside this change.
- Commit/push state: implementation commit `e2f6ff6`
  (`chore: expose keyboard action cues`) was pushed to remote `master`. There
  is no `main` branch in this repo; `origin/HEAD` points at `origin/master`.

### 2026-05-13T22:11:17Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate40/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout could not
  resolve `github.com` over SSH (`ssh: Could not resolve hostname github.com:
  -65563`), so this run continued from locally available `master` /
  `origin/master` state `6b8179c`.
- Market direction: current public docs and product positioning still validate
  Plasmate's local-first action-menu wedge. Playwright MCP centers fresh
  accessibility snapshots/refs, Stagehand and Browserbase emphasize cached
  observe/action replay, and Firecrawl/Browser Use keep packaging managed
  sessions. The near-term retention move remains richer portable SOM action
  state, not a hosted-browser pivot.
- Code changes: Rust SOM now preserves input-affordance cues:
  `inputmode`, `enterkeyhint`, `aria-autocomplete`, and
  `aria-activedescendant`. Parser packages, Python/Node/Go SDKs, Browser Use,
  LangChain, and Vercel AI action-plan surfaces now expose `inputmode`,
  `enterkeyhint`, `aria_autocomplete`, and `active_descendant` without changing
  deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert input modality and autocomplete-widget state across
  parser, SDK, and framework outputs. PRD, roadmap, SOM schema/spec, adapter
  docs, SDK docs, generated website docs, and this running state were updated
  with the rationale and next conformance step.
- Verification: `cargo build` passed with existing warnings;
  `cargo test --lib -- --test-threads=1` passed 246 tests; focused
  `cargo test --test som_compiler_test test_form_state_values_and_readonly_are_preserved
  -- --test-threads=1` passed; `./scripts/action-manifest-conformance.sh
  --quick` passed; `./scripts/action-manifest-conformance.sh --full` passed;
  `node website/build.mjs` rebuilt 39 pages; `git diff --check` passed. Full
  `cargo test -- --test-threads=1` still fails only in
  `tests/awp_integration_test.rs` because sandboxed local listener setup
  returns `Operation not permitted`.

### 2026-05-13T21:11:16Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate39/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout could not
  resolve `github.com` over SSH (`ssh: Could not resolve hostname github.com:
  -65563`), so this run continued from locally available branch state
  `codex/plasmate-improvements-2026-05-13-control-state` at `c6d39a7`.
- Market direction: current official docs and product pages still validate
  Plasmate's local-first action-state wedge. Playwright MCP snapshots expose
  fresh accessibility refs, Stagehand documents `observe()` and local/server
  action caching, Browserbase sells cached selectors plus observability, and
  Browser Use/Firecrawl continue to package browser sessions for repeated form
  work. The product response remains richer portable SOM/action menus, not a
  hosted-browser pivot.
- Code changes: Rust SOM now preserves form validation constraints
  (`minlength`, `maxlength`, `pattern`) and `aria-invalid`. Python/Node parser
  packages, Python/Node/Go SDKs, Browser Use, LangChain, and Vercel AI action
  plans now expose `autocomplete`, length constraints, `pattern`, and
  `invalid` without changing deterministic `cache_key` values.
- Fixture and docs changes: the shared action-availability SOM and expected
  manifest now assert autocomplete, length constraints, pattern, and invalid
  state across parser, SDK, and framework outputs. PRD, roadmap, SOM schema,
  SOM spec, adapter docs, SDK docs, generated website docs, and this running
  state were updated with the validation-state rationale and next conformance
  step.
- Verification: `cargo build` passed with existing warnings;
  `cargo test --lib -- --test-threads=1` passed 246 tests; focused
  `cargo test --test som_compiler_test test_form_state_values_and_readonly_are_preserved
  -- --test-threads=1` passed; `./scripts/action-manifest-conformance.sh
  --quick` passed; `./scripts/action-manifest-conformance.sh --full` passed;
  `node website/build.mjs` rebuilt 39 pages; `git diff --check` passed.
  Full `cargo test -- --test-threads=1` still fails only in
  `tests/awp_integration_test.rs` because sandboxed local listener setup
  returns `Operation not permitted`.
- Commit/push state: implementation commit `971ed7c`
  (`chore: expose form validation action cues`) was pushed to
  `codex/plasmate-improvements-2026-05-13-control-state`; local `master` was
  fast-forwarded to the same commit and pushed to remote `master`. The first
  `master` push attempt hit transient GitHub DNS failure, and the immediate
  retry succeeded. A follow-up state-only commit records the final push state
  and was pushed to both remote `master` and the automation branch.

### 2026-05-13T20:08:54Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate38/FETCH_HEAD`
  (`Operation not permitted`). This run continued from locally known
  `origin/master`/`HEAD` `0a02785`.
- Market direction: current official docs keep validating reusable action
  state over a hosted-browser pivot. Playwright MCP snapshots scope refs to a
  fresh accessibility snapshot, Stagehand `observe()` and action caching need
  validation before replay, and Firecrawl/Browser Use package persistent
  sessions around the same form-state drift problem. Plasmate's sticky response
  remains a local, portable SOM action contract across the broad SDK and
  adapter surface.
- Code changes: the Rust compiler now preserves native `readonly` state,
  textarea values, selected `<select>` values, and case-insensitive/trimmed
  ARIA booleans. Parser packages, Python/Node/Go SDKs, Browser Use, LangChain,
  and Vercel AI action-plan helpers now expose `readonly`, mark those targets
  unavailable with `blocked_reason="readonly"`, and keep deterministic
  `cache_key` values target-focused.
- Fixture and docs changes: the shared action-availability fixture now asserts
  read-only blockers and selected-option values. PRD, roadmap, SOM spec,
  integration docs, schema, and generated website docs were updated with the
  read-only/value-state rationale and next conformance direction.
- Verification: `./scripts/action-manifest-conformance.sh --quick` passed;
  `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target
  cargo build` passed with existing warnings; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target
  cargo test --lib -- --test-threads=1` passed 246 tests; focused
  `cargo test --test som_compiler_test test_form_state_values_and_readonly_are_preserved
  -- --test-threads=1` passed; `git diff --check` passed.
- Commit/push state: implementation commit `8a817ad`
  (`chore: preserve readonly action state`) was pushed to
  `codex/plasmate-improvements-2026-05-13-control-state` and fast-forwarded
  remote `master` from `0a02785` to `8a817ad`. A follow-up state-only commit
  records the successful push result.

### 2026-05-13T19:08:47Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate37/FETCH_HEAD`
  (`Operation not permitted`), and retrying from the primary checkout could
  not resolve `github.com` over SSH (`ssh: Could not resolve hostname
  github.com: -65563`). This run continued from locally known
  `origin/master`/`HEAD` `66656e3`.
- Market direction: fresh official-doc research keeps validating the
  local-first action-menu wedge. Playwright MCP snapshots bind refs to current
  accessibility state, Stagehand v3 documents local/Browserbase action caches
  that need validation, Firecrawl Interact and Browser Use Cloud package
  persistent profiles/sessions, and Cloudflare Browser Run/WebMCP is testing
  typed page-provided tools. The product response remains richer portable SOM
  action state, not a hosted-browser pivot.
- Code changes: the Rust SOM compiler and JSON Schema now preserve
  `aria-controls` and `aria-haspopup` inside `attrs.aria`, joining existing
  `aria-current` support. Python/Node parser packages, Python/Node/Go SDKs,
  Browser Use, LangChain, and Vercel AI action-plan helpers now expose
  `current`, `controls`, and `haspopup` while keeping deterministic
  `cache_key` generation target-focused.
- Fixture and docs changes: the shared action-availability SOM fixture and
  expected manifest now assert a current-page link plus a controlled popup
  select target across parser, SDK, and framework surfaces. PRD, roadmap,
  integration docs, generated website docs, SOM spec, and fixture docs now
  describe ARIA relationship state as the next action-menu reliability step.
- Verification: `./scripts/action-manifest-conformance.sh --quick` passed;
  `node website/build.mjs` rebuilt 39 pages; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target
  cargo build` passed with existing warnings; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target
  cargo test --lib -- --test-threads=1` passed 246 tests; focused
  `cargo test --test som_compiler_test test_aria_relationship_state_is_preserved_for_action_targets
  -- --test-threads=1` passed; `git diff --check` passed. Global
  `cargo fmt --check` still reports pre-existing formatting drift in unrelated
  Rust files, so only the touched Rust test was formatted.
- Commit/push state: implementation commit `b359020`
  (`chore: expose aria relationship action cues`) was pushed to
  `codex/plasmate-improvements-2026-05-13-control-state` and fast-forwarded
  remote `master` from `66656e3` to `b359020`. Follow-up state commits record
  the push outcome, and remote `master` has been fast-forwarded through this
  run.

### 2026-05-13T18:11:08Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate36/FETCH_HEAD`
  (`Operation not permitted`), and retrying from the primary checkout could
  not resolve `github.com` over SSH (`ssh: Could not resolve hostname
  github.com: -65563`). This run continued from locally known
  `origin/master`/`HEAD` `c9876d8`.
- Market direction: current docs continue to favor state-aware reusable action
  menus over a hosted-browser pivot. Playwright MCP snapshots are valid only
  against fresh page state, Stagehand v3 documents local and Browserbase action
  caches that need validation, Firecrawl/Browser Use sell persistent sessions
  around changing forms, and Cloudflare Browser Run/WebMCP is pushing typed
  website actions. Plasmate's sticky response remains local-first SOM action
  state with cross-adapter conformance.
- Code changes: Python/Node parser packages, Python/Node/Go SDKs, Browser Use,
  LangChain, and Vercel AI action-plan helpers now expose ARIA `expanded`,
  `pressed`, and `selected` cues for interactive targets without changing
  deterministic action `cache_key` values. Browser Use, LangChain, and Vercel
  AI prompt renderers now include those state cues alongside value and checked
  state.
- Fixture and docs changes: the shared action-availability SOM fixture and
  expected manifest now assert expanded, pressed, and selected state across
  parser, SDK, and framework surfaces. Updated PRD, roadmap, website generated
  docs, adapter docs, SDK docs, and fixture docs with the state-cues rationale
  and next step to promote these cases into Rust compiler/schema conformance.
- Verification: `./scripts/action-manifest-conformance.sh --quick` passed;
  `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target
  cargo build` passed with existing warnings; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target
  cargo test --lib -- --test-threads=1` passed 246 tests; `git diff --check`
  passed. A default-target Rust build/test attempt failed because restricted
  network could not download the `rusty_v8` archive. Full `cargo test --
  --test-threads=1` still fails only in `tests/awp_integration_test.rs` because
  sandboxed local listener setup returns `Operation not permitted`.
- Commit/push state: implementation commit `9ff9309`
  (`chore: expose aria action state cues`) was pushed to
  `codex/plasmate-improvements-2026-05-13-control-state` and fast-forwarded
  remote `master` from `c9876d8` to `9ff9309`. A follow-up state-only commit
  records the push result and is the current pushed `master` tip.

### 2026-05-13T17:08:35Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate35/FETCH_HEAD`
  (`Operation not permitted`), and retrying from the primary checkout could
  not resolve `github.com` over SSH (`ssh: Could not resolve hostname
  github.com: -65563`). This run continued from locally known
  `origin/master`/`HEAD` `51fccb5`. Creating a branch in the automation
  worktree was also blocked by `HEAD.lock`, so implementation and commit
  staging used the primary checkout on
  `codex/plasmate-improvements-2026-05-13-control-state`.
- Market direction: current official docs still point toward reusable action
  state rather than a hosted-browser pivot. Playwright MCP snapshots keep refs
  scoped to current accessibility state, Stagehand `observe()` and action
  caching depend on validating state before acting, and Firecrawl/Browser Use
  package managed sessions around forms that change between runs. Plasmate's
  sticky local-first response is to carry live control state in compact SOM
  action menus while preserving deterministic local cache keys.
- Code changes: Python/Node parser packages, Python/Node/Go SDKs, Browser Use,
  LangChain, and Vercel AI action-plan surfaces now preserve non-empty control
  `value` fields. Compact targets now normalize native `attrs.checked` and
  ARIA `aria.checked` into one shared `checked` field. Browser Use, LangChain,
  and Vercel AI prompt renderers now include value/checked state in action
  menu text.
- Fixture and test changes: `integrations/fixtures/action-availability.*` now
  asserts current input value, native unchecked checkbox state, and ARIA
  checked radio/menu state while keeping existing deterministic `cache_key`
  values unchanged. Adapter tests now verify those fields in rendered prompt
  contexts.
- Docs changes: updated PRD and roadmap source docs plus generated website
  source docs with the control-state market read, completed improvement log,
  and next step to extend compact action-plan state with ARIA expanded,
  pressed, and selected cues.
- Verification: `./scripts/action-manifest-conformance.sh --quick` passed;
  `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; `cargo build` passed with existing
  warnings; `cargo test --lib -- --test-threads=1` passed 246 tests; `git
  diff --check` passed.
- Commit/push state: implementation commit `ae86c9e`
  (`chore: expose action control state`) was pushed to
  `codex/plasmate-improvements-2026-05-13-control-state` and fast-forwarded
  remote `master` from `51fccb5` to `ae86c9e`. A follow-up state-only commit
  on the same branch records the push outcome and is the current pushed
  `master` tip.

### 2026-05-13T16:11:57Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate34/FETCH_HEAD`
  (`Operation not permitted`), and retrying from the primary checkout could
  not resolve `github.com` over SSH (`ssh: Could not resolve hostname
  github.com: -65563`). This run continued from locally known
  `origin/master`/`HEAD` `c6f8323`.
- Market direction: fresh official-doc research still favors reusable action
  surfaces over a hosted-browser pivot. Playwright MCP snapshots expose
  accessibility refs scoped to the current snapshot, Stagehand documents
  `observe()` plus local/server action caching, Firecrawl Interact resumes
  scraped browser sessions with prompt/code actions and persistent profiles,
  Browser Use Cloud sells stateful sessions/profiles, and Crawl4AI is moving
  LLM-friendly crawling toward cloud scale. Plasmate's sticky wedge remains a
  browser-like local SOM/action contract across the broad repo surface.
- Code changes: ARIA landmark parsing now honors space-separated fallback role
  tokens, so `role="utility search"` still produces a labelled
  search/navigation region. ARIA widget role parsing now honors fallback tokens,
  preserving menu checkbox/radio action targets when unknown role tokens
  precede known roles. Hidden stripping now treats uppercase
  `aria-hidden="TRUE"` and inline `opacity: 0` as hidden state, matching the
  existing stylesheet visibility behavior.
- Fixture and test changes: `specs/conformance/016-action-semantics.html` and
  its expected partial output now cover fallback-token search/menu roles,
  uppercase ARIA-hidden state, and inline opacity hiding. Rust compiler tests
  assert fallback-token landmark parsing, fallback-token widget parsing,
  uppercase ARIA-hidden stripping, inline opacity stripping, and the expanded
  `016-action-semantics` fixture.
- Docs changes: updated PRD and roadmap source docs plus generated website docs
  with the role-fallback/visibility market read, completed improvement log, and
  next step to promote `016-action-semantics` into parser/SDK and adapter
  conformance runners for fallback roles and hidden-state variants.
- Verification: `cargo build` passed using
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target` with existing warnings;
  `cargo test --test som_compiler_test -- --test-threads=1` passed 57 tests;
  `cargo test --lib -- --test-threads=1` passed 246 tests; `node
  website/build.mjs` rebuilt 39 pages; `./scripts/action-manifest-conformance.sh
  --quick` passed after temporarily symlinking existing primary-checkout
  `node_modules` for Node package dependencies; `git diff --check` passed.
- Commit/push state: implementation commit `513ba86`
  (`fix: tolerate aria fallback roles`) was pushed to
  `codex/plasmate-improvements-2026-05-13-vercel-action-menu` and
  fast-forwarded remote `master` from `c6f8323` to `513ba86`. This follow-up
  state entry records the successful push result.

### 2026-05-13T15:13:58Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate33/FETCH_HEAD`
  (`Operation not permitted`). This run continued from locally known
  `origin/master`/`HEAD` `0929a23`, which matched the current detached worktree.
- Market direction: current trend research still favors reusable action state
  over raw browser access. Playwright MCP-style structured snapshots,
  Stagehand/Browserbase action caching, Browser Use framework workflows, and
  Firecrawl managed browser/session features all reinforce Plasmate's
  local-first wedge: keep action semantics portable across the broad repo
  surface instead of pivoting into hosted browser infrastructure first.
- Code and fixture changes: the shared action-availability SOM fixture now
  includes ARIA menu checkbox/radio targets with deterministic cache keys; the
  Vercel AI fixture test expectation now includes those available targets; and
  `specs/conformance/016-action-semantics.html` plus expected output capture
  labelled search landmarks, ARIA menuitem checkbox/radio targets, and
  stylesheet hidden-rule whitespace/casing.
- Test coverage changes: `tests/som_compiler_test.rs` now loads the new
  `016-action-semantics` fixture directly and asserts search-region,
  menuitemcheckbox, menuitemradio, hidden stylesheet copy, visible copy, and
  interactive-count behavior.
- Docs changes: updated PRD and roadmap source docs plus generated website
  docs with the action-semantics fixture rationale, current run changes, and
  next step to wire `016-action-semantics` into parser/SDK and adapter
  conformance runners.
- Verification: `cargo build` passed using
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target` with existing warnings;
  `cargo test --test som_compiler_test -- --test-threads=1` passed 55 tests;
  Python parser, Python SDK, Browser Use, LangChain, Go SDK, Node parser, and
  Node SDK shared-manifest checks passed; `node website/build.mjs` rebuilt 39
  pages; `git diff --check` passed. The full quick action-manifest script was
  partially blocked until dependencies were installed, and Vercel AI tests
  could not run because `npm install` for `@ai-sdk/mcp` failed with
  `ENOTFOUND registry.npmjs.org`.
- Commit/push state: implementation commit `bc8aef5`
  (`chore: add action semantics fixture coverage`) was pushed to
  `codex/plasmate-improvements-2026-05-13-vercel-action-menu` and
  fast-forwarded remote `master` from `0929a23` to `bc8aef5`. This follow-up
  state entry records that push result.

### 2026-05-13T14:05:53Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate32/FETCH_HEAD`
  (`Operation not permitted`), and the primary checkout could not resolve
  `github.com` over SSH (`ssh: Could not resolve hostname github.com: -65563`).
  This run continued from the local primary checkout branch
  `codex/plasmate-improvements-2026-05-13-vercel-action-menu`, which was
  already one commit ahead of its remote branch.
- Market direction: current docs reinforce that browser-agent retention is
  moving toward trustworthy reusable action state. Playwright MCP snapshots
  expose accessibility roles and snapshot-scoped refs; Stagehand `observe()`
  and action caching reward stable target descriptions; Firecrawl Browser
  Sandbox, Browser Use Cloud, and Cloudflare Browser Run/WebMCP make managed
  browser sessions and typed website tools convenient; Crawl4AI is pushing
  LLM-friendly crawling toward cloud extraction. Plasmate should not pivot into
  hosted browser infrastructure first. The sticky wedge remains local SOM
  fidelity plus adapter conformance.
- Code changes: ARIA `role="search"` now compiles into a labelled navigation
  region; ARIA `menuitemcheckbox` and `menuitemradio` now compile as actionable
  checkbox/radio controls; stylesheet hidden-rule parsing now ignores arbitrary
  declaration whitespace and casing; a stale SOM improvements test now matches
  the case-sensitive URL path deduplication contract.
- Docs changes: updated the PRD and roadmap source docs plus generated website
  docs with the semantic-fidelity market read, completed improvement log, and
  next step to promote search/menuitem/stylesheet-hidden cases into shared
  parser and adapter fixtures.
- Verification: `node website/build.mjs` rebuilt 39 pages; `cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1` passed
  246 tests; `cargo test --test som_compiler_test --test som_improvements_test
  -- --test-threads=1` passed 74 tests; `./scripts/action-manifest-conformance.sh
  --quick` passed; `git diff --check` passed; `cargo clippy --all-targets`
  passed with existing warnings. Full `cargo test -- --test-threads=1` still
  failed only in `tests/awp_integration_test.rs` because sandboxed local
  listener setup returned `Operation not permitted`. `cargo fmt --all --check`
  still reports pre-existing formatting drift in unrelated files such as
  `src/awp/handler.rs`, `src/network/proxy.rs`, `src/main.rs`, and
  `src/mcp/tools.rs`; touched Rust files were formatted directly with
  `rustfmt`.
- Commit/push state: implementation commit `66cad78`
  (`fix: improve semantic action fidelity`) was pushed to
  `codex/plasmate-improvements-2026-05-13-vercel-action-menu` and
  fast-forwarded remote `master` from `6b10133` to `66cad78`. A follow-up
  state commit records that push result.

### 2026-05-13T13:08:36Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  could not write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate31/FETCH_HEAD`
  (`Operation not permitted`), and the primary checkout could not resolve
  `github.com` over SSH (`ssh: Could not resolve hostname github.com: -65563`).
  This run continued from locally known `origin/master`/`HEAD` `6b10133`.
- Market direction: current docs still validate the local-first action
  contract wedge. Playwright MCP uses snapshot-scoped structured refs,
  Stagehand `observe()` and Browserbase caching make repeated actions cheaper,
  Firecrawl Interact and Browser Use Cloud make managed sessions convenient,
  Crawl4AI remains strong for LLM-friendly crawling, Skyvern owns a visual
  workflow lane, and Cloudflare WebMCP points toward typed browser-native
  actions. Plasmate should keep avoiding a hosted-browser pivot and make its
  broad local SOM/action-plan contract continuously verifiable.
- Code changes: `scripts/action-manifest-conformance.sh` now supports
  `--quick` for focused shared-manifest checks and `--full` for the complete
  release gate. GitHub Actions now has a dedicated `action-manifest` job that
  installs Python, Node, and Go dependencies and runs the quick conformance
  gate on pushes and pull requests. Fixture docs now explain quick vs full
  usage.
- Docs changes: updated PRD and roadmap source docs plus generated website
  docs with the CI action-manifest rationale, completed minor-improvement log,
  and next steps to tune dependency caching and eventually promote CI from the
  quick gate to full conformance.
- Verification: `./scripts/action-manifest-conformance.sh --quick` passed;
  `./scripts/action-manifest-conformance.sh --full` passed; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed;
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings; `cargo test --lib -- --test-threads=1` passed 245 tests;
  `cargo test --bin plasmate -- --test-threads=1` passed 5 tests; `cargo
  clippy --all-targets` passed with existing warnings. Full `cargo test --
  --test-threads=1` reached integration tests and then failed only because
  sandboxed `TcpListener` setup in `tests/awp_integration_test.rs` returned
  `Operation not permitted`. `cargo fmt --all --check` still reports
  pre-existing formatting drift in unrelated Rust files; this run did not
  apply a repo-wide formatting change.
- Commit/push state: committed locally on
  `codex/plasmate-improvements-2026-05-13-vercel-action-menu`. Push attempts
  for both the review branch and `HEAD:master` failed because the environment
  could not resolve `github.com` over SSH, so remote merge to `master` is still
  blocked outside this machine.

### 2026-05-13T12:07:41Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  could not write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate30/FETCH_HEAD`
  (`Operation not permitted`), and the primary checkout could not resolve
  `github.com` over SSH (`ssh: Could not resolve hostname github.com: -65563`).
  This run continued from the locally known primary checkout state where
  `origin/master`, `origin/HEAD`, and the automation branch all pointed at
  `2ebe5b6`.
- Market direction: current official docs continue to validate Plasmate's
  local-first action-contract wedge. Playwright MCP uses structured
  accessibility snapshots with refs scoped to the current snapshot, Stagehand
  v3 `observe()` is positioned as a cacheable action-planning and validation
  surface, and Firecrawl Interact resumes scraped browser sessions with
  optional persistent profiles. The project should still avoid a hosted browser
  infrastructure pivot and instead make broad local SDK/adapter conformance a
  release feature.
- Code changes: added `scripts/action-manifest-conformance.sh`, a shared
  release-gate command that runs the action-availability expectation manifest
  across Python/Node parser packages, Go/Python/Node SDKs, Browser Use,
  LangChain, and Vercel AI. Node SDK `npm test` now builds and runs the
  action-plan fixture tests, moving TypeScript client parity out of an ad hoc
  command. Root and fixture docs now advertise the shared release gate.
- Docs changes: updated PRD and roadmap source docs plus generated website
  docs with the release-gate rationale, completed minor-improvement log, and
  next steps to wire the new command into GitHub Actions and split quick/full
  modes if runtime becomes a bottleneck.
- Verification: `./scripts/action-manifest-conformance.sh` passed across all
  manifest consumers; `node website/build.mjs` rebuilt 39 pages; `git diff
  --check` passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo
  build` passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 245 tests; `cargo test --bin plasmate` passed 5 tests; `cargo clippy
  --all-targets` passed with existing warnings. Full `cargo test` was blocked
  in `tests/awp_integration_test.rs` because sandboxed `TcpListener::bind("127.0.0.1:0")`
  returned `Operation not permitted`; unit coverage passed separately.
- Commit/push state: committed `429d73a`
  (`chore: add action manifest release gate`), pushed review branch
  `codex/plasmate-improvements-2026-05-13-vercel-action-menu`, and
  fast-forwarded remote `master` to `429d73a`. A follow-up docs-state commit
  was created and pushed after `429d73a` to record the successful push in
  `CLAUDE.md`.

### 2026-05-13T11:12:00Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate29/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout was blocked
  by DNS resolution for `github.com`, so this run continued from the current
  local primary branch, which already contained the shared adapter expectation
  manifest commit `a117a70` on top of locally known `origin/master` `c7961bb`.
- Market direction: current official docs continue to validate Plasmate's
  local-first action-contract wedge. Playwright MCP exposes structured
  accessibility snapshots with snapshot-bound refs, Stagehand/Browserbase
  foreground `observe()` and action caching, Firecrawl Interact and Browser Use
  Cloud make managed reusable sessions easy to buy, and Cloudflare Browser Run
  is adding Live View, session recordings, human-in-loop, CDP/MCP, and WebMCP.
  The startup direction should remain portable SOM/action memory rather than a
  hosted browser-infrastructure pivot.
- Code changes: Python SDK now exports `get_action_plan()` and
  `get_action_plan_cache_key()`; Node SDK now exports `getActionPlan()` and
  `getActionPlanCacheKey()`; Python parser, Node parser, Go SDK, Python SDK,
  and Node SDK fixture tests now consume
  `integrations/fixtures/action-availability.expected.json` so availability,
  blocked reasons, required state, groups, descriptions, placeholders, and
  cache keys share one expected contract.
- Docs changes: updated Python/Node SDK docs, integration fixture docs,
  PRD/roadmap source docs, and regenerated website docs with the SDK-manifest
  conformance rationale. The next step is to add one release command that runs
  Browser Use, LangChain, Vercel AI, parser-package, and SDK fixture checks
  together; Node SDK's fixture test should also move into a normal npm test
  script instead of an ad hoc compile/run command.
- Verification: Python parser tests passed; Node parser tests passed; Go SDK
  tests passed with sandbox-local `GOCACHE`; Python SDK query tests passed;
  Node SDK `npm run build` passed; Node SDK query tests passed via explicit
  TypeScript compile plus `node:test`; `node website/build.mjs` rebuilt 39
  pages; `git diff --check` passed;
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings; `cargo test --lib -- --test-threads=1` passed 245 tests
  with existing warnings. `cargo clippy --all-targets --all-features` was
  blocked by sandbox permissions because Cargo could not create
  `/Users/steve/.cargo/registry/src/.../addr2line-0.24.2`.
- Commit/push state: committed `2502514`
  (`chore: extend action manifest conformance`) and pushed review branch
  `codex/plasmate-improvements-2026-05-13-vercel-action-menu`. Remote
  `master` fast-forward was rejected because the remote contains newer work
  than local `origin/master`; fetching that newer work is currently blocked by
  DNS resolution for `github.com`, so this run could not safely merge to
  `master`.

### 2026-05-13T10:09:38Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate28/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout was blocked
  by DNS resolution for `github.com`, so this run continued from current local
  HEAD `c7961bb`.
- Market direction: current official docs still validate Plasmate's local-first
  action-contract wedge. Playwright MCP snapshots expose snapshot-bound refs,
  Stagehand/Browserbase cache actions and selectors to reduce repeated LLM
  calls, Firecrawl Interact and Browser Use Cloud sell managed sessions and
  profiles, and Cloudflare Browser Run/WebMCP points toward typed
  browser-native tools. The sticky path remains a portable local SOM action
  surface with shared conformance checks across adapters.
- Code changes: added
  `integrations/fixtures/action-availability.expected.json` as the shared
  expected compact action-target contract; Browser Use and LangChain tests now
  validate rendered prompt context against that manifest; Vercel AI runtime
  tests now compare extracted action targets against the same manifest and
  verify cache-key uniqueness.
- Docs changes: added integration fixture documentation; updated PRD and
  roadmap source docs plus generated website docs with the fixture-manifest
  rationale and next step to extend the manifest into parser-package and SDK
  conformance tests.
- Verification: Browser Use adapter test passed with repo-local
  `som_parser`; LangChain adapter test passed with the existing Python
  3.14/Pydantic v1 warning from `langchain_core`; Vercel AI `npm test`
  passed; `node website/build.mjs` rebuilt 39 pages; `git diff --check`
  passed; `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build`
  passed with existing warnings; `cargo test --lib -- --test-threads=1`
  passed 245 tests with existing warnings.

### 2026-05-13T09:10:00Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate27/FETCH_HEAD`
  (`Operation not permitted`). Retrying from the primary checkout was blocked
  by DNS resolution for `github.com`, so this run continued from locally known
  `origin/master`/`HEAD` `affc41a`.
- Market direction: current official docs continue to make reusable action
  memory the retention layer. Playwright MCP snapshots keep refs tied to fresh
  page state, Stagehand/Browserbase cache resolved actions, Firecrawl Interact
  and Browser Use Cloud package reusable hosted sessions/profiles, and
  Cloudflare WebMCP points toward typed browser-native tools. Plasmate should
  keep the local-first SOM/action-plan wedge and make deterministic action
  `cache_key` values portable across SDKs and framework prompt contexts.
- Code changes: Go SDK action plans now include deterministic `CacheKey`
  values and export `GetActionPlanCacheKey()`. Browser Use page contexts now
  render `cache_key` flags beside availability state. LangChain SOM text now
  computes and renders deterministic `cache_key` flags for interactive targets.
- Docs changes: updated the Go, Browser Use, and LangChain READMEs; updated
  PRD and roadmap source docs plus generated website docs with the cache-key
  parity rationale and the next step to move checks into a shared
  cross-adapter fixture runner.
- Verification: Browser Use adapter test passed; LangChain adapter test passed
  with the existing Python 3.14/Pydantic v1 warning from `langchain_core`; Go
  SDK tests passed with sandbox-local `GOCACHE`; `node website/build.mjs`
  rebuilt 39 pages after linking to the primary checkout's existing
  `website/node_modules`; `git diff --check` passed;
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings; `cargo test --lib -- --test-threads=1` passed 245 tests
  with existing warnings.

### 2026-05-13T08:12:00Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first. The automation worktree
  still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate26/FETCH_HEAD`
  (`Operation not permitted`), and the primary checkout could not resolve
  `github.com` over SSH. Continued from the newest local primary checkout,
  which already contained the prior two unpushed Vercel AI action-menu commits.
- Market direction: competitor docs continue to favor reusable action memory
  over a hosted-browser pivot. Playwright MCP keeps refs bound to fresh
  snapshots, while Stagehand/Browserbase-style action caching and hosted trace
  tooling make repeated workflows cheaper after first observation. Plasmate's
  best retention path remains local-first SOM action plans with deterministic
  ids, availability, and now cache keys across SDK/framework surfaces.
- Code changes: Vercel AI compact action targets now include deterministic
  `cache_key` values and export `getPlasmateActionTargetCacheKey()`. Node SOM
  parser action plans now include `cache_key` and export
  `getActionPlanCacheKey()`. Python SOM parser action plans now include
  `cache_key` and export `get_action_plan_cache_key()`.
- Docs changes: updated the Vercel AI README, Python/Node parser READMEs, PRD,
  roadmap, website PRD/roadmap sources, and regenerated website docs with the
  cache-key rationale and next step to push parity into Go, Browser Use, and
  LangChain.
- Verification: Vercel AI `npm run typecheck` passed; Vercel AI `npm test`
  passed; Node parser `npm run build && npm test` passed 52 tests; Python
  parser `python3 -m pytest tests/test_parser.py -q` passed 67 tests; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed;
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings. Default parallel `cargo test --lib` hit the existing
  auth-store HOME race once, but the focused auth test passed and
  `cargo test --lib -- --test-threads=1` passed 245 tests with existing
  warnings.
- Commit/push state: committed `bea42e5`
  (`chore: add action plan cache keys`), pushed
  `codex/plasmate-improvements-2026-05-13-vercel-action-menu`, and
  fast-forwarded remote `master` to `bea42e5`. A final `git ls-remote`
  verification hit transient DNS, but both push commands completed
  successfully. Primary checkout is clean except pre-existing untracked
  `.agents/`.

### 2026-05-13T07:10:00Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first, but this automation
  worktree still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate25/FETCH_HEAD`
  (`Operation not permitted`). Continued from locally known `origin/master`
  `fde71a8`, which matched current `HEAD` before edits.
- Market direction: official docs still favor structured, reusable action
  surfaces over hosted-browser parity chasing. Playwright MCP snapshots expose
  current refs, Stagehand v3 `observe()` returns cacheable structured actions,
  Firecrawl Interact resumes scraped browser sessions, and Cloudflare Browser
  Run/WebMCP is moving typed website tools closer to browser-native agent
  workflows. Plasmate should keep the local-first wedge and make raw SOM
  output easier for framework apps to turn into compact action menus.
- Code changes: Vercel AI now exports `extractPlasmateActionTargets()` for
  deriving compact action targets directly from raw SOM responses, including
  nested children and shadow-root elements. `formatPlasmateActionPlan()` now
  preserves blocked reasons, input type, and placeholder metadata in prompt
  text. The package now has an executable fixture test script that builds and
  checks extraction, filtering, and formatting against the shared adapter
  fixture.
- Docs changes: updated the Vercel AI README, PRD, roadmap, website PRD/roadmap
  sources, and regenerated website docs with the SOM-to-action-menu rationale
  and the completed runtime fixture step.
- Verification: Vercel AI `npm run typecheck` passed using the primary
  checkout's existing `node_modules`; Vercel AI `npm test` passed; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed;
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings; and `cargo test --lib` passed 245 tests with existing
  warnings.
- Commit/push state: committed `6c9c57e` as
  `chore: add Vercel AI SOM action extraction` from the primary checkout
  because the automation worktree could not create Git locks. Final fetch,
  branch push, and remote `master` fast-forward are blocked by DNS resolution
  for `github.com` (`ssh: Could not resolve hostname github.com: -65563`).

### 2026-05-13T06:07:54Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted first, but this automation
  worktree still cannot write shared worktree metadata at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate24/FETCH_HEAD`
  (`Operation not permitted`). The worktree was already at locally known
  `origin/master` `ebba72f`. `gh auth status` also reports an invalid saved
  token, so PR/merge work must use git remotes rather than the GitHub CLI until
  re-authenticated.
- Market direction: current official docs still validate Plasmate's local-first
  action-menu wedge. Playwright MCP uses accessibility snapshots with refs,
  Stagehand/Browserbase push `observe()` and selector/action caching, and
  Firecrawl/Browser Use keep crowding hosted browser sessions. The startup
  direction should stay focused on portable SOM action menus in SDK and
  framework adapters rather than a hosted browser-cloud pivot.
- Code changes: Vercel AI now treats any `blocked_reason` as unavailable,
  exports `normalizePlasmateActionTarget()`, exports
  `preparePlasmateActionPlan()` for filtering/limiting compact action menus,
  and exports `formatPlasmateActionPlan()` for prompt or trace text.
- Tests/docs changes: added a TypeScript fixture-style compile check for the
  Vercel AI action helpers, updated the Vercel AI README, PRD, roadmap, website
  PRD/roadmap sources, and regenerated website docs. The next step is a runtime
  Vercel AI fixture test runner so this compile check can become executable
  shared adapter coverage.
- Verification: Vercel AI `npm run typecheck` passed using the primary
  checkout's existing `node_modules`; Vercel AI `npm run build` passed; `node
  website/build.mjs` rebuilt 39 pages; `git diff --check` passed; and
  `CARGO_TARGET_DIR=/Users/steve/Git/plasmate/target cargo build` passed with
  existing warnings.
- Commit/push state: automation worktree Git metadata still cannot create
  locks, so the reviewed patch was mirrored to `/Users/steve/Git/plasmate`.
  Committed `0903bd1` as `chore: add Vercel AI action menu helpers`, pushed
  review branch `codex/plasmate-improvements-2026-05-13-vercel-action-menu`,
  and fast-forwarded remote `master` from `ebba72f` to `0903bd1`.

### 2026-05-13T05:08:17Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted in the automation worktree, but
  shared worktree metadata writes are still denied at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate23/FETCH_HEAD`
  (`Operation not permitted`). Retried from the primary checkout, but
  `github.com` DNS resolution failed. Continued from locally known
  `origin/master` at `4c39aa0`.
- Market direction: current docs continue to favor structured, reusable action
  state over a hosted browser-cloud pivot. Playwright MCP makes accessibility
  snapshots with refs the baseline, Stagehand/Browserbase are foregrounding
  cached deterministic actions and observability, and managed browser vendors
  keep crowding the hosted sessions lane. Plasmate should keep deepening
  local-first SOM/action-plan parity across adapters.
- Code changes: added shared
  `integrations/fixtures/action-availability.som.json`; Browser Use and
  LangChain tests now consume the same fixture; LangChain now marks normal
  interactive targets as `[enabled]` when `attrs.disabled` is omitted and emits
  `[blocked_reason=disabled]` for disabled targets; Vercel AI now exports
  `PlasmateActionTarget` plus `isPlasmateActionTargetAvailable()`; Browser Use
  and LangChain `__version__` exports now match package metadata.
- Docs changes: updated Browser Use, LangChain, and Vercel AI READMEs; updated
  docs/PRD.md, ROADMAP-v0.2.md, website PRD/roadmap sources, and regenerated
  website docs with the fixture-driven conformance rationale and next step.
- Verification: Browser Use adapter test passed; LangChain adapter test passed
  with the existing Python 3.14/Pydantic v1 warning from `langchain_core`;
  Vercel AI `npm run typecheck` passed; `node website/build.mjs` rebuilt 39
  pages; `git diff --check` passed; `cargo build` passed with existing
  warnings.
- Commit/push state: automation worktree still cannot write its Git index, so
  the reviewed patch was mirrored into `/Users/steve/Git/plasmate`. Committed
  `339998b` as `chore: add adapter action availability fixture`, pushed review
  branch `codex/plasmate-improvements-2026-05-13-adapter-fixture`, and
  fast-forwarded remote `master` from `4c39aa0` to `339998b`.

### 2026-05-13T04:08:53Z - Plasmate Improvements Automation

- Git sync: requested latest pull was attempted in the automation worktree, but
  shared worktree metadata writes are still denied at
  `/Users/steve/Git/plasmate/.git/worktrees/plasmate22/FETCH_HEAD`
  (`Operation not permitted`). Retried from the primary checkout, but
  `github.com` DNS resolution failed. Continued from locally known
  `origin/master` at `62de9d3`.
- Market direction: current docs continue to crowd the hosted browser lane.
  Playwright MCP uses structured accessibility snapshots with refs, Stagehand
  v3 `observe()` and action caching turn page state into reusable actions,
  Firecrawl Interact and Browser Use Cloud sell managed browsers/profiles/CDP,
  and Cloudflare Browser Run is expanding hosted MCP/CDP/WebMCP distribution.
  Plasmate should keep its local-first SOM/action-plan wedge and push
  availability cues through framework adapters.
- Code changes: Browser Use integration now exposes sync/async
  `extract_action_plan` helpers and renders `enabled`, disabled
  `blocked_reason`, required, type, group, and description state in page
  context; LangChain SOM text output now marks disabled/enabled/required/group
  and description state on interactive targets; Vercel AI SDK integration now
  exports `plasmateActionGuidance` for system prompts.
- Tests/docs changes: added focused Browser Use and LangChain adapter tests,
  updated adapter READMEs, updated PRD and roadmap source docs plus generated
  website docs. Next step is to promote adapter availability checks into shared
  cross-adapter conformance fixtures.
- Verification: Browser Use adapter test passed; LangChain adapter test passed
  with an existing Python 3.14/Pydantic v1 warning from `langchain_core`;
  Vercel AI `npm run typecheck` passed after temporarily symlinking existing
  primary-checkout `node_modules`; `node website/build.mjs` rebuilt 39 pages;
  `git diff --check` passed; `cargo build` passed with existing warnings; and
  `cargo test --lib` passed 245 tests with existing warnings.
- Commit/push state: committed `469478f` as
  `chore: surface action availability in adapters`, pushed review branch
  `codex/plasmate-improvements-2026-05-13-adapter-availability`, and
  fast-forwarded remote `master` from `62de9d3` to `469478f`. A follow-up
  state-only commit records this final push state.

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
