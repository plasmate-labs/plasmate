import { describe, it } from 'node:test';
import * as assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { Som } from './types';
import type { ActionPlanItem } from './query';
import {
  findActionTargetByCacheKey,
  findActionTargetByHtmlId,
  findActionTargetById,
  findByRole,
  findById,
  findByHtmlId,
  findByTag,
  findInteractive,
  findByText,
  flatElements,
  getActionPlan,
  getActionPlanCacheKey,
  getActionPlanFingerprint,
  getActionPlanIndex,
  getActionPlanSummary,
  getEnabledActionPlan,
  getTokenEstimate,
} from './query';

const fixture: Som = {
  som_version: '1.0',
  url: 'https://example.com',
  title: 'Test Page',
  lang: 'en',
  regions: [
    {
      id: 'r_header',
      role: 'header',
      elements: [
        { id: 'e1', role: 'heading', text: 'Welcome to Example', attrs: { level: 1 } },
        {
          id: 'e2',
          role: 'link',
          text: 'Home',
          actions: ['click'],
          attrs: { href: '/' },
        },
      ],
    },
    {
      id: 'r_main',
      role: 'main',
      label: 'Main content',
      elements: [
        { id: 'e3', html_id: 'main-copy', role: 'paragraph', text: 'Hello World' },
        {
          id: 'e4',
          role: 'button',
          text: 'Submit',
          actions: ['click'],
          hints: ['primary'],
        },
        {
          id: 'e5',
          role: 'text_input',
          label: 'Email',
          actions: ['type', 'clear'],
          attrs: { input_type: 'email', placeholder: 'you@example.com', required: true },
        },
        {
          id: 'e6',
          role: 'section',
          children: [
            { id: 'e7', role: 'paragraph', text: 'Nested paragraph' },
            {
              id: 'e8',
              role: 'checkbox',
              text: 'Accept terms',
              actions: ['toggle'],
              attrs: { checked: false },
            },
          ],
        },
      ],
    },
    {
      id: 'r_footer',
      role: 'footer',
      elements: [
        { id: 'e9', role: 'paragraph', text: 'Copyright 2025' },
      ],
    },
  ],
  meta: {
    html_bytes: 8000,
    som_bytes: 2000,
    element_count: 9,
    interactive_count: 4,
  },
};

const shadowFixture: Som = {
  som_version: '1.0',
  url: 'https://example.com/shadow',
  title: 'Shadow',
  lang: 'en',
  regions: [
    {
      id: 'r_main',
      role: 'main',
      elements: [
        {
          id: 'host',
          role: 'section',
          shadow: {
            mode: 'open',
            elements: [
              { id: 'shadow_text', role: 'paragraph', text: 'Inside shadow root' },
              {
                id: 'shadow_button',
                role: 'button',
                text: 'Confirm',
                actions: ['click'],
                attrs: { aria: { pressed: false } },
              },
            ],
          },
        },
      ],
    },
  ],
  meta: {
    html_bytes: 1000,
    som_bytes: 500,
    element_count: 3,
    interactive_count: 1,
  },
};

function loadActionAvailabilityFixture(): { som: Som; action_targets: ActionPlanItem[] } {
  const fixtureDir = resolve(process.cwd(), '../../integrations/fixtures');
  return {
    som: JSON.parse(readFileSync(resolve(fixtureDir, 'action-availability.som.json'), 'utf8')),
    action_targets: JSON.parse(
      readFileSync(resolve(fixtureDir, 'action-availability.expected.json'), 'utf8'),
    ).action_targets,
  };
}

describe('findByRole', () => {
  it('finds regions by role', () => {
    const mains = findByRole(fixture, 'main');
    assert.equal(mains.length, 1);
    assert.equal(mains[0].id, 'r_main');
  });

  it('returns empty array for missing role', () => {
    assert.deepEqual(findByRole(fixture, 'dialog'), []);
  });
});

