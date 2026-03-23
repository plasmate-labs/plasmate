"""
Plasmate SOM integration for Browser Use.

Provides SOM-based content extraction that can replace or complement
Browser Use's default DOM serialization, reducing token costs by 90%+.

Example::

    from plasmate_browser_use import PlasmateExtractor

    extractor = PlasmateExtractor()
    context = extractor.get_page_context("https://example.com")
    print(context)
"""

from .extractor import PlasmateExtractor
from .utils import token_count_comparison, estimate_tokens

__all__ = [
    "PlasmateExtractor",
    "token_count_comparison",
    "estimate_tokens",
]
__version__ = "0.3.0"
