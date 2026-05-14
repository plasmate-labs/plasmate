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
    group = "group"
    separator = "separator"
    details = "details"
    iframe = "iframe"


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
    disabled: Optional[bool] = None
    group: Optional[str] = None


class ListItem(BaseModel):
    """A list item."""

    model_config = {"extra": "forbid"}

    text: str


class ElementAttrs(BaseModel):
    """Role-specific attributes for an element."""

    model_config = {"extra": "forbid"}

    href: Optional[str] = None
    target: Optional[str] = None
    rel: Optional[str] = None
    download: Optional[bool | str] = None
    input_type: Optional[str] = None
    value: Optional[str] = None
    placeholder: Optional[str] = None
    required: Optional[bool] = None
    readonly: Optional[bool] = None
    disabled: Optional[bool] = None
    inert: Optional[bool] = None
    checked: Optional[bool] = None
    group: Optional[str] = None
    multiple: Optional[bool] = None
    options: Optional[List[SelectOption]] = None
    selected_values: Optional[List[str]] = None
    size: Optional[int | str] = None
    level: Optional[int] = Field(default=None, ge=1, le=6)
    alt: Optional[str] = None
    src: Optional[str] = None
    ordered: Optional[bool] = None
    items: Optional[List[ListItem]] = None
    headers: Optional[List[str]] = None
    rows: Optional[List[List[str]]] = None
    section_label: Optional[str] = None
    legend: Optional[str] = None
    open: Optional[bool] = None
    summary: Optional[str] = None
    contenteditable: Optional[bool | str] = None
    tabindex: Optional[int | str] = None
    accesskey: Optional[str] = None
    title: Optional[str] = None
    labelledby: Optional[str] = None
    describedby: Optional[str] = None
    spellcheck: Optional[bool | str] = None
    name: Optional[str] = None
    accept: Optional[str] = None
    capture: Optional[bool | str] = None
    autocomplete: Optional[str] = None
    inputmode: Optional[str] = None
    enterkeyhint: Optional[str] = None
    autocapitalize: Optional[str] = None
    dirname: Optional[str] = None
    form: Optional[str] = None
    list: Optional[str] = None
    popovertarget: Optional[str] = None
    popovertargetaction: Optional[str] = None
    commandfor: Optional[str] = None
    command: Optional[str] = None
    popover: Optional[str] = None
    button_type: Optional[str] = None
    formaction: Optional[str] = None
    formmethod: Optional[str] = None
    formenctype: Optional[str] = None
    formtarget: Optional[str] = None
    formnovalidate: Optional[bool] = None
    minlength: Optional[int | str] = None
    maxlength: Optional[int | str] = None
    min: Optional[int | str] = None
    max: Optional[int | str] = None
    step: Optional[str] = None
    pattern: Optional[str] = None
    description: Optional[str] = None
    aria: Optional[Dict[str, bool | str]] = None
    has_srcdoc: Optional[bool] = None
    srcdoc_preview: Optional[str] = None
    sandbox: Optional[str] = None
    allow: Optional[str] = None
    width: Optional[str] = None
    height: Optional[str] = None


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
    shadow: Optional["SomShadowRoot"] = None


class SomShadowRoot(BaseModel):
    """Shadow DOM content attached to a SOM element."""

    model_config = {"extra": "forbid"}

    mode: str
    elements: List[SomElement]


class SomRegion(BaseModel):
    """A semantic region within the page."""

    model_config = {"extra": "forbid"}

    id: str
    role: RegionRole
    label: Optional[str] = None
    action: Optional[str] = None
    method: Optional[str] = None
    target: Optional[str] = None
    enctype: Optional[str] = None
    novalidate: Optional[bool] = None
    accept_charset: Optional[str] = None
    autocomplete: Optional[str] = None
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
