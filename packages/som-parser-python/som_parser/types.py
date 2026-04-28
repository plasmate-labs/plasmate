"""Pydantic v2 models for SOM (Semantic Object Model)."""

from __future__ import annotations

from enum import Enum
from typing import Any, Dict, List, Optional

from pydantic import BaseModel


class RegionRole(str, Enum):
    NAVIGATION = "navigation"
    MAIN = "main"
    ASIDE = "aside"
    HEADER = "header"
    FOOTER = "footer"
    FORM = "form"
    DIALOG = "dialog"
    CONTENT = "content"


class ElementRole(str, Enum):
    LINK = "link"
    BUTTON = "button"
    TEXT_INPUT = "text_input"
    TEXTAREA = "textarea"
    SELECT = "select"
    CHECKBOX = "checkbox"
    RADIO = "radio"
    HEADING = "heading"
    IMAGE = "image"
    LIST = "list"
    TABLE = "table"
    PARAGRAPH = "paragraph"
    SECTION = "section"
    SEPARATOR = "separator"
    DETAILS = "details"
    IFRAME = "iframe"


class ElementAction(str, Enum):
    CLICK = "click"
    TYPE = "type"
    CLEAR = "clear"
    SELECT = "select"
    TOGGLE = "toggle"


class SemanticHint(str, Enum):
    ACTIVE = "active"
    BADGE = "badge"
    CARD = "card"
    COLLAPSED = "collapsed"
    DANGER = "danger"
    DISABLED = "disabled"
    ERROR = "error"
    EXPANDED = "expanded"
    HERO = "hero"
    HIDDEN = "hidden"
    LARGE = "large"
    LOADING = "loading"
    MODAL = "modal"
    NOTIFICATION = "notification"
    PRIMARY = "primary"
    REQUIRED = "required"
    SECONDARY = "secondary"
    SELECTED = "selected"
    SMALL = "small"
    STICKY = "sticky"
    SUCCESS = "success"
    WARNING = "warning"


class SelectOption(BaseModel):
    value: str
    text: str
    selected: Optional[bool] = None


class ListItem(BaseModel):
    text: str


class SomElementAttrs(BaseModel):
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
    level: Optional[int] = None
    alt: Optional[str] = None
    src: Optional[str] = None
    ordered: Optional[bool] = None
    items: Optional[List[ListItem]] = None
    headers: Optional[List[str]] = None
    rows: Optional[List[List[str]]] = None
    section_label: Optional[str] = None
    open: Optional[bool] = None
    summary: Optional[str] = None
    has_srcdoc: Optional[bool] = None
    srcdoc_preview: Optional[str] = None
    name: Optional[str] = None
    sandbox: Optional[str] = None
    allow: Optional[str] = None
    width: Optional[str] = None
    height: Optional[str] = None


class ShadowRoot(BaseModel):
    mode: str
    elements: List[SomElement]


class SomElement(BaseModel):
    id: str
    role: ElementRole
    html_id: Optional[str] = None
    text: Optional[str] = None
    label: Optional[str] = None
    actions: Optional[List[ElementAction]] = None
    attrs: Optional[SomElementAttrs] = None
    children: Optional[List[SomElement]] = None
    hints: Optional[List[SemanticHint]] = None
    shadow: Optional[ShadowRoot] = None


class SomRegion(BaseModel):
    id: str
    role: RegionRole
    label: Optional[str] = None
    action: Optional[str] = None
    method: Optional[str] = None
    elements: List[SomElement]


class SomMeta(BaseModel):
    html_bytes: int
    som_bytes: int
    element_count: int
    interactive_count: int


class LinkElement(BaseModel):
    rel: str
    href: str
    type: Optional[str] = None
    hreflang: Optional[str] = None


class StructuredData(BaseModel):
    json_ld: Optional[List[Dict[str, Any]]] = None
    open_graph: Optional[Dict[str, str]] = None
    twitter_card: Optional[Dict[str, str]] = None
    meta: Optional[Dict[str, str]] = None
    links: Optional[List[LinkElement]] = None


class Som(BaseModel):
    som_version: str
    url: str
    title: str
    lang: str
    regions: List[SomRegion]
    meta: SomMeta
    structured_data: Optional[StructuredData] = None
