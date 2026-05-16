"""Convert SOM documents to concise text representations for LLM consumption."""

from __future__ import annotations

import json
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
    form_flags = []
    for key in ("target", "enctype", "accept_charset", "autocomplete"):
        if region.get(key):
            form_flags.append(f"{key}={region[key]}")
    if region.get("novalidate") is True:
        form_flags.append("novalidate=true")
    if form_flags:
        header += " " + " ".join(f"[{flag}]" for flag in form_flags)
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
    state_str = _action_state_to_text(elem, bool(actions))
    if state_str:
        hint_str = f"{hint_str} {state_str}".rstrip()

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
            aria = attrs.get("aria") or {}
            if attrs.get("checked") or aria.get("checked") is True:
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


def _fnv1a32(value: str) -> str:
    hash_value = 0x811C9DC5
    for char in value:
        hash_value ^= ord(char)
        hash_value = (hash_value * 0x01000193) & 0xFFFFFFFF
    return f"{hash_value:08x}"


def _compact_string(value: object) -> str | None:
    return value if isinstance(value, str) and value else None


def _action_cache_key(elem: dict[str, Any]) -> str:
    attrs = elem.get("attrs") or {}
    actions = elem.get("actions")
    action_values = sorted(actions) if isinstance(actions, list) else []
    parts = [
        _compact_string(elem.get("id")),
        _compact_string(elem.get("role")),
        _compact_string(elem.get("label") or elem.get("text")),
        ",".join(str(action) for action in action_values) or None,
        _compact_string(attrs.get("name")),
        _compact_string(attrs.get("href")),
        _compact_string(attrs.get("input_type")),
        _compact_string(attrs.get("group")),
        _compact_string(attrs.get("placeholder")),
    ]
    encoded = json.dumps(parts, separators=(",", ":"))
    return f"plasmate-action:v1:{_fnv1a32(encoded)}"


