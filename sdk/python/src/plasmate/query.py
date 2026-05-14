"""SOM query helpers for searching and traversing SOM documents."""

from __future__ import annotations

import json
from typing import Dict, List, Optional, Union

from .types import Som, SomElement, SomRegion


def find_by_role(som: Som, role: str) -> List[SomRegion]:
    """Return all regions matching the given role."""
    return [r for r in som.regions if r.role == role]


def find_by_id(som: Som, element_id: str) -> Optional[SomElement]:
    """Find a single element by its ID, searching all regions recursively."""
    for region in som.regions:
        for element in region.elements:
            result = _find_element_by_id(element, element_id)
            if result is not None:
                return result
    return None


def find_by_tag(som: Som, tag: str) -> List[SomElement]:
    """Find all elements matching the given element role (tag)."""
    results: List[SomElement] = []
    for region in som.regions:
        for element in region.elements:
            _collect_by_role(element, tag, results)
    return results


def find_interactive(som: Som) -> List[SomElement]:
    """Return all interactive elements (those with actions)."""
    results: List[SomElement] = []
    for region in som.regions:
        for element in region.elements:
            _collect_interactive(element, results)
    return results


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
    encoded = json.dumps(parts, separators=(",", ":"))
    return f"plasmate-action:v1:{_fnv1a32(encoded)}"


def get_action_plan(som: Som) -> List[Dict[str, object]]:
    """Return compact action targets for agent planning."""
    plan: List[Dict[str, object]] = []
    for element in find_interactive(som):
        attrs = element.attrs
        item: Dict[str, object] = {
            "id": element.id,
            "role": element.role.value,
            "actions": element.actions or [],
            "enabled": True,
        }
        label = element.label or element.text
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
            if attrs.name:
                item["name"] = attrs.name
            if attrs.autocomplete:
                item["autocomplete"] = attrs.autocomplete
            if attrs.inputmode:
                item["inputmode"] = attrs.inputmode
            if attrs.enterkeyhint:
                item["enterkeyhint"] = attrs.enterkeyhint
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
            if attrs.accesskey:
                item["accesskey"] = attrs.accesskey
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
            if attrs.pattern:
                item["pattern"] = attrs.pattern
            if attrs.description:
                item["description"] = attrs.description
            if attrs.checked is not None:
                item["checked"] = attrs.checked
            elif attrs.aria and "checked" in attrs.aria:
                item["checked"] = attrs.aria["checked"]
            if attrs.aria:
                for aria_key in (
                    "expanded",
                    "pressed",
                    "selected",
                    "current",
                    "controls",
                    "haspopup",
                    "invalid",
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
                ):
                    if aria_key in attrs.aria:
                        item_key = (
                            "aria_autocomplete"
                            if aria_key == "autocomplete"
                            else aria_key
                        )
                        item[item_key] = attrs.aria[aria_key]
            if attrs.required is not None:
                item["required"] = attrs.required
            if attrs.readonly is not None:
                item["readonly"] = attrs.readonly
            if attrs.disabled is not None:
                item["disabled"] = attrs.disabled
                if attrs.disabled:
                    item["enabled"] = False
                    item["blocked_reason"] = "disabled"
            elif attrs.readonly:
                item["enabled"] = False
                item["blocked_reason"] = "readonly"
            if attrs.group:
                item["group"] = attrs.group
        item["cache_key"] = get_action_plan_cache_key(item)
        plan.append(item)
    return plan


def find_by_text(som: Som, text: str) -> List[SomElement]:
    """Find all elements whose text contains the given substring (case-insensitive)."""
    results: List[SomElement] = []
    lower = text.lower()
    for region in som.regions:
        for element in region.elements:
            _collect_by_text(element, lower, results)
    return results


def flat_elements(som: Som) -> List[SomElement]:
    """Return all elements flattened into a single list."""
    results: List[SomElement] = []
    for region in som.regions:
        for element in region.elements:
            _flatten(element, results)
    return results


def get_token_estimate(som: Union[Som, dict]) -> int:
    """Estimate the token count of a SOM document.

    Uses a simple heuristic of ~4 characters per token on the JSON representation.
    """
    if isinstance(som, Som):
        text = som.model_dump_json()
    else:
        text = json.dumps(som)
    return len(text) // 4


# ---- Internal helpers ----


def _find_element_by_id(element: SomElement, element_id: str) -> Optional[SomElement]:
    if element.id == element_id:
        return element
    if element.children:
        for child in element.children:
            result = _find_element_by_id(child, element_id)
            if result is not None:
                return result
    if element.shadow:
        for child in element.shadow.elements:
            result = _find_element_by_id(child, element_id)
            if result is not None:
                return result
    return None


def _collect_by_role(element: SomElement, role: str, results: List[SomElement]) -> None:
    if element.role == role:
        results.append(element)
    if element.children:
        for child in element.children:
            _collect_by_role(child, role, results)
    if element.shadow:
        for child in element.shadow.elements:
            _collect_by_role(child, role, results)


def _collect_interactive(element: SomElement, results: List[SomElement]) -> None:
    if element.actions:
        results.append(element)
    if element.children:
        for child in element.children:
            _collect_interactive(child, results)
    if element.shadow:
        for child in element.shadow.elements:
            _collect_interactive(child, results)


def _collect_by_text(element: SomElement, lower_text: str, results: List[SomElement]) -> None:
    if element.text and lower_text in element.text.lower():
        results.append(element)
    if element.children:
        for child in element.children:
            _collect_by_text(child, lower_text, results)
    if element.shadow:
        for child in element.shadow.elements:
            _collect_by_text(child, lower_text, results)


def _flatten(element: SomElement, results: List[SomElement]) -> None:
    results.append(element)
    if element.children:
        for child in element.children:
            _flatten(child, results)
    if element.shadow:
        for child in element.shadow.elements:
            _flatten(child, results)
