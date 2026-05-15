"""Query, filter, and search utilities for SOM objects."""

from __future__ import annotations

import json
from typing import Callable, Dict, List, Optional, Union

from .types import (
    ElementAction,
    ElementRole,
    RegionRole,
    SemanticHint,
    Som,
    SomElement,
    SomRegion,
)


def _collect_elements(elements: List[SomElement]) -> List[SomElement]:
    """Recursively collect all elements including nested children."""
    result: List[SomElement] = []
    for el in elements:
        result.append(el)
        if el.children:
            result.extend(_collect_elements(el.children))
        if el.shadow:
            result.extend(_collect_elements(el.shadow.elements))
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


def find_by_html_id(som: Som, html_id: str) -> Optional[SomElement]:
    """Find an element by its original HTML id. Returns None if not found."""
    for el in get_all_elements(som):
        if el.html_id == html_id:
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


def find_by_action(
    som: Som, action: Union[ElementAction, str]
) -> List[SomElement]:
    """Find all elements that expose a specific action."""
    if isinstance(action, str):
        action = ElementAction(action)
    return [el for el in get_all_elements(som) if el.actions and action in el.actions]


def find_by_hint(
    som: Som, hint: Union[SemanticHint, str]
) -> List[SomElement]:
    """Find all elements tagged with a specific semantic hint."""
    if isinstance(hint, str):
        hint = SemanticHint(hint)
    return [el for el in get_all_elements(som) if el.hints and hint in el.hints]


def get_interactive_elements(som: Som) -> List[SomElement]:
    """Get all elements that have actions."""
    return [el for el in get_all_elements(som) if el.actions]


def _fnv1a32(value: str) -> str:
    hash_value = 0x811C9DC5
    for char in value:
        hash_value ^= ord(char)
        hash_value = (hash_value * 0x01000193) & 0xFFFFFFFF
    return f"{hash_value:08x}"


def _compact_string(value: object) -> Optional[str]:
    return value if isinstance(value, str) and value else None


def get_action_plan_cache_key(item: Dict[str, object]) -> str:
    """Return a deterministic key for caching or comparing an action target."""
    actions = item.get("actions")
    action_values = sorted(actions) if isinstance(actions, list) else []
    parts = [
        _compact_string(item.get("id")),
        _compact_string(item.get("role")),
        _compact_string(item.get("label")),
        ",".join(str(action) for action in action_values) or None,
        _compact_string(item.get("name")),
        _compact_string(item.get("href")),
        _compact_string(item.get("input_type")),
        _compact_string(item.get("group")),
        _compact_string(item.get("placeholder")),
    ]
    for provenance_key in ("test_id", "data_action"):
        value = _compact_string(item.get(provenance_key))
        if value:
            parts.append(value)
    encoded = json.dumps(parts, separators=(",", ":"))
    return f"plasmate-action:v1:{_fnv1a32(encoded)}"


def _copy_form_context(item: Dict[str, object], region: SomRegion) -> None:
    if region.action:
        item["form_action"] = region.action
    if region.method:
        item["form_method"] = region.method
    if region.target:
        item["form_target"] = region.target
    if region.enctype:
        item["form_enctype"] = region.enctype
    if region.novalidate is not None:
        item["form_novalidate"] = region.novalidate
    if region.accept_charset:
        item["form_accept_charset"] = region.accept_charset
    if region.autocomplete:
        item["form_autocomplete"] = region.autocomplete


