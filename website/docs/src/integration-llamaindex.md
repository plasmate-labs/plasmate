# LlamaIndex Integration

Use Plasmate as a document reader in your LlamaIndex RAG pipelines — structured SOM output for **10-16x fewer tokens** than raw HTML readers.

Source: [`llama-index-readers-plasmate`](https://github.com/run-llama/llama_index/pull/21144)

## Installation

```bash
pip install plasmate llama-index-readers-plasmate
```

## Quick Start

```python
from llama_index.readers.plasmate import PlasmateWebReader

reader = PlasmateWebReader()
documents = reader.load_data(urls=[
    "https://example.com",
    "https://docs.python.org/3/",
])

# Use in your RAG pipeline
from llama_index.core import VectorStoreIndex
index = VectorStoreIndex.from_documents(documents)
query_engine = index.as_query_engine()
response = query_engine.query("What is this page about?")
```

## Why Plasmate for RAG?

Standard web readers (`SimpleWebPageReader`, `BeautifulSoupWebReader`) return raw HTML or basic text extraction. Plasmate returns structured semantic content:

- **10-16x fewer tokens** per page — cheaper embeddings and queries
- **Preserved document hierarchy** — headings, sections, lists stay structured
- **Clean text extraction** — no scripts, styles, or layout noise
- **Metadata included** — title, language, byte sizes, element counts

## Configuration

```python
PlasmateWebReader(
    binary="plasmate",    # Path to plasmate binary
    timeout=30,           # Timeout per page in seconds
    budget=None,          # Optional SOM token budget
    javascript=True,      # Enable JS execution
)
```

## Document Metadata

Each loaded document includes metadata:

| Field | Description |
|-------|-------------|
| `url` | Source URL |
| `title` | Page title |
| `html_bytes` | Original HTML size |
| `som_bytes` | SOM output size |
| `element_count` | Total SOM elements |
| `compression_ratio` | HTML → SOM ratio |

## Links

- [GitHub PR #21144](https://github.com/run-llama/llama_index/pull/21144)
- [LlamaIndex Docs](https://docs.llamaindex.ai)
- [Plasmate Python SDK](sdk-python)
