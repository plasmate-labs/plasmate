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
  enabled: true,
  label: 'Work email',
  input_type: 'email',
  value: 'ops@example.com',
  placeholder: 'name@company.com',
  description: 'Use your work email',
  required: true,
  group: 'Account',
})

const save = targets.find((target) => target.id === 'e_save')
assert.equal(isPlasmateActionTargetAvailable(save), false)
assert.equal(getPlasmateActionTargetCacheKey(save), save.cache_key)
assert.equal(save.enabled, false)
assert.equal(save.disabled, true)
assert.equal(save.blocked_reason, 'disabled')

const availableTargets = preparePlasmateActionPlan(targets)
assert.deepEqual(
  availableTargets.map((target) => target.id),
  ['e_email', 'e_plan', 'e_compact', 'e_annual']
)

const formatted = formatPlasmateActionPlan(targets, {
  includeUnavailable: true,
})
assert.match(
  formatted,
  /\[e_email\] text_input "Work email" \(type\) \[enabled\] \[cache_key=plasmate-action:v1:91875850\] \[required\] \[type=email\] \[value=ops@example\.com\] \[placeholder=name@company\.com\] \[group=Account\]/
)
assert.match(formatted, /\[e_compact\].*\[checked=false\]/)
assert.match(formatted, /\[e_annual\].*\[checked=true\]/)
assert.match(
  formatted,
  /\[e_save\] button "Save" \(click\) \[blocked\] \[cache_key=plasmate-action:v1:4d0e8356\] \[blocked_reason=disabled\]/
)
