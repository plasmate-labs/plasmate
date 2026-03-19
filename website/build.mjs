#!/usr/bin/env node
/**
 * Plasmate docs build script.
 * Converts markdown in docs/src/ to branded HTML in docs/.
 */
import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync } from 'node:fs';
import { join, basename } from 'node:path';
import { marked } from 'marked';

const SRC = join(import.meta.dirname, 'docs', 'src');
const OUT = join(import.meta.dirname, 'docs');

const NAV = [
  { slug: 'overview', label: 'Overview' },
  { slug: 'quickstart', label: 'Quick Start' },
  { slug: 'spec', label: 'Product Spec' },
  { slug: 'som', label: 'SOM Reference' },
  { slug: 'awp', label: 'AWP Protocol' },
  { slug: 'awp-mvp', label: 'AWP MVP v0.1' },
  { slug: 'roadmap', label: 'Roadmap' },
  { slug: 'coverage', label: 'Coverage' },
  { slug: 'brand', label: 'Brand Guide' },
];

function template(title, body, currentSlug) {
  const sidebar = NAV.map(n => {
    const active = n.slug === currentSlug ? ' class="active"' : '';
    const href = n.slug === 'overview' ? '.' : n.slug;
    return `<a href="${href}"${active}>${n.label}</a>`;
  }).join('\n          ');

  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <title>${title} - Plasmate Docs</title>
  <meta name="description" content="Plasmate documentation: ${title}" />

  <link rel="icon" href="/favicon.ico" sizes="any" />
  <link rel="icon" href="/favicon-32x32.png" type="image/png" sizes="32x32" />
  <link rel="apple-touch-icon" href="/apple-touch-icon.png" />

  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap" rel="stylesheet" />

  <style>
    :root {
      --plasma-white: #F0EDE8;
      --core-black: #0D0D0D;
      --void: #111110;
      --deep: #1A1918;
      --smoke: #2A2825;
      --ash: #6B6560;
      --dim: #8A8480;
      --ion: #D4C5A0;
      --ember: #E8853A;
      --arc: #3D8FD4;
      --corona: #F5C842;

      --font-display: "Space Grotesk", system-ui, sans-serif;
      --font-body: "IBM Plex Sans", system-ui, sans-serif;
      --font-mono: "IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, monospace;

      --sidebar-w: 240px;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      background: var(--void);
      color: var(--plasma-white);
      font-family: var(--font-body);
      font-size: 15px;
      line-height: 1.65;
      -webkit-font-smoothing: antialiased;
    }

    .layout {
      display: flex;
      min-height: 100vh;
    }

    /* ---- Sidebar ---- */
    .sidebar {
      position: fixed;
      top: 0;
      left: 0;
      width: var(--sidebar-w);
      height: 100vh;
      overflow-y: auto;
      background: var(--core-black);
      border-right: 1px solid var(--smoke);
      padding: 22px 16px 40px;
      display: flex;
      flex-direction: column;
      gap: 4px;
      z-index: 10;
    }

    .sidebar-brand {
      font-family: var(--font-display);
      font-weight: 600;
      font-size: 14px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--plasma-white);
      text-decoration: none;
      padding: 0 8px 14px;
      border-bottom: 1px solid var(--smoke);
      margin-bottom: 10px;
      display: flex;
      align-items: center;
      gap: 10px;
    }

    .sidebar-brand .dot {
      width: 8px;
      height: 8px;
      border-radius: 999px;
      background: var(--ember);
    }

    .sidebar a {
      display: block;
      padding: 7px 10px;
      border-radius: 8px;
      color: var(--dim);
      text-decoration: none;
      font-size: 13px;
      font-family: var(--font-body);
      font-weight: 500;
      transition: color 0.1s, background 0.1s;
    }

    .sidebar a:hover {
      color: var(--plasma-white);
      background: rgba(240,237,232,0.06);
    }

    .sidebar a.active {
      color: var(--plasma-white);
      background: rgba(232,133,58,0.14);
    }

    .sidebar-foot {
      margin-top: auto;
      padding-top: 16px;
      border-top: 1px solid var(--smoke);
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--ash);
      display: flex;
      flex-direction: column;
      gap: 4px;
    }

    .sidebar-foot a { padding: 4px 10px; font-size: 11px; font-family: var(--font-mono); }

    /* ---- Content ---- */
    .content {
      margin-left: var(--sidebar-w);
      flex: 1;
      max-width: 820px;
      padding: 36px 40px 80px;
    }

    /* ---- Typography ---- */
    .content h1 {
      font-family: var(--font-display);
      font-weight: 650;
      font-size: 32px;
      line-height: 1.15;
      letter-spacing: -0.015em;
      margin-bottom: 18px;
      padding-bottom: 14px;
      border-bottom: 1px solid var(--smoke);
    }

    .content h2 {
      font-family: var(--font-display);
      font-weight: 600;
      font-size: 22px;
      line-height: 1.25;
      margin-top: 38px;
      margin-bottom: 12px;
      color: var(--ion);
    }

    .content h3 {
      font-family: var(--font-display);
      font-weight: 600;
      font-size: 17px;
      margin-top: 28px;
      margin-bottom: 8px;
    }

    .content h4, .content h5, .content h6 {
      font-family: var(--font-display);
      font-weight: 600;
      font-size: 15px;
      margin-top: 22px;
      margin-bottom: 6px;
      color: var(--dim);
    }

    .content p { margin-bottom: 14px; }

    .content a { color: var(--arc); text-decoration: none; }
    .content a:hover { text-decoration: underline; }

    .content strong { color: var(--plasma-white); font-weight: 600; }

    .content ul, .content ol {
      padding-left: 22px;
      margin-bottom: 14px;
    }

    .content li { margin-bottom: 4px; }

    .content blockquote {
      border-left: 3px solid var(--ember);
      padding: 10px 16px;
      margin: 16px 0;
      background: rgba(232,133,58,0.06);
      border-radius: 0 8px 8px 0;
      color: rgba(240,237,232,0.88);
    }

    .content blockquote p { margin-bottom: 0; }

    .content hr {
      border: none;
      height: 1px;
      background: var(--smoke);
      margin: 30px 0;
    }

    /* ---- Code ---- */
    .content code {
      font-family: var(--font-mono);
      font-size: 13px;
      background: rgba(240,237,232,0.08);
      padding: 2px 6px;
      border-radius: 4px;
      color: var(--ion);
    }

    .content pre {
      background: var(--core-black);
      border: 1px solid var(--smoke);
      border-radius: 10px;
      padding: 16px 18px;
      overflow-x: auto;
      margin: 14px 0 18px;
      line-height: 1.55;
    }

    .content pre code {
      background: none;
      padding: 0;
      border-radius: 0;
      color: rgba(240,237,232,0.88);
      font-size: 13px;
    }

    /* ---- Tables ---- */
    .content table {
      width: 100%;
      border-collapse: collapse;
      margin: 14px 0 18px;
      font-size: 13px;
      font-family: var(--font-mono);
    }

    .content th {
      text-align: left;
      padding: 10px 12px;
      border-bottom: 2px solid var(--smoke);
      color: var(--ash);
      font-weight: 500;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      font-size: 11px;
    }

    .content td {
      padding: 8px 12px;
      border-bottom: 1px solid rgba(240,237,232,0.08);
      color: rgba(240,237,232,0.84);
    }

    .content tr:hover td {
      background: rgba(240,237,232,0.03);
    }

    /* ---- Images ---- */
    .content img {
      max-width: 100%;
      border-radius: 8px;
    }

    /* ---- Mobile ---- */
    @media (max-width: 860px) {
      .sidebar {
        position: static;
        width: 100%;
        height: auto;
        border-right: none;
        border-bottom: 1px solid var(--smoke);
        flex-direction: row;
        flex-wrap: wrap;
        gap: 2px;
        padding: 12px;
      }

      .sidebar-brand {
        width: 100%;
        border-bottom: none;
        padding-bottom: 8px;
        margin-bottom: 4px;
      }

      .sidebar-foot { display: none; }

      .layout { flex-direction: column; }

      .content {
        margin-left: 0;
        padding: 24px 18px 60px;
      }
    }
  </style>
