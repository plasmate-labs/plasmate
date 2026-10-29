"""SOM query helpers for searching and traversing SOM documents."""

from __future__ import annotations

import json
from typing import Dict, List, Literal, Optional, Union

from .types import ElementRole, Som, SomElement, SomRegion


_MEDIA_TAG_ALIASES = {
    "audio": "audio",
    "audios": "audio",
    "audio_item": "audio",
    "audio_items": "audio",
    "canvas": "canvas",
    "canvases": "canvas",
    "canvas_item": "canvas",
    "canvas_items": "canvas",
    "video": "video",
    "videos": "video",
    "video_item": "video",
    "video_items": "video",
}
_MEDIA_GROUP_ALIASES = {"media", "media_item", "mediaitem", "media_items"}


def find_by_role(som: Som, role: str) -> List[SomRegion]:
    """Return all regions matching a canonical role or selector alias."""
    normalized_role = {
        "main": "main",
        "main_content": "main",
        "main_contents": "main",
        "main_region": "main",
        "main_regions": "main",
        "maincontent": "main",
        "maincontents": "main",
        "nav": "navigation",
        "navs": "navigation",
        "navigation": "navigation",
        "navigations": "navigation",
        "nav_region": "navigation",
        "nav_regions": "navigation",
        "navigation_region": "navigation",
        "navigation_regions": "navigation",
        "search": "navigation",
        "searches": "navigation",
        "search_nav": "navigation",
        "search_navigation": "navigation",
        "search_navigations": "navigation",
        "aside": "aside",
        "asides": "aside",
        "sidebar": "aside",
        "sidebars": "aside",
        "complementary": "aside",
        "header": "header",
        "headers": "header",
        "banner": "header",
        "footer": "footer",
        "footers": "footer",
        "contentinfo": "footer",
        "content_info": "footer",
        "form": "form",
        "forms": "form",
        "dialog": "dialog",
        "dialogs": "dialog",
        "content": "content",
        "contents": "content",
        "article": "content",
        "articles": "content",
        "region": "content",
        "regions": "content",
        "content_region": "content",
        "content_regions": "content",
    }
    normalized_role = normalized_role.get(
        _normalize_role_alias(role), _normalize_role_alias(role)
    )
    return [r for r in som.regions if r.role == normalized_role]


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
    """Find elements by accessible label or its compiled attribute fallbacks."""
    results: List[SomElement] = []
    normalized_label = _normalize_label_text(label)
    if not normalized_label:
        return results
    folded_label = normalized_label.lower()
    for element in flat_elements(som):
        labels = _label_candidates(element)
        if (exact and any(normalized_label == _normalize_label_text(value) for value in labels)) or (
            not exact
            and any(
                folded_label in _normalize_label_text(value).lower() for value in labels
            )
        ):
            results.append(element)
    return results