def get_action_plan(som: Som) -> List[Dict[str, object]]:
    """Return compact action targets for agent planning."""
    plan: List[Dict[str, object]] = []
    form_context_by_id: Dict[str, SomRegion] = {}
    for region in som.regions:
        for el in _collect_elements(region.elements):
            if el.actions:
                form_context_by_id[el.id] = region
    for el in get_interactive_elements(som):
        attrs = el.attrs
        item: Dict[str, object] = {
            "id": el.id,
            "role": el.role.value,
            "actions": [action.value for action in el.actions or []],
            "enabled": True,
        }
        if el.html_id:
            item["html_id"] = el.html_id
        label = el.label or el.text
        if label:
            item["label"] = label
        if attrs:
            if attrs.href:
                item["href"] = attrs.href
            if attrs.target:
                item["target"] = attrs.target
            if attrs.rel:
                item["rel"] = attrs.rel
            if attrs.download is not None:
                item["download"] = attrs.download
            if attrs.alt:
                item["alt"] = attrs.alt
            if attrs.src:
                item["src"] = attrs.src
            if attrs.name:
                item["name"] = attrs.name
            if attrs.accept:
                item["accept"] = attrs.accept
            if attrs.capture is not None:
                item["capture"] = attrs.capture
            if attrs.multiple is not None:
                item["multiple"] = attrs.multiple
            if attrs.options:
                item["options"] = [
                    option.model_dump(exclude_none=True) for option in attrs.options
                ]
            if attrs.selected_values:
                item["selected_values"] = attrs.selected_values
            if attrs.size is not None:
                item["size"] = attrs.size
            if attrs.autocomplete:
                item["autocomplete"] = attrs.autocomplete
            if attrs.inputmode:
                item["inputmode"] = attrs.inputmode
            if attrs.enterkeyhint:
                item["enterkeyhint"] = attrs.enterkeyhint
            if attrs.autocapitalize:
                item["autocapitalize"] = attrs.autocapitalize
            if attrs.dirname:
                item["dirname"] = attrs.dirname
            if attrs.dir:
                item["dir"] = attrs.dir
            if attrs.lang:
                item["lang"] = attrs.lang
            if attrs.form:
                item["form"] = attrs.form
            if attrs.list:
                item["list"] = attrs.list
            if attrs.popovertarget:
                item["popovertarget"] = attrs.popovertarget
            if attrs.popovertargetaction:
                item["popovertargetaction"] = attrs.popovertargetaction
            if attrs.commandfor:
                item["commandfor"] = attrs.commandfor
            if attrs.command:
                item["command"] = attrs.command
            if attrs.popover:
                item["popover"] = attrs.popover
            if attrs.button_type:
                item["button_type"] = attrs.button_type
            if attrs.formaction:
                item["formaction"] = attrs.formaction
            if attrs.formmethod:
                item["formmethod"] = attrs.formmethod
            if attrs.formenctype:
                item["formenctype"] = attrs.formenctype
            if attrs.formtarget:
                item["formtarget"] = attrs.formtarget
            if attrs.formnovalidate is not None:
                item["formnovalidate"] = attrs.formnovalidate
            if attrs.accesskey:
                item["accesskey"] = attrs.accesskey
            if attrs.title:
                item["title"] = attrs.title
            if attrs.aria_label:
                item["aria_label"] = attrs.aria_label
            if attrs.aria_description:
                item["aria_description"] = attrs.aria_description
            if attrs.labelledby:
                item["labelledby"] = attrs.labelledby
            if attrs.describedby:
                item["describedby"] = attrs.describedby
            if attrs.spellcheck is not None:
                item["spellcheck"] = attrs.spellcheck
            if attrs.draggable is not None:
                item["draggable"] = attrs.draggable
            if attrs.input_type:
                item["input_type"] = attrs.input_type
            if attrs.value:
                item["value"] = attrs.value
            if attrs.placeholder:
                item["placeholder"] = attrs.placeholder
            if attrs.minlength is not None:
                item["minlength"] = attrs.minlength
            if attrs.maxlength is not None:
                item["maxlength"] = attrs.maxlength
            if attrs.min is not None:
                item["min"] = attrs.min
            if attrs.max is not None:
                item["max"] = attrs.max
            if attrs.step:
                item["step"] = attrs.step
            if attrs.pattern:
                item["pattern"] = attrs.pattern
            if attrs.description:
                item["description"] = attrs.description
            if attrs.test_id:
                item["test_id"] = attrs.test_id
            if attrs.data_action:
                item["data_action"] = attrs.data_action
            if attrs.data_state:
                item["data_state"] = attrs.data_state
            if attrs.checked is not None:
                item["checked"] = attrs.checked
            elif attrs.aria and "checked" in attrs.aria:
                item["checked"] = attrs.aria["checked"]
            if attrs.aria:
                for aria_key in (
                    "expanded",
                    "readonly",
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
                    "modal",
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
                    "rowindex",
                    "colindex",
                    "rowcount",
                    "colcount",
                    "grabbed",
                    "dropeffect",
                    "valuemin",
                    "valuemax",
                    "valuenow",
                    "valuetext",
                ):
                    if aria_key in attrs.aria:
                        item_key = (
                            "aria_autocomplete"
                            if aria_key == "autocomplete"
                            else "aria_placeholder"
                            if aria_key == "placeholder"
                            else aria_key
                        )
                        item[item_key] = attrs.aria[aria_key]
            if attrs.required is not None:
                item["required"] = attrs.required
            if attrs.readonly is not None:
                item["readonly"] = attrs.readonly
            elif attrs.aria and attrs.aria.get("readonly") is not None:
                item["readonly"] = attrs.aria["readonly"]
            if attrs.disabled is not None:
                item["disabled"] = attrs.disabled
                if attrs.disabled:
                    item["enabled"] = False
                    item["blocked_reason"] = "disabled"
            if attrs.inert is not None:
                item["inert"] = attrs.inert
                if attrs.inert:
                    item["enabled"] = False
                    item["blocked_reason"] = "inert"
            elif item.get("readonly") is True and item.get("enabled") is not False:
                item["enabled"] = False
                item["blocked_reason"] = "readonly"
            if attrs.group:
                item["group"] = attrs.group
        if el.id in form_context_by_id:
            _copy_form_context(item, form_context_by_id[el.id])
        item["cache_key"] = get_action_plan_cache_key(item)
        plan.append(item)
    return plan


