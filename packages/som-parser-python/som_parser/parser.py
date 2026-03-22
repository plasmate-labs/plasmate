"""Parse and validate SOM JSON."""

from __future__ import annotations

import json
from typing import Any, Union

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


def from_plasmate(json_output: str) -> Som:
    """Parse raw Plasmate CLI JSON output into a Som object.

    Plasmate CLI outputs JSON that may be the SOM directly or wrapped
    in a container object with a ``som`` key.

    Args:
        json_output: Raw JSON string from Plasmate CLI.

    Returns:
        A validated Som instance.

    Raises:
        ValueError: If the output cannot be parsed.
    """
    try:
        data = json.loads(json_output)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON from Plasmate: {e}") from e

    # Handle wrapped output: {"som": {...}}
    if isinstance(data, dict) and "som" in data and "som_version" not in data:
        data = data["som"]

    return Som.model_validate(data)
