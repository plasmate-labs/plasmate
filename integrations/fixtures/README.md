# Integration Fixtures

Shared fixtures keep framework adapters aligned with the SOM contract emitted
by the Rust core and parser packages.

## Action Availability

- `action-availability.som.json` is the canonical SOM payload for compact
  action-target availability checks.
- `action-availability.expected.json` is the expected adapter contract for that
  payload: ids, roles, labels, actions, availability, blocked reasons, cache
  keys, required state, group context, input metadata, and descriptions.

Browser Use, LangChain, and Vercel AI tests consume the expectation file
directly. When action-plan semantics change, update the SOM fixture and the
expectation file together so adapters fail consistently instead of drifting via
hard-coded assertions.