def _action_state_to_text(elem: dict[str, Any], interactive: bool = False) -> str:
    attrs = elem.get("attrs") or {}
    flags: list[str] = []
    aria = attrs.get("aria") if isinstance(attrs.get("aria"), dict) else {}
    readonly = attrs.get("readonly") is True or aria.get("readonly") is True
    if attrs.get("disabled") is True:
        flags.append("[disabled]")
        flags.append("[blocked_reason=disabled]")
    elif attrs.get("inert") is True:
        flags.append("[inert]")
        flags.append("[blocked_reason=inert]")
    elif readonly:
        flags.append("[readonly]")
        flags.append("[blocked_reason=readonly]")
    elif interactive:
        flags.append("[enabled]")
    if interactive:
        flags.append(f"[cache_key={_action_cache_key(elem)}]")
    if interactive and elem.get("html_id"):
        flags.append(f'[html_id="{elem["html_id"]}"]')
    if attrs.get("required") is True:
        flags.append("[required]")
    if readonly and "[readonly]" not in flags:
        flags.append("[readonly]")
    if attrs.get("inert") is True and "[inert]" not in flags:
        flags.append("[inert]")
    if attrs.get("value"):
        flags.append(f'[value="{attrs["value"]}"]')
    if attrs.get("name"):
        flags.append(f'[name="{attrs["name"]}"]')
    if attrs.get("accept"):
        flags.append(f'[accept="{attrs["accept"]}"]')
    if "capture" in attrs:
        flags.append(f'[capture="{attrs["capture"]}"]')
    if "multiple" in attrs:
        flags.append(f'[multiple="{attrs["multiple"]}"]')
    if attrs.get("selected_values"):
        flags.append(f'[selected_values="{",".join(attrs["selected_values"])}"]')
    if "size" in attrs:
        flags.append(f'[size="{attrs["size"]}"]')
    if attrs.get("autocomplete"):
        flags.append(f'[autocomplete="{attrs["autocomplete"]}"]')
    if attrs.get("inputmode"):
        flags.append(f'[inputmode="{attrs["inputmode"]}"]')
    if attrs.get("enterkeyhint"):
        flags.append(f'[enterkeyhint="{attrs["enterkeyhint"]}"]')
    if attrs.get("autocapitalize"):
        flags.append(f'[autocapitalize="{attrs["autocapitalize"]}"]')
    if attrs.get("dirname"):
        flags.append(f'[dirname="{attrs["dirname"]}"]')
    if attrs.get("form"):
        flags.append(f'[form="{attrs["form"]}"]')
    if attrs.get("list"):
        flags.append(f'[list="{attrs["list"]}"]')
    if attrs.get("target"):
        flags.append(f'[target="{attrs["target"]}"]')
    if attrs.get("rel"):
        flags.append(f'[rel="{attrs["rel"]}"]')
    if attrs.get("hreflang"):
        flags.append(f'[hreflang="{attrs["hreflang"]}"]')
    if attrs.get("type"):
        flags.append(f'[type="{attrs["type"]}"]')
    if attrs.get("referrerpolicy"):
        flags.append(f'[referrerpolicy="{attrs["referrerpolicy"]}"]')
    if "download" in attrs:
        flags.append(f'[download="{attrs["download"]}"]')
    for command_key in (
        "popovertarget",
        "popovertargetaction",
        "commandfor",
        "command",
        "popover",
        "button_type",
        "formaction",
        "formmethod",
        "formenctype",
        "formtarget",
    ):
        if attrs.get(command_key):
            flags.append(f'[{command_key}="{attrs[command_key]}"]')
    if "formnovalidate" in attrs:
        flags.append(f'[formnovalidate="{attrs["formnovalidate"]}"]')
    if attrs.get("accesskey"):
        flags.append(f'[accesskey="{attrs["accesskey"]}"]')
    if attrs.get("title"):
        flags.append(f'[title="{attrs["title"]}"]')
    if attrs.get("source_role"):
        flags.append(f'[source_role="{attrs["source_role"]}"]')
    if attrs.get("test_id"):
        flags.append(f'[test_id="{attrs["test_id"]}"]')
    if "spellcheck" in attrs:
        flags.append(f'[spellcheck="{attrs["spellcheck"]}"]')
    if "draggable" in attrs:
        flags.append(f'[draggable="{attrs["draggable"]}"]')
    for constraint_key in ("minlength", "maxlength", "min", "max", "step", "pattern"):
        if constraint_key in attrs:
            flags.append(f'[{constraint_key}="{attrs[constraint_key]}"]')
    if "checked" in attrs:
        flags.append(f'[checked="{attrs["checked"]}"]')
    elif isinstance(attrs.get("aria"), dict) and "checked" in attrs["aria"]:
        flags.append(f'[checked="{attrs["aria"]["checked"]}"]')
    if aria:
        for state_key in (
            "expanded",
            "pressed",
            "selected",
            "current",
            "controls",
            "haspopup",
            "invalid",
            "placeholder",
            "autocomplete",
            "active_descendant",
            "errormessage",
            "keyshortcuts",
            "roledescription",
            "busy",
            "live",
            "atomic",
            "relevant",
            "owns",
            "flowto",
            "details",
            "multiline",
            "multiselectable",
            "orientation",
            "sort",
            "level",
            "posinset",
            "setsize",
            "grabbed",
            "dropeffect",
            "valuemin",
            "valuemax",
            "valuenow",
            "valuetext",
        ):
            if state_key in aria:
                output_key = (
                    "aria_autocomplete"
                    if state_key == "autocomplete"
                    else "aria_placeholder"
                    if state_key == "placeholder"
                    else state_key
                )
                flags.append(f'[{output_key}="{aria[state_key]}"]')
    if attrs.get("group"):
        flags.append(f'[group="{attrs["group"]}"]')
    if attrs.get("description"):
        flags.append(f'[description="{attrs["description"]}"]')
    return " ".join(flags)


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
