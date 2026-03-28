# HTML vs Markdown vs SOM: Which Format Should Your AI Agent Use?

Every AI agent that browses the web faces the same question: how do you represent a web page to a language model?

The default answer -- raw HTML -- is expensive and slow. A typical page dumps 30,000+ tokens into your context window, most of it CSS classes and layout divs. But what are the actual alternatives? And do they work?

We ran 150 benchmark tasks across GPT-4o and Claude Sonnet 4 to find out. The results surprised us.

---

## The Three Representations

When an agent needs to understand a web page, there are three common approaches:

### 1. Raw HTML

The DOM as-is. Every `<div>`, every `class="sc-1234 flex items-center gap-2"`, every inline script. This is what most agents send today.

```html
<div class="sc-1234 flex items-center gap-2 px-4 py-2">
  <a href="/about" class="text-blue-500 hover:underline
     font-medium tracking-tight text-sm">About</a>
  <span class="text-gray-400">|</span>
  <a href="/pricing" class="text-blue-500 hover:underline
     font-medium tracking-tight text-sm">Pricing</a>
</div>
```

**Pros:** Complete fidelity to the DOM. No information lost.

**Cons:** 80-95% of tokens are noise (styling, scripts, tracking). Expensive. Slow.

### 2. Markdown

Strip the HTML to readable text, preserving structure through Markdown conventions. This is what tools like Jina Reader and many scraping libraries produce.

```markdown
[About](/about) | [Pricing](/pricing)
```

**Pros:** Dramatically fewer tokens. Human-readable.

**Cons:** Loses interactive elements. No way to know what's clickable. Navigation tasks become guesswork.

### 3. SOM (Semantic Object Model)

A structured JSON representation that preserves meaning and interactivity while stripping presentation noise. Each element includes its semantic role and available actions.

```json
{
  "role": "navigation",
  "children": [
    { "role": "link", "text": "About", "ref": "e1", "attrs": {"href": "/about"}, "actions": ["click"] },
    { "role": "link", "text": "Pricing", "ref": "e2", "attrs": {"href": "/pricing"}, "actions": ["click"] }
  ]
}
```

**Pros:** Minimal tokens. Preserves interactivity. Clear semantic roles.

**Cons:** Requires a SOM-aware fetcher (like Plasmate).

---

## Token Cost Comparison

We measured input tokens across 50 web pages (news sites, documentation, e-commerce, government sites, social platforms). The differences are stark:

| Format | Avg Input Tokens | vs HTML |
|--------|------------------|---------|
| HTML | 33,181 | 1.0x |
| SOM | 8,301 | **4.0x fewer** |
| Markdown | 4,542 | **7.3x fewer** |

Markdown wins on raw token count -- it strips everything. But tokens aren't the whole story.

### Cost Per 1,000 Pages (at $3/M input tokens)

| Format | Cost | Savings vs HTML |
|--------|------|-----------------|
| HTML | $99.54 | -- |
| SOM | $24.90 | 75% |
| Markdown | $13.63 | 86% |

If you're just extracting text, Markdown is cheaper. But if your agent needs to *interact* with pages -- click buttons, fill forms, navigate -- Markdown falls apart.

---

## The Latency Surprise

Here's where it gets interesting. We expected Markdown to be fastest (fewest tokens = fastest inference). That's true for GPT-4o:

### GPT-4o Latency (seconds)

| Format | Avg Latency |
|--------|-------------|
| HTML | 2.7s |
| Markdown | 1.9s |
| SOM | **1.4s** |

SOM beats both. Why? Two reasons:

1. **Structured input parses faster.** JSON with clear roles lets the model skip the "what is this?" step.
2. **Less ambiguity = shorter reasoning chains.** When a link is explicitly marked `"role": "link", "actions": ["click"]`, the model doesn't need to infer interactivity from context.

### Claude Sonnet 4 Latency (seconds)

| Format | Avg Latency |
|--------|-------------|
| HTML | 16.2s |
| **Markdown** | **25.2s** |
| SOM | **8.5s** |

Wait -- Markdown is *slower* than HTML on Claude? Yes. And SOM is nearly 3x faster than Markdown.

