"""SOM query helpers for searching and traversing SOM documents."""

from __future__ import annotations

import json
import math
from typing import List, Optional, Union

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

    Uses a simple heuristic of ~4 bytes per token. For typed SOM objects, this
    uses the compiler-reported `meta.som_bytes`; dicts fall back to JSON length.
    """
    if isinstance(som, Som):
        return math.ceil(som.meta.som_bytes / 4)
    else:
        text = json.dumps(som)
    return math.ceil(len(text) / 4)


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
