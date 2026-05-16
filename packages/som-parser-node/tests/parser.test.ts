import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, it, expect } from 'vitest';
import {
  parseSom,
  isValidSom,
  fromPlasmate,
  getAllElements,
  findByAction,
  findActionTarget,
  findActionTargetByCacheKey,
  findActionTargetByHtmlId,
  findActionTargetById,
  findActionTargetByLabel,
  findActionTargetByTestId,
  findActionTargetsByLabel,
  findByHint,
  findByRole,
  findById,
  findByHtmlId,
  findByLabel,
  findByText,
  getActionPlan,
  getActionPlanCacheKey,
  getActionPlanIndex,
  getEnabledActionPlan,
  getInteractiveElements,
  getLinks,
  getForms,
  getInputs,
  getHeadings,
  getText,
  getTextByRegion,
  getCompressionRatio,
  toMarkdown,
  filter,
} from '../src/index.js';
import type { Som } from '../src/index.js';

type ExpectedActionTarget = { id: string; cache_key: string; [key: string]: unknown };

const FIXTURE: Som = {
  som_version: '0.1',
  url: 'https://example.com/',
  title: 'Example Domain',
  lang: 'en',
  regions: [
    {
      id: 'r_nav',
      role: 'navigation',
      elements: [
        {
          id: 'e_1',
          role: 'link',
          html_id: 'home-link',
          text: 'Home',
          actions: ['click'],
          attrs: { href: '/' },
        },
        { id: 'e_2', role: 'link', text: 'About', actions: ['click'], attrs: { href: '/about' } },
      ],
    },
    {
      id: 'r_content',
      role: 'content',
      elements: [
        { id: 'e_3', role: 'heading', text: 'Welcome', attrs: { level: 1 } },
        { id: 'e_4', role: 'paragraph', text: 'This is a test page.' },
        {
          id: 'e_5',
          role: 'link',
          text: 'Learn more',
          actions: ['click'],
          attrs: { href: 'https://example.org' },
        },
        { id: 'e_6', role: 'image', attrs: { src: '/logo.png', alt: 'Logo' } },
      ],
    },
    {
      id: 'r_form',
      role: 'form',
      action: '/search',
      method: 'GET',
      elements: [
        {
          id: 'e_7',
          role: 'text_input',
          label: 'Search',
          actions: ['type', 'clear'],
          attrs: { input_type: 'text', name: 'q', placeholder: 'Search...' },
          hints: ['required'],
        },
        { id: 'e_8', role: 'button', text: 'Go', actions: ['click'] },
      ],
    },
  ],
  meta: {
    html_bytes: 5000,
    som_bytes: 800,
    element_count: 8,
    interactive_count: 5,
  },
};

const FIXTURE_JSON = JSON.stringify(FIXTURE);
const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');

function loadActionAvailabilityFixture(): { som: Som; action_targets: ExpectedActionTarget[] } {
  const fixtureDir = resolve(REPO_ROOT, 'integrations/fixtures');
  return {
    som: JSON.parse(readFileSync(resolve(fixtureDir, 'action-availability.som.json'), 'utf8')),
    action_targets: JSON.parse(
      readFileSync(resolve(fixtureDir, 'action-availability.expected.json'), 'utf8'),
    ).action_targets,
  };
}

const SHADOW_FIXTURE: Som = {
  ...FIXTURE,
  regions: [
    {
      id: 'r_content',
      role: 'content',
      elements: [
        {
          id: 'host',
          role: 'section',
          text: 'Widget host',
          shadow: {
            mode: 'open',
            elements: [
              {
                id: 'shadow_action',
                role: 'button',
                text: 'Shadow Save',
                actions: ['click'],
              },
              {
                id: 'shadow_link',
                role: 'link',
                text: 'Shadow Docs',
                actions: ['click'],
                attrs: { href: '/shadow-docs' },
              },
            ],
          },
        },
      ],
    },
  ],
  meta: { html_bytes: 1000, som_bytes: 500, element_count: 3, interactive_count: 2 },
};

// ---- Parser tests ----

