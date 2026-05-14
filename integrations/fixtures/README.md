# Integration Fixtures

Shared fixtures keep framework adapters aligned with the SOM contract emitted
by the Rust core and parser packages.

## Action Availability

- `action-availability.som.json` is the canonical SOM payload for compact
  action-target availability checks.
- `action-availability.expected.json` is the expected adapter contract for that
  payload: ids, roles, labels, actions, availability, blocked reasons, cache
  keys, required state, group context, link target/rel/download cues, input metadata, descriptions, value and
  checked state, ARIA expanded/pressed/selected cues, ARIA current/controls/
  haspopup relationship cues, validation constraints (`autocomplete`,
  `inputmode`, `enterkeyhint`, `aria_autocomplete`, `active_descendant`,
  `form`, `list`, `popovertarget`, `popovertargetaction`, `commandfor`,
  `command`, `accesskey`, `errormessage`, `keyshortcuts`,
  `roledescription`, `busy`, `live`, `atomic`, `relevant`, `owns`, `flowto`,
  `details`, `orientation`, `sort`, `valuemin`, `valuemax`, `valuenow`,
  `valuetext`, `minlength`, `maxlength`, `min`, `max`, `step`, `pattern`,
  `invalid`), and ARIA menu
  checkbox/radio targets.

Browser Use, LangChain, Vercel AI, Python parser, Node parser, Python SDK, and
Go SDK tests consume the expectation file directly. When action-plan semantics
change, update the SOM fixture and the expectation file together so adapters
and SDKs fail consistently instead of drifting via hard-coded assertions.

Run the full cross-runtime release gate with:

```bash
./scripts/action-manifest-conformance.sh
```

For faster pre-merge feedback, run the narrow shared-manifest checks with:

```bash
./scripts/action-manifest-conformance.sh --quick
```

The script checks the Python and Node parser packages, Go/Python/Node SDKs, and
Browser Use, LangChain, and Vercel AI adapters against the same expectation
manifest. CI runs the quick gate on pull requests and pushes; maintainers should
use the full gate before changing action-plan semantics or publishing SDK and
adapter releases.