describe('findById', () => {
  it('finds a top-level element', () => {
    const el = findById(fixture, 'e4');
    assert.equal(el?.role, 'button');
    assert.equal(el?.text, 'Submit');
  });

  it('finds a nested child element', () => {
    const el = findById(fixture, 'e7');
    assert.equal(el?.role, 'paragraph');
    assert.equal(el?.text, 'Nested paragraph');
  });

  it('finds an element inside a shadow root', () => {
    const el = findById(shadowFixture, 'shadow_button');
    assert.equal(el?.role, 'button');
    assert.equal(el?.attrs?.aria?.pressed, false);
  });

  it('returns undefined for missing ID', () => {
    assert.equal(findById(fixture, 'e999'), undefined);
  });
});

describe('findByHtmlId', () => {
  it('finds an element by original HTML id', () => {
    const el = findByHtmlId(fixture, 'main-copy');
    assert.equal(el?.id, 'e3');
  });

  it('returns undefined for missing original HTML id', () => {
    assert.equal(findByHtmlId(fixture, 'missing'), undefined);
  });
});

describe('findByTag', () => {
  it('finds elements by role', () => {
    const paragraphs = findByTag(fixture, 'paragraph');
    assert.equal(paragraphs.length, 3);
  });

  it('returns empty for unused role', () => {
    assert.deepEqual(findByTag(fixture, 'table'), []);
  });
});

describe('findInteractive', () => {
  it('returns all elements with actions', () => {
    const interactive = findInteractive(fixture);
    assert.equal(interactive.length, 4);
    const ids = interactive.map((el) => el.id);
    assert.deepEqual(ids.sort(), ['e2', 'e4', 'e5', 'e8']);
  });

  it('includes interactive elements inside shadow roots', () => {
    const interactive = findInteractive(shadowFixture);
    assert.deepEqual(interactive.map((el) => el.id), ['shadow_button']);
  });
});

describe('findByText', () => {
  it('finds elements containing text (case-insensitive)', () => {
    const results = findByText(fixture, 'hello');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'e3');
  });

  it('matches partial text', () => {
    const results = findByText(fixture, 'WELCOME');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'e1');
  });

  it('finds text inside shadow roots', () => {
    const results = findByText(shadowFixture, 'inside shadow');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'shadow_text');
  });

  it('finds by label text', () => {
    const results = findByText(fixture, 'email');
    assert.deepEqual(results.map((el) => el.id), ['e5']);
  });

  it('supports case-sensitive exact text and label matching', () => {
    assert.deepEqual(findByText(fixture, 'Email', { exact: true }).map((el) => el.id), ['e5']);
    assert.deepEqual(findByText(fixture, 'email', { exact: true }), []);
  });

  it('returns empty for no match', () => {
    assert.deepEqual(findByText(fixture, 'nonexistent'), []);
  });
});

