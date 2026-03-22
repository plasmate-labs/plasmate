"""Pydantic v2 models for the SOM (Semantic Object Model) JSON Schema."""

from __future__ import annotations

from enum import Enum
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field


class RegionRole(str, Enum):
    """The semantic role of a region."""

    navigation = "navigation"
    main = "main"
    aside = "aside"
    header = "header"
    footer = "footer"
    form = "form"
    dialog = "dialog"
    content = "content"


class ElementRole(str, Enum):
    """The semantic role of an element."""

    link = "link"
    button = "button"
    text_input = "text_input"
    textarea = "textarea"
    select = "select"
    checkbox = "checkbox"
    radio = "radio"
    heading = "heading"
    image = "image"
    list = "list"
    table = "table"
    paragraph = "paragraph"
    section = "section"
    separator = "separator"


class SemanticHint(str, Enum):
    """A semantic hint inferred from CSS class names."""

    active = "active"
    badge = "badge"
    card = "card"
    collapsed = "collapsed"
    danger = "danger"
    disabled = "disabled"
    error = "error"
    expanded = "expanded"
    hero = "hero"
    hidden = "hidden"
    large = "large"
    loading = "loading"
    modal = "modal"
    notification = "notification"
    primary = "primary"
    required = "required"
    secondary = "secondary"
    selected = "selected"
    small = "small"
    sticky = "sticky"
    success = "success"
    warning = "warning"


class SelectOption(BaseModel):
    """A select option with value and display text."""

    model_config = {"extra": "forbid"}

    value: str
    text: str
    selected: Optional[bool] = None


class ListItem(BaseModel):
    """A list item."""

    model_config = {"extra": "forbid"}

    text: str


class ElementAttrs(BaseModel):
    """Role-specific attributes for an element."""

    model_config = {"extra": "forbid"}

    href: Optional[str] = None
    input_type: Optional[str] = None
    value: Optional[str] = None
    placeholder: Optional[str] = None
    required: Optional[bool] = None
    disabled: Optional[bool] = None
    checked: Optional[bool] = None
    group: Optional[str] = None
    multiple: Optional[bool] = None
    options: Optional[List[SelectOption]] = None
    level: Optional[int] = Field(default=None, ge=1, le=6)
    alt: Optional[str] = None
    src: Optional[str] = None
    ordered: Optional[bool] = None
    items: Optional[List[ListItem]] = None
    headers: Optional[List[str]] = None
    rows: Optional[List[List[str]]] = None
    section_label: Optional[str] = None


class SomElement(BaseModel):
    """A semantic element within a region."""

    model_config = {"extra": "forbid"}

    id: str
    role: ElementRole
    text: Optional[str] = None
    label: Optional[str] = None
    actions: Optional[List[str]] = None
    attrs: Optional[ElementAttrs] = None
    children: Optional[List[SomElement]] = None
    hints: Optional[List[SemanticHint]] = None


class SomRegion(BaseModel):
    """A semantic region within the page."""

    model_config = {"extra": "forbid"}

    id: str
    role: RegionRole
    label: Optional[str] = None
    action: Optional[str] = None
    method: Optional[str] = None
    elements: List[SomElement]


class SomMeta(BaseModel):
    """Metadata about the SOM compilation."""

    model_config = {"extra": "forbid"}

    html_bytes: int = Field(ge=0)
    som_bytes: int = Field(ge=0)
    element_count: int = Field(ge=0)
    interactive_count: int = Field(ge=0)


class LinkElement(BaseModel):
    """A <link> element with rel and href."""

    model_config = {"extra": "forbid"}

    rel: str
    href: str
    type: Optional[str] = None
    hreflang: Optional[str] = None


class StructuredData(BaseModel):
    """Structured data extracted from the page head."""

    model_config = {"extra": "forbid"}

    json_ld: Optional[List[Dict[str, Any]]] = None
    open_graph: Optional[Dict[str, str]] = None
    twitter_card: Optional[Dict[str, str]] = None
    meta: Optional[Dict[str, str]] = None
    links: Optional[List[LinkElement]] = None


class Som(BaseModel):
    """A SOM (Semantic Object Model) document."""

    model_config = {"extra": "forbid"}

    som_version: str
    url: str
    title: str
    lang: str
    regions: List[SomRegion]
    meta: SomMeta
    structured_data: Optional[StructuredData] = None
