import { describe, it } from 'node:test';
import * as assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { Som } from './types';
import {
  findActionTarget,
  findActionTargetByCacheKey,
  findActionTargetByHtmlId,
  findActionTargetById,
  findActionTargetByLabel,
  findActionTargetByTestId,
  findActionTargetsByAction,
  findActionTargetsByLabel,
  findActionTargetsByRole,
  findByRole,
  findById,
  findByHtmlId,
  findByLabel,
  findByTag,
  findInteractive,
  findByText,
  flatElements,
  getActionPlan,
  getActionPlanCacheKey,
  getActionPlanIndex,
  getEnabledActionPlan,
  getCompressionRatio,
  getTokenEstimate,
} from './query';

type ExpectedActionTarget = { id: string; cache_key: string; [key: string]: unknown };

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
          html_id: 'home-link',
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
        { id: 'e3', role: 'paragraph', text: 'Hello World' },
        {
          id: 'e4',
          role: 'button',
          text: 'Submit',
          actions: ['click'],
          attrs: { button_type: 'submit' },
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

describe('findActionTargetsByAction aliases', () => {
  it('normalizes MCP action aliases and finds submit/reset controls', () => {
    assert.deepEqual(findActionTargetsByAction(fixture, 'action : submit').map((item) => item.id), ['e4']);
    assert.deepEqual(findActionTargetsByAction(fixture, 'action=type_text').map((item) => item.id), ['e5']);
    for (const alias of ['clear_text', 'clear-text', 'clear text', 'action:clear-text']) {
      assert.deepEqual(findActionTargetsByAction(fixture, alias).map((item) => item.id), ['e5'], alias);
    }
    assert.deepEqual(findActionTargetsByAction(fixture, '"action:submit"').map((item) => item.id), ['e4']);
    assert.deepEqual(findActionTargetsByAction(fixture, "'type_text'").map((item) => item.id), ['e5']);
    assert.deepEqual(findActionTargetsByAction(fixture, 'role : submit').map((item) => item.id), ['e4']);
    assert.deepEqual(findActionTargetsByAction(fixture, 'ROLE=TYPE_TEXT').map((item) => item.id), ['e5']);

    const resetFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                {
                  id: 'e10',
                  role: 'button',
                  text: 'Reset',
                  actions: ['click'],
                  attrs: { button_type: 'reset' },
                },
              ],
            }
          : region,
      ),
    };
    assert.deepEqual(findActionTargetsByAction(resetFixture, 'reset').map((item) => item.id), ['e10']);
  });
});