def find_by_tag(som: Som, tag: str) -> List[SomElement]:
    """Find all elements matching an element role or common HTML tag alias."""
    role_aliases = {
        "a": "link",
        "buttons": "button",
        "article": "section",
        "button": "button",
        "menuitem": "button",
        "menuitems": "button",
        "option": "button",
        "options": "button",
        "tab": "button",
        "tabs": "button",
        "treeitem": "button",
        "treeitems": "button",
        "checkboxes": "checkbox",
        "check_box": "checkbox",
        "check_boxes": "checkbox",
        "comboboxes": "select",
        "combo_boxes": "select",
        "div": "group",
        "fieldset": "group",
        "fieldsets": "group",
        "figure": "group",
        "figures": "group",
        "hr": "separator",
        "img": "image",
        "images": "image",
        "iframe": "iframe",
        "iframes": "iframe",
        "input": "text_input",
        "inputs": "text_input",
        "search": "text_input",
        "search_control": "text_input",
        "search_controls": "text_input",
        "search_field": "text_input",
        "search_fields": "text_input",
        "search_input": "text_input",
        "search_inputs": "text_input",
        "search_box": "text_input",
        "search_boxes": "text_input",
        "query": "text_input",
        "queries": "text_input",
        "query_box": "text_input",
        "query_boxes": "text_input",
        "query_control": "text_input",
        "query_controls": "text_input",
        "query_field": "text_input",
        "query_fields": "text_input",
        "query_input": "text_input",
        "query_inputs": "text_input",
        "ol": "list",
        "dl": "list",
        "menu": "list",
        "blockquote": "paragraph",
        "blockquotes": "paragraph",
        "quote": "paragraph",
        "quotes": "paragraph",
        "menu_item": "button",
        "menuitem": "button",
        "menu_item_checkbox": "checkbox",
        "menu_item_checkboxes": "checkbox",
        "menuitemcheckbox": "checkbox",
        "menuitemcheckboxes": "checkbox",
        "menu_item_radio": "radio",
        "menu_item_radios": "radio",
        "menuitemradio": "radio",
        "menuitemradios": "radio",
        "p": "paragraph",
        "summary": "details",
        "detail": "details",
        "summaries": "details",
        "textarea": "textarea",
        "textareas": "textarea",
        "text_box": "text_input",
        "text_boxes": "text_input",
        "textbox": "text_input",
        "textboxes": "text_input",
        "text_inputs": "text_input",
        "searchbox": "text_input",
        "searchboxes": "text_input",
        "slider": "text_input",
        "sliders": "text_input",
        "spinbutton": "text_input",
        "spinbuttons": "text_input",
        "spin_button": "text_input",
        "spin_buttons": "text_input",
        "combobox": "select",
        "combo_box": "select",
        "combo_boxes": "select",
        "listbox": "select",
        "list_box": "select",
        "list_boxes": "select",
        "select": "select",
        "selects": "select",
        "switch": "checkbox",
        "tab": "button",
        "ul": "list",
        "unordered_list": "list",
        "ordered_list": "list",
        "definition_list": "list",
        "links": "link",
        "listboxes": "select",
        "radios": "radio",
        "radio_button": "radio",
        "radio_buttons": "radio",
        "radiobutton": "radio",
        "radiobuttons": "radio",
        "radiogroup": "group",
        "radiogroups": "group",
        "progress": "group",
        "progressbar": "group",
        "progressbars": "group",
        "meter": "group",
        "meters": "group",
        "output": "group",
        "outputs": "group",
        "tables": "table",
        "data_table": "table",
        "data_tables": "table",
        "data_grid": "table",
        "data_grids": "table",
        "grid": "table",
        "grids": "table",
        "table_grid": "table",
        "table_grids": "table",
        "treegrid": "table",
        "treegrids": "table",
        "tree_grids": "table",
        "tree_grid": "table",
    }
    tag = _normalize_role_alias(tag)
    heading_level = None
    if tag.startswith("heading"):
        suffix = tag.removeprefix("heading").lstrip("-_")
        if suffix in "123456":
            heading_level = int(suffix)
    elif tag.startswith("h"):
        suffix = tag[1:].lstrip("_")
        if suffix in "123456":
            heading_level = int(suffix)
    media_source_role = _MEDIA_TAG_ALIASES.get(tag)
    if media_source_role is not None:
        return [
            element
            for element in flat_elements(som)
            if element.attrs is not None
            and element.attrs.source_role == media_source_role
        ]
    if tag in _MEDIA_GROUP_ALIASES:
        return [element for element in flat_elements(som) if _is_media_element(element)]
    tag = role_aliases.get(tag, tag)
    results: List[SomElement] = []
    for region in som.regions:
        for element in region.elements:
            _collect_by_tag(element, tag, heading_level, results)
    return results


