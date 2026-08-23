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

## Active governor constraint (2026-08-23)

- Window: `457915d` .. `8286203`
- Decision: `NARROW`
- Merged this run: #163
- Prohibited next: another compiled `attrs.options` / `attrs.caption` /
  `attrs.items` / `attrs.rows` one-surface copy in parser, SDK, CLI, or MCP
  text extractors. Do not reopen `#` selector matching (region id, SOM
  element id, or `html_id`); #158 closed that advertised contract gap.
  Do not copy #160 click DOM-miss fail-closed, #161 unparseable Python
  tool-JSON fail-closed, or #163 `type_text` disabled/readonly fail-closed
  onto adjacent handlers (`clear`, `toggle`, `select_option`) or SDKs.
- Allowed next (pick one distinct journey): privacy-safe diagnostics, a
  real missed regression, or docs that remove an integration failure.

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