describe('parseSom', () => {
  it('parses a valid JSON string', () => {
    const som = parseSom(FIXTURE_JSON);
    expect(som.title).toBe('Example Domain');
    expect(som.regions).toHaveLength(3);
  });

  it('parses a valid object', () => {
    const som = parseSom(FIXTURE);
    expect(som.url).toBe('https://example.com/');
  });

  it('parses group roles and legend attrs', () => {
    const som = parseSom({
      ...FIXTURE,
      regions: [
        {
          id: 'r_form',
          role: 'form',
          elements: [
            {
              id: 'e_group',
              role: 'group',
              label: 'Contact preference',
              attrs: { legend: 'Contact preference', disabled: true },
            },
          ],
        },
      ],
    });
    const group = som.regions[0].elements[0];
    expect(group.role).toBe('group');
    expect(group.attrs?.legend).toBe('Contact preference');
    expect(group.attrs?.disabled).toBe(true);
  });

  it('throws on invalid input', () => {
    expect(() => parseSom('{}')).toThrow('Invalid SOM');
    expect(() => parseSom('not json')).toThrow();
  });

  it('throws on missing required fields', () => {
    expect(() => parseSom({ som_version: '0.1' })).toThrow('Invalid SOM');
    expect(() => parseSom({ som_version: '0.1', url: 'x', title: 'y' })).toThrow('Invalid SOM');
  });
});

describe('isValidSom', () => {
  it('returns true for valid SOM', () => {
    expect(isValidSom(FIXTURE)).toBe(true);
  });

  it('returns false for null/undefined', () => {
    expect(isValidSom(null)).toBe(false);
    expect(isValidSom(undefined)).toBe(false);
  });

  it('returns false for non-objects', () => {
    expect(isValidSom('string')).toBe(false);
    expect(isValidSom(42)).toBe(false);
  });

  it('returns false for missing fields', () => {
    expect(isValidSom({})).toBe(false);
    expect(isValidSom({ som_version: '0.1' })).toBe(false);
  });
});

describe('fromPlasmate', () => {
  it('parses clean JSON', () => {
    const som = fromPlasmate(FIXTURE_JSON);
    expect(som.title).toBe('Example Domain');
  });

  it('parses wrapped SOM JSON', () => {
    const som = fromPlasmate(JSON.stringify({ som: FIXTURE }));
    expect(som.title).toBe('Example Domain');
  });

  it('extracts JSON from surrounding text', () => {
    const output = `Processing https://example.com/...\n${FIXTURE_JSON}\nDone.`;
    const som = fromPlasmate(output);
    expect(som.title).toBe('Example Domain');
  });

  it('extracts wrapped JSON from surrounding text', () => {
    const output = `Processing...\n${JSON.stringify({ som: FIXTURE })}\nDone.`;
    const som = fromPlasmate(output);
    expect(som.title).toBe('Example Domain');
  });

  it('throws when no JSON found', () => {
    expect(() => fromPlasmate('no json here')).toThrow();
  });
});

// ---- Query tests ----

describe('getAllElements', () => {
  it('returns all 8 elements', () => {
    expect(getAllElements(FIXTURE)).toHaveLength(8);
  });

  it('includes shadow-root elements', () => {
    expect(getAllElements(SHADOW_FIXTURE).map((el) => el.id)).toEqual([
      'host',
      'shadow_action',
      'shadow_link',
    ]);
  });
});

describe('findByRole', () => {
  it('finds all links', () => {
    const links = findByRole(FIXTURE, 'link');
    expect(links).toHaveLength(3);
    expect(links.map((l) => l.text)).toEqual(['Home', 'About', 'Learn more']);
  });

  it('finds headings', () => {
    expect(findByRole(FIXTURE, 'heading')).toHaveLength(1);
  });

  it('returns empty for missing role', () => {
    expect(findByRole(FIXTURE, 'table')).toHaveLength(0);
  });
});

describe('findById', () => {
  it('finds existing element', () => {
    const el = findById(FIXTURE, 'e_3');
    expect(el).toBeDefined();
    expect(el!.role).toBe('heading');
    expect(el!.text).toBe('Welcome');
  });

  it('returns undefined for missing id', () => {
    expect(findById(FIXTURE, 'e_999')).toBeUndefined();
  });

  it('finds elements inside shadow roots', () => {
    expect(findById(SHADOW_FIXTURE, 'shadow_action')?.text).toBe('Shadow Save');
  });
});