def _is_media_element(element: SomElement) -> bool:
    return element.role == ElementRole.image or (
        element.attrs is not None
        and element.attrs.source_role in {"audio", "video", "canvas"}
    )


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
    label = item.get("label")
    normalized_label = _normalize_label_text(label) if isinstance(label, str) else None
    parts = [
        _compact_string(item.get("id")),
        _compact_string(item.get("role")),
        normalized_label,
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
        if not _normalize_label_text(label or ""):
            label = element.text
        if not _normalize_label_text(label or ""):
            candidates = _label_candidates(element)
            label = next(
                (
                    candidate
                    for candidate in candidates
                    if _normalize_label_text(candidate)
                ),
                None,
            )
        if label and _normalize_label_text(label):
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
) -> Dict[str, Dict[str, object]]:
    """Return action targets indexed for replay and grouped by role/action."""
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    index: Dict[str, Dict[str, object]] = {
        "by_id": {},
        "by_cache_key": {},
        "by_html_id": {},
        "by_test_id": {},
        "by_label": {},
        "by_role": {},
        "by_action": {},
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
            if bucket_key == "by_label" and isinstance(value, str):
                value = _normalize_label_text(value)
            if isinstance(value, str) and value not in index[bucket_key]:
                index[bucket_key][value] = item
        role = item.get("role")
        if isinstance(role, str):
            index["by_role"].setdefault(role, []).append(item)
        actions = item.get("actions")
        if isinstance(actions, list):
            for action in actions:
                if isinstance(action, str):
                    index["by_action"].setdefault(action, []).append(item)
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

    ``by="auto"`` checks SOM id, deterministic cache key, HTML id, test id, then
    a normalized accessible label.
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
        normalized_value = _normalize_label_text(value)
        if not normalized_value:
            return None
        found = index["by_label"].get(normalized_value)
        if found is not None:
            return found
        folded_value = normalized_value.lower()
        plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
        return next(
            (
                item
                for item in plan
                if isinstance(item.get("label"), str)
                and _normalize_label_text(item["label"]).lower() == folded_value
            ),
            None,
        )
    if by not in buckets:
        raise ValueError("by must be one of: auto, id, cache_key, html_id, test_id, label")
    if by != "label":
        return index[buckets[by]].get(value)
    normalized_value = _normalize_label_text(value)
    if not normalized_value:
        return None
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    return next(
        (
            item
            for item in plan
            if isinstance(item.get("label"), str)
            and _normalize_label_text(item["label"]) == normalized_value
        ),
        None,
    )


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
    normalized_label = _normalize_label_text(label)
    if not normalized_label:
        return results
    folded_label = normalized_label.lower()
    for item in plan:
        item_label = item.get("label")
        if not isinstance(item_label, str):
            continue
        if exact:
            if normalized_label == _normalize_label_text(item_label):
                results.append(item)
        elif folded_label in _normalize_label_text(item_label).lower():
            results.append(item)
    return results


def find_action_targets_by_role(
    som: Som,
    role: str,
    *,
    enabled_only: bool = False,
) -> List[Dict[str, object]]:
    """Return compact action targets whose SOM role matches the requested role.

    Accept the same common HTML/ARIA role aliases as ``find_by_tag`` so callers
    can use ``a``/``button``/``input`` and related spellings consistently.
    """
    index = get_action_plan_index(som, enabled_only=enabled_only)
    normalized_alias = _normalize_role_alias(role)
    media_source_role = _MEDIA_TAG_ALIASES.get(normalized_alias)
    plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
    if media_source_role is not None:
        return [
            item
            for item in plan
            if item.get("source_role") == media_source_role
        ]
    if normalized_alias in _MEDIA_GROUP_ALIASES:
        return [
            item
            for item in plan
            if item.get("role") == ElementRole.image.value
            or item.get("source_role") in {"audio", "video", "canvas"}
        ]
    normalized_role = {
        "a": "link",
        "article": "section",
        "button": "button",
        "buttons": "button",
        "menuitem": "button",
        "menuitems": "button",
        "checkboxes": "checkbox",
        "check_box": "checkbox",
        "check_boxes": "checkbox",
        "combobox": "select",
        "combo_box": "select",
        "comboboxes": "select",
        "combo_boxes": "select",
        "definition_list": "list",
        "div": "group",
        "fieldset": "group",
        "fieldsets": "group",
        "figure": "group",
        "figures": "group",
        "hr": "separator",
        "img": "image",
        "images": "image",
        "iframe": "iframe",
        "iframes": "iframe",
        "input": "text_input",
        "inputs": "text_input",
        "listbox": "select",
        "listboxes": "select",
        "list_box": "select",
        "list_boxes": "select",
        "links": "link",
        "menu": "list",
        "menu_item": "button",
        "menuitem": "button",
        "menu_item_checkbox": "checkbox",
        "menu_item_checkboxes": "checkbox",
        "menuitemcheckbox": "checkbox",
        "menuitemcheckboxes": "checkbox",
        "menu_item_radio": "radio",
        "menu_item_radios": "radio",
        "menuitemradio": "radio",
        "menuitemradios": "radio",
        "ol": "list",
        "option": "button",
        "options": "button",
        "ordered_list": "list",
        "p": "paragraph",
        "blockquote": "paragraph",
        "blockquotes": "paragraph",
        "quote": "paragraph",
        "quotes": "paragraph",
        "radio": "radio",
        "radios": "radio",
        "radio_button": "radio",
        "radio_buttons": "radio",
        "radiobutton": "radio",
        "radiobuttons": "radio",
        "radiogroup": "group",
        "radiogroups": "group",
        "progress": "group",
        "progressbar": "group",
        "progressbars": "group",
        "meter": "group",
        "meters": "group",
        "output": "group",
        "outputs": "group",
        "search": "text_input",
        "search_box": "text_input",
        "search_boxes": "text_input",
        "search_control": "text_input",
        "search_controls": "text_input",
        "search_field": "text_input",
        "search_fields": "text_input",
        "search_input": "text_input",
        "search_inputs": "text_input",
        "slider": "text_input",
        "sliders": "text_input",
        "spinbutton": "text_input",
        "spinbuttons": "text_input",
        "spin_button": "text_input",
        "spin_buttons": "text_input",
        "query": "text_input",
        "queries": "text_input",
        "query_box": "text_input",
        "query_boxes": "text_input",
        "query_control": "text_input",
        "query_controls": "text_input",
        "query_field": "text_input",
        "query_fields": "text_input",
        "query_input": "text_input",
        "query_inputs": "text_input",
        "select": "select",
        "selects": "select",
        "summary": "details",
        "detail": "details",
        "summaries": "details",
        "switch": "checkbox",
        "tab": "button",
        "tabs": "button",
        "textarea": "textarea",
        "textareas": "textarea",
        "text_box": "text_input",
        "text_boxes": "text_input",
        "textbox": "text_input",
        "textboxes": "text_input",
        "tables": "table",
        "data_table": "table",
        "data_tables": "table",
        "data_grid": "table",
        "data_grids": "table",
        "grid": "table",
        "grids": "table",
        "table_grid": "table",
        "table_grids": "table",
        "treegrid": "table",
        "treegrids": "table",
        "tree_grids": "table",
        "tree_grid": "table",
        "treeitem": "button",
        "treeitems": "button",
        "text_inputs": "text_input",
        "searchbox": "text_input",
        "searchboxes": "text_input",
        "ul": "list",
        "list_box": "select",
        "list_boxes": "select",
        "unordered_list": "list",
    }.get(normalized_alias, normalized_alias)
    results = index["by_role"].get(normalized_role, [])
    return list(results) if isinstance(results, list) else []


def find_action_targets_by_action(
    som: Som,
    action: str,
    *,
    enabled_only: bool = False,
) -> List[Dict[str, object]]:
    """Return compact action targets that expose the requested action.

    Accept the same interaction aliases as the MCP/CLI selector contract.
    Compiled SOM actions remain canonical (``type`` and ``select``), so the
    aliases are normalized only for this lookup.
    """
    index = get_action_plan_index(som, enabled_only=enabled_only)
    normalized_action = " ".join(action.strip().lower().split())
    if (
        len(normalized_action) >= 2
        and normalized_action[0] == normalized_action[-1]
        and normalized_action[0] in "\"'"
    ):
        normalized_action = normalized_action[1:-1].strip()
    if normalized_action.startswith("action"):
        separator = normalized_action[len("action") :].lstrip()
        if separator.startswith((":", "=")):
            normalized_action = " ".join(separator[1:].strip().split())
    normalized_action = {
        "clear_text": "clear",
        "clear-text": "clear",
        "clear text": "clear",
        "type_text": "type",
        "type-text": "type",
        "type text": "type",
        "select_option": "select",
        "select-option": "select",
        "select option": "select",
    }.get(normalized_action, normalized_action)
    results = list(index["by_action"].get(normalized_action, []))
    if normalized_action in {"submit", "reset"}:
        seen_ids = {item.get("id") for item in results}
        plan = get_enabled_action_plan(som) if enabled_only else get_action_plan(som)
        for item in plan:
            button_type = item.get("button_type")
            matches_submitter = isinstance(button_type, str) and (
                button_type.lower() == normalized_action
                or (normalized_action == "submit" and button_type.lower() == "image")
            )
            if matches_submitter and item.get("id") not in seen_ids:
                results.append(item)
                seen_ids.add(item.get("id"))
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
    matches = find_action_targets_by_label(
        som, label, exact=True, enabled_only=enabled_only
    )
    return matches[0] if matches else None


def find_by_text(
    som: Som, text: str, *, exact: bool = False
) -> List[SomElement]:
    """Find elements by visible or compiled text metadata.

    Matching is case-insensitive substring search by default. Set ``exact`` to
    require a case-sensitive match after whitespace normalization.
    """
    results: List[SomElement] = []
    normalized_text = _normalize_label_text(text)
    if not normalized_text:
        return results
    for region in som.regions:
        for element in region.elements:
            _collect_by_text(element, normalized_text, exact, results)
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

    Uses the canonical ``meta.som_bytes`` count with a ~4 bytes-per-token
    heuristic. Dicts without SOM metadata keep the legacy JSON-size fallback.
    """
    if isinstance(som, Som):
        som_bytes = som.meta.som_bytes
    else:
        meta = som.get("meta")
        som_bytes = meta.get("som_bytes") if isinstance(meta, dict) else None
        if (
            not isinstance(som_bytes, int)
            or isinstance(som_bytes, bool)
            or som_bytes < 0
        ):
            return (len(json.dumps(som)) + 3) // 4
    return (som_bytes + 3) // 4


def get_compression_ratio(som: Union[Som, dict]) -> float:
    """Return source HTML bytes divided by serialized SOM bytes.

    Empty SOMs return ``inf``. Dict inputs must include non-negative integer
    ``html_bytes`` and ``som_bytes`` metadata values.
    """
    if isinstance(som, Som):
        html_bytes = som.meta.html_bytes
        som_bytes = som.meta.som_bytes
    else:
        meta = som.get("meta")
        html_bytes = meta.get("html_bytes") if isinstance(meta, dict) else None
        som_bytes = meta.get("som_bytes") if isinstance(meta, dict) else None
        if any(
            not isinstance(value, int) or isinstance(value, bool) or value < 0
            for value in (html_bytes, som_bytes)
        ):
            raise ValueError(
                "SOM metadata must include non-negative html_bytes and som_bytes"
            )
    if som_bytes == 0:
        return float("inf")
    return html_bytes / som_bytes


# ---- Internal helpers ----


def _normalize_role_alias(value: str) -> str:
    """Normalize role aliases so hyphenated and spaced forms behave alike."""
    normalized = " ".join(value.strip().lower().split())
    if len(normalized) >= 2 and normalized[0] == normalized[-1] and normalized[0] in "\"'":
        normalized = normalized[1:-1].strip()
    if normalized.startswith("role"):
        remainder = normalized[4:].lstrip()
        if remainder.startswith((":", "=")):
            normalized = remainder[1:].strip()
    if len(normalized) >= 2 and normalized[0] == normalized[-1] and normalized[0] in "\"'":
        normalized = normalized[1:-1].strip()
    return "_".join(normalized.replace("-", " ").split())


def _normalize_label_text(value: str) -> str:
    """Collapse label whitespace while preserving case for exact matches."""
    return " ".join(
        value.replace("\u200b", " ").replace("\u2060", " ").replace("\ufeff", " ").split()
    )


def _label_candidates(element: SomElement) -> List[str]:
    """Return accessible labels and compiled attribute fallbacks."""
    labels = [element.label] if element.label else []
    if element.attrs:
        aria_label = (element.attrs.aria or {}).get("label")
        labels.extend(
            value
            for value in (
                aria_label,
                element.attrs.title,
                element.attrs.placeholder,
                element.attrs.alt,
            )
            if isinstance(value, str) and value
        )
    return labels


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


def _collect_by_tag(
    element: SomElement,
    role: str,
    heading_level: Optional[int],
    results: List[SomElement],
) -> None:
    if heading_level is not None:
        if element.role == "heading" and element.attrs and element.attrs.level == heading_level:
            results.append(element)
    elif element.role == role:
        results.append(element)
    if element.children:
        for child in element.children:
            _collect_by_tag(child, role, heading_level, results)
    if element.shadow:
        for child in element.shadow.elements:
            _collect_by_tag(child, role, heading_level, results)


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
    normalized_text: str,
    exact: bool,
    results: List[SomElement],
) -> None:
    parts = []
    if element.text:
        parts.append(element.text)
    if element.label:
        parts.append(element.label)
    if element.attrs and element.attrs.description:
        parts.append(element.attrs.description)
    if element.attrs:
        parts.extend(
            value
            for value in (
                element.attrs.alt,
                element.attrs.placeholder,
                element.attrs.title,
                element.attrs.value,
                element.attrs.summary,
                element.attrs.legend,
                element.attrs.section_label,
            )
            if value
        )
        for key in ("valuetext", "roledescription"):
            value = (element.attrs.aria or {}).get(key)
            if isinstance(value, str) and value:
                parts.append(value)
    if element.attrs and element.attrs.items:
        parts.extend(item.text for item in element.attrs.items if item.text)
        parts.extend(item.term for item in element.attrs.items if item.term)
        parts.extend(
            item.description for item in element.attrs.items if item.description
        )
    if element.attrs and element.attrs.options:
        parts.extend(option.text for option in element.attrs.options if option.text)
    if element.attrs and element.attrs.caption:
        parts.append(element.attrs.caption)
    if element.attrs and element.attrs.headers:
        parts.extend(header for header in element.attrs.headers if header)
    if element.attrs and element.attrs.rows:
        for row in element.attrs.rows:
            parts.extend(cell for cell in row if cell)
    if any(
        (
            _normalize_label_text(part) == normalized_text
            if exact
            else normalized_text.lower() in _normalize_label_text(part).lower()
        )
        for part in parts
    ):
        results.append(element)
    if element.children:
        for child in element.children:
            _collect_by_text(child, normalized_text, exact, results)
    if element.shadow:
        for child in element.shadow.elements:
            _collect_by_text(child, normalized_text, exact, results)


def _flatten(element: SomElement, results: List[SomElement]) -> None:
    results.append(element)
    if element.children:
        for child in element.children:
            _flatten(child, results)
    if element.shadow:
        for child in element.shadow.elements:
            _flatten(child, results)
