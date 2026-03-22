# Coverage Scorecard (JS)

Real-world compression across the same curated set of 100 agent-relevant pages, with JavaScript execution enabled. Plasmate's SOM compiler reduces HTML to a semantic representation that agents can reason about efficiently. Higher compression = fewer tokens = lower cost.

- Data source: `coverage-js.json`
- Generator: `plasmate coverage --urls bench/top100.txt --output website/docs/coverage-js.json`
- Update cadence: scheduled GitHub Action

<div id="coverage-summary" style="margin: 16px 0; padding: 12px 14px; border: 1px solid rgba(240,237,232,0.12); border-radius: 10px; background: rgba(240,237,232,0.04);">
  Loading coverage data...
</div>

<div style="overflow-x:auto;">
  <table id="coverage-table">
    <thead>
      <tr>
        <th>URL</th>
        <th>HTTP</th>
        <th>Compression</th>
        <th>HTML bytes</th>
        <th>SOM bytes</th>
        <th>Elements</th>
        <th>Interactive</th>
        <th>Parse ms</th>
        <th>JS scripts</th>
        <th>JS failed</th>
        <th>Status</th>
      </tr>
    </thead>
    <tbody></tbody>
  </table>
</div>

<script>
(async function() {
  const summaryEl = document.getElementById('coverage-summary');
  const tbody = document.querySelector('#coverage-table tbody');

  function esc(s) {
    return String(s || '').replace(/[&<>\"']/g, (c) => ({
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&quot;',
      "'": '&#39;'
    }[c]));
  }

  function fmt(n) {
    if (n === null || n === undefined) return '';
    return String(n);
  }

  function fmtRatio(n) {
    if (n === null || n === undefined) return '';
    return n.toFixed(1) + 'x';
  }

  function statusBadge(status) {
    const s = String(status || '');
    if (s === 'ok') return `<span style="display:inline-block; padding: 2px 8px; border-radius: 999px; border: 1px solid rgba(240,237,232,0.18); background: rgba(240,237,232,0.04); color: #3bd4a7; font-family: var(--font-mono); font-size: 11px;">ok</span>`;
    if (s === 'blocked') return `<span style="display:inline-block; padding: 2px 8px; border-radius: 999px; border: 1px solid rgba(240,237,232,0.18); background: rgba(240,237,232,0.04); color: #F5C842; font-family: var(--font-mono); font-size: 11px;">\uD83D\uDD12 blocked</span>`;
    return `<span style="display:inline-block; padding: 2px 8px; border-radius: 999px; border: 1px solid rgba(240,237,232,0.18); background: rgba(240,237,232,0.04); color: #E8853A; font-family: var(--font-mono); font-size: 11px;">${esc(s)}</span>`;
  }

  try {
    const res = await fetch('coverage-js.json', { cache: 'no-store' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();

    const s = data.summary || {};
    const parseable = (s.urls_total || 0) - (s.blocked || 0);
    summaryEl.innerHTML = `
      <div style="display:flex; flex-wrap:wrap; gap: 14px; align-items:baseline;">
        <div><strong>Parsed</strong>: ${fmt(s.ok)} / ${fmt(parseable)} sites (${fmt((s.parsed_percent || 0).toFixed ? s.parsed_percent.toFixed(1) : s.parsed_percent)}%)</div>
        <div><strong>Median compression</strong>: ${fmtRatio(s.median_ratio)}</div>
        <div><strong>Mean compression</strong>: ${fmtRatio(s.mean_ratio)}</div>
        <div><strong>Blocked</strong>: ${fmt(s.blocked)} <span style="color: rgba(240,237,232,0.52); font-size: 11px;">(sites that returned 403/401)</span></div>
        <div style="color: rgba(240,237,232,0.72); font-family: var(--font-mono); font-size: 11px;">Generated: ${esc(data.generated_at_utc || '')}, version: ${esc(data.plasmate_version || '')}</div>
      </div>
    `;

    const results = Array.isArray(data.results) ? data.results : [];

    // Sort by compression ratio descending (best results first).
    results.sort((a, b) => (b.compression_ratio || 0) - (a.compression_ratio || 0));

    tbody.innerHTML = results.map(r => {
      const url = r.final_url || r.input_url;
      const http = r.http_status || '';
      const parseMs = r.pipeline_ms != null ? r.pipeline_ms : r.fetch_ms;

      return `
        <tr>
          <td style="max-width: 320px; overflow:hidden; text-overflow: ellipsis; white-space: nowrap;">
            <a href="${esc(url)}" target="_blank" rel="noreferrer">${esc(url)}</a>
          </td>
          <td>${esc(http)}</td>
          <td>${fmtRatio(r.compression_ratio)}</td>
          <td>${esc(fmt(r.html_bytes))}</td>
          <td>${esc(fmt(r.som_bytes))}</td>
          <td>${esc(fmt(r.element_count))}</td>
          <td>${esc(fmt(r.interactive_count))}</td>
          <td>${esc(fmt(parseMs))}</td>
          <td>${esc(fmt(r.js_total_scripts))}</td>
          <td>${esc(fmt(r.js_failed))}</td>
          <td>${statusBadge(r.status)}</td>
        </tr>
      `;
    }).join('');

  } catch (e) {
    summaryEl.innerHTML = `<strong>Failed to load coverage-js.json</strong><br/>${esc(e && e.message ? e.message : e)}`;
    tbody.innerHTML = '';
  }
})();
</script>
