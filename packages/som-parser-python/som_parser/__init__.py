"""som-parser: Parse and query SOM (Semantic Object Model) output."""

from .types import (
    ElementAction,
    ElementRole,
    LinkElement,
    ListItem,
    RegionRole,
    SelectOption,
    SemanticHint,
    Som,
    SomElement,
    SomElementAttrs,
    SomMeta,
    SomRegion,
    StructuredData,
)
from .parser import from_plasmate, is_valid_som, parse_som
from .query import (
    filter_elements,
    find_by_id,
    find_by_role,
    find_by_text,
    get_all_elements,
    get_compression_ratio,
    get_forms,
    get_headings,
    get_inputs,
    get_interactive_elements,
    get_links,
    get_text,
    get_text_by_region,
    to_markdown,
)

__all__ = [
    # Types
    "ElementAction",
    "ElementRole",
    "LinkElement",
    "ListItem",
    "RegionRole",
    "SelectOption",
    "SemanticHint",
    "Som",
    "SomElement",
    "SomElementAttrs",
    "SomMeta",
    "SomRegion",
    "StructuredData",
    # Parser
    "from_plasmate",
    "is_valid_som",
    "parse_som",
    # Query
    "filter_elements",
    "find_by_id",
    "find_by_role",
    "find_by_text",
    "get_all_elements",
    "get_compression_ratio",
    "get_forms",
    "get_headings",
    "get_inputs",
    "get_interactive_elements",
    "get_links",
    "get_text",
    "get_text_by_region",
    "to_markdown",
]

__version__ = "0.3.0"