</head>
<body>
  <div class="layout">
    <nav class="sidebar" aria-label="Documentation navigation">
      <a class="sidebar-brand" href="https://plasmate.app"><img src="/brand/plasmate-mark.png" alt="" width="22" height="22" style="vertical-align: -3px;" />Plasmate</a>
      ${sidebar}
      <div class="sidebar-foot">
        <a href="https://github.com/plasmate-labs/plasmate">GitHub</a>
        <a href="https://plasmate.app">plasmate.app</a>
        <span style="padding: 4px 10px;">Apache 2.0 / v0.1</span>
      </div>
    </nav>
    <main class="content">
      ${body}
    </main>
  </div>
</body>
</html>`;
}

// Strip YAML frontmatter
function stripFrontmatter(md) {
  if (md.startsWith('---')) {
    const end = md.indexOf('---', 3);
    if (end !== -1) return md.slice(end + 3).trim();
  }
  return md;
}

// Extract first H1 as title
function extractTitle(md) {
  const m = md.match(/^#\s+(.+)$/m);
  return m ? m[1].trim() : 'Plasmate Docs';
}

// Configure marked
marked.setOptions({
  gfm: true,
  breaks: false,
});

// Build
if (!existsSync(OUT)) mkdirSync(OUT, { recursive: true });

const files = readdirSync(SRC).filter(f => f.endsWith('.md'));
let built = 0;

for (const file of files) {
  const raw = readFileSync(join(SRC, file), 'utf-8');
  const md = stripFrontmatter(raw);
  const title = extractTitle(md);
  const html = marked.parse(md);
  const slug = basename(file, '.md');
  const page = template(title, html, slug);

  writeFileSync(join(OUT, `${slug}.html`), page);
  built++;
  console.log(`  ${slug}.html  (${title})`);
}

console.log(`\nBuilt ${built} pages -> ${OUT}/`);
