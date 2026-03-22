"""Convert SOM documents to the text format Browser Use feeds to the LLM.

Browser Use normally gives the LLM a DOM tree like::

    [123]<button type="submit" />
      Submit Order
    [456]<input name="email" placeholder="you@example.com" />

Our SOM format is more compact and semantic::

    [1] button "Submit Order"
    [2] input(email) "Email" placeholder="you@example.com"

Both formats use ``[N]`` integer indices so the LLM can reference elements
in its action output. The SOM version uses ~10x fewer tokens because it
strips layout noise, merges redundant nodes, and uses semantic roles instead
of raw HTML tags.
"""

from __future__ import annotations

import json
from typing import Any, Optional


def som_to_browser_use_state(
    som: dict[str, Any],
    index_map: Optional[dict[str, int]] = None,
) -> str:
    """Convert a SOM dict to Browser Use-compatible text for LLM consumption.

    The output mirrors the format Browser Use feeds to the agent, but using
    SOM's semantic structure instead of raw DOM. Interactive elements are
    prefixed with ``[N]`` indices that the agent uses to specify click/type
    targets.

    Args:
        som: A SOM document dict (as returned by Plasmate).
        index_map: Optional mapping of SOM element ID -> integer index.
            If not provided, interactive elements are shown with their
            SOM IDs directly.

    Returns:
        Formatted text string for LLM consumption.
    """
    lines: list[str] = []

    # Page header (matches Browser Use's state description format)
    lines.append(f"[Tab] {som.get('title', 'Untitled')}")
    lines.append(f"[URL] {som.get('url', '')}")
    lines.append("")

    # Regions and elements
    for region in som.get("regions", []):
        lines.append(_region_header(region))
        for elem in region.get("elements", []):
            text = _element_to_text(elem, indent=1, index_map=index_map)
            if text:
                lines.append(text)
        lines.append("")

    # Compression stats footer
    meta = som.get("meta", {})
    html_bytes = meta.get("html_bytes", 0)
    som_bytes = meta.get("som_bytes", 0)
    ratio = f"{html_bytes / som_bytes:.1f}x" if som_bytes else "N/A"
    el_count = meta.get("element_count", 0)
    interactive = meta.get("interactive_count", 0)
    lines.append(f"[SOM] {html_bytes:,} -> {som_bytes:,} bytes ({ratio}) | {el_count} elements, {interactive} interactive")

    return "\n".join(lines)


def token_count_comparison(
    som: dict[str, Any],
    som_text: Optional[str] = None,
) -> dict[str, Any]:
    """Compare token counts between raw HTML and SOM representation.

    Useful for benchmarking and demonstrating the token savings Plasmate
    provides over traditional browser backends.

    Args:
        som: A SOM document dict.
        som_text: Pre-rendered SOM text. If not provided, it's generated
            from the SOM dict.

    Returns:
        Dict with token counts and savings ratio::

            {
                "html_bytes": 87234,
                "som_bytes": 4521,
                "html_tokens_est": 21808,
                "som_tokens_est": 1130,
                "byte_ratio": 19.3,
                "token_ratio": 19.3,
                "token_savings_pct": 94.8,
            }
    """
    meta = som.get("meta", {})
    html_bytes = meta.get("html_bytes", 0)
    som_bytes = meta.get("som_bytes", 0)

    if som_text is None:
        som_text = som_to_browser_use_state(som)

    som_text_bytes = len(som_text.encode("utf-8"))

    # Token estimation: ~4 chars per token (standard heuristic)
    html_tokens = html_bytes // 4
    som_tokens = som_text_bytes // 4

    byte_ratio = html_bytes / som_bytes if som_bytes else 0
    token_ratio = html_tokens / som_tokens if som_tokens else 0
    savings_pct = (1 - som_tokens / html_tokens) * 100 if html_tokens else 0

    return {
        "html_bytes": html_bytes,
        "som_bytes": som_bytes,
        "som_text_bytes": som_text_bytes,
        "html_tokens_est": html_tokens,
        "som_tokens_est": som_tokens,
        "byte_ratio": round(byte_ratio, 1),
        "token_ratio": round(token_ratio, 1),
        "token_savings_pct": round(savings_pct, 1),
    }


# ---- Internal formatting helpers ----


def _region_header(region: dict[str, Any]) -> str:
    role = region["role"]
    label = region.get("label", "")
    header = f"--- {role}"
    if label:
        header += f' "{label}"'
    if region.get("action"):
        method = region.get("method", "GET")
        header += f" -> {region['action']} [{method}]"
    header += " ---"
    return header


