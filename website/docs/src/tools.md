# Auditing & CI/CD Tools

Tools built on Plasmate for site auditing, accessibility testing, and CI/CD integration.

## Site Auditor

Crawl and audit websites for SEO and content quality issues. 10x faster than Chrome-based tools like Screaming Frog.

```bash
pip install plasmate plasmate-audit

# Audit a site (crawls up to 50 pages)
plasmate-audit https://example.com

# More pages, JSON output
plasmate-audit https://example.com --max-pages 200 --json
```

**Python API:**

```python
from plasmate_audit import audit_site

result = audit_site("https://example.com", max_pages=50)
print(f"Score: {result['score']}/100")
print(f"Errors: {result['errors']}, Warnings: {result['warnings']}")
```

**Checks:** missing titles, heading structure, empty links, thin content, images without alt text.

[GitHub](https://github.com/plasmate-labs/plasmate-audit)

---

## Accessibility Auditor

Analyze what a screen reader would "see" using Plasmate's Semantic Object Model.

```bash
pip install plasmate plasmate-a11y

python -m plasmate_a11y https://example.com
```

**Python API:**

```python
from plasmate_a11y import audit_url

result = audit_url("https://example.com")
for issue in result['issues']:
    print(f"[{issue['severity']}] {issue['message']}")
```

**Checks:** heading hierarchy, image alt text, link text quality, form labels, page language, landmark regions.

[GitHub](https://github.com/plasmate-labs/plasmate-a11y)

---

## GitHub Action

Fetch web pages with Plasmate in your CI/CD workflows.

```yaml
- uses: plasmate-labs/som-action@v1
  with:
    url: https://example.com
  id: som

- run: echo "Page title: ${{ steps.som.outputs.title }}"
```

**Outputs:** `som` (full SOM JSON), `title` (page title), `tokens` (token count).

Use cases:
- Verify deployed content after deploy
- Monitor page structure changes
- Content regression testing
- Automated accessibility checks in CI

[GitHub](https://github.com/plasmate-labs/som-action)

---

## Jupyter Notebooks

Interactive notebooks for learning Plasmate. Run locally or on Google Colab.

```bash
pip install plasmate jupyter tiktoken pandas matplotlib
git clone https://github.com/plasmate-labs/notebooks
cd notebooks
jupyter notebook
```

**Notebooks:**
1. Getting Started - explore SOM structure
2. Token Comparison - measure HTML vs SOM across 10 sites
3. Batch Extraction - process 20+ URLs in parallel
4. Research Agent - build a web research pipeline

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/plasmate-labs/notebooks/blob/main/01-getting-started.ipynb)

[GitHub](https://github.com/plasmate-labs/notebooks)

---

## Replit Template

Try Plasmate in your browser with zero setup.

[![Run on Replit](https://replit.com/badge/github/plasmate-labs/replit-template)](https://replit.com/github/plasmate-labs/replit-template)

[GitHub](https://github.com/plasmate-labs/replit-template)
