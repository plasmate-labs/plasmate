import {
  extractPlasmateActionTargets,
  findPlasmateActionTarget,
  findPlasmateActionTargetsByAction,
  findPlasmateActionTargetsByRole,
  formatPlasmateActionPlan,
  indexPlasmateActionTargets,
  isPlasmateActionTargetAvailable,
  normalizePlasmateActionTarget,
  preparePlasmateActionPlan,
  type PlasmateSom,
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
const indexedTargets = indexPlasmateActionTargets(fixtureTargets)
const buttonTargets = findPlasmateActionTargetsByRole(fixtureTargets, 'button', {
  includeUnavailable: true,
})
const clickTargets = findPlasmateActionTargetsByAction(fixtureTargets, 'click', {
  includeUnavailable: true,
})
const targetByCacheKey = findPlasmateActionTarget(fixtureTargets, normalized.cache_key ?? '', {
  by: 'cache_key',
  includeUnavailable: true,
})
const visiblePromptMenu = formatPlasmateActionPlan(fixtureTargets, {
  maxTargets: 2,
})
const fixtureSom: PlasmateSom = {
  regions: [
    {
      elements: [
        {
          id: 'e_shadow',
          role: 'custom_element',
          shadow: {
            elements: [
              {
                id: 'e_shadow_button',
                role: 'button',
                text: 'Shadow save',
                actions: ['click'],
                attrs: { disabled: false },
              },
            ],
          },
        },
      ],
    },
  ],
}
const extractedFromSom = extractPlasmateActionTargets(fixtureSom)

const shouldBeBoolean: boolean =
  isPlasmateActionTargetAvailable(normalized) &&
  availableOnly.every(isPlasmateActionTargetAvailable) &&
  extractedFromSom.every(isPlasmateActionTargetAvailable)
const shouldBeString: string = visiblePromptMenu

void shouldBeBoolean
void shouldBeString
void indexedTargets
void buttonTargets
void clickTargets
void targetByCacheKey
