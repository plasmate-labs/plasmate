import { describe, it } from 'node:test';
import * as assert from 'node:assert/strict';
import type { Som } from './types';
import {
  findByRole,
  findById,
  findByTag,
  findInteractive,
  findByText,
  flatElements,
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
        { id: 'e3', role: 'paragraph', text: 'Hello World' },
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

  it('returns undefined for missing ID', () => {
    assert.equal(findById(fixture, 'e999'), undefined);
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

  it('returns empty for no match', () => {
    assert.deepEqual(findByText(fixture, 'nonexistent'), []);
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
});

describe('getTokenEstimate', () => {
  it('estimates tokens from som_bytes', () => {
    const tokens = getTokenEstimate(fixture);
    assert.equal(tokens, 500); // 2000 / 4
  });
});
