# LlamaIndex Integration

Use the shipped [Python SDK](sdk-python) to load web pages into LlamaIndex RAG pipelines as clean SOM text instead of raw HTML. Output size and token use vary by page and tokenizer.

This repository does not ship `llama-index-readers-plasmate` or `PlasmateWebReader`. Call `Plasmate.extract_text` and wrap the result in `llama_index.core.Document`.

## Installation

```bash
pip install plasmate llama-index
```

Requires the `plasmate` binary on your PATH.

## Quick Start

```python
from llama_index.core import Document, VectorStoreIndex
from plasmate import Plasmate

with Plasmate() as client:
    text = client.extract_text("https://example.com", selector="main")

documents = [
    Document(text=text, metadata={"url": "https://example.com"}),
]
index = VectorStoreIndex.from_documents(documents)
query_engine = index.as_query_engine()
response = query_engine.query("What is this page about?")
```

When the index also needs title or SOM sizes, call `fetch_page` on the same client and copy those fields into `Document.metadata`. Persistent click/type flows use `open_page`; see the Python SDK.

## Why Plasmate for RAG?

Standard web readers (`SimpleWebPageReader`, `BeautifulSoupWebReader`) return raw HTML or basic text extraction. The shipped SDK returns compiled page text:

- **Compact by design** - non-semantic markup is removed; benchmark embedding and query costs on your corpus
- **Region filters** - `selector="main"` strips nav/footer before indexing
- **Clean text extraction** - no scripts, styles, or layout noise
- **Optional structure** - `fetch_page` still exposes title, language, and byte sizes when you need them

There is no shipped LlamaIndex reader class, `load_data()` helper, or automatic HTML fallback.

## Links

- [LlamaIndex Docs](https://docs.llamaindex.ai)
- [Plasmate Python SDK](sdk-python)
