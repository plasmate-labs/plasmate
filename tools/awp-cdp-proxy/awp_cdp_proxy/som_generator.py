"""Generate SOM from HTML - a lightweight reference implementation.

This is NOT as sophisticated as Plasmate's Rust SOM compiler.
It demonstrates that SOM can be generated from any HTML source,
proving the format is not locked to a single implementation.

Uses Python's html.parser and the som-parser Pydantic types to
produce a valid SOM object from raw HTML.
"""

from __future__ import annotations

import hashlib
import json
import re
from html.parser import HTMLParser
from typing import Optional
from urllib.parse import urljoin

from som_parser.types import (
    ElementAction,
    ElementRole,
    ListItem,
    RegionRole,
    SelectOption,
    Som,
    SomElement,
    SomElementAttrs,
    SomMeta,
    SomRegion,
)

# Map HTML tags to SOM element roles
TAG_TO_ROLE: dict[str, ElementRole] = {
    "a": ElementRole.LINK,
    "button": ElementRole.BUTTON,
    "input": ElementRole.TEXT_INPUT,
    "textarea": ElementRole.TEXTAREA,
    "select": ElementRole.SELECT,
    "h1": ElementRole.HEADING,
    "h2": ElementRole.HEADING,
    "h3": ElementRole.HEADING,
    "h4": ElementRole.HEADING,
    "h5": ElementRole.HEADING,
    "h6": ElementRole.HEADING,
    "img": ElementRole.IMAGE,
    "p": ElementRole.PARAGRAPH,
    "ul": ElementRole.LIST,
    "ol": ElementRole.LIST,
    "table": ElementRole.TABLE,
    "hr": ElementRole.SEPARATOR,
    "section": ElementRole.SECTION,
    "article": ElementRole.SECTION,
}

# Map HTML semantic elements to SOM region roles
TAG_TO_REGION: dict[str, RegionRole] = {
    "nav": RegionRole.NAVIGATION,
    "main": RegionRole.MAIN,
    "aside": RegionRole.ASIDE,
    "header": RegionRole.HEADER,
    "footer": RegionRole.FOOTER,
    "form": RegionRole.FORM,
    "dialog": RegionRole.DIALOG,
}

# Tags to skip entirely
SKIP_TAGS = {"script", "style", "noscript", "template", "svg", "head"}

# Heading level map
HEADING_LEVELS = {"h1": 1, "h2": 2, "h3": 3, "h4": 4, "h5": 5, "h6": 6}

# Input types that map to text_input
TEXT_INPUT_TYPES = {
    "text", "email", "password", "search", "tel", "url", "number", "date",
    "datetime-local", "month", "week", "time",
}

# Max text length for element text
MAX_TEXT_LENGTH = 200


def _generate_element_id(origin: str, role: str, name: str, dom_path: str) -> str:
    """Generate a deterministic element ID per the AWP spec."""
    input_str = f"{origin}|{role}|{name.lower().strip()[:100]}|{dom_path}"
    hash_hex = hashlib.sha256(input_str.encode()).hexdigest()[:12]
    return f"e_{hash_hex}"


def _generate_region_id(role: str, index: int) -> str:
    """Generate a region ID."""
    return f"r_{role}_{index}"


def _clean_text(text: str) -> str:
    """Collapse whitespace and trim text."""
    cleaned = re.sub(r"\s+", " ", text).strip()
    if len(cleaned) > MAX_TEXT_LENGTH:
        return cleaned[: MAX_TEXT_LENGTH - 3] + "..."
    return cleaned


def _is_hidden(attrs: dict[str, str]) -> bool:
    """Check if an element is hidden via common HTML patterns."""
    if "hidden" in attrs:
        return True
    if attrs.get("aria-hidden") == "true":
        return True
    style = attrs.get("style", "")
    if "display:none" in style.replace(" ", "") or "display: none" in style:
        return True
    if "visibility:hidden" in style.replace(" ", "") or "visibility: hidden" in style:
        return True
    return False


class _StackFrame:
    """Tracks parser state for one open HTML tag."""

    def __init__(self, tag: str, attrs: dict[str, str], dom_path: str):
        self.tag = tag
        self.attrs = attrs
        self.dom_path = dom_path
        self.text_parts: list[str] = []
        self.children: list[SomElement] = []
        self.child_index = 0
        # For lists
        self.list_items: list[ListItem] = []
        # For tables
        self.table_headers: list[str] = []
        self.table_rows: list[list[str]] = []
        self.current_row: list[str] = []
        # For select
        self.options: list[SelectOption] = []