describe('findActionTargetsByRole aliases', () => {
  it('normalizes case, whitespace, and plural role aliases', () => {
    assert.deepEqual(findActionTargetsByRole(fixture, ' BUTTONS ').map((item) => item.id), ['e4']);
    assert.deepEqual(findActionTargetsByRole(fixture, 'text-input').map((item) => item.id), ['e5']);
    for (const alias of ['text field', 'text fields']) {
      assert.deepEqual(findActionTargetsByRole(fixture, alias).map((item) => item.id), ['e5'], alias);
    }
    assert.deepEqual(findActionTargetsByRole(fixture, 'role:button').map((item) => item.id), ['e4']);
    assert.deepEqual(findActionTargetsByRole(fixture, 'role=text-input').map((item) => item.id), ['e5']);
    assert.deepEqual(findActionTargetsByRole(fixture, 'role : text-input').map((item) => item.id), ['e5']);
    assert.deepEqual(findActionTargetsByRole(fixture, 'role = text-input').map((item) => item.id), ['e5']);
    assert.deepEqual(findActionTargetsByRole(fixture, 'role: "button"').map((item) => item.id), ['e4']);
    assert.deepEqual(findActionTargetsByRole(fixture, '"role:button"').map((item) => item.id), ['e4']);
    const selectFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e11', role: 'select', text: 'Country', actions: ['select'] },
              ],
            }
          : region,
      ),
    };
    for (const alias of ['combo boxes', 'list boxes']) {
      assert.deepEqual(
        findActionTargetsByRole(selectFixture, alias).map((item) => item.id),
        ['e11'],
        alias,
      );
    }
    const fieldsetFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e12', role: 'group', text: 'Profile', actions: ['click'] },
              ],
            }
          : region,
      ),
    };
    assert.deepEqual(
      findActionTargetsByRole(fieldsetFixture, 'fieldsets').map((item) => item.id),
      ['e12'],
    );
    const structuralFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e14', role: 'group', text: 'Progress', actions: ['click'] },
                { id: 'e15', role: 'details', text: 'Details', actions: ['click'] },
              ],
            }
          : region,
      ),
    };
    for (const alias of ['figure', 'radiogroups', 'progressbar', 'meters', 'outputs']) {
      assert.deepEqual(
        findActionTargetsByRole(structuralFixture, alias).map((item) => item.id),
        ['e14'],
        alias,
      );
    }
    for (const alias of ['detail', 'summaries']) {
      assert.deepEqual(
        findActionTargetsByRole(structuralFixture, alias).map((item) => item.id),
        ['e15'],
        alias,
      );
    }
    const sectionFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e16', role: 'section', text: 'Article', actions: ['click'] },
              ],
            }
          : region,
      ),
    };
    for (const alias of ['article', 'articles', 'sections']) {
      assert.deepEqual(
        findActionTargetsByRole(sectionFixture, alias).map((item) => item.id),
        ['e16'],
        alias,
      );
    }
    const tableFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e13', role: 'table', text: 'Orders', actions: ['click'] },
              ],
            }
          : region,
      ),
    };
    assert.deepEqual(
      findActionTargetsByRole(tableFixture, 'treegrids').map((item) => item.id),
      ['e13'],
    );
    assert.deepEqual(findActionTargetsByRole(fixture, 'unknown-role'), []);
  });

  it('accepts HTML input type aliases for action targets', () => {
    for (const alias of ['email', 'date', 'datetime-local', 'number', 'password', 'url', 'queryboxes']) {
      assert.deepEqual(findActionTargetsByRole(fixture, alias).map((item) => item.id), ['e5'], alias);
    }
  });

  it('accepts media aliases for actionable source-role elements', () => {
    const mediaFixture: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_media',
          role: 'main',
          elements: [
            { id: 'image', role: 'image', actions: ['click'] },
            { id: 'audio', role: 'group', actions: ['click'], attrs: { source_role: 'audio' } },
            { id: 'video', role: 'group', actions: ['click'], attrs: { source_role: 'video' } },
            { id: 'canvas', role: 'group', actions: ['click'], attrs: { source_role: 'canvas' } },
          ],
        },
      ],
    };
    for (const alias of ['audio-item', 'audioitem', 'audioitems']) {
      assert.deepEqual(findActionTargetsByRole(mediaFixture, alias).map((item) => item.id), ['audio'], alias);
    }
    assert.deepEqual(findActionTargetsByRole(mediaFixture, 'media-items').map((item) => item.id), [
      'image',
      'audio',
      'video',
      'canvas',
    ]);
  });
});

