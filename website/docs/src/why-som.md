      <h1>Why SOM: The Case for a Semantic Web Format for AI Agents</h1>

      <p>The web was built for humans looking at pixels. AI agents don't need pixels. They need meaning.</p>

      <p>Every day, millions of agent API calls send raw HTML to language models, paying for CSS classes, script tags, tracking pixels, and layout divs that carry zero semantic value. SOM fixes this.</p>

      <hr>

      <h2>The Problem</h2>

      <p>A typical web page weighs 300-500KB of HTML. Between <strong>80% and 95%</strong> of that markup is presentation: class names, inline styles, script blocks, SVG paths, tracking pixels, and deeply nested layout divs. None of it carries meaning.</p>

      <p>But when an AI agent reads a web page, all of that noise goes straight into the context window. And context windows cost money.</p>

      <div class="hero-stats">
        <div class="stat-card">
          <div class="stat-value">$10/M</div>
          <div class="stat-label">GPT-4 Input Tokens</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">80-95%</div>
          <div class="stat-label">HTML That's Noise</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">$50+/day</div>
          <div class="stat-label">Wasted at 1K pages/day</div>
        </div>
      </div>

      <p>Here's the deeper issue: the DOM is a <strong>rendering tree</strong>, not a <strong>meaning tree</strong>. It tells you WHERE things go on screen, not WHAT things are. A <code>&lt;div&gt;</code> with twelve CSS classes might be a navigation link, a button, a heading, or a decorative container. The DOM doesn't know and doesn't care. It was designed to paint pixels, not convey semantics.</p>

      <p>AI agents deserve better input than a rendering tree with the renderer removed.</p>

      <hr>

      <h2>What SOM Is</h2>

      <p><strong>SOM (Semantic Object Model)</strong> is a structured JSON representation of web content designed for machine consumption. It takes the meaningful content of a web page and expresses it in a format that LLMs can process efficiently.</p>

      <p>Instead of this:</p>

      <div class="compare-box">
        <div class="compare-label">Raw HTML</div>
        <pre><code>&lt;div class="sc-1234 flex items-center gap-2"&gt;
  &lt;a href="/about" class="text-blue-500 hover:underline
     font-medium tracking-tight"&gt;About&lt;/a&gt;