class SOMHTMLParser(HTMLParser):
    """Parse HTML into SOM regions and elements."""

    def __init__(self, url: str = "", origin: str = ""):
        super().__init__()
        self.url = url
        self.origin = origin

        # Output
        self.regions: list[SomRegion] = []
        self.title = ""
        self.lang = "en"

        # State
        self.stack: list[_StackFrame] = []
        self.region_stack: list[tuple[str, dict[str, str], list[SomElement]]] = []
        self.skip_depth = 0
        self.in_head = False
        self.in_title = False
        self.title_parts: list[str] = []
        self.body_elements: list[SomElement] = []
        self.element_ids: set[str] = set()
        self.region_counts: dict[str, int] = {}

    def _attrs_dict(self, attrs: list[tuple[str, Optional[str]]]) -> dict[str, str]:
        return {k: (v or "") for k, v in attrs}

    def _current_dom_path(self) -> str:
        parts = []
        for frame in self.stack:
            parts.append(str(frame.child_index))
        return "/".join(parts)

    def _unique_element_id(self, role: str, name: str, dom_path: str) -> str:
        eid = _generate_element_id(self.origin, role, name, dom_path)
        if eid in self.element_ids:
            counter = 2
            while f"{eid}_{counter}" in self.element_ids:
                counter += 1
            eid = f"{eid}_{counter}"
        self.element_ids.add(eid)
        return eid

    def handle_starttag(self, tag: str, attrs: list[tuple[str, Optional[str]]]):
        tag = tag.lower()
        attr_dict = self._attrs_dict(attrs)

        if tag == "head":
            self.in_head = True
            return
        if tag == "title" and self.in_head:
            self.in_title = True
            return
        if tag == "html":
            self.lang = attr_dict.get("lang", "en")
            return

        if self.in_head:
            return

        # Skip hidden elements
        if _is_hidden(attr_dict):
            self.skip_depth += 1
            return

        if self.skip_depth > 0:
            self.skip_depth += 1
            return

        if tag in SKIP_TAGS:
            self.skip_depth += 1
            return

        # Track parent child index
        if self.stack:
            self.stack[-1].child_index += 1

        dom_path = self._current_dom_path()

        # Region tracking
        region_role = TAG_TO_REGION.get(tag)
        aria_role = attr_dict.get("role", "")
        if not region_role and aria_role in (
            "navigation", "main", "complementary", "banner",
            "contentinfo", "dialog", "alertdialog",
        ):
            role_map = {
                "navigation": RegionRole.NAVIGATION,
                "main": RegionRole.MAIN,
                "complementary": RegionRole.ASIDE,
                "banner": RegionRole.HEADER,
                "contentinfo": RegionRole.FOOTER,
                "dialog": RegionRole.DIALOG,
                "alertdialog": RegionRole.DIALOG,
            }
            region_role = role_map.get(aria_role)

        if region_role:
            self.region_stack.append((region_role.value, attr_dict, []))

        # Push stack frame
        frame = _StackFrame(tag, attr_dict, dom_path)
        self.stack.append(frame)

    def handle_endtag(self, tag: str):
        tag = tag.lower()

        if tag == "head":
            self.in_head = False
            return
        if tag == "title":
            self.in_title = False
            self.title = _clean_text("".join(self.title_parts))
            return

        if self.in_head:
            return

        if self.skip_depth > 0:
            self.skip_depth -= 1
            return

        if not self.stack:
            return

        frame = self.stack[-1]
        if frame.tag != tag:
            # Mismatched tag -- try to find the matching one
            for i in range(len(self.stack) - 1, -1, -1):
                if self.stack[i].tag == tag:
                    # Pop everything up to and including the match
                    while len(self.stack) > i:
                        self._close_frame()
                    return
            return

        self._close_frame()

    def _close_frame(self):
        if not self.stack:
            return

        frame = self.stack.pop()
        tag = frame.tag
        attr_dict = frame.attrs
        text = _clean_text("".join(frame.text_parts))
        dom_path = frame.dom_path

        element: Optional[SomElement] = None

        # Build element based on tag type
        role = TAG_TO_ROLE.get(tag)
        aria_role = attr_dict.get("role", "")

        # Override role based on ARIA
        if aria_role == "button":
            role = ElementRole.BUTTON
        elif aria_role == "checkbox":
            role = ElementRole.CHECKBOX
        elif aria_role == "radio":
            role = ElementRole.RADIO
        elif aria_role in ("img", "image"):
            role = ElementRole.IMAGE

        if tag == "input":
            input_type = attr_dict.get("type", "text").lower()
            if input_type in TEXT_INPUT_TYPES:
                role = ElementRole.TEXT_INPUT
            elif input_type == "checkbox":
                role = ElementRole.CHECKBOX
            elif input_type == "radio":
                role = ElementRole.RADIO
            elif input_type in ("submit", "button", "reset"):
                role = ElementRole.BUTTON

        if role:
            label = attr_dict.get("aria-label") or attr_dict.get("title") or None
            actions = self._get_actions(role, tag, attr_dict)
            som_attrs = self._build_attrs(role, tag, attr_dict, frame)

            # For buttons from input, use value as text
            if tag == "input" and role == ElementRole.BUTTON:
                text = attr_dict.get("value", text) or text

            # For images, use alt as text
            if role == ElementRole.IMAGE:
                alt = attr_dict.get("alt", "")
                if not alt and attr_dict.get("role") == "presentation":
                    # Skip decorative images
                    role = None
                else:
                    text = alt or text

            if role:
                eid = self._unique_element_id(role.value, text or label or "", dom_path)
                element = SomElement(
                    id=eid,
                    role=role,
                    text=text or None,
                    label=label,
                    actions=actions or None,
                    attrs=som_attrs,
                    children=frame.children or None,
                )

        # Add non-role tags that have significant text as paragraphs
        if not element and tag in ("div", "span", "td", "th", "li", "dt", "dd"):
            # Only if they have direct text and no children already captured it
            if text and not frame.children:
                # Don't create paragraph elements for every div -- only if it has meaningful text
                if len(text) > 1:
                    # These are handled by parent context
                    pass

        # Handle list items
        if tag == "li" and self.stack:
            parent = self.stack[-1]
            if parent.tag in ("ul", "ol"):
                parent.list_items.append(ListItem(text=text or ""))

        # Handle table cells
        if tag in ("td", "th") and self.stack:
            parent = self.stack[-1]
            if parent.tag == "tr":
                parent.current_row.append(text or "")

        # Handle table rows
        if tag == "tr" and self.stack:
            parent = self.stack[-1]
            if parent.tag in ("table", "thead", "tbody", "tfoot"):
                # Find the table frame
                table_frame = None
                for f in reversed(self.stack):
                    if f.tag == "table":
                        table_frame = f
                        break
                if table_frame:
                    if any(f.tag == "thead" for f in self.stack):
                        table_frame.table_headers = frame.current_row
                    else:
                        if len(table_frame.table_rows) < 20:
                            table_frame.table_rows.append(frame.current_row)

        # Handle select options
        if tag == "option" and self.stack:
            for f in reversed(self.stack):
                if f.tag == "select":
                    f.options.append(SelectOption(
                        value=attr_dict.get("value", text or ""),
                        text=text or "",
                        selected="selected" in attr_dict or None,
                    ))
                    break

        # Add element to parent or region
        if element:
            self._add_element(element)
        elif frame.children:
            # Pass children up
            for child in frame.children:
                self._add_element(child)

        # Close region if this tag opened one
        if tag in TAG_TO_REGION or (
            attr_dict.get("role", "") in (
                "navigation", "main", "complementary", "banner",
                "contentinfo", "dialog", "alertdialog",
            )
        ):
            if self.region_stack:
                role_str, region_attrs, elements = self.region_stack.pop()
                if elements:
                    count = self.region_counts.get(role_str, 0)
                    self.region_counts[role_str] = count + 1
                    region_id = _generate_region_id(role_str, count)

                    region_label = region_attrs.get("aria-label") or None
                    region = SomRegion(
                        id=region_id,
                        role=RegionRole(role_str),
                        label=region_label,
                        elements=elements,
                    )
                    # Add form-specific attrs
                    if role_str == "form":
                        region.action = region_attrs.get("action")
                        region.method = region_attrs.get("method", "").upper() or None

                    self.regions.append(region)

        # Pass text up to parent
        if self.stack and text:
            self.stack[-1].text_parts.append(text + " ")

    def _add_element(self, element: SomElement):
        """Add an element to the current region or body."""
        if self.region_stack:
            self.region_stack[-1][2].append(element)
        elif self.stack:
            self.stack[-1].children.append(element)
        else:
            self.body_elements.append(element)

    def _get_actions(
        self, role: ElementRole, tag: str, attrs: dict[str, str]
    ) -> list[ElementAction]:
        """Determine available actions for an element."""
        if attrs.get("disabled") is not None:
            return []

        actions = []
        if role in (ElementRole.LINK, ElementRole.BUTTON):
            actions.append(ElementAction.CLICK)
        elif role in (ElementRole.TEXT_INPUT, ElementRole.TEXTAREA):
            actions.extend([ElementAction.TYPE, ElementAction.CLEAR])
        elif role == ElementRole.SELECT:
            actions.append(ElementAction.SELECT)
        elif role == ElementRole.CHECKBOX:
            actions.append(ElementAction.TOGGLE)
        elif role == ElementRole.RADIO:
            actions.append(ElementAction.CLICK)

        return actions

    def _build_attrs(
        self,
        role: ElementRole,
        tag: str,
        attr_dict: dict[str, str],
        frame: _StackFrame,
    ) -> Optional[SomElementAttrs]:
        """Build SOM element attributes from HTML attributes."""
        kwargs: dict = {}

        if role == ElementRole.LINK:
            href = attr_dict.get("href", "")
            if href and self.url:
                href = urljoin(self.url, href)
            kwargs["href"] = href or None

        elif role in (ElementRole.TEXT_INPUT, ElementRole.TEXTAREA):
            if tag == "input":
                kwargs["input_type"] = attr_dict.get("type", "text")
            kwargs["placeholder"] = attr_dict.get("placeholder") or None
            kwargs["value"] = attr_dict.get("value") or None
            kwargs["required"] = "required" in attr_dict or None
            kwargs["disabled"] = "disabled" in attr_dict or None

        elif role == ElementRole.SELECT:
            kwargs["options"] = frame.options or None
            kwargs["multiple"] = "multiple" in attr_dict or None
            kwargs["required"] = "required" in attr_dict or None

        elif role == ElementRole.CHECKBOX or role == ElementRole.RADIO:
            kwargs["checked"] = "checked" in attr_dict or None
            kwargs["disabled"] = "disabled" in attr_dict or None
            if role == ElementRole.RADIO:
                kwargs["group"] = attr_dict.get("name") or None

        elif role == ElementRole.HEADING:
            kwargs["level"] = HEADING_LEVELS.get(tag)

        elif role == ElementRole.IMAGE:
            kwargs["alt"] = attr_dict.get("alt") or None
            src = attr_dict.get("src", "")
            if src and self.url:
                src = urljoin(self.url, src)
            kwargs["src"] = src or None

        elif role == ElementRole.LIST:
            kwargs["ordered"] = tag == "ol" or None
            kwargs["items"] = frame.list_items or None

        elif role == ElementRole.TABLE:
            kwargs["headers"] = frame.table_headers or None
            kwargs["rows"] = frame.table_rows or None

        elif role == ElementRole.BUTTON:
            kwargs["disabled"] = "disabled" in attr_dict or None

        # Filter out all-None attrs
        if any(v is not None for v in kwargs.values()):
            return SomElementAttrs(**kwargs)
        return None

    def handle_data(self, data: str):
        if self.in_title:
            self.title_parts.append(data)
            return
        if self.in_head or self.skip_depth > 0:
            return
        if self.stack:
            self.stack[-1].text_parts.append(data)

    def handle_entityref(self, name: str):
        self.handle_data(f"&{name};")

    def handle_charref(self, name: str):
        self.handle_data(f"&#{name};")