function loadActionAvailabilityFixture(): { som: Som; action_targets: ExpectedActionTarget[] } {
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

  it('accepts common region selector aliases', () => {
    const som: Som = {
      ...fixture,
      regions: [
        ...fixture.regions,
        { id: 'r_nav', role: 'navigation', elements: [] },
        { id: 'r_content', role: 'content', elements: [] },
      ],
    };
    assert.deepEqual(findByRole(som, ' MAIN-CONTENT ').map((region) => region.id), ['r_main']);
    assert.deepEqual(findByRole(som, '"main-content"').map((region) => region.id), ['r_main']);
    assert.deepEqual(findByRole(som, 'role:main').map((region) => region.id), ['r_main']);
    assert.deepEqual(findByRole(som, 'role : main').map((region) => region.id), ['r_main']);
    assert.deepEqual(findByRole(som, 'role=navigation').map((region) => region.id), ['r_nav']);
    assert.deepEqual(findByRole(som, "role = 'navigation'").map((region) => region.id), ['r_nav']);
    assert.deepEqual(findByRole(som, 'navigation').map((region) => region.id), ['r_nav']);
    assert.deepEqual(findByRole(som, 'article').map((region) => region.id), ['r_content']);
    for (const alias of ['content-info', 'content_info']) {
      assert.deepEqual(findByRole(som, alias).map((region) => region.id), ['r_footer'], alias);
    }

    const asideSom: Som = {
      ...som,
      regions: [...som.regions, { id: 'r_aside', role: 'aside', elements: [] }],
    };
    for (const alias of ['aside', 'asides', 'sidebar', 'sidebars', 'complementary']) {
      assert.deepEqual(findByRole(asideSom, alias).map((region) => region.id), ['r_aside'], alias);
    }
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
  it('finds elements by original HTML id', () => {
    assert.equal(findByHtmlId(fixture, 'home-link')?.id, 'e2');
  });

  it('returns undefined for missing HTML id', () => {
    assert.equal(findByHtmlId(fixture, 'missing-html-id'), undefined);
  });
});

describe('findByLabel', () => {
  it('finds label-only controls', () => {
    const results = findByLabel(fixture, 'email');
    assert.deepEqual(results.map((el) => el.id), ['e5']);
  });

  it('supports exact case-sensitive label matching', () => {
    assert.deepEqual(findByLabel(fixture, 'Email', { exact: true }).map((el) => el.id), ['e5']);
    assert.deepEqual(findByLabel(fixture, 'email', { exact: true }), []);
  });

  it('normalizes whitespace in exact and partial label matches', () => {
    const som: Som = {
      ...fixture,
      regions: fixture.regions.map((region) => ({
        ...region,
        elements: region.elements.map((element) =>
          element.id === 'e5' ? { ...element, label: 'Account\n settings' } : element,
        ),
      })),
    };

    assert.deepEqual(findByLabel(som, 'Account settings', { exact: true }).map((el) => el.id), ['e5']);
    assert.deepEqual(findByLabel(som, 'account settings').map((el) => el.id), ['e5']);
  });

  it('returns no matches for empty exact labels', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_empty_label',
          role: 'content',
          elements: [{ id: 'empty', role: 'button', label: '  ' }],
        },
      ],
    };

    assert.deepEqual(findByLabel(som, '', { exact: true }), []);
    assert.deepEqual(findByLabel(som, '\n\t', { exact: true }), []);
  });

  it('matches compiled label fallbacks', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_fallbacks',
          role: 'main',
          elements: [
            { id: 'aria', role: 'button', attrs: { aria: { label: 'Open menu' } } },
            { id: 'title', role: 'button', attrs: { title: 'Close dialog' } },
            { id: 'placeholder', role: 'text_input', attrs: { placeholder: 'Search products' } },
            { id: 'alt', role: 'image', attrs: { alt: 'Product preview' } },
          ],
        },
      ],
    };

    assert.deepEqual(findByLabel(som, 'menu').map((el) => el.id), ['aria']);
    assert.deepEqual(findByLabel(som, 'Close dialog', { exact: true }).map((el) => el.id), ['title']);
    assert.deepEqual(findByLabel(som, 'products').map((el) => el.id), ['placeholder']);
    assert.deepEqual(findByLabel(som, 'preview').map((el) => el.id), ['alt']);
  });
});