def get_enabled_action_plan(som: Som) -> List[Dict[str, object]]:
    """Return compact action targets that are currently available."""
    return [item for item in get_action_plan(som) if item.get("enabled") is not False]


def get_action_plan_index(
    som: Som, *, enabled_only: bool = False
) -> Dict[str, object]:
    """Return action targets indexed by SOM id, cache key, and original HTML id."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    index: Dict[str, object] = {
        "by_id": {},
        "by_cache_key": {},
        "by_cache_key_all": {},
        "by_html_id": {},
        "by_html_id_all": {},
        "duplicate_cache_keys": [],
        "duplicate_html_ids": [],
    }
    by_id = index["by_id"]
    by_cache_key = index["by_cache_key"]
    by_cache_key_all = index["by_cache_key_all"]
    by_html_id = index["by_html_id"]
    by_html_id_all = index["by_html_id_all"]
    for item in plan:
        target_id = item.get("id")
        if isinstance(target_id, str) and target_id not in by_id:
            by_id[target_id] = item
        cache_key = item.get("cache_key")
        if isinstance(cache_key, str):
            if cache_key not in by_cache_key:
                by_cache_key[cache_key] = item
            by_cache_key_all.setdefault(cache_key, []).append(item)
        html_id = item.get("html_id")
        if isinstance(html_id, str):
            if html_id not in by_html_id:
                by_html_id[html_id] = item
            by_html_id_all.setdefault(html_id, []).append(item)
    index["duplicate_cache_keys"] = sorted(
        key for key, items in by_cache_key_all.items() if len(items) > 1
    )
    index["duplicate_html_ids"] = sorted(
        key for key, items in by_html_id_all.items() if len(items) > 1
    )
    return index


def get_action_plan_fingerprint(som: Som, *, enabled_only: bool = False) -> str:
    """Return a deterministic fingerprint for the current compact action plan."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    rows = sorted(
        [
            [
                item.get("cache_key"),
                item.get("enabled") is not False,
                item.get("blocked_reason"),
            ]
            for item in plan
        ],
        key=lambda row: str(row[0]),
    )
    encoded = json.dumps(rows, separators=(",", ":"))
    return f"plasmate-plan:v1:{_fnv1a32(encoded)}"


