      <h1>Why SOM: The Case for a Semantic Web Format for AI Agents</h1>

      <p>The web was built for humans looking at pixels. AI agents don't need pixels. They need meaning.</p>

      <p>Every day, millions of agent API calls send raw HTML to language models, paying for CSS classes, script tags, tracking pixels, and layout divs that carry zero semantic value. SOM fixes this.</p>

      <hr>

      <h2>The Problem</h2>

      <p>Web pages commonly contain presentation and runtime markup—class names, inline styles, script blocks, SVG paths, tracking elements, and deeply nested layout containers—that is not useful to every agent task.</p>

      <p>But when an AI agent reads a web page, all of that noise goes straight into the context window. And context windows cost money.</p>

      <p>How much of that material can be removed depends on the page, the SOM
      selector and budget, JavaScript mode, serialization, and the downstream
      tokenizer. Measure the exact workflow rather than applying a universal
      savings percentage.</p>

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

      <p>The example preserves the link's meaning and action while omitting its
      presentation classes. Whether it uses fewer tokens, and by how much,
      depends on the surrounding page and tokenizer.</p>

      <h3>Key Properties</h3>

      <ul>
        <li><strong>Semantic roles</strong> (link, button, heading, paragraph, form, input) instead of div/span/a</li>
        <li><strong>Actionable attributes only</strong> (href, value, placeholder) instead of class, style, data-*</li>
        <li><strong>Region-based structure</strong> (navigation, content, form, footer) instead of arbitrary nesting</li>
        <li><strong>Explicit interactivity</strong>: every interactive element is marked with its available actions (click, type, select)</li>
        <li><strong>Structured data extraction</strong>: JSON-LD, OpenGraph, and meta tags normalized into a clean object</li>
      </ul>

      <hr>

      <h2>Retained Output-Size Evidence</h2>

      <p>The retained v0.5.1 public-web snapshots attempted 98 URLs per run. The
      non-JavaScript snapshot recorded a 9.98x median serialized-byte ratio over
      83 successful inputs; the JavaScript snapshot recorded a 9.32x median
      over 82 successful inputs. Blocked and failed URLs remain in the full
      denominator.</p>

      <p>These are historical observational byte ratios. They are not universal
      token savings, cost savings, latency, or task-success claims, and the
      legacy snapshots predate the current provenance/corpus-digest schema. See
      the <a href="coverage">retained non-JavaScript snapshot</a>, the <a
      href="coverage-js">JavaScript snapshot</a>, and the <a
      href="https://github.com/plasmate-labs/plasmate/blob/master/docs/BENCHMARKING.md">benchmark policy</a>.</p>

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
        <li><strong>Representation cost varies.</strong> Image and text tokenization depends on the model and input. Measure both paths for the selected model instead of assuming a fixed multiplier.</li>
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
        <li><strong>Agent framework developers</strong> (Browser Use, LangChain, CrewAI): structured page data out of the box; measure output size and token use for the target workflow</li>
        <li><strong>Enterprise AI teams</strong>: predictable, structured web data instead of HTML soup. No more prompt-engineering around broken DOM structures.</li>
        <li><strong>Web processing at scale</strong>: structured output can omit irrelevant markup, but cost impact must be measured on the actual corpus, tokenizer, and task.</li>
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