describe('findByHtmlId', () => {
  it('finds existing element by original HTML id', () => {
    expect(findByHtmlId(FIXTURE, 'home-link')?.id).toBe('e_1');
  });

  it('returns undefined for missing HTML id', () => {
    expect(findByHtmlId(FIXTURE, 'missing-html-id')).toBeUndefined();
  });
});

describe('findByLabel', () => {
  it('finds label-only controls', () => {
    const results = findByLabel(FIXTURE, 'search');
    expect(results.map((el) => el.id)).toEqual(['e_7']);
  });

  it('supports exact case-sensitive label matching', () => {
    expect(findByLabel(FIXTURE, 'Search', { exact: true }).map((el) => el.id)).toEqual(['e_7']);
    expect(findByLabel(FIXTURE, 'search', { exact: true })).toEqual([]);
  });
});

describe('findByText', () => {
  it('finds by substring (case-insensitive)', () => {
    const results = findByText(FIXTURE, 'home');
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('e_1');
  });

  it('finds by label text', () => {
    const results = findByText(FIXTURE, 'search');
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('e_7');
  });

  it('finds text inside shadow roots', () => {
    const results = findByText(SHADOW_FIXTURE, 'shadow docs');
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('shadow_link');
  });

  it('finds by exact match', () => {
    const results = findByText(FIXTURE, 'Home', { exact: true });
    expect(results).toHaveLength(1);
    // Case-sensitive exact
    expect(findByText(FIXTURE, 'home', { exact: true })).toHaveLength(0);
  });

  it('returns empty for no match', () => {
    expect(findByText(FIXTURE, 'nonexistent')).toHaveLength(0);
  });
});

describe('getInteractiveElements', () => {
  it('returns elements with actions', () => {
    const interactive = getInteractiveElements(FIXTURE);
    expect(interactive).toHaveLength(5);
    const ids = interactive.map((e) => e.id);
    expect(ids).toContain('e_1');
    expect(ids).toContain('e_2');
    expect(ids).toContain('e_5');
    expect(ids).toContain('e_7');
    expect(ids).toContain('e_8');
  });

  it('includes interactive shadow-root elements', () => {
    const ids = getInteractiveElements(SHADOW_FIXTURE).map((e) => e.id);
    expect(ids).toEqual(['shadow_action', 'shadow_link']);
  });
});

describe('findByAction', () => {
  it('finds clickable elements', () => {
    expect(findByAction(FIXTURE, 'click').map((el) => el.id)).toEqual([
      'e_1',
      'e_2',
      'e_5',
      'e_8',
    ]);
  });

  it('finds typeable elements', () => {
    expect(findByAction(FIXTURE, 'type').map((el) => el.id)).toEqual(['e_7']);
  });
});

describe('findByHint', () => {
  it('finds required elements', () => {
    expect(findByHint(FIXTURE, 'required').map((el) => el.id)).toEqual(['e_7']);
  });

  it('returns empty for missing hints', () => {
    expect(findByHint(FIXTURE, 'danger')).toEqual([]);
  });
});

