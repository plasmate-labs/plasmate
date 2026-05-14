import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'

import {
  extractPlasmateActionTargets,
  formatPlasmateActionPlan,
  getPlasmateActionTargetCacheKey,
  isPlasmateActionTargetAvailable,
  preparePlasmateActionPlan,
} from '../dist/index.js'

const fixtureUrl = new URL(
  '../../fixtures/action-availability.som.json',
  import.meta.url
)
const expectationsUrl = new URL(
  '../../fixtures/action-availability.expected.json',
  import.meta.url
)
const fixture = JSON.parse(await readFile(fixtureUrl, 'utf8'))
const expectations = JSON.parse(await readFile(expectationsUrl, 'utf8'))
const expectedTargets = expectations.action_targets
const targets = extractPlasmateActionTargets(fixture)

assert.equal(targets.length, expectedTargets.length)

for (const expected of expectedTargets) {
  const target = targets.find((item) => item.id === expected.id)
  assert.ok(target, `missing ${expected.id}`)

  for (const [key, value] of Object.entries(expected)) {
    assert.deepEqual(target[key], value, `${expected.id}.${key}`)
  }
}

assert.equal(
  new Set(expectedTargets.map((target) => target.cache_key)).size,
  expectedTargets.length
)

const email = targets.find((target) => target.id === 'e_email')
assert.deepEqual(email, {
  id: 'e_email',
  cache_key: 'plasmate-action:v1:91875850',
  role: 'text_input',
  actions: ['type'],
  enabled: false,
  label: 'Work email',
  input_type: 'email',
  value: 'ops@example.com',
  autocomplete: 'email',
  inputmode: 'email',
  enterkeyhint: 'next',
  autocapitalize: 'none',
  dirname: 'email.dir',
  form: 'settings-form',
  list: 'email-suggestions',
  placeholder: 'name@company.com',
  spellcheck: false,
  minlength: 6,
  maxlength: 64,
  pattern: '.+@example\\.com',
  invalid: 'grammar',
  aria_placeholder: 'Work email address',
  aria_autocomplete: 'list',
  active_descendant: 'email-suggestion-1',
  errormessage: 'email-error',
  multiline: true,
  description: 'Use your work email',
  readonly: true,
  blocked_reason: 'readonly',
  required: true,
  group: 'Account',
})

const save = targets.find((target) => target.id === 'e_save')
assert.equal(isPlasmateActionTargetAvailable(save), false)
assert.equal(getPlasmateActionTargetCacheKey(save), save.cache_key)
assert.equal(save.enabled, false)
assert.equal(save.disabled, true)
assert.equal(save.blocked_reason, 'disabled')
assert.equal(save.popovertarget, 'save-status')
assert.equal(save.popovertargetaction, 'show')
assert.equal(save.commandfor, 'save-status')
assert.equal(save.command, 'toggle-popover')

const availableTargets = preparePlasmateActionPlan(targets)
assert.deepEqual(
  availableTargets.map((target) => target.id),
  ['e_plan', 'e_compact', 'e_annual', 'e_quota', 'e_billing']
)

const formatted = formatPlasmateActionPlan(targets, {
  includeUnavailable: true,
})
assert.match(
  formatted,
  /\[e_email\] text_input "Work email" \(type\) \[blocked\] \[cache_key=plasmate-action:v1:91875850\] \[blocked_reason=readonly\] \[required\] \[readonly\] \[type=email\] \[value=ops@example\.com\] \[autocomplete=email\] \[inputmode=email\] \[enterkeyhint=next\] \[autocapitalize=none\] \[dirname=email\.dir\] \[form=settings-form\] \[list=email-suggestions\].*\[spellcheck=false\] \[placeholder=name@company\.com\].*\[invalid=grammar\] \[aria_placeholder=Work email address\] \[aria_autocomplete=list\] \[active_descendant=email-suggestion-1\] \[errormessage=email-error\] \[group=Account\]/
)
assert.match(formatted, /\[e_compact\].*\[checked=false\]/)
assert.match(formatted, /\[e_compact\].*\[pressed=false\]/)
assert.match(formatted, /\[e_annual\].*\[checked=true\]/)
assert.match(formatted, /\[e_annual\].*\[selected=true\]/)
assert.match(formatted, /\[e_plan\].*\[expanded=false\]/)
assert.match(formatted, /\[e_plan\].*\[controls=plan-options\]/)
assert.match(formatted, /\[e_plan\].*\[haspopup=listbox\]/)
assert.match(formatted, /\[e_plan\].*\[multiselectable=true\]/)
assert.match(formatted, /\[e_plan\].*\[level=2\].*\[posinset=1\].*\[setsize=3\]/)
assert.match(formatted, /\[e_quota\].*\[min=1\].*\[max=100\].*\[step=5\]/)
assert.match(
  formatted,
  /\[e_quota\].*\[orientation=horizontal\].*\[valuemin=1\].*\[valuemax=100\].*\[valuenow=40\].*\[valuetext=40 seats\]/
)
assert.match(formatted, /\[e_billing\].*\[current=page\]/)
assert.match(formatted, /\[e_billing\].*\[target=_blank\]/)
assert.match(formatted, /\[e_billing\].*\[rel=noopener\]/)
assert.match(formatted, /\[e_billing\].*\[download=billing\.csv\]/)
assert.match(
  formatted,
  /\[e_save\] button "Save" \(click\) \[blocked\] \[cache_key=plasmate-action:v1:4d0e8356\] \[blocked_reason=disabled\].*\[popovertarget=save-status\].*\[popovertargetaction=show\].*\[commandfor=save-status\].*\[command=toggle-popover\]/
)
