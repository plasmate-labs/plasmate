# Plasmate hourly builder loop

- Status: active Codex automation contract
- Cadence: hourly, overlap prohibited
- Role: bounded product-improvement loop
- Repository: `/Users/dbhurley/Git/plasmate`
- Source of truth: `plasmate-labs/plasmate` `master`

## Mission

Use each run as an opportunity to ship at most one small, verified improvement
that makes Plasmate more reliable, compatible, measurable, or useful to an AI
agent. There is no commit quota. A truthful no-change result is preferred to
filler work, duplicate work, weak proof, scope expansion, or repository debris.

Optimize for successful agent tasks and trustworthy semantic delivery, not raw
output reduction, benchmark cherry-picking, tool count, lines changed, or commit
frequency.

## Start gate

1. Read `AGENTS.md`, this contract, the current implementation and relevant
   tests; use `docs/PRD.md` only where it still agrees with current code.
2. Refuse overlap when another automation, merge, release, deployment, or dirty
   checkout is unresolved.
3. Require the checkout to be clean, on `master`, and fast-forwardable to
   `upstream/master`. Fetch/prune both remotes before selecting work.
4. Inspect recent commits and the previous four-hour governor decision. Do not
   duplicate a recent change or work outside an allowed lane.
5. Record the base SHA, named agent journey, before evidence, hypothesis,
   intended delta, focused proof, scope cap, and stop condition.

If these gates are not met, report a no-change or blocked run. Never stash,
overwrite, reset, or absorb unrelated work.

## Active governor constraint (2026-08-29)

