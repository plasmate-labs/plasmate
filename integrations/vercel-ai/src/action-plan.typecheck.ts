import {
  formatPlasmateActionPlan,
  isPlasmateActionTargetAvailable,
  normalizePlasmateActionTarget,
  preparePlasmateActionPlan,
  type PlasmateActionTarget,
} from './index'

const fixtureTargets: PlasmateActionTarget[] = [
  {
    id: 'e_email',
    role: 'text_input',
    label: 'Work email',
    actions: ['type'],
    required: true,
    description: 'Use your work email',
    group: 'Account',
  },
  {
    id: 'e_save',
    role: 'button',
    text: 'Save',
    actions: ['click'],
    disabled: true,
    description: 'Unavailable until required fields are complete',
  },
  {
    id: 'e_plan',
    role: 'select',
    label: 'Plan',
    actions: ['select'],
    required: true,
    group: 'Billing',
  },
]

const normalized = normalizePlasmateActionTarget(fixtureTargets[1])
const availableOnly = preparePlasmateActionPlan(fixtureTargets)
const visiblePromptMenu = formatPlasmateActionPlan(fixtureTargets, {
  maxTargets: 2,
})

const shouldBeBoolean: boolean =
  isPlasmateActionTargetAvailable(normalized) &&
  availableOnly.every(isPlasmateActionTargetAvailable)
const shouldBeString: string = visiblePromptMenu

void shouldBeBoolean
void shouldBeString
