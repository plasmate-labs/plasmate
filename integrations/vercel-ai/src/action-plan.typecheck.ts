import {
  extractPlasmateActionTargets,
  formatPlasmateActionPlan,
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
    lang: 'ar',
    dir: 'rtl',
    translate: false,
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
