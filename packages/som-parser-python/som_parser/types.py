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
    GROUP = "group"
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
    disabled: Optional[bool] = None
    group: Optional[str] = None


class ListItem(BaseModel):
    text: str


class SomElementAttrs(BaseModel):
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
    level: Optional[int] = None
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
    aria_label: Optional[str] = None
    aria_description: Optional[str] = None
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
    dir: Optional[str] = None
    lang: Optional[str] = None
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
    test_id: Optional[str] = None
    data_action: Optional[str] = None
    data_state: Optional[str] = None
    aria: Optional[Dict[str, bool | str]] = None
    has_srcdoc: Optional[bool] = None
    srcdoc_preview: Optional[str] = None
    sandbox: Optional[str] = None
    allow: Optional[str] = None
    width: Optional[str] = None
    height: Optional[str] = None


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
    shadow: Optional["SomShadowRoot"] = None


class SomShadowRoot(BaseModel):
    mode: str
    elements: List[SomElement]


class SomRegion(BaseModel):
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