describe('getActionPlan', () => {
  it('returns compact action targets', () => {
    const plan = getActionPlan(FIXTURE);
    expect(plan[0]).toEqual({
      id: 'e_1',
      cache_key: 'plasmate-action:v1:4f5af432',
      role: 'link',
      actions: ['click'],
      enabled: true,
      label: 'Home',
      html_id: 'home-link',
      href: '/',
    });
    expect(plan.at(-2)).toEqual({
      id: 'e_7',
      cache_key: 'plasmate-action:v1:0b6b537f',
      role: 'text_input',
      actions: ['type', 'clear'],
      enabled: true,
      label: 'Search',
      name: 'q',
      input_type: 'text',
      placeholder: 'Search...',
      form_action: '/search',
      form_method: 'GET',
    });
  });

  it('marks disabled targets unavailable', () => {
    const disabledSom: Som = {
      ...FIXTURE,
      regions: [
        {
          id: 'r_form',
          role: 'form',
          elements: [
            {
              id: 'locked',
              role: 'button',
              text: 'Archive',
              actions: ['click'],
              attrs: { disabled: true },
            },
          ],
        },
      ],
      meta: { html_bytes: 100, som_bytes: 50, element_count: 1, interactive_count: 1 },
    };

    expect(getActionPlan(disabledSom)).toEqual([
      {
        id: 'locked',
        cache_key: 'plasmate-action:v1:2de92b9a',
        role: 'button',
        actions: ['click'],
        enabled: false,
        label: 'Archive',
        disabled: true,
        blocked_reason: 'disabled',
      },
    ]);
  });

  it('returns deterministic cache keys for equivalent action targets', () => {
    expect(
      getActionPlanCacheKey({
        id: 'e_7',
        role: 'text_input',
        actions: ['type', 'clear'],
        enabled: true,
        label: 'Search',
        name: 'q',
        input_type: 'text',
        placeholder: 'Search...',
      }),
    ).toBe('plasmate-action:v1:0b6b537f');
  });

  it('matches the shared action availability manifest', () => {
    const { som, action_targets } = loadActionAvailabilityFixture();

    expect(getActionPlan(som)).toEqual(action_targets);
  });

  it('indexes action targets for replay', () => {
    const { som, action_targets } = loadActionAvailabilityFixture();
    const save = action_targets.find((target) => target.id === 'e_save')!;
    const index = getActionPlanIndex(som);

    expect(index.byId.e_save).toEqual(save);
    expect(index.byCacheKey[save.cache_key]).toEqual(save);
    expect(index.byHtmlId['save-button']).toEqual(save);
    expect(index.byTestId['settings-save']).toEqual(save);
    expect(index.byLabel.Save).toEqual(save);
    expect(findActionTarget(som, 'e_save')).toEqual(save);
    expect(findActionTarget(som, save.cache_key)).toEqual(save);
    expect(findActionTarget(som, 'save-button')).toEqual(save);
    expect(findActionTarget(som, 'settings-save')).toEqual(save);
    expect(findActionTarget(som, 'Save', { by: 'label' })).toEqual(save);
    expect(findActionTarget(som, 'settings-save', { by: 'test_id' })).toEqual(save);
    expect(findActionTargetById(som, 'e_save')).toEqual(save);
    expect(findActionTargetByCacheKey(som, save.cache_key)).toEqual(save);
    expect(findActionTargetByHtmlId(som, 'save-button')).toEqual(save);
    expect(findActionTargetByTestId(som, 'settings-save')).toEqual(save);
    expect(findActionTargetByLabel(som, 'Save')).toEqual(save);
    expect(findActionTargetsByLabel(som, 'save')).toEqual([save]);
    expect(findActionTarget(som, 'settings-save', { enabledOnly: true })).toBeUndefined();
    expect(findActionTarget(som, 'Save', { by: 'label', enabledOnly: true })).toBeUndefined();
  });

  it('filters blocked targets from enabled action indexes', () => {
    const { som } = loadActionAvailabilityFixture();
    const enabled = getEnabledActionPlan(som);
    const index = getActionPlanIndex(som, { enabledOnly: true });

    expect(enabled.every((target) => target.enabled)).toBe(true);
    expect(index.byId.e_save).toBeUndefined();
    expect(index.byTestId['settings-save']).toBeUndefined();
    expect(index.byLabel.Save).toBeUndefined();
    expect(index.byId.e_plan).toBeDefined();
  });
});

describe('getLinks', () => {
  it('returns all links with text, href, id', () => {
    const links = getLinks(FIXTURE);
    expect(links).toHaveLength(3);
    expect(links[0]).toEqual({ text: 'Home', href: '/', id: 'e_1' });
    expect(links[2]).toEqual({ text: 'Learn more', href: 'https://example.org', id: 'e_5' });
  });

  it('falls back to labels for accessible-only links', () => {
    const links = getLinks({
      ...FIXTURE,
      regions: [
        {
          id: 'r_nav',
          role: 'navigation',
          elements: [
            {
              id: 'icon_docs',
              role: 'link',
              label: 'Docs',
              actions: ['click'],
              attrs: { href: '/docs' },
            },
          ],
        },
      ],
      meta: { html_bytes: 100, som_bytes: 50, element_count: 1, interactive_count: 1 },
    });

    expect(links).toEqual([{ text: 'Docs', href: '/docs', id: 'icon_docs' }]);
  });
});

