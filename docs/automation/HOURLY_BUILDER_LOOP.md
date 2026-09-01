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

## Active governor constraint (2026-09-01)

- Window: `688d58b` .. `7db8cf6`
- Decision: `NARROW`
- Merged this run: `155b7b8` inspect compact expanded (`7db8cf6`)
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
  Do not copy #241 select `.options` / `.selectedIndex` IDL onto `input`,
  `textarea`, `contenteditable`, `clear` handler changes, or SDK wrappers;
  do not invent `selected` on non-option tags, change textarea value or
  details open persistence, or add inspect compact `selectedIndex`.
  Do not copy #242 table `rowspan` grid fill onto extract_text, CLI, CDP,
  SDKs, or adjacent tags (`ul`, `ol`, `dl`); do not invent a compiled
  `rowspan` attr, change `colspan`, or copy table headers onto lists.
  Do not copy #243 input/textarea `.readOnly` IDL onto `select`,
  `contenteditable`, `dialog`, paragraphs, `clear`/`toggle`/`select_option`
  handlers, or SDK wrappers; do not invent readonly on non-field tags,
  change disabled IDL, or add inspect compact `readonly`.
  Do not copy #245 input/textarea/select `.required` IDL onto
  `contenteditable`, `dialog`, paragraphs, `clear`/`toggle` handlers,
  or SDK wrappers; do not invent required on non-field tags, change
  readonly/disabled IDL, or add inspect compact `required`.
  Do not copy #246 element `.hidden` IDL onto extract_text, CLI, CDP,
  extract_links, inspect compact fields, or adjacent hide attributes
  (`aria-hidden`, `style=display:none`); do not change `hidden="until-found"`
  compile behavior, invent hidden on untouched elements, or add inspect
  compact `hidden`.
  Do not copy labelable control `.labels` onto `button`, `meter`,
  `progress`, `output`, `click`/`clear`/`toggle` handlers, SDKs, or
  inspect compact fields; do not add `htmlFor` IDL or invent labels on
  paragraphs.
  Do not copy #249 input/textarea `.placeholder` IDL onto `select`,
  `contenteditable`, paragraphs, `clear`/`type_text` handlers, or SDK
  wrappers; do not invent placeholder on non-field tags, change
  value/readonly IDL, or add inspect compact `placeholder`.
  Do not copy #251 input/textarea `setSelectionRange`/`select` onto other
  tags, `selectionStart`/`selectionEnd`/`setRangeText`, handlers, or SDK
  wrappers; do not invent compiled caret/selection attrs or inspect
  compact selection fields.
  Do not copy #253 form `.elements` IDL onto fieldset, `document.forms`,
  RadioNodeList, form named getters, inspect compact fields, CLI, CDP, or
  SDKs; do not invent `elements` on non-form tags or compile the collection
  as an attribute.
  Do not copy #255 input/textarea `.maxLength` IDL onto `select`,
  `contenteditable`, paragraphs, `clear`/`type_text` handlers, or SDK
  wrappers; do not invent maxlength on non-field tags, change
  value/readonly IDL, add `minLength`, or add inspect compact `maxlength`.
  Do not copy #256 textarea `rows`/`cols` compile onto extract_text, CLI,
  CDP, extract_links, input, select, paragraphs, or SDKs; do not invent
  HTML defaults (2/20), add JS `rows`/`cols` IDL, or change table
  `attrs.rows`.
  Do not copy #258 img/area `.alt` IDL onto extract_text, CLI, CDP, SDKs,
  input, video, paragraphs, or adjacent attributes (`srcset`, `sizes`,
  `title`); do not invent `alt` on non-image tags, add inspect compact
  `alt`, or change compile of present `alt`.
  Do not copy #260 HTML `setAttribute` ASCII-lowercasing onto XML/SVG
  case-sensitive names, IDL properties, extract_text, CLI, CDP, or SDKs;
  do not fold attribute values or invent lowercase names on serialize of
  already-lowercased attrs.
  Do not copy #261 `insertAdjacentHTML` onto `insertAdjacentText` /
  `insertAdjacentElement` / `outerHTML`, extract_text, CLI, CDP, SDKs, or
  the native DOM bridge; do not add `afterend` script execution.
  Do not copy #262 `replaceChildren` onto `replaceWith` copies,
  `innerHTML` setters, extract_text, CLI, CDP, SDKs, or the native DOM
  bridge.
  Do not copy #263 table `insertRow`/`insertCell` onto `deleteRow` /
  `deleteCell`, extract_text, CLI, CDP, SDKs, parser `attrs.rows` copies,
  or the native DOM bridge; do not invent table rows on paragraphs or
  lists.
  Do not copy #265 select.add onto HTMLOptionsCollection.add, select.remove,
  extract_text, CLI, CDP, SDKs, or the native DOM bridge; do not invent
  options on paragraphs or inputs, or add inspect compact option lists.
  Do not copy #267 toggleAttribute onto classList.toggle, setAttributeNS,
  extract_text, CLI, CDP, SDKs, or the native DOM bridge; do not invent
  toggle on non-element nodes, add inspect compact toggled attrs, or copy
  force=true/false onto adjacent APIs.
  Do not copy #269 ARIA `role=alert`/`role=status` live-region compile onto
  extract_text, CLI, CDP, extract_links, SDKs, or other ARIA live roles
  (`log`, `timer`, `marquee`); do not invent actions, compact live fields,
  or a new element role.
  Do not copy #270 form.reset onto form.submit, fieldset, document.forms,
  extract_text, CLI, CDP, SDKs, or the native DOM bridge; do not invent
  defaultValue public IDL, reset orphan controls/paragraphs, or add inspect
  compact reset fields.
  Do not copy #272 `<figcaption>` compile work onto extract_text, CLI, CDP,
  extract_links, SDKs, or adjacent tags (`figure`, `caption`, `legend`);
  do not invent `attrs.caption` from figcaption or copy caption text onto
  the image label.
  Do not copy #274 input `.min`/`.max` IDL onto `textarea`, `select`,
  `contenteditable`, paragraphs, meter, progress, `clear`/`type_text`
  handlers, or SDK wrappers; do not invent min/max on non-input tags,
  change maxLength IDL, add `minLength`/`step`, or add inspect compact
  `min`/`max`.
  Do not copy #276 input `.pattern` IDL onto `textarea`, `select`,
  `contenteditable`, paragraphs, `clear`/`type_text` handlers, or SDK
  wrappers; do not invent pattern on non-input tags, change min/max/
  maxLength IDL, add `step`, or add inspect compact `pattern`.
  Do not copy #278 select `.value` IDL onto `input`, `textarea`,
  `contenteditable`, paragraphs, `clear`/`type_text` handlers, or SDK
  wrappers; do not invent `selected` on non-option tags, change textarea
  value or input pattern persistence, or add inspect compact `value`.
  Do not copy #280 input `.accept` IDL onto `textarea`, `select`,
  `contenteditable`, paragraphs, `clear`/`type_text` handlers, or SDK
  wrappers; do not invent accept on non-input tags, change pattern/
  min/max IDL, add inspect compact `accept`, or add upload APIs.
  Do not copy #282 inspect compact `expanded` onto fetch_page, CLI, CDP,
  or SDKs; do not invent missing expanded, copy it onto paragraphs or
  checkboxes, read non-boolean compiled aria-expanded, or add
  `aria-pressed` / `aria-selected` compact fields.
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
  compact value copy, details open IDL copy, select selectedIndex
  IDL copy, table rowspan grid copy, input readOnly IDL copy,
  field required IDL copy, element hidden IDL copy, labelable
  control `.labels` copy, field placeholder IDL copy, field
  setSelectionRange/select copy, form `.elements` IDL copy, field
  maxLength IDL copy, textarea rows/cols compile copy,
  img/area alt IDL copy, setAttribute ASCII-lowercase copy,
  insertAdjacentHTML copy, replaceChildren copy, table
  insertRow/insertCell copy, select.add copy, toggleAttribute copy,
  ARIA alert/status live-region compile copy, form.reset copy,
  figcaption paragraph compile copy, input min/max IDL copy,
  input pattern IDL copy, select value IDL copy, input accept
  IDL copy, or inspect compact expanded copy.

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