describe('findByTag', () => {
  it('finds elements by role', () => {
    const paragraphs = findByTag(fixture, 'paragraph');
    assert.equal(paragraphs.length, 3);
  });

  it('accepts case-insensitive selector aliases', () => {
    assert.deepEqual(findByTag(fixture, 'BUTTONS').map((el) => el.id), ['e4']);
    assert.deepEqual(findByTag(fixture, 'role: text-input').map((el) => el.id), ['e5']);
    assert.deepEqual(findByTag(fixture, '"role=paragraph"').map((el) => el.id), ['e3', 'e7', 'e9']);
    assert.deepEqual(findByTag(fixture, 'search box').map((el) => el.id), ['e5']);
    assert.deepEqual(findByTag(fixture, 'role=query_input').map((el) => el.id), ['e5']);
    for (const alias of [
      'search',
      'searches',
      'search-control',
      'search-field',
      'search-inputs',
      'query',
      'queries',
      'query-boxes',
      'query-control',
      'query-field',
      'query-inputs',
    ]) {
      assert.deepEqual(findByTag(fixture, alias).map((el) => el.id), ['e5'], alias);
    }
  });

  it('accepts HTML input type aliases', () => {
    for (const alias of [
      'text',
      'email',
      'number',
      'password',
      'tel',
      'url',
      'date',
      'datetime',
      'datetime-local',
      'date-time',
      'date-time-local',
      'month',
      'time',
      'week',
      'color',
      'range',
      'hidden',
      'file',
      'queryboxes',
    ]) {
      assert.deepEqual(findByTag(fixture, alias).map((el) => el.id), ['e5'], alias);
    }
  });

  it('accepts spaced text box aliases', () => {
    for (const alias of [
      'text_box',
      'text-box',
      'text boxes',
      'text-boxes',
      'text field',
      'text fields',
    ]) {
      assert.deepEqual(findByTag(fixture, alias).map((el) => el.id), ['e5'], alias);
    }
  });

  it('accepts spaced textarea aliases', () => {
    const textareaFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e10', role: 'textarea', text: 'Notes', actions: ['type', 'clear'] },
              ],
            }
          : region,
      ),
    };
    for (const alias of ['text_area', 'text-area', 'text area', 'text_areas', 'text-areas', 'text areas']) {
      assert.deepEqual(findByTag(textareaFixture, alias).map((el) => el.id), ['e10'], alias);
      assert.deepEqual(findActionTargetsByRole(textareaFixture, alias).map((item) => item.id), ['e10'], alias);
    }
  });

  it('accepts data table aliases', () => {
    const som: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [...region.elements, { id: 'e12', role: 'table', text: 'Orders' }],
            }
          : region,
      ),
    };
    for (const alias of ['data_table', 'data-tables', 'data tables']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['e12'], alias);
    }
  });

  it('accepts table and grid aliases', () => {
    const som: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [...region.elements, { id: 'e12', role: 'table', text: 'Orders' }],
            }
          : region,
      ),
    };
    for (const alias of [
      'data_grid',
      'data-grids',
      'data tables',
      'grid',
      'grids',
      'table_grid',
      'table-grids',
      'treegrid',
      'treegrids',
      'tree grids',
    ]) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['e12'], alias);
    }
  });

  it('accepts common HTML tag aliases', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_tags',
          role: 'main',
          elements: [
            { id: 'article', role: 'section' },
            { id: 'group', role: 'group' },
            { id: 'separator', role: 'separator' },
            { id: 'image', role: 'image' },
            { id: 'list', role: 'list' },
            { id: 'paragraph', role: 'paragraph' },
            { id: 'details', role: 'details' },
          ],
        },
      ],
      meta: fixture.meta,
    };
    const aliases: Record<string, string> = {
      article: 'article',
      div: 'group',
      fieldset: 'group',
      hr: 'separator',
      img: 'image',
      ol: 'list',
      p: 'paragraph',
      summary: 'details',
      ul: 'list',
    };
    for (const [alias, id] of Object.entries(aliases)) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), [id], alias);
    }
  });

  it('accepts structural and details aliases', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_structural',
          role: 'main',
          elements: [
            { id: 'group', role: 'group' },
            { id: 'details', role: 'details' },
          ],
        },
      ],
    };
    for (const alias of ['figure', 'radiogroup', 'progress', 'progressbar', 'meter', 'output']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['group'], alias);
    }
    for (const alias of ['detail', 'summaries']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['details'], alias);
    }
    const sectionSom: Som = {
      ...som,
      regions: [
        {
          id: 'r_section',
          role: 'main',
          elements: [{ id: 'section', role: 'section', text: 'A section' }],
        },
      ],
    };
    for (const alias of ['article', 'articles', 'sections']) {
      assert.deepEqual(findByTag(sectionSom, alias).map((el) => el.id), ['section'], alias);
    }
  });

  it('accepts blockquote aliases for paragraph elements', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_quote',
          role: 'main',
          elements: [{ id: 'quote', role: 'paragraph', text: 'A cited thought' }],
        },
      ],
    };
    for (const alias of ['blockquote', 'blockquotes', 'quote', 'quotes']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['quote'], alias);
    }
  });

  it('accepts ARIA paragraph aliases', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_messages',
          role: 'main',
          elements: [
            { id: 'alert', role: 'paragraph', text: 'Saved' },
            { id: 'status', role: 'paragraph', text: 'Ready' },
            { id: 'tooltip', role: 'paragraph', text: 'Help' },
          ],
        },
      ],
    };
    for (const alias of ['alert', 'status', 'tooltip']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['alert', 'status', 'tooltip'], alias);
    }
  });

  it('accepts media aliases for source-role elements', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_media',
          role: 'main',
          elements: [
            { id: 'image', role: 'image' },
            { id: 'audio', role: 'group', attrs: { source_role: 'audio' } },
            { id: 'video', role: 'group', attrs: { source_role: 'video' } },
            { id: 'canvas', role: 'group', attrs: { source_role: 'canvas' } },
          ],
        },
      ],
    };
    for (const alias of ['audio', 'audios', 'audio-item', 'audioitem', 'audio_items', 'audioitems']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['audio'], alias);
    }
    for (const alias of ['video', 'videos', 'video-item', 'videoitem', 'video_items', 'videoitems']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['video'], alias);
    }
    for (const alias of ['canvas', 'canvases', 'canvas-item', 'canvasitem', 'canvas_items', 'canvasitems']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['canvas'], alias);
    }
    for (const alias of ['media', 'media-item', 'media_items', 'mediaitems']) {
      assert.deepEqual(findByTag(som, alias).map((el) => el.id), ['image', 'audio', 'video', 'canvas'], alias);
    }
  });

  it('accepts ARIA button-role aliases', () => {
    for (const alias of [
      'menuitem',
      'menu-items',
      'option',
      'tabs',
      'tree-item',
    ]) {
      assert.deepEqual(findByTag(fixture, alias).map((el) => el.id), ['e4'], alias);
    }
    assert.deepEqual(findByTag(fixture, 'switch').map((el) => el.id), ['e8']);
    const selectFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e11', role: 'select', text: 'Country', actions: ['select'] },
              ],
            }
          : region,
      ),
    };
    for (const alias of [
      'combobox',
      'combo-box',
      'combo boxes',
      'listbox',
      'list-box',
      'list boxes',
    ]) {
      assert.deepEqual(findByTag(selectFixture, alias).map((el) => el.id), ['e11'], alias);
    }
    const radioFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e10', role: 'radio', text: 'Use saved address', actions: ['toggle'] },
              ],
            }
          : region,
      ),
    };
    assert.deepEqual(findByTag(radioFixture, 'menu-item-radio').map((el) => el.id), ['e10']);
    for (const alias of ['radio button', 'radio buttons', 'radiobutton', 'radiobuttons']) {
      assert.deepEqual(findByTag(radioFixture, alias).map((el) => el.id), ['e10'], alias);
    }
    const checkboxFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e12', role: 'checkbox', text: 'Remember me', actions: ['toggle'] },
              ],
            }
          : region,
      ),
    };
    for (const alias of ['check box', 'check boxes']) {
      assert.deepEqual(findByTag(checkboxFixture, alias).map((el) => el.id), ['e8', 'e12'], alias);
    }
    const spinButtonFixture: Som = {
      ...fixture,
      regions: fixture.regions.map((region) =>
        region.role === 'main'
          ? {
              ...region,
              elements: [
                ...region.elements,
                { id: 'e13', role: 'text_input', text: 'Quantity', actions: ['type'] },
              ],
            }
          : region,
      ),
    };
    for (const alias of ['slider', 'sliders', 'spin button', 'spin buttons', 'range']) {
      assert.deepEqual(findByTag(spinButtonFixture, alias).map((el) => el.id), ['e5', 'e13'], alias);
    }
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

  it('supports exact text matching without case folding', () => {
    assert.deepEqual(findByText(fixture, 'Hello World', { exact: true }).map((el) => el.id), ['e3']);
    assert.deepEqual(findByText(fixture, 'hello world', { exact: true }), []);
    assert.deepEqual(findByText(fixture, 'Hello', { exact: true }), []);
  });

  it('normalizes whitespace in text matches', () => {
    const som: Som = {
      ...fixture,
      regions: fixture.regions.map((region) => ({
        ...region,
        elements: region.elements.map((element) =>
          element.id === 'e3' ? { ...element, text: 'Hello\n  world' } : element,
        ),
      })),
    };

    assert.deepEqual(findByText(som, 'hello world').map((el) => el.id), ['e3']);
  });

  it('trims invisible separators at text boundaries', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_boundary',
          role: 'main',
          elements: [{ id: 'e_boundary', role: 'paragraph', text: '\u200BSave changes\uFEFF' }],
        },
      ],
    };

    assert.deepEqual(findByText(som, 'Save changes').map((el) => el.id), ['e_boundary']);
  });

  it('matches label-only controls', () => {
    const results = findByText(fixture, 'email');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'e5');
  });

  it('matches user-facing element metadata', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_metadata',
          role: 'main',
          elements: [
            { id: 'e_alt', role: 'image', attrs: { alt: 'Product wordmark' } },
            {
              id: 'e_placeholder',
              role: 'text_input',
              attrs: { placeholder: 'Search documentation' },
            },
            {
              id: 'e_value',
              role: 'button',
              attrs: { value: 'Search documentation' },
            },
            {
              id: 'e_progress',
              role: 'section',
              attrs: {
                aria: {
                  valuetext: '50 percent complete',
                  roledescription: 'progress indicator',
                },
              },
            },
            {
              id: 'e_aria_label',
              role: 'button',
              attrs: { aria: { label: 'Open settings' } },
            },
          ],
        },
      ],
    };

    assert.deepEqual(findByText(som, 'wordmark').map((el) => el.id), ['e_alt']);
    assert.deepEqual(findByText(som, 'documentation').map((el) => el.id), [
      'e_placeholder',
      'e_value',
    ]);
    assert.deepEqual(findByText(som, '50 percent').map((el) => el.id), ['e_progress']);
    assert.deepEqual(findByText(som, 'progress indicator').map((el) => el.id), [
      'e_progress',
    ]);
    assert.deepEqual(findByText(som, 'settings').map((el) => el.id), ['e_aria_label']);
  });

  it('finds text inside shadow roots', () => {
    const results = findByText(shadowFixture, 'inside shadow');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'shadow_text');
  });

  it('returns empty for no match', () => {
    assert.deepEqual(findByText(fixture, 'nonexistent'), []);
  });

  it('does not match every element for an empty query', () => {
    assert.deepEqual(findByText(fixture, '   '), []);
    assert.deepEqual(findByLabel(fixture, '\n\t'), []);
  });

  it('finds compiled list items', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_main',
          role: 'main',
          elements: [
            {
              id: 'e_list',
              role: 'list',
              attrs: {
                ordered: false,
                items: [{ text: 'Install' }, { text: 'Configure' }],
              },
            },
          ],
        },
      ],
    };

    const results = findByText(som, 'install');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'e_list');
    assert.deepEqual(
      findByText(som, 'Configure').map((el) => el.id),
      ['e_list'],
    );
    assert.deepEqual(findByText(som, 'nonexistent'), []);
  });

  it('finds compiled table captions', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_main',
          role: 'main',
          elements: [
            {
              id: 'e_table',
              role: 'table',
              attrs: {
                caption: 'Plans',
                headers: ['Plan', 'Price'],
                rows: [
                  ['Starter', '$9'],
                  ['Pro', '$29'],
                ],
              },
            },
          ],
        },
      ],
    };

    const results = findByText(som, 'plans');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'e_table');
    assert.deepEqual(
      findByText(som, 'Plans').map((el) => el.id),
      ['e_table'],
    );
    assert.deepEqual(findByText(som, 'nonexistent'), []);
  });

  it('finds compiled table rows', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_main',
          role: 'main',
          elements: [
            {
              id: 'e_table',
              role: 'table',
              attrs: {
                headers: ['Plan', 'Price'],
                rows: [
                  ['Starter', '$9'],
                  ['Pro', '$29'],
                ],
              },
            },
          ],
        },
      ],
    };

    const results = findByText(som, 'starter');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'e_table');
    assert.deepEqual(
      findByText(som, 'Pro').map((el) => el.id),
      ['e_table'],
    );
    assert.deepEqual(findByText(som, 'nonexistent'), []);
  });

  it('finds compiled select options', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_main',
          role: 'main',
          elements: [
            {
              id: 'e_select',
              role: 'select',
              attrs: {
                options: [
                  { value: 'us', text: 'United States' },
                  { value: 'ca', text: 'Canada' },
                ],
              },
            },
          ],
        },
      ],
    };

    const results = findByText(som, 'united states');
    assert.equal(results.length, 1);
    assert.equal(results[0].id, 'e_select');
    assert.deepEqual(
      findByText(som, 'Canada').map((el) => el.id),
      ['e_select'],
    );
    assert.deepEqual(findByText(som, 'nonexistent'), []);
  });
});