Claude appears to struggle with ambiguous Markdown when the task requires interaction. The model spends more time reasoning about what elements are clickable, what actions are available, and how to express those actions. With SOM, that information is explicit.

---

## Category Breakdown

Not all tasks are equal. We tested extraction (pull data from pages), comparison (analyze multiple items), navigation (click through to find information), and adversarial tasks (complex anti-bot pages).

### HTML/SOM Token Ratio by Category

| Category | HTML/SOM Ratio | Notes |
|----------|----------------|-------|
| Extraction | 3.8x | SOM wins, but margin is smaller |
| Comparison | 4.2x | Multi-item pages benefit from structure |
| Navigation | **5.4x** | Interactivity data is dense in SOM |
| Adversarial | **6.0x** | Anti-bot clutter inflates HTML massively |

For adversarial pages (Cloudflare challenges, heavy JavaScript), HTML explodes with noise while SOM stays lean. The 6x ratio means you're paying 6x more for HTML on the hardest pages.

### Where Markdown Fails

Markdown works great for "read this article and summarize it." It breaks down for:

- **Form filling**: Markdown can't represent input fields, dropdowns, or submit buttons
- **Navigation**: No way to know which text is a clickable link vs decorative
- **Stateful interactions**: Multi-step flows (add to cart, checkout) require element references
- **Dynamic content**: JavaScript-rendered content often doesn't survive Markdown conversion

In our benchmark, Markdown tasks requiring interaction had a **23% higher failure rate** than SOM.

---

## When to Use What

### Use Markdown when:

- Pure text extraction (summarize this article)
- No interaction needed
- Budget is the only constraint
- You control the source (your own docs, known-good pages)

### Use SOM when:

- Agents need to click, type, or navigate
- Multi-step workflows
- Unknown/adversarial pages
- Latency matters (SOM is fastest on both models)
- You want consistent structure across diverse sites

### Use HTML when:

- You need pixel-perfect DOM fidelity
- Building a browser automation tool that maps directly to selectors
- Debugging what the page actually contains
- Never (for LLM input -- it's just wasteful)

The honest recommendation: **default to SOM** unless you have a specific reason not to. It's faster, cheaper than HTML, and handles interactive tasks that Markdown can't.

---

## Getting Started with Plasmate

Plasmate is the reference implementation of SOM. Three ways to use it:

### 1. CLI

```bash
# Install
npm install -g plasmate

# Fetch a page as SOM
plasmate fetch https://example.com

# Output formats: som (default), markdown, html
plasmate fetch https://example.com --format markdown
```

### 2. MCP Server (Claude Desktop / Cursor)

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "npx",
      "args": ["-y", "plasmate", "mcp"]
    }
  }
}
```

Now Claude can browse the web using SOM directly.

### 3. Python SDK

```python
from plasmate import Plasmate

client = Plasmate()

# Get a page as SOM
som = client.fetch("https://news.ycombinator.com")

# Access structured data
for item in som.get_by_role("listitem"):
    title = item.get_text()
    link = item.get_attr("href")
    print(f"{title}: {link}")

# Or get markdown for simple extraction
markdown = client.fetch("https://example.com", format="markdown")
```

For authenticated browsing (sites that require login), see the [Authenticated Browsing Guide](/guide-authenticated-browsing).

---

## The Data

All numbers in this post come from WebTaskBench, an open benchmark of 150 web tasks across 50 real-world URLs. You can run it yourself:

```bash
git clone https://github.com/nicholasday/webtaskbench
cd webtaskbench
npm install
npm run eval -- --model gpt-4o --format som
```

Full results: [WebTaskBench Results](https://github.com/nicholasday/webtaskbench/tree/main/results)

---

## Further Reading

- [SOM Spec v1.0](/som-spec) -- The complete specification
- [SOM-first Websites](/som-first-sites) -- Sites that publish native SOM endpoints
- [Authenticated Browsing](/guide-authenticated-browsing) -- Handling logins and cookies

---

**Links:**
- [GitHub](https://github.com/nicholasday/plasmate)
- [npm](https://www.npmjs.com/package/plasmate)
- [Docs](https://docs.plasmate.app)
