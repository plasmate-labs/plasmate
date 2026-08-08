"""Parse and validate SOM JSON."""

from __future__ import annotations

import json
from typing import Any, Optional, Union

from pydantic import ValidationError

from .types import Som


def parse_som(input: Union[str, dict]) -> Som:
    """Parse a JSON string or dict into a validated Som object.

    Args:
        input: A JSON string or a dictionary conforming to the SOM schema.

    Returns:
        A validated Som instance.

    Raises:
        ValueError: If the input is not valid JSON.
        ValidationError: If the input does not conform to the SOM schema.
    """
    if isinstance(input, str):
        try:
            data = json.loads(input)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON: {e}") from e
    elif isinstance(input, dict):
        data = input
    else:
        raise TypeError(f"Expected str or dict, got {type(input).__name__}")

    return Som.model_validate(data)


def is_valid_som(input: Any) -> bool:
    """Check if input conforms to the SOM schema.

    Args:
        input: A JSON string, dict, or any other value.

    Returns:
        True if the input is valid SOM, False otherwise.
    """
    try:
        parse_som(input)
        return True
    except (ValueError, ValidationError, TypeError):
        return False


def _extract_json_objects(text: str) -> list[Any]:
    """Extract complete JSON objects from mixed Plasmate CLI output."""
    decoder = json.JSONDecoder()
    objects: list[Any] = []
    position = 0
    while position < len(text):
        idx = text.find("{", position)
        if idx == -1:
            break
        try:
            data, end = decoder.raw_decode(text, idx)
        except json.JSONDecodeError:
            position = idx + 1
            continue
        objects.append(data)
        position = end
    return objects


def from_plasmate(json_output: str) -> Som:
    """Parse raw Plasmate CLI JSON output into a Som object.

    Plasmate CLI outputs JSON that may be the SOM directly or wrapped
    in a container object with a ``som`` key. It may also include progress
    lines before or after the JSON payload.

    Args:
        json_output: Raw JSON string from Plasmate CLI.

    Returns:
        A validated Som instance.

    Raises:
        ValueError: If the output cannot be parsed.
    """
    objects = _extract_json_objects(json_output)
    if not objects:
        raise ValueError("No JSON object found in Plasmate output")

    last_error: Optional[ValidationError] = None
    result: Optional[Som] = None
    for data in objects:
        # Handle wrapped output: {"som": {...}}
        if isinstance(data, dict) and "som" in data and "som_version" not in data:
            data = data["som"]

        try:
            result = Som.model_validate(data)
        except ValidationError as exc:
            last_error = exc

    if result is not None:
        return result
    assert last_error is not None
    raise last_error
