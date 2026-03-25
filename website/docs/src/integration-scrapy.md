# Scrapy Integration

Use Plasmate as a drop-in downloader middleware for Scrapy, the most popular Python web scraping framework.

## Install

```bash
pip install plasmate scrapy-plasmate
```

## Setup

Add to your Scrapy project's `settings.py`:

```python
DOWNLOADER_MIDDLEWARES = {
    'scrapy_plasmate.PlasmateDownloaderMiddleware': 543,
}
```

## Usage

```python
import scrapy
from scrapy_plasmate.utils import extract_text, extract_links

class MySpider(scrapy.Spider):
    name = 'my_spider'
    start_urls = ['https://example.com']

    def parse(self, response):
        som = response.meta.get('plasmate_som', {})
        yield {
            'url': response.url,
            'title': som.get('title', ''),
            'text': extract_text(som),
            'links': extract_links(som),
        }
```

## How It Works

The middleware intercepts requests and routes them through Plasmate instead of the default HTTP downloader. The SOM is stored in `response.meta['plasmate_som']` for easy access in your spider.

If Plasmate fails for any URL, the middleware falls back to the standard Scrapy downloader automatically.

## Utilities

```python
from scrapy_plasmate.utils import (
    extract_text,      # All text content
    extract_links,     # All links with text
    extract_headings,  # All headings with levels
    extract_tables,    # Table data
)
```

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `PLASMATE_ENABLED` | `True` | Enable/disable the middleware |
| `PLASMATE_TIMEOUT` | `30` | Timeout in seconds per request |
| `PLASMATE_JAVASCRIPT` | `True` | Enable JavaScript execution |

## Links

- [GitHub](https://github.com/plasmate-labs/scrapy-plasmate)
- [Scrapy Docs](https://docs.scrapy.org)