&lt;/div&gt;</code></pre>
      </div>

      <p>SOM gives you this:</p>

      <div class="compare-box">
        <div class="compare-label">SOM Output</div>
        <pre><code>{
  "role": "link",
  "text": "About",
  "attrs": { "href": "/about" },
  "actions": ["click"]
}</code></pre>
      </div>

      <p>Same information. Fraction of the tokens. And the agent actually knows it's a clickable link.</p>

      <h3>Key Properties</h3>

      <ul>
        <li><strong>Semantic roles</strong> (link, button, heading, paragraph, form, input) instead of div/span/a</li>
        <li><strong>Actionable attributes only</strong> (href, value, placeholder) instead of class, style, data-*</li>
        <li><strong>Region-based structure</strong> (navigation, content, form, footer) instead of arbitrary nesting</li>
        <li><strong>Explicit interactivity</strong>: every interactive element is marked with its available actions (click, type, select)</li>
        <li><strong>Structured data extraction</strong>: JSON-LD, OpenGraph, and meta tags normalized into a clean object</li>
      </ul>

      <hr>

      <h2>The Numbers</h2>

      <p>We benchmarked SOM against raw HTML on <strong>49 real-world websites</strong>. Not toy examples. Real production pages from Google Cloud, Reddit, Stripe, The New York Times, and 45 others.</p>

      <div class="hero-stats">
        <div class="stat-card">
          <div class="stat-value">16.6x</div>
          <div class="stat-label">Overall Compression</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">10.5x</div>
          <div class="stat-label">Median Compression</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">94%</div>
          <div class="stat-label">Cost Savings</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">49</div>
          <div class="stat-label">Sites Tested</div>
        </div>
      </div>

      <table>
        <thead>
          <tr>
            <th>Model</th>
            <th>HTML Cost</th>
            <th>SOM Cost</th>
            <th>Savings</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>GPT-4 ($10/M)</td>
            <td>$50,397</td>
            <td>$3,042</td>
            <td class="savings">$47,355 (94%)</td>
          </tr>
          <tr>
            <td>GPT-4o ($2.50/M)</td>
            <td>$12,599</td>
            <td>$761</td>
            <td class="savings">$11,839 (94%)</td>
          </tr>
          <tr>
            <td>Claude Sonnet ($3/M)</td>
            <td>$15,119</td>
            <td>$913</td>
            <td class="savings">$14,207 (94%)</td>
          </tr>
        </tbody>
      </table>

      <p>Best case: <strong>cloud.google.com</strong> compressed 116.9x, from 464K tokens down to 4K. Even minimal sites like postgresql.org still showed 1.2x compression.</p>

      <p><a href="benchmark-cost">See the full benchmark with all 49 sites &rarr;</a></p>

      <hr>

      <h2>Why Not Just Strip Tags?</h2>

      <p>Common objection: "Just use BeautifulSoup or Cheerio to strip HTML tags. Problem solved."</p>

      <p>Not quite. Tag stripping is the wrong tool for this job:</p>

      <ul>
        <li><strong>Loses structure.</strong> You can't tell a navigation link from a content link from a footer link. They all become plain text.</li>
        <li><strong>Loses interactivity.</strong> You don't know what's clickable, typeable, or selectable. An agent needs to act on pages, not just read them.</li>
        <li><strong>Loses hierarchy.</strong> Headings, sections, and regions disappear. The page becomes a flat wall of text.</li>
        <li><strong>Lossy in the wrong direction.</strong> Tag stripping removes structure but keeps text noise: hidden elements, aria labels scattered everywhere, inline script content that leaked through.</li>
      </ul>

      <p>SOM is selective. It removes noise but preserves meaning. A stripped page is text. A SOM page is a structured document with roles, regions, and actions.</p>

      <hr>

      <h2>Why Not the Accessibility Tree?</h2>

      <p>Accessibility trees are designed for screen readers. They solve a related but fundamentally different problem.</p>

      <ul>
        <li><strong>Browser-dependent.</strong> You need a full browser runtime to generate an accessibility tree. SOM works from raw HTML, no browser required.</li>
        <li><strong>Visual layout information.</strong> Accessibility trees include bounding boxes, visual states, and layout hints that agents don't need.</li>
        <li><strong>Verbose.</strong> Every DOM node gets an accessibility role, even purely decorative ones. The tree inherits the DOM's depth and redundancy.</li>
        <li><strong>Not designed for action.</strong> Accessibility trees describe what things are for human assistive technology. SOM describes what things are AND what an agent can do with them.</li>
      </ul>

      <p>SOM is purpose-built for agent consumption: flat regions, semantic roles, explicit action annotations. It's what you'd design if you started from "what does an LLM need?" instead of "what does a screen reader need?"</p>

      <hr>

      <h2>Why Not Screenshots + Vision?</h2>

      <p>Vision models can look at screenshots. So why not just send a screenshot?</p>

      <ul>
        <li><strong>Token cost.</strong> Image tokens are 4-10x more expensive than text tokens. A screenshot of a page costs far more than its SOM representation.</li>
        <li><strong>Hallucination.</strong> Vision models hallucinate UI elements. They'll "see" buttons that aren't there and miss ones that are.</li>
        <li><strong>No structured data.</strong> You can't extract JSON-LD, form values, or link targets from pixels.</li>
        <li><strong>No interaction model.</strong> You can't identify elements by selector from a screenshot. You can't tell the model "click the third link in the navigation" if all it has is an image.</li>
      </ul>

      <p>Screenshots are appropriate for visual verification: "does this page look right?" They're not appropriate for primary page understanding. SOM gives agents the structured, actionable data they actually need.</p>

      <hr>

      <h2>SOM as a Standard</h2>

      <p>SOM isn't locked inside Plasmate. It's an open specification designed to be consumed by any tool, framework, or agent.</p>

      <ul>
        <li><strong><a href="som-spec">SOM Spec v1.0</a></strong> is published and stable</li>
        <li><strong>Standalone parsers</strong> available on <a href="https://www.npmjs.com/package/som-parser">npm</a> (<code>som-parser</code>) and <a href="https://pypi.org/project/som-parser/">PyPI</a> (<code>som-parser</code>)</li>
        <li><strong>Zero dependency</strong> on Plasmate to consume SOM output</li>
        <li><strong>JSON Schema validation</strong> available for tooling and CI</li>
        <li><strong>Apache 2.0 licensed</strong> with no IP restrictions</li>
      </ul>

      <p>You can generate SOM with Plasmate and consume it with anything. Or build your own SOM generator. The format is the standard, not the tool.</p>

      <hr>

      <h2>Who Benefits</h2>

      <ul>
        <li><strong>Agent framework developers</strong> (Browser Use, LangChain, CrewAI): lower token costs, faster inference, structured page data out of the box</li>
        <li><strong>Enterprise AI teams</strong>: predictable, structured web data instead of HTML soup. No more prompt-engineering around broken DOM structures.</li>
        <li><strong>Web scraping at scale</strong>: 10x reduction in LLM costs. When you're processing millions of pages, 94% savings is the difference between viable and bankrupt.</li>
        <li><strong>Tool-use agents</strong>: explicit action annotations tell the model exactly what's clickable, typeable, and selectable. No guessing.</li>
      </ul>

      <hr>

      <h2>Get Started</h2>

      <p>Try SOM in under a minute:</p>

      <pre><code># Install Plasmate
cargo install plasmate

# Fetch any URL as SOM
plasmate fetch https://example.com</code></pre>

      <p>Use SOM in your project:</p>

      <pre><code># Node.js
npm install som-parser

# Python
pip install som-parser</code></pre>

      <h3>Learn More</h3>

      <ul>
        <li><a href="som-spec">Read the SOM Spec v1.0</a></li>
        <li><a href="benchmark-cost">See the full Cost Analysis benchmark</a></li>
        <li><a href="som">Browse the SOM Reference</a></li>
        <li><a href="https://github.com/plasmate-labs/plasmate">Contribute on GitHub</a></li>
      </ul>