- Window: `0929266` .. `a12bc22`
- Decision: `NARROW`
- Merged this run: #239
- Prohibited next: another compiled `attrs.options` / `attrs.caption` /
  `attrs.items` / `attrs.rows` one-surface copy in parser, SDK, CLI, or MCP
  text extractors. Do not reopen `#` selector matching (region id, SOM
  element id, or `html_id`); #158 closed that advertised contract gap.
  Do not copy #160 click DOM-miss fail-closed, #161 unparseable Python
  tool-JSON fail-closed, or #163 `type_text` disabled/readonly fail-closed
  onto adjacent handlers (`clear`, `toggle`, `select_option`) or SDKs.
  Do not copy #165 `session_status` disabled/readonly counts onto other
  tools, traces, or SDKs. Do not copy #167 iframe `src` collection onto
  CLI or CDP link extractors. Do not copy #168 Python `extract_links` onto
  Node or Go SDKs as a one-surface method add. Do not copy #170 Go module
  path / `FetchPageOptions` docs onto marketing, tweets, or remaining
  nickel-org mentions outside published SDK install surfaces. Do not copy
  #172 SOM-reference field/table rewrites onto other docs, marketing, or
  SDK READMEs. Do not copy #173 `<area href>` compile work onto
  extract_links, CLI, CDP, or adjacent tags (`object`, `embed`, `source`).
  Do not copy #175 `extract_text` compiled-label fallback onto CLI, CDP,
  parser, or SDK text extractors. Do not copy #177 `<picture>` nested-img
  src/alt inheritance onto extract_links, CLI, CDP, or adjacent tags
  (`object`, `embed`, `source`). Do not copy #179 native `<search>`
  landmark mapping onto extract_links, CLI, CDP, selectors, or adjacent
  tags (`form`, `object`, `embed`, `source`). Do not copy #181
  `hidden="until-found"` compile work onto extract_text, CLI, CDP,
  extract_links, or adjacent hide attributes (`aria-hidden`,
  `style=display:none`). Do not copy #183 click compiled `html_id`
  lookup onto adjacent handlers (`clear`, `toggle`, `select_option`) or
  SDKs; `type_text` already has this path. Do not copy #185
  `<time datetime>` compile work onto extract_text, CLI, CDP,
  extract_links, or adjacent tags (`data-*`, `ins`, `del`). Do not copy
  #187 bare `contenteditable` type-target compile work onto `type_text` /
  `clear` mutation, extract_text, CLI, CDP, extract_links, or adjacent
  attributes (`spellcheck`, `inputmode`, `enterkeyhint`). Do not copy
  #189 Node `openPage` flat-SOM mapping onto Python or Go SDKs. Do not
  copy #190 LangChain `type_text` tool-name fix onto other integrations
  or SDK wrappers. Do not copy #191 `<progress>` / `<meter>` compile
  work onto extract_text, CLI, CDP, extract_links, or adjacent tags
  (`output`, `dialog`, `data`). Do not copy #193 toggle compiled ARIA
  switch/checkbox onto adjacent handlers (`clear`, `select_option`,
  `click`) or SDKs; do not extend toggle to other ARIA roles (`tab`,
  `option`, `treeitem`) or add html_id lookup / disabled fail-closed
  to toggle. Do not copy #196 wrapping `<label>` without control-id
  compile work onto extract_text, CLI, CDP, extract_links, or SDKs;
  do not extend wrapping-label node indexing onto explicit `for=` /
  `aria-labelledby` paths already covered. Do not copy #198 native radio
  `select_option` onto adjacent handlers (`clear`, `toggle`, `click`) or
  SDKs; do not extend `select_option` to ARIA radio roles or add html_id
  lookup / disabled fail-closed. Do not copy #200 `<video>` src/poster
  compile work onto extract_text, CLI, CDP, extract_links, or adjacent
  tags (`audio`, `object`, `embed`, `source`); do not inherit nested
  `<source>` or advertise play/pause/seek. Do not copy #201 inspect
  compact form `action`/`method` onto fetch_page, CLI, CDP, or SDKs; do
  not add `enctype` / `target` / `novalidate`. Do not copy #202 `<img
  usemap>` compile work onto extract_links, CLI, CDP, or adjacent tags
  (`object`, `embed`, `source`); do not inherit usemap onto `<picture>`
  or compile `ismap`. Do not copy #204 inspect compact link `href` onto
  fetch_page, CLI, CDP, or SDKs; do not add `target` / `rel` / `download`.
  Do not copy #206 file-input omit `type`/`clear` onto `type_text` /
  `clear` mutation, extract_text, CLI, CDP, or SDKs; do not add an upload
  API, a new file role, or html_id lookup, and do not extend the omit to
  adjacent input types (`password`, `hidden`, `range`). Do not copy #208
  `<ol>` `start`/`reversed` compile work onto extract_text, CLI, CDP,
  extract_links, or SDKs; do not add `type` / `li value`, invent defaults
  on plain lists, or copy those attrs onto `ul`. Do not copy #210 ARIA
  `role=heading` / `aria-level` compile work onto extract_text, CLI, CDP,
  extract_links, or SDKs; do not invent default levels, copy compact
  `level` onto non-heading roles, or extend to other ARIA outline
  attributes. Do not copy #212 native `<img srcset>` compile work onto
  extract_links, CLI, CDP, or adjacent tags (`object`, `embed`, `source`);
  do not invent `src` from `srcset`, copy `srcset` onto `<picture>`, or
  add `sizes`. Do not copy #214 native `<canvas>` width/height compile
  work onto extract_text, CLI, CDP, extract_links, or adjacent tags
  (`svg`, `object`, `embed`); do not invent `src` from nested `<img>`,
  advertise draw/play actions, or copy these attrs onto `svg`/`object`/
  `embed`. Do not copy #216 inspect compact image `src` onto fetch_page,
  CLI, CDP, or SDKs; do not invent `src` from `srcset`, copy `src` onto
  links, iframes, or video, or add `srcset`/`sizes`. Do not copy #218
  inspect compact heading `level` onto fetch_page, CLI, CDP, or SDKs; do
  not invent missing or out-of-range levels, copy `level` onto non-heading
  roles, or add outline attrs beyond compiled 1-6. Do not copy #220
  `<blockquote cite>` compile work onto extract_text, CLI, CDP, or
  adjacent tags (`q`, `ins`, `del`); do not invent cite from nested
  `<cite>` text or inline `<q cite>`. Do not copy #222 `<area>`
  shape/coords compile work onto extract_links, CLI, CDP, or adjacent
  tags (`object`, `embed`, `source`, `a`, `img`); do not invent default
  `shape=rect` or copy whitespace/absent geometry. Do not copy #224
  native `autofocus` compile work onto extract_text, CLI, CDP,
  extract_links, or adjacent attributes (`required`, `disabled`,
  `readonly`); do not invent autofocus when absent or copy it onto
  links or paragraphs. Do not copy #225 inspect compact control `name`
  onto fetch_page, CLI, CDP, or SDKs; do not invent missing or
  whitespace names, copy `name` onto paragraphs or unnamed links, or
  add `id`/`for`. Do not copy #227 native element `lang` compile work
  onto extract_text, CLI, CDP, extract_links, or adjacent attributes
  (`xml:lang`, `dir`, `translate`); do not inherit `html lang`, invent
  whitespace/absent lang, or copy `hreflang` into `lang`. Do not copy
  #229 Python `ElementAttrs` extra=allow onto Node or Go SDKs, CLI,
  CDP, parsers, or other Python models (`Som`, `Region`, `Element`);
  structural models stay fail-closed. Do not add named `lang` /
  `autofocus` fields to Python `ElementAttrs` as a follow-on. Do not
  copy #231 textarea `.value` IDL child-text mapping onto `select`,
  `contenteditable`, `clear` handler changes, or SDK wrappers; do not
  invent a textarea `value` content attribute or change input attribute
  persistence. Do not copy #233 inspect compact `disabled` onto
  fetch_page, CLI, CDP, or SDKs; do not invent missing disabled, copy it
  onto paragraphs or unnamed links, or add `readonly` / `required`
  compact fields. Do not copy #235 inspect compact `checked` onto
  fetch_page, CLI, CDP, or SDKs; do not invent missing checked, copy it
  onto paragraphs, buttons, or links, add `readonly` / `required`
  compact fields, or read `attrs.aria.checked`. Do not copy #237 inspect
  compact `value` onto fetch_page, CLI, CDP, or SDKs; do not invent
  missing or whitespace-only values, copy `value` onto paragraphs,
  buttons, links, or checkboxes, stringify non-string compiled values,
  or add `readonly` / `required` / `placeholder` compact fields.
  Do not copy #239 details `.open` IDL onto `dialog`, `select`,
  `contenteditable`, `clear` handler changes, or SDK wrappers; do not
  invent an `open` content attribute on non-details tags, change checkbox
  `checked` or textarea value persistence, or add inspect compact `open`.
- Allowed next (pick one distinct journey): a real missed regression, or
  a published-docs integration failure that is not another SDK install-path
  rewrite, SOM-reference field rewrite, extract_text label fallback copy,
  picture-img fallback copy, native-search landmark copy,
  hidden-until-found copy, click compiled-html_id lookup copy,
  time-datetime compile copy, contenteditable type-target compile copy,
  Node openPage flat-SOM copy, LangChain type_text rename copy,
  progress/meter compile copy, toggle ARIA-switch copy, wrapping-label
  without control-id compile copy, native-radio select_option copy,
  video src/poster compile copy, inspect compact form action/method copy,
  img usemap compile copy, inspect compact link href copy, file-input
  type/clear omit copy, ol start/reversed compile copy, aria heading
  role/level compile copy, img srcset compile copy, canvas
  width/height compile copy, inspect compact image src copy, inspect
  compact heading level copy, blockquote cite compile copy, area
  shape/coords compile copy, autofocus compile copy, inspect compact
  control name copy, element lang compile copy, Python ElementAttrs
  extra=allow copy, textarea value IDL child-text copy, inspect
  compact disabled copy, inspect compact checked copy, inspect
  compact value copy, or details open IDL copy.

## Preferred lanes

Select one demonstrated gap from current code and evidence:

1. SDK or integration compatibility with a reproducible fixture.
2. Deterministic SOM/action behavior or protocol compatibility.
3. Clear, bounded error handling and recovery for an agent journey.
4. Reproducible benchmark, conformance, or release evidence.
5. Documentation that removes a real integration failure and stays aligned with
   implementation and retained measurements.
6. Privacy-safe measurement or diagnostics that prove delivered value without
   collecting page content, raw URLs, credentials, or session data.
7. A regression test for a real failure that current coverage misses.

Generic cleanup, dependency churn, speculative abstractions, generated content,
test-count inflation, and unsupported performance or cost claims do not qualify.

## Review-required boundaries

Do not autonomously change or publish auth/profile encryption, capability
tokens, cookie/session handling, private-network or redirect policy, proxy/TLS
behavior, browser/process containment, cache persistence, secret handling,
release manifests, package/crate/registry publication, CI/release policy, DNS,
or universal security/performance claims. A narrow regression proof around an
existing contract may be proposed, but behavioral changes in these areas stop
for human review.

Never access authenticated/private pages, production credentials, customer
sessions, or private browsing data. Public pages remain untrusted input.

## Change and proof sequence

1. Reproduce the gap using a checked-in, local, or synthetic fixture.
2. Implement the smallest reversible delta; keep one outcome per run.
3. Add or strengthen an independent focused proof that fails before and passes
   after. Do not weaken expectations to accept the candidate.
4. Review the diff for raw URLs/content, secrets, unbounded output, network
   expansion, cache/protocol compatibility, generated artifacts, and claim drift.
5. Run the focused proof, `cargo fmt --check`, `cargo test`, and
   `cargo clippy --all-targets --all-features -- -D warnings`. Run SDK,
   integration, conformance, benchmark, or real-Chrome checks when affected.
6. Re-fetch `upstream/master`. If it moved, reconcile without losing proof and
   rerun affected gates.
7. Publish one atomic outcome. Push directly to `master` only when repository
   rules permit it and the push is a fast-forward. When protected `master`
   requires review, create exactly one short-lived automation branch and one
   pull request for the verified outcome. Do not discard valid work because
   review is required. The four-hour governor owns final review and merge or
   rejection.

Do not create issues, persistent feature branches, or additional worktrees. Do
not start a second builder pull request while one is open. Do not amend or
force-push shared history. End on current `master` with a clean working tree.
An open pull request that is waiting for required checks is normal governor
work, not a human blocker.

## Final report

Report run/base/head; selected journey and hypothesis; before/after evidence;
files and behavior changed; focused/full checks; accepted/no-change/blocked;
commit and publication state; governor compatibility; residual risk; and exact
human decision needed. Never include page content, raw URLs from measurements,
credentials, cookies, or private session data.
