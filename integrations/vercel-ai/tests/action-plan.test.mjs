import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'

import {
  extractPlasmateActionTargets,
  findPlasmateActionTargetByCacheKey,
  findPlasmateActionTargetByHtmlId,
  findPlasmateActionTargetById,
  formatPlasmateActionPlan,
  getPlasmateActionPlanFingerprint,
  getPlasmateActionPlanIndex,
  getPlasmateActionPlanSummary,
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
  html_id: 'work-email',
  cache_key: 'plasmate-action:v1:e0588ddd',
  role: 'text_input',
  actions: ['type'],
  enabled: false,
  label: 'Work email',
  input_type: 'email',
  name: 'email',
  value: 'ops@example.com',
  autocomplete: 'email',
  inputmode: 'email',
  enterkeyhint: 'next',
  autocapitalize: 'none',
  dirname: 'email.dir',
  dir: 'ltr',
  lang: 'en-US',
  form: 'settings-form',
  form_action: '/settings',
  form_method: 'POST',
  form_target: '_self',
  form_enctype: 'multipart/form-data',
  form_novalidate: true,
  form_accept_charset: 'UTF-8',
  form_autocomplete: 'off',
  list: 'email-suggestions',
  placeholder: 'name@company.com',
  title: 'Credential field',
  aria_label: 'Work email',
  aria_description: 'Use your work email',
  labelledby: 'email-label',
  describedby: 'email-help',
  test_id: 'settings-email',
  data_state: 'readonly',
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
assert.equal(save.button_type, 'submit')
assert.equal(save.formaction, '/settings/save')
assert.equal(save.formmethod, 'post')
assert.equal(save.formenctype, 'application/x-www-form-urlencoded')
assert.equal(save.formtarget, '_top')
assert.equal(save.formnovalidate, true)
assert.equal(save.test_id, 'settings-save')
assert.equal(save.data_action, 'save-settings')
assert.equal(save.data_state, 'blocked')

const preview = targets.find((target) => target.id === 'e_preview')
assert.equal(isPlasmateActionTargetAvailable(preview), false)
assert.equal(preview.enabled, false)
assert.equal(preview.inert, true)
assert.equal(preview.blocked_reason, 'inert')

const availableTargets = preparePlasmateActionPlan(targets)
assert.deepEqual(
  availableTargets.map((target) => target.id),
  ['e_upload', 'e_image_submit', 'e_plan', 'e_compact', 'e_annual', 'e_quota', 'e_billing']
)

const replayIndex = getPlasmateActionPlanIndex(targets)
assert.equal(replayIndex.by_id.e_save.id, 'e_save')
assert.equal(replayIndex.by_cache_key[save.cache_key].id, 'e_save')
assert.equal(replayIndex.by_cache_key_all[save.cache_key].length, 1)
assert.equal(replayIndex.by_html_id['save-settings'].id, 'e_save')
assert.deepEqual(replayIndex.duplicate_cache_keys, [])
assert.deepEqual(replayIndex.duplicate_html_ids, [])
assert.equal(findPlasmateActionTargetById(targets, 'e_save').id, 'e_save')
assert.equal(
  findPlasmateActionTargetByCacheKey(targets, save.cache_key).id,
  'e_save'
)
assert.equal(
  findPlasmateActionTargetByHtmlId(targets, 'save-settings').id,
  'e_save'
)
const enabledReplayIndex = getPlasmateActionPlanIndex(targets, {
  includeUnavailable: false,
})
assert.equal(enabledReplayIndex.by_id.e_save, undefined)
assert.equal(enabledReplayIndex.by_html_id['save-settings'], undefined)

const duplicateReplayTargets = [
  ...targets,
  { ...save, id: 'e_save_copy' },
]
const duplicateReplayIndex =
  getPlasmateActionPlanIndex(duplicateReplayTargets)
assert.deepEqual(duplicateReplayIndex.duplicate_cache_keys, [save.cache_key])
assert.equal(
  duplicateReplayIndex.by_cache_key_all[save.cache_key].length,
  2
)
assert.deepEqual(duplicateReplayIndex.duplicate_html_ids, ['save-settings'])
assert.equal(
  duplicateReplayIndex.by_html_id_all['save-settings'].length,
  2
)

const summary = getPlasmateActionPlanSummary(targets)
assert.equal(summary.total, 10)
assert.equal(summary.enabled, 7)
assert.equal(summary.disabled, 3)
assert.equal(summary.with_cache_key, 10)
assert.equal(summary.unique_cache_keys, 10)
assert.deepEqual(summary.duplicate_cache_keys, [])
assert.equal(summary.with_html_id, 3)
assert.deepEqual(summary.duplicate_html_ids, [])
assert.equal(summary.with_test_id, 2)
assert.equal(summary.with_data_action, 1)
assert.equal(summary.with_data_state, 2)
assert.deepEqual(summary.by_role, {
  button: 3,
  checkbox: 1,
  link: 1,
  radio: 1,
  select: 1,
  text_input: 3,
})
assert.deepEqual(summary.blocked_reasons, {
  disabled: 1,
  inert: 1,
  readonly: 1,
})
assert.match(summary.fingerprint, /^plasmate-plan:v1:/)
assert.equal(getPlasmateActionPlanFingerprint(targets), summary.fingerprint)
assert.equal(
  getPlasmateActionPlanFingerprint(targets, { includeUnavailable: false }),
  summary.enabled_fingerprint
)
assert.notEqual(summary.fingerprint, summary.enabled_fingerprint)

const formatted = formatPlasmateActionPlan(targets, {
  includeUnavailable: true,
})
assert.match(
  formatted,
  /\[e_email\] text_input "Work email" \(type\) \[blocked\] \[cache_key=plasmate-action:v1:e0588ddd\].*\[test_id=settings-email\] \[data_state=readonly\].*\[blocked_reason=readonly\] \[required\] \[readonly\] \[type=email\] \[value=ops@example\.com\] \[name=email\] \[autocomplete=email\] \[inputmode=email\] \[enterkeyhint=next\] \[autocapitalize=none\] \[dirname=email\.dir\] \[dir=ltr\] \[lang=en-US\] \[form=settings-form\].*\[form_action=\/settings\] \[form_method=POST\] \[form_target=_self\] \[form_enctype=multipart\/form-data\] \[form_novalidate=true\] \[form_accept_charset=UTF-8\] \[form_autocomplete=off\] \[list=email-suggestions\].*\[title=Credential field\] \[aria_label=Work email\] \[aria_description=Use your work email\] \[labelledby=email-label\] \[describedby=email-help\] \[spellcheck=false\] \[placeholder=name@company\.com\].*\[invalid=grammar\] \[aria_placeholder=Work email address\] \[aria_autocomplete=list\] \[active_descendant=email-suggestion-1\] \[errormessage=email-error\] \[group=Account\]/
)
assert.match(formatted, /\[e_email\].*\[html_id=work-email\]/)
assert.match(
  formatted,
  /\[e_upload\].*\[type=file\].*\[name=evidence\].*\[accept=image\/png,\.\pdf\].*\[capture=environment\].*\[multiple=true\]/
)
assert.match(
  formatted,
  /\[e_image_submit\].*\[alt=Upload receipt\].*\[src=\/assets\/upload-receipt\.svg\].*\[type=image\].*\[value=receipt\].*\[name=intent\].*\[button_type=submit\]/
)
assert.match(formatted, /\[e_compact\].*\[checked=false\]/)
assert.match(formatted, /\[e_compact\].*\[pressed=false\]/)
assert.match(formatted, /\[e_annual\].*\[checked=true\]/)
assert.match(formatted, /\[e_annual\].*\[selected=true\]/)
assert.match(formatted, /\[e_plan\].*\[expanded=false\]/)
assert.match(formatted, /\[e_plan\].*\[controls=plan-options\]/)
assert.match(formatted, /\[e_plan\].*\[haspopup=listbox\]/)
assert.match(formatted, /\[e_plan\].*\[multiple=true\]/)
assert.match(
  formatted,
  /\[e_plan\].*\[options=starter:Starter\|team:Team\(selected\)\|enterprise:Enterprise\(disabled\|group:Growth\)\|agency:Agency\(selected\|group:Growth\)\]/
)
assert.match(formatted, /\[e_plan\].*\[selected_values=team,agency\]/)
assert.match(formatted, /\[e_plan\].*\[size=4\]/)
assert.match(formatted, /\[e_plan\].*\[multiselectable=true\]/)
assert.match(formatted, /\[e_plan\].*\[level=2\].*\[posinset=1\].*\[setsize=3\]/)
assert.match(formatted, /\[e_plan\].*\[modal=true\]/)
assert.match(formatted, /\[e_plan\].*\[rowindex=2\].*\[colindex=1\].*\[rowcount=4\].*\[colcount=2\]/)
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
  /\[e_save\] button "Save" \(click\) \[blocked\] \[cache_key=plasmate-action:v1:e6fa1cf6\].*\[test_id=settings-save\].*\[data_action=save-settings\].*\[data_state=blocked\].*\[blocked_reason=disabled\].*\[popovertarget=save-status\].*\[popovertargetaction=show\].*\[commandfor=save-status\].*\[command=toggle-popover\].*\[button_type=submit\].*\[formaction=\/settings\/save\].*\[formmethod=post\].*\[formenctype=application\/x-www-form-urlencoded\].*\[formtarget=_top\].*\[formnovalidate=true\]/
)
assert.match(
  formatted,
  /\[e_preview\] button "Preview changes" \(click\) \[blocked\] \[cache_key=plasmate-action:v1:a7067d8d\] \[blocked_reason=inert\] \[inert\]/
)
