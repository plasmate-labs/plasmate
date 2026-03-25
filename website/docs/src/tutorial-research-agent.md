# Build a Web Research Agent with Plasmate

This tutorial walks through building a simple web research agent that uses Plasmate to read web pages. No LLM API key required - we focus on the content extraction pipeline.

## What We're Building

A Python script that:
1. Takes a research topic
2. Fetches relevant web pages using Plasmate
3. Extracts structured content from each page
4. Combines findings into a research summary
5. Reports token usage and estimated costs

## Prerequisites

```bash
pip install plasmate
```

Verify it works:

```bash
plasmate fetch https://example.com
```

You should see a JSON SOM (Semantic Object Model) output.

## Step 1: Fetch a Page

The simplest Plasmate interaction:

```python
import subprocess
import json

def fetch_page(url: str) -> dict:
    """Fetch a URL with Plasmate and return the SOM."""
    result = subprocess.run(
        ["plasmate", "fetch", url],
        capture_output=True, text=True, timeout=30
    )
    if result.returncode != 0:
        return None
    return json.loads(result.stdout)

# Try it
som = fetch_page("https://en.wikipedia.org/wiki/Artificial_intelligence")
print(f"Title: {som['title']}")
print(f"Regions: {len(som['regions'])}")
```

The SOM output is structured into **regions** (navigation, content, forms) containing **elements** (headings, paragraphs, links, tables).

## Step 2: Extract Content

Pull the useful information out of the SOM:

```python
def extract_content(som: dict) -> dict:
    """Extract structured content from a SOM."""
    title = som.get("title", "")
    headings = []
    paragraphs = []
    links = []

    for region in som.get("regions", []):
        for el in region.get("elements", []):
            role = el.get("role", "")
            text = el.get("text", "")

            if role == "heading":
                level = el.get("attrs", {}).get("level", 2)
                headings.append({"level": level, "text": text})
            elif role == "paragraph" and text:
                paragraphs.append(text)
            elif role == "link":
                href = el.get("attrs", {}).get("href", "")
                if href and text:
                    links.append({"text": text, "url": href})

    return {
        "title": title,
        "headings": headings,
        "paragraphs": paragraphs,
        "links": links,
        "word_count": sum(len(p.split()) for p in paragraphs),
    }

content = extract_content(som)
print(f"Title: {content['title']}")
print(f"Headings: {len(content['headings'])}")
print(f"Paragraphs: {len(content['paragraphs'])}")
print(f"Links: {len(content['links'])}")
print(f"Words: {content['word_count']}")
```

## Step 3: Batch Research

Fetch multiple pages about a topic:

```python
from concurrent.futures import ThreadPoolExecutor, as_completed

def research_topic(urls: list, max_concurrent: int = 5) -> list:
    """Fetch and extract content from multiple URLs."""
    results = []

    with ThreadPoolExecutor(max_workers=max_concurrent) as executor:
        futures = {executor.submit(fetch_page, url): url for url in urls}

        for future in as_completed(futures):
            url = futures[future]
            try:
                som = future.result()
                if som:
                    content = extract_content(som)
                    content["url"] = url
                    results.append(content)
                    print(f"  Fetched: {content['title'][:50]}")
            except Exception as e:
                print(f"  Failed: {url} ({e})")

    return results

# Research "AI agents"
urls = [
    "https://en.wikipedia.org/wiki/Intelligent_agent",
    "https://docs.python.org/3/library/asyncio.html",
    "https://www.anthropic.com/research",
    "https://openai.com/index/practices-for-governing-agentic-ai-systems",
    "https://www.reuters.com/technology/artificial-intelligence/",
]

print("Researching AI agents...")
findings = research_topic(urls)
print(f"\nFetched {len(findings)} pages")
```

## Step 4: Summarize Findings

Combine the extracted content into a structured research document:

```python
def summarize_research(findings: list, topic: str) -> str:
    """Create a research summary from extracted content."""
    lines = [f"# Research Summary: {topic}", ""]

    for finding in findings:
        lines.append(f"## {finding['title']}")
        lines.append(f"Source: {finding['url']}")
        lines.append(f"Words: {finding['word_count']}")
        lines.append("")

        # Include top headings as an outline
        for h in finding["headings"][:5]:
            prefix = "#" * (h["level"] + 1)
            lines.append(f"{prefix} {h['text']}")

        # Include first 3 paragraphs
        for p in finding["paragraphs"][:3]:
            lines.append(f"\n{p[:200]}...")

        lines.append("\n---\n")

    # Token comparison
    total_words = sum(f["word_count"] for f in findings)
    est_tokens = total_words * 4 // 3  # rough estimate
    lines.append(f"## Stats")
    lines.append(f"- Pages fetched: {len(findings)}")
    lines.append(f"- Total words: {total_words}")
    lines.append(f"- Estimated tokens (SOM): ~{est_tokens}")
    lines.append(f"- Estimated tokens (Chrome HTML): ~{est_tokens * 16} (16x more)")
    lines.append(f"- Estimated GPT-4 cost (SOM): ${est_tokens * 30 / 1000000:.4f}")
    lines.append(f"- Estimated GPT-4 cost (Chrome): ${est_tokens * 16 * 30 / 1000000:.4f}")

    return "\n".join(lines)

summary = summarize_research(findings, "AI Agents")
print(summary)
```

## Step 5: Save and Compare

```python
# Save the summary
with open("research-summary.md", "w") as f:
    f.write(summary)

print(f"\nSaved to research-summary.md")
print(f"\nToken savings vs Chrome: {16}x fewer tokens")
print(f"That's the difference between $0.01 and $0.16 per research run at GPT-4 rates.")
print(f"At 1000 research runs/day: $10 vs $160.")
```

## What's Next

This pipeline produces clean, structured content ready to feed into any LLM. To build a full research agent:

1. **Add an LLM** - Send the extracted content to GPT-4, Claude, or a local model for synthesis
2. **Add search** - Use a search API to discover URLs dynamically instead of hardcoding them
3. **Add follow-up** - Extract links from the SOM and follow them for deeper research
4. **Add caching** - Use the [SOM Cache](https://cache.plasmate.app) to avoid re-fetching pages

## Full Script

The complete script is available in our [quickstart-python](https://github.com/plasmate-labs/quickstart-python) template.

## Links

- [Plasmate](https://plasmate.app) - Install with `pip install plasmate`
- [SOM Specification](https://docs.plasmate.app/som-spec) - Full format reference
- [Jupyter Notebooks](https://github.com/plasmate-labs/notebooks) - Interactive versions of these examples
- [W3C Community Group](https://www.w3.org/community/web-content-browser-ai/) - Help shape the standard
