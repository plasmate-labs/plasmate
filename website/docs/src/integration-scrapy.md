# Scrapy Integration

Use the shipped [Python SDK](sdk-python) inside Scrapy spiders when you need structured SOM instead of raw HTML. Output size and token use depend on the page and model tokenizer.

This repository does not ship a `scrapy_plasmate` package or downloader middleware. Call `Plasmate.fetch_page` from spider code.

## Installation

```bash
pip install plasmate scrapy
```

Requires the `plasmate` binary on your PATH.

## Quick Start

```python
import scrapy
from plasmate import Plasmate


class ExampleSpider(scrapy.Spider):
    name = "example"
    start_urls = ["https://example.com"]

    def parse(self, response):
        with Plasmate() as client:
            som = client.fetch_page(response.url, selector="main")
        yield {
            "url": response.url,
            "title": som.get("title", ""),
            "regions": som.get("regions"),
        }
```

When the spider only needs readable text or outbound URLs, use `extract_text` or `extract_links` on the same client instead of a second HTML parse.

## Fetch vs middleware

There is no shipped downloader middleware, `response.meta` SOM hook, or automatic Scrapy fallback. Persistent click/type flows use `open_page` on the same client; see the Python SDK.

SOM removes non-semantic markup, but token and cost differences vary with the pages and model tokenizer. Benchmark the full crawl before planning context or spend.

## Links

- [Scrapy Docs](https://docs.scrapy.org)
- [Plasmate Python SDK](sdk-python)