def _element_to_text(
    elem: dict[str, Any],
    indent: int = 1,
    index_map: Optional[dict[str, int]] = None,
) -> str:
    prefix = "  " * indent
    role = elem.get("role", "")
    eid = elem.get("id", "")
    text = elem.get("text", "")
    label = elem.get("label", "")
    hints = elem.get("hints", [])
    attrs = elem.get("attrs") or {}
    actions = elem.get("actions")

    hint_str = " " + " ".join(f"({h})" for h in hints) if hints else ""

    # Resolve index: use integer index if available, else SOM ID
    idx_str = ""
    if actions:
        if index_map and eid in index_map:
            idx_str = f"[{index_map[eid]}]"
        else:
            idx_str = f"[{eid}]"

    # Interactive elements
    if actions:
        display = label or text
        if role == "link":
            href = attrs.get("href", "")
            line = f'{prefix}{idx_str} link "{display}" -> {href}{hint_str}'
        elif role == "button":
            line = f'{prefix}{idx_str} button "{display}"{hint_str}'
        elif role == "text_input":
            parts = [f"input({attrs.get('input_type', 'text')})"]
            if label:
                parts.append(f'"{label}"')
            if attrs.get("placeholder"):
                parts.append(f'placeholder="{attrs["placeholder"]}"')
            if attrs.get("value"):
                parts.append(f'value="{attrs["value"]}"')
            if attrs.get("required"):
                parts.append("(required)")
            line = f"{prefix}{idx_str} {' '.join(parts)}{hint_str}"
        elif role == "textarea":
            parts = ["textarea"]
            if label:
                parts.append(f'"{label}"')
            if attrs.get("placeholder"):
                parts.append(f'placeholder="{attrs["placeholder"]}"')
            line = f"{prefix}{idx_str} {' '.join(parts)}{hint_str}"
        elif role == "select":
            options = attrs.get("options", [])
            opt_texts = [o.get("text", o.get("value", "")) for o in options[:5]]
            desc = f'select "{label or text}"'
            if opt_texts:
                desc += f" [{', '.join(opt_texts)}]"
            if len(options) > 5:
                desc += f" (+{len(options) - 5} more)"
            line = f"{prefix}{idx_str} {desc}{hint_str}"
        elif role == "checkbox":
            state = "checked" if attrs.get("checked") else "unchecked"
            line = f'{prefix}{idx_str} checkbox "{label or text}" ({state}){hint_str}'
        elif role == "radio":
            desc = f'radio "{label or text}"'
            if attrs.get("group"):
                desc += f" group={attrs['group']}"
            if attrs.get("checked"):
                desc += " (selected)"
            line = f"{prefix}{idx_str} {desc}{hint_str}"
        else:
            line = f'{prefix}{idx_str} {role} "{display}"{hint_str}'
    else:
        # Non-interactive elements
        if role == "heading":
            level = attrs.get("level", 1)
            line = f"{prefix}h{level}: {text}"
        elif role == "paragraph":
            line = f"{prefix}{text}" if text else ""
        elif role == "image":
            alt = attrs.get("alt", "")
            line = f"{prefix}[img] {alt}" if alt else f"{prefix}[img]"
        elif role == "list":
            line = _list_to_text(attrs, prefix)
        elif role == "table":
            line = _table_to_text(attrs, prefix)
        elif role == "separator":
            line = ""  # Skip separators to save tokens
        elif role == "section":
            section_label = attrs.get("section_label", "")
            line = f"{prefix}# {section_label}" if section_label else ""
        elif text:
            line = f"{prefix}{text}"
        else:
            line = ""

    # Recurse into children
    children_text = ""
    for child in elem.get("children", []) or []:
        child_line = _element_to_text(child, indent + 1, index_map)
        if child_line:
            children_text += "\n" + child_line

    if not line and not children_text:
        return ""
    return (line or "") + children_text


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
        result.append(f"{prefix}  (+{len(items) - 5} more)")
    return "\n".join(result)


def _table_to_text(attrs: dict[str, Any], prefix: str) -> str:
    headers = attrs.get("headers", [])
    rows = attrs.get("rows", [])
    if not headers and not rows:
        return f"{prefix}[table empty]"
    parts: list[str] = []
    if headers:
        parts.append(f"{prefix}{' | '.join(headers)}")
    for row in rows[:3]:
        parts.append(f"{prefix}{' | '.join(row)}")
    if len(rows) > 3:
        parts.append(f"{prefix}(+{len(rows) - 3} more rows)")
    return "\n".join(parts) if parts else f"{prefix}[table {len(rows)} rows]"
