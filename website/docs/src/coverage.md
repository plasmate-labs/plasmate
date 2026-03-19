# Coverage Scorecard

This page shows Plasmate's real-world coverage across a curated set of 100 agent-relevant pages.

- Data source: `coverage.json`
- Generator: `plasmate coverage --urls bench/top100.txt --output website/docs/coverage.json`
- Update cadence: scheduled GitHub Action (nightly)
- Current mode: HTML-only (JS disabled) until V8 heap limits are raised and the harness is hardened against fatal OOM on heavy sites

<div id="coverage-summary" style="margin: 16px 0; padding: 12px 14px; border: 1px solid rgba(240,237,232,0.12); border-radius: 10px; background: rgba(240,237,232,0.04);">
  Loading coverage data...
</div>

<div style="overflow-x:auto;">
  <table id="coverage-table">
    <thead>
      <tr>
        <th>Status</th>
        <th>URL</th>
        <th>HTTP</th>
        <th>Title</th>
        <th>HTML bytes</th>
        <th>SOM bytes</th>
        <th>Elements</th>
        <th>Interactive</th>
        <th>Fetch ms</th>
        <th>Pipeline ms</th>
        <th>Error</th>
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

  function statusBadge(status) {
    const s = String(status || '');
    const color = s === 'ok' ? '#3bd4a7' : (s === 'thin' ? '#F5C842' : '#E8853A');
    return `<span style="display:inline-block; padding: 2px 8px; border-radius: 999px; border: 1px solid rgba(240,237,232,0.18); background: rgba(240,237,232,0.04); color: ${color}; font-family: var(--font-mono); font-size: 11px;">${esc(s)}</span>`;
  }

  try {
    const res = await fetch('coverage.json', { cache: 'no-store' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();

    const s = data.summary || {};
    summaryEl.innerHTML = `
      <div style="display:flex; flex-wrap:wrap; gap: 14px; align-items:baseline;">
        <div><strong>OK</strong>: ${fmt(s.ok)} / ${fmt(s.urls_total)} (${fmt((s.ok_percent || 0).toFixed ? s.ok_percent.toFixed(1) : s.ok_percent)}%)</div>
        <div><strong>Thin</strong>: ${fmt(s.thin)}</div>
        <div><strong>Failed</strong>: ${fmt(s.failed)}</div>
        <div style="color: rgba(240,237,232,0.72); font-family: var(--font-mono); font-size: 11px;">Generated: ${esc(data.generated_at_utc || '')}, version: ${esc(data.plasmate_version || '')}</div>
      </div>
    `;

    const results = Array.isArray(data.results) ? data.results : [];
    tbody.innerHTML = results.map(r => {
      const url = r.final_url || r.input_url;
      const title = r.title || '';
      const http = r.http_status || '';
      const err = r.error || '';

      return `
        <tr>
          <td>${statusBadge(r.status)}</td>
          <td style="max-width: 320px; overflow:hidden; text-overflow: ellipsis; white-space: nowrap;">
            <a href="${esc(url)}" target="_blank" rel="noreferrer">${esc(url)}</a>
          </td>
          <td>${esc(http)}</td>
          <td style="max-width: 240px; overflow:hidden; text-overflow: ellipsis; white-space: nowrap;">${esc(title)}</td>
          <td>${esc(fmt(r.html_bytes))}</td>
          <td>${esc(fmt(r.som_bytes))}</td>
          <td>${esc(fmt(r.element_count))}</td>
          <td>${esc(fmt(r.interactive_count))}</td>
          <td>${esc(fmt(r.fetch_ms))}</td>
          <td>${esc(fmt(r.pipeline_ms))}</td>
          <td style="max-width: 280px; overflow:hidden; text-overflow: ellipsis; white-space: nowrap;">${esc(err)}</td>
        </tr>
      `;
    }).join('');

  } catch (e) {
    summaryEl.innerHTML = `<strong>Failed to load coverage.json</strong><br/>${esc(e && e.message ? e.message : e)}`;
    tbody.innerHTML = '';
  }
})();
</script>
