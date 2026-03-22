"""Query, filter, and search utilities for SOM objects."""

from __future__ import annotations

from typing import Callable, Dict, List, Optional, Union

from .types import ElementRole, RegionRole, Som, SomElement, SomRegion


def _collect_elements(elements: List[SomElement]) -> List[SomElement]:
    """Recursively collect all elements including nested children."""
    result: List[SomElement] = []
    for el in elements:
        result.append(el)
        if el.children:
            result.extend(_collect_elements(el.children))
    return result


def get_all_elements(som: Som) -> List[SomElement]:
    """Flatten all elements from all regions, including nested children."""
    result: List[SomElement] = []
    for region in som.regions:
        result.extend(_collect_elements(region.elements))
    return result


def find_by_role(som: Som, role: Union[ElementRole, str]) -> List[SomElement]:
    """Find all elements with a specific role.

    Args:
        som: The parsed SOM object.
        role: An ElementRole enum value or string (e.g. "link").
    """
    if isinstance(role, str):
        role = ElementRole(role)
    return [el for el in get_all_elements(som) if el.role == role]


def find_by_id(som: Som, id: str) -> Optional[SomElement]:
    """Find an element by its SOM id. Returns None if not found."""
    for el in get_all_elements(som):
        if el.id == id:
            return el
    return None


def find_by_text(
    som: Som, text: str, *, exact: bool = False
) -> List[SomElement]:
    """Find elements containing text.

    Args:
        som: The parsed SOM object.
        text: The text to search for.
        exact: If True, match the full text exactly (case-sensitive).
               If False (default), case-insensitive substring match.
    """
    results: List[SomElement] = []
    for el in get_all_elements(som):
        el_text = el.text or ""
        el_label = el.label or ""
        if exact:
            if text == el_text or text == el_label:
                results.append(el)
        else:
            text_lower = text.lower()
            if text_lower in el_text.lower() or text_lower in el_label.lower():
                results.append(el)
    return results


def get_interactive_elements(som: Som) -> List[SomElement]:
    """Get all elements that have actions."""
    return [el for el in get_all_elements(som) if el.actions]


def get_links(som: Som) -> List[Dict[str, Optional[str]]]:
    """Extract all links as dicts with text, href, and id."""
    links: List[Dict[str, Optional[str]]] = []
    for el in find_by_role(som, ElementRole.LINK):
        href = el.attrs.href if el.attrs else None
        links.append({"text": el.text, "href": href, "id": el.id})
    return links


def get_forms(som: Som) -> List[SomRegion]:
    """Get all form regions."""
    return [r for r in som.regions if r.role == RegionRole.FORM]


def get_inputs(som: Som) -> List[SomElement]:
    """Get all input elements (text_input, textarea, select, checkbox, radio)."""
    input_roles = {
        ElementRole.TEXT_INPUT,
        ElementRole.TEXTAREA,
        ElementRole.SELECT,
        ElementRole.CHECKBOX,
        ElementRole.RADIO,
    }
    return [el for el in get_all_elements(som) if el.role in input_roles]


def get_headings(som: Som) -> List[Dict[str, object]]:
    """Extract heading hierarchy as a list of dicts with level, text, and id."""
    headings: List[Dict[str, object]] = []
    for el in find_by_role(som, ElementRole.HEADING):
        level = el.attrs.level if el.attrs and el.attrs.level is not None else 0
        headings.append({"level": level, "text": el.text, "id": el.id})
    return headings


def get_text(som: Som) -> str:
    """Extract all visible text content from the SOM."""
    parts: List[str] = []
    for el in get_all_elements(som):
        if el.text:
            parts.append(el.text)
        elif el.label:
            parts.append(el.label)
    return "\n".join(parts)


def get_text_by_region(som: Som) -> List[Dict[str, object]]:
    """Extract text grouped by region."""
    results: List[Dict[str, object]] = []
    for region in som.regions:
        texts: List[str] = []
        for el in _collect_elements(region.elements):
            if el.text:
                texts.append(el.text)
            elif el.label:
                texts.append(el.label)
        results.append({
            "region_id": region.id,
            "role": region.role.value,
            "label": region.label,
            "text": "\n".join(texts),
        })
    return results


def get_compression_ratio(som: Som) -> float:
    """Return html_bytes / som_bytes compression ratio."""
    if som.meta.som_bytes == 0:
        return float("inf")
    return som.meta.html_bytes / som.meta.som_bytes


def to_markdown(som: Som) -> str:
    """Convert a SOM object to readable markdown."""
    lines: List[str] = []
    lines.append(f"# {som.title}")
    lines.append(f"URL: {som.url}")
    lines.append("")

    for region in som.regions:
        role_label = region.role.value.title()
        if region.label:
            lines.append(f"## {role_label}: {region.label}")
        else:
            lines.append(f"## {role_label}")
        lines.append("")

        for el in _collect_elements(region.elements):
            _element_to_markdown(el, lines)

        lines.append("")

    return "\n".join(lines)


def _element_to_markdown(el: SomElement, lines: List[str]) -> None:
    """Append markdown for a single element."""
    role = el.role

    if role == ElementRole.HEADING:
        level = el.attrs.level if el.attrs and el.attrs.level else 1
        prefix = "#" * (level + 2)  # offset by 2 since region is ##
        lines.append(f"{prefix} {el.text or ''}")
    elif role == ElementRole.PARAGRAPH:
        lines.append(el.text or "")
        lines.append("")
    elif role == ElementRole.LINK:
        href = el.attrs.href if el.attrs else "#"
        lines.append(f"[{el.text or ''}]({href})")
    elif role == ElementRole.BUTTON:
        lines.append(f"[Button: {el.text or ''}]")
    elif role == ElementRole.IMAGE:
        alt = el.attrs.alt if el.attrs else ""
        src = el.attrs.src if el.attrs else ""
        lines.append(f"![{alt}]({src})")
    elif role in (ElementRole.TEXT_INPUT, ElementRole.TEXTAREA):
        label = el.label or ""
        placeholder = ""
        if el.attrs and el.attrs.placeholder:
            placeholder = f' placeholder="{el.attrs.placeholder}"'
        lines.append(f"[Input: {label}{placeholder}]")
    elif role == ElementRole.SELECT:
        lines.append(f"[Select: {el.label or el.text or ''}]")
    elif role in (ElementRole.CHECKBOX, ElementRole.RADIO):
        checked = ""
        if el.attrs and el.attrs.checked:
            checked = "x"
        lines.append(f"[{checked}] {el.text or el.label or ''}")
    elif role == ElementRole.LIST:
        if el.attrs and el.attrs.items:
            for item in el.attrs.items:
                lines.append(f"- {item.text}")
    elif role == ElementRole.SEPARATOR:
        lines.append("---")
    else:
        if el.text:
            lines.append(el.text)


def filter_elements(
    som: Som, predicate: Callable[[SomElement], bool]
) -> List[SomElement]:
    """Filter all elements using a predicate function."""
    return [el for el in get_all_elements(som) if predicate(el)]
