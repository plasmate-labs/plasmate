"""SOM query helpers for searching and traversing SOM documents."""

from __future__ import annotations

import json
from typing import Dict, List, Literal, Optional, Union

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


def find_by_html_id(som: Som, html_id: str) -> Optional[SomElement]:
    """Find a single element by its original HTML id."""
    for element in flat_elements(som):
        if element.html_id == html_id:
            return element
    return None


def find_by_label(som: Som, label: str, *, exact: bool = False) -> List[SomElement]:
    """Find elements by accessible label."""
    results: List[SomElement] = []
    for element in flat_elements(som):
        element_label = element.label or ""
        if exact:
            if label == element_label:
                results.append(element)
        elif label.lower() in element_label.lower():
            results.append(element)
    return results


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
        region_elements: List[SomElement] = []
        for root in region.elements:
            _flatten(root, region_elements)
        for el in region_elements:
            if el.actions:
                form_context_by_id[el.id] = region
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
        if element.html_id:
            item["html_id"] = element.html_id
        if attrs:
            if attrs.href:
                item["href"] = attrs.href
            if attrs.target:
                item["target"] = attrs.target
            if attrs.rel:
                item["rel"] = attrs.rel
            if attrs.hreflang:
                item["hreflang"] = attrs.hreflang
            if attrs.type:
                item["type"] = attrs.type
            if attrs.referrerpolicy:
                item["referrerpolicy"] = attrs.referrerpolicy
            if attrs.download is not None:
                item["download"] = attrs.download
            if attrs.name:
                item["name"] = attrs.name
            if attrs.accept:
                item["accept"] = attrs.accept
            if attrs.capture is not None:
                item["capture"] = attrs.capture
            if attrs.multiple is not None:
                item["multiple"] = attrs.multiple
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
            if attrs.source_role:
                item["source_role"] = attrs.source_role
            if attrs.test_id:
                item["test_id"] = attrs.test_id
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
                    "label",
                    "labelledby",
                    "describedby",
                ):
                    if aria_key in attrs.aria:
                        item_key = (
                            "aria_autocomplete"
                            if aria_key == "autocomplete"
                            else "aria_placeholder"
                            if aria_key == "placeholder"
                            else "aria_label"
                            if aria_key == "label"
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
        if element.id in form_context_by_id:
            _copy_form_context(item, form_context_by_id[element.id])
        item["cache_key"] = get_action_plan_cache_key(item)
        plan.append(item)
    return plan


def get_enabled_action_plan(som: Som) -> List[Dict[str, object]]:
    """Return compact action targets that are currently safe to offer."""
    return [item for item in get_action_plan(som) if item.get("enabled") is not False]


def get_action_plan_index(
    som: Som, *, enabled_only: bool = False
) -> Dict[str, Dict[str, Dict[str, object]]]:
    """Return action targets indexed by id, cache key, HTML id, test id, and label."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    index: Dict[str, Dict[str, Dict[str, object]]] = {
        "by_id": {},
        "by_cache_key": {},
        "by_html_id": {},
        "by_test_id": {},
        "by_label": {},
    }
    for item in plan:
        for source_key, bucket_key in (
            ("id", "by_id"),
            ("cache_key", "by_cache_key"),
            ("html_id", "by_html_id"),
            ("test_id", "by_test_id"),
            ("label", "by_label"),
        ):
            value = item.get(source_key)
            if isinstance(value, str) and value not in index[bucket_key]:
                index[bucket_key][value] = item
    return index


ActionTargetLookupKey = Literal["auto", "id", "cache_key", "html_id", "test_id", "label"]


def find_action_target(
    som: Som,
    value: str,
    *,
    by: ActionTargetLookupKey = "auto",
    enabled_only: bool = False,
) -> Optional[Dict[str, object]]:
    """Return the compact action target matching a replay id.

    ``by="auto"`` checks SOM id, deterministic cache key, HTML id, then test id.
    Use ``by="label"`` only when the label is unique enough for the page.
    """
    index = get_action_plan_index(som, enabled_only=enabled_only)
    buckets = {
        "id": "by_id",
        "cache_key": "by_cache_key",
        "html_id": "by_html_id",
        "test_id": "by_test_id",
        "label": "by_label",
    }
    if by == "auto":
        for bucket in ("by_id", "by_cache_key", "by_html_id", "by_test_id"):
            found = index[bucket].get(value)
            if found is not None:
                return found
        return None
    if by not in buckets:
        raise ValueError("by must be one of: auto, id, cache_key, html_id, test_id, label")
    return index[buckets[by]].get(value)


def find_action_targets_by_label(
    som: Som,
    label: str,
    *,
    exact: bool = False,
    enabled_only: bool = False,
) -> List[Dict[str, object]]:
    """Return compact action targets whose accessible label matches text."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    results: List[Dict[str, object]] = []
    for item in plan:
        item_label = item.get("label")
        if not isinstance(item_label, str):
            continue
        if exact:
            if label == item_label:
                results.append(item)
        elif label.lower() in item_label.lower():
            results.append(item)
    return results


def find_action_target_by_id(
    som: Som, target_id: str, *, enabled_only: bool = False
) -> Optional[Dict[str, object]]:
    """Return the compact action target matching a SOM element id."""
    return find_action_target(som, target_id, by="id", enabled_only=enabled_only)


def find_action_target_by_cache_key(
    som: Som, cache_key: str, *, enabled_only: bool = False
) -> Optional[Dict[str, object]]:
    """Return the compact action target matching a deterministic cache key."""
    return find_action_target(som, cache_key, by="cache_key", enabled_only=enabled_only)


def find_action_target_by_html_id(
    som: Som, html_id: str, *, enabled_only: bool = False
) -> Optional[Dict[str, object]]:
    """Return the compact action target matching an original HTML id."""
    return find_action_target(som, html_id, by="html_id", enabled_only=enabled_only)


def find_action_target_by_test_id(
    som: Som, test_id: str, *, enabled_only: bool = False
) -> Optional[Dict[str, object]]:
    """Return the compact action target matching a test locator attribute."""
    return find_action_target(som, test_id, by="test_id", enabled_only=enabled_only)


def find_action_target_by_label(
    som: Som, label: str, *, enabled_only: bool = False
) -> Optional[Dict[str, object]]:
    """Return the first compact action target matching an exact accessible label."""
    return find_action_target(som, label, by="label", enabled_only=enabled_only)


def find_by_text(som: Som, text: str) -> List[SomElement]:
    """Find all elements whose text or label contains the substring."""
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
    if (element.text and lower_text in element.text.lower()) or (
        element.label and lower_text in element.label.lower()
    ):
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