def get_action_plan_summary(som: Som) -> Dict[str, object]:
    """Return compact action-plan counts and fingerprints for replay validation."""
    plan = get_action_plan(som)
    by_role: Dict[str, int] = {}
    blocked_reasons: Dict[str, int] = {}
    cache_key_counts: Dict[str, int] = {}
    html_id_counts: Dict[str, int] = {}
    enabled_count = 0
    with_cache_key = 0
    with_html_id = 0
    with_test_id = 0
    with_data_action = 0
    with_data_state = 0
    for item in plan:
        role = item.get("role")
        if isinstance(role, str):
            by_role[role] = by_role.get(role, 0) + 1
        cache_key = item.get("cache_key")
        if isinstance(cache_key, str) and cache_key:
            with_cache_key += 1
            cache_key_counts[cache_key] = cache_key_counts.get(cache_key, 0) + 1
        html_id = item.get("html_id")
        if isinstance(html_id, str) and html_id:
            with_html_id += 1
            html_id_counts[html_id] = html_id_counts.get(html_id, 0) + 1
        if isinstance(item.get("test_id"), str) and item["test_id"]:
            with_test_id += 1
        if isinstance(item.get("data_action"), str) and item["data_action"]:
            with_data_action += 1
        if isinstance(item.get("data_state"), str) and item["data_state"]:
            with_data_state += 1
        if item.get("enabled") is False:
            reason = item.get("blocked_reason")
            reason_key = reason if isinstance(reason, str) and reason else "unknown"
            blocked_reasons[reason_key] = blocked_reasons.get(reason_key, 0) + 1
        else:
            enabled_count += 1
    duplicate_cache_keys = sorted(
        key for key, count in cache_key_counts.items() if count > 1
    )
    duplicate_html_ids = sorted(
        key for key, count in html_id_counts.items() if count > 1
    )
    return {
        "fingerprint": get_action_plan_fingerprint(som),
        "enabled_fingerprint": get_action_plan_fingerprint(som, enabled_only=True),
        "total": len(plan),
        "enabled": enabled_count,
        "disabled": len(plan) - enabled_count,
        "with_cache_key": with_cache_key,
        "unique_cache_keys": len(cache_key_counts),
        "duplicate_cache_keys": duplicate_cache_keys,
        "with_html_id": with_html_id,
        "duplicate_html_ids": duplicate_html_ids,
        "with_test_id": with_test_id,
        "with_data_action": with_data_action,
        "with_data_state": with_data_state,
        "by_role": dict(sorted(by_role.items())),
        "blocked_reasons": dict(sorted(blocked_reasons.items())),
    }


def find_action_target_by_id(
    som: Som, target_id: str, *, enabled_only: bool = False
) -> Optional[Dict[str, object]]:
    """Return the compact action target matching a SOM element id."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    for item in plan:
        if item.get("id") == target_id:
            return item
    return None


def find_action_target_by_html_id(
    som: Som, html_id: str, *, enabled_only: bool = False
) -> Optional[Dict[str, object]]:
    """Return the compact action target matching an original HTML id."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    for item in plan:
        if item.get("html_id") == html_id:
            return item
    return None


def find_action_target_by_cache_key(
    som: Som, cache_key: str, *, enabled_only: bool = False
) -> Optional[Dict[str, object]]:
    """Return the compact action target matching a deterministic cache key."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    for item in plan:
        if item.get("cache_key") == cache_key:
            return item
    return None


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
