"""LangChain document loader that returns pages as SOM documents."""

from __future__ import annotations

from typing import Any, Iterator, Optional, Sequence

from langchain_core.document_loaders import BaseLoader
from langchain_core.documents import Document

from plasmate import Plasmate

from .som_output import som_to_text


class PlasmateSOMLoader(BaseLoader):
    """Load web pages as SOM-formatted LangChain Documents.

    Each URL produces a ``Document`` whose ``page_content`` is the concise
    SOM text representation (optimised for LLM consumption) and whose
    ``metadata`` contains the full SOM statistics and structured data.

    Args:
        urls: One or more URLs to load.
        client: An existing Plasmate client. If ``None``, one is created.
        budget: Optional per-page SOM token budget.
        javascript: Whether to execute JavaScript (default ``True``).

    Example::

        loader = PlasmateSOMLoader(["https://example.com"])
        docs = loader.load()
        print(docs[0].page_content)  # concise SOM text
        print(docs[0].metadata["url"])
    """

    def __init__(
        self,
        urls: Sequence[str],
        *,
        client: Optional[Plasmate] = None,
        budget: Optional[int] = None,
        javascript: bool = True,
    ):
        self.urls = list(urls)
        self.client = client or Plasmate()
        self.budget = budget
        self.javascript = javascript

    def lazy_load(self) -> Iterator[Document]:
        """Lazily load URLs one at a time, yielding Documents."""
        for url in self.urls:
            kwargs: dict[str, Any] = {}
            if self.budget is not None:
                kwargs["budget"] = self.budget
            if not self.javascript:
                kwargs["javascript"] = False

            som = self.client.fetch_page(url, **kwargs)
            text = som_to_text(som)
            meta = som.get("meta", {})

            metadata: dict[str, Any] = {
                "url": som.get("url", url),
                "title": som.get("title", ""),
                "lang": som.get("lang", "en"),
                "html_bytes": meta.get("html_bytes", 0),
                "som_bytes": meta.get("som_bytes", 0),
                "element_count": meta.get("element_count", 0),
                "interactive_count": meta.get("interactive_count", 0),
            }

            # Include structured data fields when present
            sd = som.get("structured_data")
            if sd:
                if sd.get("meta", {}).get("description"):
                    metadata["description"] = sd["meta"]["description"]
                if sd.get("open_graph"):
                    metadata["open_graph"] = sd["open_graph"]
                if sd.get("json_ld"):
                    metadata["json_ld"] = sd["json_ld"]

            yield Document(page_content=text, metadata=metadata)
