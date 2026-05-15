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


def find_by_html_id(som: Som, html_id: str) -> Optional[SomElement]:
    """Find a single element by its original HTML id."""
    for element in flat_elements(som):
        if element.html_id == html_id:
            return element
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
        if element.html_id:
            item["html_id"] = element.html_id
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
        if element.id in form_context_by_id:
            _copy_form_context(item, form_context_by_id[element.id])
        item["cache_key"] = get_action_plan_cache_key(item)
        plan.append(item)
    return plan


def get_enabled_action_plan(som: Som) -> List[Dict[str, object]]:
    """Return compact action targets that are currently available."""
    return [item for item in get_action_plan(som) if item.get("enabled") is not False]


def get_action_plan_index(
    som: Som, *, enabled_only: bool = False
) -> Dict[str, Dict[str, Dict[str, object]]]:
    """Return action targets indexed by SOM id, cache key, and original HTML id."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    index: Dict[str, Dict[str, Dict[str, object]]] = {
        "by_id": {},
        "by_cache_key": {},
        "by_html_id": {},
    }
    for item in plan:
        target_id = item.get("id")
        if isinstance(target_id, str) and target_id not in index["by_id"]:
            index["by_id"][target_id] = item
        cache_key = item.get("cache_key")
        if isinstance(cache_key, str) and cache_key not in index["by_cache_key"]:
            index["by_cache_key"][cache_key] = item
        html_id = item.get("html_id")
        if isinstance(html_id, str) and html_id not in index["by_html_id"]:
            index["by_html_id"][html_id] = item
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
    enabled_count = 0
    with_cache_key = 0
    with_html_id = 0
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
        if item.get("enabled") is False:
            reason = item.get("blocked_reason")
            reason_key = reason if isinstance(reason, str) and reason else "unknown"
            blocked_reasons[reason_key] = blocked_reasons.get(reason_key, 0) + 1
        else:
            enabled_count += 1
    duplicate_cache_keys = sorted(
        key for key, count in cache_key_counts.items() if count > 1
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


def find_by_text(som: Som, text: str, *, exact: bool = False) -> List[SomElement]:
    """Find all elements whose text or label matches the given text.

    Exact matches are case-sensitive. Substring matches are case-insensitive.
    """
    results: List[SomElement] = []
    lower = text.lower()
    for region in som.regions:
        for element in region.elements:
            _collect_by_text(element, text, lower, exact, results)
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


def _collect_by_text(
    element: SomElement,
    text: str,
    lower_text: str,
    exact: bool,
    results: List[SomElement],
) -> None:
    values = [value for value in (element.text, element.label) if value]
    if exact:
        matched = any(value == text for value in values)
    else:
        matched = any(lower_text in value.lower() for value in values)
    if matched:
        results.append(element)
    if element.children:
        for child in element.children:
            _collect_by_text(child, text, lower_text, exact, results)
    if element.shadow:
        for child in element.shadow.elements:
            _collect_by_text(child, text, lower_text, exact, results)


def _flatten(element: SomElement, results: List[SomElement]) -> None:
    results.append(element)
    if element.children:
        for child in element.children:
            _flatten(child, results)
    if element.shadow:
        for child in element.shadow.elements:
            _flatten(child, results)
