import { describe, it, expect } from 'vitest';
import {
  parseSom,
  isValidSom,
  fromPlasmate,
  getAllElements,
  findByRole,
  findById,
  findByText,
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
        { id: 'e_1', role: 'link', text: 'Home', actions: ['click'], attrs: { href: '/' } },
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
        {
          id: 'e_6',
          role: 'image',
          html_id: 'logo',
          attrs: { src: '/logo.png', alt: 'Logo' },
          shadow: {
            mode: 'open',
            elements: [
              {
                id: 'e_shadow',
                role: 'button',
                text: 'Shadow Action',
                actions: ['click'],
              },
            ],
          },
        },
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
          attrs: { input_type: 'text', placeholder: 'Search...' },
        },
        { id: 'e_8', role: 'button', text: 'Go', actions: ['click'] },
      ],
    },
  ],
  meta: {
    html_bytes: 5000,
    som_bytes: 800,
    element_count: 9,
    interactive_count: 6,
  },
};

const FIXTURE_JSON = JSON.stringify(FIXTURE);

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

  it('extracts JSON from surrounding text', () => {
    const output = `Processing https://example.com/...\n${FIXTURE_JSON}\nDone.`;
    const som = fromPlasmate(output);
    expect(som.title).toBe('Example Domain');
  });

  it('throws when no JSON found', () => {
    expect(() => fromPlasmate('no json here')).toThrow();
  });
});

// ---- Query tests ----

describe('getAllElements', () => {
  it('returns all 9 elements', () => {
    expect(getAllElements(FIXTURE)).toHaveLength(9);
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
    expect(interactive).toHaveLength(6);
    const ids = interactive.map((e) => e.id);
    expect(ids).toContain('e_1');
    expect(ids).toContain('e_2');
    expect(ids).toContain('e_5');
    expect(ids).toContain('e_shadow');
    expect(ids).toContain('e_7');
    expect(ids).toContain('e_8');
  });
});

describe('getLinks', () => {
  it('returns all links with text, href, id', () => {
    const links = getLinks(FIXTURE);
    expect(links).toHaveLength(3);
    expect(links[0]).toEqual({ text: 'Home', href: '/', id: 'e_1' });
    expect(links[2]).toEqual({ text: 'Learn more', href: 'https://example.org', id: 'e_5' });
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
    expect(text).toContain('Shadow Action');
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
    expect(buttons).toHaveLength(2);
    expect(buttons.map((b) => b.text)).toEqual(['Shadow Action', 'Go']);
  });

  it('returns empty for no matches', () => {
    expect(filter(FIXTURE, () => false)).toHaveLength(0);
  });
});