describe('getForms', () => {
  it('returns form regions', () => {
    const forms = getForms(FIXTURE);
    expect(forms).toHaveLength(1);
    expect(forms[0].id).toBe('r_form');
    expect(forms[0].action).toBe('/search');
  });
});

describe('getInputs', () => {
  it('returns input elements', () => {
    const inputs = getInputs(FIXTURE);
    expect(inputs).toHaveLength(1);
    expect(inputs[0].role).toBe('text_input');
    expect(inputs[0].label).toBe('Search');
  });
});

describe('getHeadings', () => {
  it('returns headings with level', () => {
    const headings = getHeadings(FIXTURE);
    expect(headings).toHaveLength(1);
    expect(headings[0]).toEqual({ level: 1, text: 'Welcome', id: 'e_3' });
  });
});

describe('getText', () => {
  it('extracts all text content', () => {
    const text = getText(FIXTURE);
    expect(text).toContain('Home');
    expect(text).toContain('Welcome');
    expect(text).toContain('This is a test page.');
    expect(text).toContain('Learn more');
    expect(text).toContain('Search');
    expect(text).toContain('Go');
  });
});

describe('getTextByRegion', () => {
  it('groups text by region', () => {
    const byRegion = getTextByRegion(FIXTURE);
    expect(byRegion).toHaveLength(3);

    const nav = byRegion.find((r) => r.region === 'r_nav')!;
    expect(nav.role).toBe('navigation');
    expect(nav.text).toContain('Home');
    expect(nav.text).toContain('About');

    const content = byRegion.find((r) => r.region === 'r_content')!;
    expect(content.text).toContain('Welcome');

    const form = byRegion.find((r) => r.region === 'r_form')!;
    expect(form.text).toContain('Search');
  });
});

describe('getCompressionRatio', () => {
  it('calculates html_bytes / som_bytes', () => {
    const ratio = getCompressionRatio(FIXTURE);
    expect(ratio).toBe(5000 / 800);
    expect(ratio).toBeCloseTo(6.25);
  });

  it('returns infinity when som_bytes is zero', () => {
    expect(getCompressionRatio({ ...FIXTURE, meta: { ...FIXTURE.meta, som_bytes: 0 } })).toBe(
      Infinity,
    );
  });
});

describe('toMarkdown', () => {
  it('produces markdown with title', () => {
    const md = toMarkdown(FIXTURE);
    expect(md).toContain('# Example Domain');
  });

  it('includes headings at correct level', () => {
    const md = toMarkdown(FIXTURE);
    expect(md).toContain('## Welcome');
  });

  it('includes links as markdown links', () => {
    const md = toMarkdown(FIXTURE);
    expect(md).toContain('[Home](/)');
    expect(md).toContain('[About](/about)');
    expect(md).toContain('[Learn more](https://example.org)');
  });

  it('uses labels for markdown links without text', () => {
    const md = toMarkdown({
      ...FIXTURE,
      regions: [
        {
          id: 'r_nav',
          role: 'navigation',
          elements: [
            {
              id: 'icon_docs',
              role: 'link',
              label: 'Docs',
              actions: ['click'],
              attrs: { href: '/docs' },
            },
          ],
        },
      ],
      meta: { html_bytes: 100, som_bytes: 50, element_count: 1, interactive_count: 1 },
    });

    expect(md).toContain('[Docs](/docs)');
  });

  it('includes paragraph text', () => {
    const md = toMarkdown(FIXTURE);
    expect(md).toContain('This is a test page.');
  });

  it('includes image', () => {
    const md = toMarkdown(FIXTURE);
    expect(md).toContain('![Logo](/logo.png)');
  });

  it('includes form section', () => {
    const md = toMarkdown(FIXTURE);
    expect(md).toContain('### Form');
    expect(md).toContain('**Search** (text_input)');
    expect(md).toContain('[Go] (button)');
  });
});

describe('filter', () => {
  it('filters elements by predicate', () => {
    const buttons = filter(FIXTURE, (el) => el.role === 'button');
    expect(buttons).toHaveLength(1);
    expect(buttons[0].text).toBe('Go');
  });

  it('returns empty for no matches', () => {
    expect(filter(FIXTURE, () => false)).toHaveLength(0);
  });
});
