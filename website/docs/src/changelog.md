      <h1>Changelog</h1>
      <p>All notable changes to Plasmate, tracked by version and date.</p>

      <h2><span class="version-tag">v0.3.0</span> <span class="date-tag">2026-03-22</span></h2>
      <p>SPA rendering, interaction APIs, plugin system, and multi-session support. ~25K lines Rust, 200+ tests passing.</p>
      <ul>
        <li><span class="change-type feat">feat</span> <strong>Network request interception</strong> - block, modify, or mock responses.</li>
        <li><span class="change-type feat">feat</span> <strong>TLS configuration</strong> - cipher suite and fingerprint tuning.</li>
        <li><span class="change-type feat">feat</span> <strong>Wasm plugin system</strong> - optional feature flag, wasmtime runtime.</li>
        <li><span class="change-type feat">feat</span> <strong>SPA rendering bridge</strong> - bidirectional V8/rcdom DOM sync, SOM recompiled after JS.</li>
        <li><span class="change-type feat">feat</span> <strong>Timer queue drain</strong> for React/Vue deferred rendering.</li>
        <li><span class="change-type feat">feat</span> <strong>page.click(), page.type(), page.waitForSelector()</strong> via DOM bridge.</li>
        <li><span class="change-type feat">feat</span> <strong>Deep SPA hydration</strong> - insertBefore, replaceChild, cloneNode, classList.</li>
        <li><span class="change-type feat">feat</span> <strong>Multi-page session manager</strong> - 50 concurrent sessions per instance.</li>
        <li><span class="change-type feat">feat</span> <strong>Auth profiles</strong> with encrypted cookie storage.</li>
        <li><span class="change-type feat">feat</span> <strong>Screenshots protocol surface</strong> (SOM fallback until renderer lands).</li>
      </ul>

      <h3>Stats</h3>
      <ul>
        <li><strong>Binary:</strong> 45.8 MB default / 54 MB with plugins</li>
        <li><strong>Memory:</strong> 28 MB RSS per process</li>
        <li><strong>Tests:</strong> 200+ passing</li>
        <li><strong>Codebase:</strong> ~25K lines Rust</li>
      </ul>

      <hr />

      <h2><span class="version-tag">v0.2.0</span> <span class="date-tag">2026-03-21</span></h2>
      <p>V8 integration, CDP server, SDK expansion, and coverage tooling.</p>
      <ul>
        <li><span class="change-type feat">feat</span> <strong>V8 JavaScript integration</strong> with full DOM shim.</li>
        <li><span class="change-type feat">feat</span> <strong>CDP server</strong> for Puppeteer/Playwright compatibility.</li>
        <li><span class="change-type feat">feat</span> <strong>MCP server mode</strong> (stdio JSON-RPC).</li>
        <li><span class="change-type feat">feat</span> <strong>SOM Specification v1.0</strong> with JSON Schema and conformance test suite.</li>
        <li><span class="change-type feat">feat</span> <strong>Node.js SDK</strong> with full TypeScript types.</li>
        <li><span class="change-type feat">feat</span> <strong>Python SDK</strong> with Pydantic models.</li>
        <li><span class="change-type feat">feat</span> <strong>Go SDK</strong> with structs, client, and query helpers.</li>
        <li><span class="change-type feat">feat</span> <strong>Throughput CI</strong> benchmark expansion to 100 URLs across 13 categories.</li>
        <li><span class="change-type feat">feat</span> <strong>CDP cookie jar</strong> (getCookies, setCookies, deleteCookies, clearBrowserCookies).</li>
        <li><span class="change-type docs">docs</span> Browser Use and LangChain integration pages.</li>
        <li><span class="change-type docs">docs</span> Interactive coverage scorecards (nightly HTML, weekly JS).</li>
      </ul>

      <hr />

      <h2><span class="version-tag">v0.1.1</span> <span class="date-tag">2026-03-20</span></h2>
      <p>Major JS runtime improvements. Coverage jumps from 71% to 82%.</p>
      <ul>
        <li><span class="change-type feat">feat</span> Functional <strong>MutationObserver</strong> for SPA framework support (React, Vue, Next.js). Observer callbacks now fire on DOM mutations, enabling client-side rendering in the headless engine.</li>
        <li><span class="change-type feat">feat</span> <strong>URL/URLSearchParams polyfills</strong> with try-catch constructor probing. Detects partial V8 built-ins and only overrides when necessary.</li>
        <li><span class="change-type feat">feat</span> <strong>JS coverage scorecard</strong> tracking 100 real-world sites with per-site element counts, JS failure rates, and error details.</li>
        <li><span class="change-type fix">fix</span> <strong>Browser-realistic HTTP headers</strong> to avoid anti-bot blocking (zero speed/memory cost).</li>
        <li><span class="change-type fix">fix</span> <strong>Pre/post JS SOM comparison</strong> keeps whichever SOM has more elements, preventing JS from degrading content.</li>
        <li><span class="change-type fix">fix</span> Skip <code>type="module"</code> external scripts (prevents <code>SyntaxError: Cannot use import statement outside a module</code>).</li>
        <li><span class="change-type fix">fix</span> <code>var self = globalThis</code> for SPA frameworks that reference <code>self</code>.</li>
        <li><span class="change-type perf">perf</span> Configurable JS safety budgets: <code>--max-external-scripts</code>, <code>--max-external-script-kb</code>, <code>--external-script-timeout-ms</code>.</li>
      </ul>

      <h3>Coverage Results</h3>
      <ul>
        <li><strong>HTML scorecard:</strong> 80/100 Full (80%)</li>
        <li><strong>JS scorecard:</strong> 79/98 Full (80.6%)</li>
        <li><strong>New Full sites:</strong> amazon.com, ebay.com (via MutationObserver)</li>
        <li><strong>Remaining thin:</strong> 14 sites needing deeper DOM mutation support</li>
        <li><strong>Failed:</strong> 4 (anti-bot: etsy, tripadvisor, wsj; non-HTML: httpbin)</li>
      </ul>

      <hr />

      <h2><span class="version-tag">v0.1.0</span> <span class="date-tag">2026-03-01</span></h2>
      <p>Initial public release. The browser engine for agents.</p>
      <ul>
        <li><span class="change-type feat">feat</span> <strong>Semantic Object Model (SOM)</strong> compiler: HTML to structured, token-efficient representation.</li>
        <li><span class="change-type feat">feat</span> <strong>Agent Web Protocol (AWP)</strong>: 7-method protocol for agent-native browsing (navigate, snapshot, click, type, scroll, select, extract).</li>
        <li><span class="change-type feat">feat</span> <strong>V8 JavaScript runtime</strong> with DOM shim for JS-rendered pages.</li>
        <li><span class="change-type feat">feat</span> <strong>CDP compatibility layer</strong> for Puppeteer/Playwright drop-in usage.</li>
        <li><span class="change-type feat">feat</span> <strong>MCP server mode</strong> (<code>plasmate mcp</code>) with stateful tools: open_page, evaluate, click, close_page.</li>
        <li><span class="change-type feat">feat</span> <strong>Cookie-based auth profiles</strong> with AES-256-GCM encryption at rest.</li>
        <li><span class="change-type feat">feat</span> <strong>Chrome extension bridge</strong> for importing auth sessions from a real browser.</li>
        <li><span class="change-type feat">feat</span> <strong>Docker image</strong> on GHCR (multi-arch).</li>
        <li><span class="change-type feat">feat</span> <strong>Node.js and Python SDKs</strong>.</li>
        <li><span class="change-type perf">perf</span> 4-5ms per page (50x faster than Chrome, 5x faster than Lightpanda).</li>
        <li><span class="change-type perf">perf</span> ~30MB memory for 100 concurrent pages.</li>
        <li><span class="change-type perf">perf</span> 10.4x token compression vs raw DOM.</li>
      </ul>

      <h3>Architecture</h3>
      <ul>
        <li><strong>Language:</strong> Rust (16,740 lines at launch, 171 tests)</li>
        <li><strong>Binary:</strong> 43 MB single file, no dependencies</li>
        <li><strong>License:</strong> Apache 2.0</li>
      </ul>
