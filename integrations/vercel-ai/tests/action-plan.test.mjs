import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'

import {
  extractPlasmateActionTargets,
  formatPlasmateActionPlan,
  isPlasmateActionTargetAvailable,
  preparePlasmateActionPlan,
} from '../dist/index.js'

const fixtureUrl = new URL(
  '../../fixtures/action-availability.som.json',
  import.meta.url
)
const fixture = JSON.parse(await readFile(fixtureUrl, 'utf8'))
const targets = extractPlasmateActionTargets(fixture)

assert.equal(targets.length, 3)

const email = targets.find((target) => target.id === 'e_email')
assert.deepEqual(email, {
  id: 'e_email',
  role: 'text_input',
  actions: ['type'],
  enabled: true,
  label: 'Work email',
  input_type: 'email',
  placeholder: 'name@company.com',
  description: 'Use your work email',
  required: true,
  group: 'Account',
})

const save = targets.find((target) => target.id === 'e_save')
assert.equal(isPlasmateActionTargetAvailable(save), false)
assert.equal(save.enabled, false)
assert.equal(save.disabled, true)
assert.equal(save.blocked_reason, 'disabled')

const availableTargets = preparePlasmateActionPlan(targets)
assert.deepEqual(
  availableTargets.map((target) => target.id),
  ['e_email', 'e_plan']
)

const formatted = formatPlasmateActionPlan(targets, {
  includeUnavailable: true,
})
assert.match(
  formatted,
  /\[e_email\] text_input "Work email" \(type\) \[enabled\] \[required\] \[type=email\] \[placeholder=name@company\.com\] \[group=Account\]/
)
assert.match(
  formatted,
  /\[e_save\] button "Save" \(click\) \[blocked\] \[blocked_reason=disabled\]/
)
