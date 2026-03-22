"""Convert SOM documents to concise text representations for LLM consumption."""

from __future__ import annotations

from typing import Any


def som_to_text(som: dict[str, Any]) -> str:
    """Convert a SOM dict to a concise text representation for LLM context.

    The output format is designed to be token-efficient while preserving all
    information an agent needs: page structure, interactive elements with IDs,
    and content summaries. A typical page produces under 2000 tokens.

    Args:
        som: A SOM document dict (as returned by Plasmate).

    Returns:
        A formatted text string suitable for LLM consumption.
    """
    lines: list[str] = []

    # Page header
    lines.append(f"Page: {som.get('title', 'Untitled')}")
    lines.append(f"URL: {som.get('url', '')}")

    # Surface description from structured data if available
    sd = som.get("structured_data")
    if sd:
        desc = (sd.get("meta") or {}).get("description")
        if desc:
            lines.append(f"Description: {desc}")

    lines.append("")

    # Regions
    for region in som.get("regions", []):
        lines.append(_region_header(region))
        for elem in region.get("elements", []):
            text = _element_to_text(elem)
            if text:
                lines.append(text)
        lines.append("")

    # Compression stats
    meta = som.get("meta", {})
    html_bytes = meta.get("html_bytes", 0)
    som_bytes = meta.get("som_bytes", 0)
    ratio = f"{html_bytes / som_bytes:.1f}x" if som_bytes else "N/A"
    lines.append("---")
    lines.append(
        f"{html_bytes:,} → {som_bytes:,} bytes ({ratio}) | "
        f"{meta.get('element_count', 0)} elements, "
        f"{meta.get('interactive_count', 0)} interactive"
    )

    return "\n".join(lines)


def _region_header(region: dict[str, Any]) -> str:
    role = region["role"]
    label = region.get("label", "")
    header = f"## {role}"
    if label:
        header += f' "{label}"'
    if region.get("action"):
        method = region.get("method", "GET")
        header += f" -> {region['action']} [{method}]"
    return header


def _element_to_text(elem: dict[str, Any], indent: int = 1) -> str:
    prefix = "  " * indent
    role = elem.get("role", "")
    eid = elem.get("id", "")
    text = elem.get("text", "")
    label = elem.get("label", "")
    hints = elem.get("hints", [])
    attrs = elem.get("attrs") or {}
    actions = elem.get("actions")

    hint_str = " " + " ".join(f"[{h}]" for h in hints) if hints else ""

    # Interactive elements: show ID so the agent can reference them
    if actions:
        display = label or text
        if role == "link":
            href = attrs.get("href", "")
            return f'{prefix}[{eid}] link "{display}" -> {href}{hint_str}'
        if role == "button":
            return f'{prefix}[{eid}] button "{display}"{hint_str}'
        if role == "text_input":
            parts = [f"input({attrs.get('input_type', 'text')})"]
            if label:
                parts.append(f'"{label}"')
            if attrs.get("placeholder"):
                parts.append(f'placeholder="{attrs["placeholder"]}"')
            if attrs.get("value"):
                parts.append(f'value="{attrs["value"]}"')
            if attrs.get("required"):
                parts.append("[required]")
            return f"{prefix}[{eid}] {' '.join(parts)}{hint_str}"
        if role == "textarea":
            parts = ["textarea"]
            if label:
                parts.append(f'"{label}"')
            if attrs.get("placeholder"):
                parts.append(f'placeholder="{attrs["placeholder"]}"')
            return f"{prefix}[{eid}] {' '.join(parts)}{hint_str}"
        if role == "select":
            options = attrs.get("options", [])
            opt_texts = [o.get("text", o.get("value", "")) for o in options[:5]]
            desc = f'select "{label or text}"'
            if opt_texts:
                desc += f" [{', '.join(opt_texts)}]"
            if len(options) > 5:
                desc += f" (+{len(options) - 5} more)"
            return f"{prefix}[{eid}] {desc}{hint_str}"
        if role == "checkbox":
            state = "checked" if attrs.get("checked") else "unchecked"
            return f'{prefix}[{eid}] checkbox "{label or text}" ({state}){hint_str}'
        if role == "radio":
            desc = f'radio "{label or text}"'
            if attrs.get("group"):
                desc += f" group={attrs['group']}"
            if attrs.get("checked"):
                desc += " (selected)"
            return f"{prefix}[{eid}] {desc}{hint_str}"
        return f'{prefix}[{eid}] {role} "{display}"{hint_str}'

    # Non-interactive elements
    if role == "heading":
        level = attrs.get("level", 1)
        return f"{prefix}h{level}: {text}"
    if role == "paragraph":
        return f"{prefix}p: {text}" if text else ""
    if role == "image":
        alt = attrs.get("alt", "")
        return f"{prefix}image: {alt}" if alt else f"{prefix}image"
    if role == "list":
        return _list_to_text(attrs, prefix)
    if role == "table":
        return _table_to_text(attrs, prefix)
    if role == "separator":
        return f"{prefix}---"
    if role == "section":
        section_label = attrs.get("section_label", "")
        return f"{prefix}section: {section_label}" if section_label else ""
    if text:
        return f"{prefix}{role}: {text}"
    return ""


def _list_to_text(attrs: dict[str, Any], prefix: str) -> str:
    items = attrs.get("items", [])
    ordered = attrs.get("ordered", False)
    if not items:
        return ""
    result: list[str] = []
    for i, item in enumerate(items[:5]):
        marker = f"{i + 1}." if ordered else "-"
        result.append(f"{prefix}{marker} {item.get('text', '')}")
    if len(items) > 5:
        result.append(f"{prefix}  [{len(items) - 5} more items]")
    return "\n".join(result)


def _table_to_text(attrs: dict[str, Any], prefix: str) -> str:
    headers = attrs.get("headers", [])
    rows = attrs.get("rows", [])
    if not headers and not rows:
        return f"{prefix}table (empty)"
    parts: list[str] = []
    if headers:
        parts.append(f"{prefix}table: {' | '.join(headers)}")
    for row in rows[:3]:
        parts.append(f"{prefix}  {' | '.join(row)}")
    if len(rows) > 3:
        parts.append(f"{prefix}  [{len(rows) - 3} more rows]")
    return "\n".join(parts) if parts else f"{prefix}table ({len(rows)} rows)"