def html_to_som(html: str, url: str = "", title: str = "") -> Som:
    """Convert raw HTML into a SOM object.

    Args:
        html: The full HTML string (document.documentElement.outerHTML).
        url: The page URL (used for resolving relative links).
        title: Optional page title override.

    Returns:
        A valid Som object with regions, elements, and metadata.
    """
    from urllib.parse import urlparse

    origin = ""
    if url:
        parsed = urlparse(url)
        origin = f"{parsed.scheme}://{parsed.netloc}"

    parser = SOMHTMLParser(url=url, origin=origin)
    parser.feed(html)

    # Close any unclosed tags
    while parser.stack:
        parser._close_frame()

    # If no regions were created, wrap body elements in a content region
    regions = parser.regions
    if parser.body_elements:
        # Add remaining body elements to a content region
        count = parser.region_counts.get("content", 0)
        regions.append(SomRegion(
            id=_generate_region_id("content", count),
            role=RegionRole.CONTENT,
            label=None,
            elements=parser.body_elements,
        ))

    if not regions:
        regions = [SomRegion(
            id="r_content_0",
            role=RegionRole.CONTENT,
            label=None,
            elements=[],
        )]

    # Count elements
    element_count = 0
    interactive_count = 0
    for region in regions:
        for el in region.elements:
            element_count += 1
            if el.actions:
                interactive_count += 1
            if el.children:
                for child in el.children:
                    element_count += 1
                    if child.actions:
                        interactive_count += 1

    html_bytes = len(html.encode("utf-8"))
    som_obj = Som(
        som_version="0.1",
        url=url or "about:blank",
        title=title or parser.title or "",
        lang=parser.lang,
        regions=regions,
        meta=SomMeta(
            html_bytes=html_bytes,
            som_bytes=0,  # filled below
            element_count=element_count,
            interactive_count=interactive_count,
        ),
    )

    # Compute som_bytes from the serialized JSON
    som_json = som_obj.model_dump_json()
    som_obj.meta.som_bytes = len(som_json.encode("utf-8"))

    return som_obj
