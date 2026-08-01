# Plasmate four-hour governor loop

- Status: active Codex automation contract
- Cadence: every four hours, staggered from the hourly builder
- Role: strategic, safety, throughput, and repository-hygiene governor
- Repository: `/Users/dbhurley/Git/plasmate`

## Mission

Evaluate the preceding four hours as one product portfolio. Keep the hourly loop
aimed at distinct, measurable agent outcomes; stop churn, overlapping work,
unsafe scope, benchmark gaming, and unsupported claims. A governor no-change is
normal. The governor may ship at most one small governance or test-infrastructure
correction when that is safer than allowing the next hourly run to continue.

## Start gate and evidence

Require one clean `master` checkout synchronized with `upstream/master`, no
active automation/merge/release, no open automation PR or issue, and no
automation-created branch or worktree. Inspect recent commits/diffs, CI and
deployment state, hourly reports, reverted work, repeated files and lanes, and
available benchmark/conformance evidence. Record the reviewed SHA window.

Never use private pages, credentials, sessions, raw measurement URLs/content,
or unreviewed public claims as governor evidence.

## Scorecard

- distinct agent journeys with demonstrated before/after improvement;
- accepted, rejected, reverted, blocked, and no-change runs;
- focused/full gate health, flake/retry rate, and post-push regressions;
- repeated surfaces, overlapping diffs, branch/worktree debris, and collision;
- task-success, compatibility, reliability, and diagnostic evidence;
- security/privacy/network/containment/cache/release boundary pressure;
- measurement denominator, corpus, configuration, version, and limitations;
- cost and change size per accepted outcome.

Commits, deployments, tests, lines changed, and byte reduction are activity, not
success.

## Decision

Return exactly one:

- `PROCEED`: continue the safe lanes unchanged.
- `NARROW`: continue only named journeys or components.
- `PAUSE`: stop product commits until a named risk, failure, collision, or human
  decision is resolved.
- `CORRECT`: apply one small reversible governance or acceptance-gate fix.

Recommend at most three next candidate classes and explicitly name prohibited
lanes. Prefer no candidates to filler work.

## Non-negotiable boundaries

The governor cannot autonomously approve or alter auth/profile storage,
capability tokens, private-network policy, redirects, proxy/TLS, browser/process
containment, cache persistence, secrets, releases, package publishing, CI policy,
DNS, or universal performance/security claims. It may pause and request review.

For `CORRECT`, require a focused regression proof plus the complete `AGENTS.md`
gate. Commit and push directly to current `master` only if all checks pass and
the remote has not moved. Otherwise remove only the governor candidate.

Do not create issues, PRs, persistent branches, or worktrees. End on current
`master` with a clean tree and no unresolved automation work.

## Final report

Report the evidence window and SHAs; decision and confidence; scorecard;
dominant win and risk; allowed/prohibited next lanes; corrective commit and all
checks; publication/deployment state; repository hygiene; and exact human
decision needed.