describe('getActionPlan', () => {
  it('returns compact action targets', () => {
    const plan = getActionPlan(fixture);

    assert.deepEqual(plan[0], {
      id: 'e2',
      cache_key: 'plasmate-action:v1:da48f406',
      role: 'link',
      actions: ['click'],
      enabled: true,
      label: 'Home',
      href: '/',
    });
    assert.deepEqual(plan[2], {
      id: 'e5',
      cache_key: 'plasmate-action:v1:f08859f9',
      role: 'text_input',
      actions: ['type', 'clear'],
      enabled: true,
      label: 'Email',
      input_type: 'email',
      placeholder: 'you@example.com',
      required: true,
    });
  });

  it('matches the shared action availability manifest', () => {
    const { som, action_targets } = loadActionAvailabilityFixture();

    assert.deepEqual(getActionPlan(som), action_targets);
  });

  it('returns deterministic cache keys for equivalent targets', () => {
    assert.equal(
      getActionPlanCacheKey({
        id: 'e5',
        role: 'text_input',
        actions: ['type', 'clear'],
        enabled: true,
        label: 'Email',
        input_type: 'email',
        placeholder: 'you@example.com',
        required: true,
      }),
      'plasmate-action:v1:f08859f9',
    );
  });

  it('finds action targets by cache key', () => {
    const target = findActionTargetByCacheKey(fixture, 'plasmate-action:v1:f08859f9');

    assert.equal(target?.id, 'e5');
    assert.equal(findActionTargetByCacheKey(fixture, 'missing'), undefined);
  });

  it('finds action targets by SOM and HTML ids', () => {
    const target = findActionTargetById(fixture, 'e5');
    assert.equal(target?.cache_key, 'plasmate-action:v1:f08859f9');
    assert.equal(findActionTargetById(fixture, 'e3'), undefined);

    const { som } = loadActionAvailabilityFixture();
    assert.equal(findActionTargetByHtmlId(som, 'save-settings')?.id, 'e_save');
    assert.equal(findActionTargetByHtmlId(fixture, 'main-copy'), undefined);
  });

  it('returns enabled action targets', () => {
    const { som, action_targets } = loadActionAvailabilityFixture();

    assert.deepEqual(
      getEnabledActionPlan(som),
      action_targets.filter((target) => target.enabled),
    );
  });

  it('returns action plan summaries for replay validation', () => {
    const { som } = loadActionAvailabilityFixture();
    const summary = getActionPlanSummary(som);

    assert.equal(summary.fingerprint, getActionPlanFingerprint(som));
    assert.equal(
      summary.enabledFingerprint,
      getActionPlanFingerprint(som, { enabledOnly: true }),
    );
    assert.notEqual(summary.fingerprint, summary.enabledFingerprint);
    assert.equal(summary.total, 10);
    assert.equal(summary.enabled, 7);
    assert.equal(summary.disabled, 3);
    assert.equal(summary.withCacheKey, 10);
    assert.equal(summary.uniqueCacheKeys, 10);
    assert.deepEqual(summary.duplicateCacheKeys, []);
    assert.equal(summary.withHtmlId, 3);
    assert.deepEqual(summary.byRole, {
      button: 3,
      checkbox: 1,
      link: 1,
      radio: 1,
      select: 1,
      text_input: 3,
    });
    assert.deepEqual(summary.blockedReasons, {
      disabled: 1,
      inert: 1,
      readonly: 1,
    });
  });

  it('indexes action targets for replay lookups', () => {
    const { som, action_targets } = loadActionAvailabilityFixture();
    const index = getActionPlanIndex(som);

    assert.deepEqual(index.byId.e_save, findActionTargetById(som, 'e_save'));
    assert.deepEqual(index.byCacheKey[action_targets[0].cache_key], action_targets[0]);
    assert.equal(index.byHtmlId['save-settings'].id, 'e_save');

    const enabledIndex = getActionPlanIndex(som, { enabledOnly: true });
    assert.equal(enabledIndex.byId.e_disabled, undefined);
    assert.equal(enabledIndex.byHtmlId['disabled-control'], undefined);
  });
});

describe('flatElements', () => {
  it('flattens all elements including nested children', () => {
    const all = flatElements(fixture);
    assert.equal(all.length, 9);
  });

  it('includes nested children in order', () => {
    const ids = flatElements(fixture).map((el) => el.id);
    assert.deepEqual(ids, ['e1', 'e2', 'e3', 'e4', 'e5', 'e6', 'e7', 'e8', 'e9']);
  });

  it('includes shadow-root elements in order after their host', () => {
    const ids = flatElements(shadowFixture).map((el) => el.id);
    assert.deepEqual(ids, ['host', 'shadow_text', 'shadow_button']);
  });
});

describe('getTokenEstimate', () => {
  it('estimates tokens from som_bytes', () => {
    const tokens = getTokenEstimate(fixture);
    assert.equal(tokens, 500); // 2000 / 4
  });
});