describe('getActionPlan', () => {
  it('treats spec-compliant aria-disabled controls as unavailable', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_main',
          role: 'main',
          elements: [
            {
              id: 'aria-disabled-button',
              role: 'button',
              text: 'Archive',
              actions: ['click'],
              attrs: { aria: { disabled: true } },
            },
          ],
        },
      ],
    };

    assert.deepEqual(getActionPlan(som)[0], {
      id: 'aria-disabled-button',
      cache_key: 'plasmate-action:v1:046c903b',
      role: 'button',
      actions: ['click'],
      enabled: false,
      label: 'Archive',
      disabled: true,
      blocked_reason: 'disabled',
    });
    assert.deepEqual(getEnabledActionPlan(som), []);
  });

  it('returns compact action targets', () => {
    const plan = getActionPlan(fixture);

    assert.deepEqual(plan[0], {
      id: 'e2',
      cache_key: 'plasmate-action:v1:da48f406',
      role: 'link',
      actions: ['click'],
      enabled: true,
      label: 'Home',
      html_id: 'home-link',
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

  it('indexes action targets for replay', () => {
    const { som, action_targets } = loadActionAvailabilityFixture();
    const save = action_targets.find((target) => target.id === 'e_save')!;
    const billing = action_targets[action_targets.length - 1];
    const index = getActionPlanIndex(som);

    assert.deepEqual(index.byId.e_save, save);
    assert.deepEqual(index.byCacheKey[save.cache_key], save);
    assert.deepEqual(index.byHtmlId['save-button'], save);
    assert.deepEqual(index.byTestId['settings-save'], save);
    assert.deepEqual(index.byLabel.Save, save);
    assert.deepEqual(index.byRole.button.map((target) => target.id), ['e_save', 'e_preview']);
    assert.deepEqual(index.byAction.click.map((target) => target.id), [
      'e_save',
      'e_preview',
      'e_billing',
    ]);
    assert.deepEqual(findActionTarget(som, 'e_save'), save);
    assert.deepEqual(findActionTarget(som, save.cache_key), save);
    assert.deepEqual(findActionTarget(som, 'save-button'), save);
    assert.deepEqual(findActionTarget(som, 'settings-save'), save);
    assert.deepEqual(findActionTarget(som, 'Save'), save);
    assert.deepEqual(findActionTarget(som, 'save'), save);
    assert.deepEqual(findActionTarget(som, 'Save', { by: 'label' }), save);
    assert.deepEqual(findActionTarget(som, 'settings-save', { by: 'test_id' }), save);
    assert.deepEqual(findActionTargetById(som, 'e_save'), save);
    assert.deepEqual(findActionTargetByCacheKey(som, save.cache_key), save);
    assert.deepEqual(findActionTargetByHtmlId(som, 'save-button'), save);
    assert.deepEqual(findActionTargetByTestId(som, 'settings-save'), save);
    assert.deepEqual(findActionTargetByLabel(som, 'Save'), save);
    assert.deepEqual(findActionTargetsByLabel(som, 'save'), [save]);
    assert.equal(findActionTargetByLabel(som, '  \n\t'), undefined);
    assert.deepEqual(findActionTargetsByLabel(som, '\t  '), []);
    assert.equal(findActionTarget(som, '  ', { by: 'label' }), undefined);
    assert.equal(findActionTarget(som, '\n\t'), undefined);
    assert.deepEqual(findActionTargetsByRole(som, 'button'), [
      action_targets[2],
      action_targets[3],
    ]);
    assert.deepEqual(findActionTargetsByAction(som, 'click'), [
      action_targets[2],
      action_targets[3],
      billing,
    ]);
    assert.equal(findActionTarget(som, 'settings-save', { enabledOnly: true }), undefined);
    assert.equal(findActionTarget(som, 'Save', { by: 'label', enabledOnly: true }), undefined);
    assert.equal(findActionTarget(som, 'sAvE', { enabledOnly: true }), undefined);
    assert.deepEqual(findActionTargetsByRole(som, 'button', { enabledOnly: true }), []);
    assert.deepEqual(findActionTargetsByAction(som, 'click', { enabledOnly: true }), [
      billing,
    ]);
  });

  it('normalizes whitespace when finding action labels', () => {
    const som: Som = {
      ...fixture,
      regions: fixture.regions.map((region) => ({
        ...region,
        elements: region.elements.map((element) =>
          element.id === 'e5' ? { ...element, label: 'Account\n settings' } : element,
        ),
      })),
    };

    assert.equal(findActionTargetByLabel(som, 'Account settings')?.id, 'e5');
    assert.deepEqual(findActionTargetsByLabel(som, 'account settings').map((item) => item.id), ['e5']);
  });

  it('uses compiled label fallbacks for action targets', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_fallback_action',
          role: 'main',
          elements: [
            {
              id: 'fallback-action',
              role: 'button',
              actions: ['click'],
              attrs: { aria: { label: 'Open settings' } },
            },
          ],
        },
      ],
    };

    assert.equal(findActionTargetByLabel(som, 'Open settings')?.id, 'fallback-action');
    assert.deepEqual(findActionTargetsByLabel(som, 'settings').map((item) => item.id), [
      'fallback-action',
    ]);
  });

  it('skips blank labels before using compiled fallbacks', () => {
    const som: Som = {
      ...fixture,
      regions: [
        {
          id: 'r_blank_label_action',
          role: 'main',
          elements: [
            {
              id: 'blank-label-action',
              role: 'button',
              label: '  \n\t',
              text: 'Fallback text',
              actions: ['click'],
              attrs: { title: 'Fallback title' },
            },
          ],
        },
      ],
    };

    const [target] = getActionPlan(som);
    assert.equal(target.label, 'Fallback text');
    assert.equal(findActionTargetByLabel(som, 'Fallback text')?.id, 'blank-label-action');
  });

  it('filters blocked targets from enabled action indexes', () => {
    const { som } = loadActionAvailabilityFixture();
    const enabled = getEnabledActionPlan(som);
    const index = getActionPlanIndex(som, { enabledOnly: true });

    assert.equal(enabled.every((target) => target.enabled), true);
    assert.equal(index.byId.e_save, undefined);
    assert.equal(index.byTestId['settings-save'], undefined);
    assert.equal(index.byLabel.Save, undefined);
    assert.equal(index.byRole.button, undefined);
    assert.deepEqual(index.byAction.click.map((target) => target.id), ['e_billing']);
    assert.notEqual(index.byId.e_plan, undefined);
  });

  it('preserves replay targets with reserved object-property names', () => {
    const identifierSom: Som = {
      som_version: '1.0',
      url: 'fixture',
      title: '',
      lang: 'en',
      regions: [
        {
          id: 'r_main',
          role: 'main',
          elements: [
            {
              id: '__proto__',
              role: 'button',
              html_id: 'constructor',
              label: 'toString',
              actions: ['click'],
              attrs: { test_id: 'hasOwnProperty' },
            },
          ],
        },
      ],
      meta: { html_bytes: 1, som_bytes: 1, element_count: 1, interactive_count: 1 },
    };
    const index = getActionPlanIndex(identifierSom);
    const byHtmlId = index.byHtmlId as Record<string, { id?: string }>;
    const byTestId = index.byTestId as Record<string, { id?: string }>;
    const byLabel = index.byLabel as Record<string, { id?: string }>;

    assert.equal(index.byId['__proto__']?.id, '__proto__');
    assert.equal(byHtmlId['constructor']?.id, '__proto__');
    assert.equal(byTestId['hasOwnProperty']?.id, '__proto__');
    assert.equal(byLabel['toString']?.id, '__proto__');

    const bucketSom = {
      ...identifierSom,
      regions: [
        {
          ...identifierSom.regions[0],
          elements: [{ id: 'bucket-collision', role: '__proto__', actions: ['__proto__'] }],
        },
      ],
    } as unknown as Som;
    const bucketIndex = getActionPlanIndex(bucketSom);

    assert.deepEqual(bucketIndex.byRole['__proto__'].map((target) => target.id), [
      'bucket-collision',
    ]);
    assert.deepEqual(bucketIndex.byAction['__proto__'].map((target) => target.id), [
      'bucket-collision',
    ]);
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

describe('getCompressionRatio', () => {
  it('returns the source-to-SOM byte ratio', () => {
    assert.equal(getCompressionRatio(fixture), 4);
  });

  it('returns Infinity when the SOM has no bytes', () => {
    assert.equal(
      getCompressionRatio({ ...fixture, meta: { ...fixture.meta, som_bytes: 0 } }),
      Infinity,
    );
  });
});
