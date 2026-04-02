"""Tests for _extract_last_json hardened parser."""

import pytest

from plasmate.client import _extract_last_json


class TestExtractLastJson:
    """Ensure the three-phase parser handles real-world malformed output."""

    def test_clean_object(self):
        assert _extract_last_json('{"title": "Example"}') == {"title": "Example"}

    def test_clean_array(self):
        assert _extract_last_json('[1, 2, 3]') == [1, 2, 3]

    def test_whitespace_padding(self):
        assert _extract_last_json('  \n{"a": 1}\n  ') == {"a": 1}

    def test_progress_line_before_json(self):
        """Plasmate may emit a status line before the JSON payload."""
        text = 'Fetching https://example.com...\n{"title": "Example", "url": "https://example.com"}'
        result = _extract_last_json(text)
        assert result == {"title": "Example", "url": "https://example.com"}

    def test_multiple_progress_lines(self):
        text = "Starting...\nConnecting...\nRendering JS...\n{\"ok\": true}"
        assert _extract_last_json(text) == {"ok": True}

    def test_json_embedded_in_log_line(self):
        """JSON embedded within a log message (no clean line break)."""
        text = 'INFO: result = {"status": "done", "count": 42} [finished]'
        result = _extract_last_json(text)
        assert result == {"status": "done", "count": 42}

    def test_multiple_json_objects_returns_last(self):
        """When multiple JSON objects exist, return the last complete one."""
        text = '{"partial": true}\n{"final": true}'
        result = _extract_last_json(text)
        assert result == {"final": True}

    def test_nested_braces_in_strings(self):
        """Braces inside JSON string values must not confuse the parser."""
        text = '{"code": "if (x) { y }", "valid": true}'
        result = _extract_last_json(text)
        assert result == {"code": "if (x) { y }", "valid": True}

    def test_empty_string(self):
        assert _extract_last_json("") is None

    def test_none_input(self):
        # Defensive: should not crash even if None somehow gets through
        assert _extract_last_json(None) is None  # type: ignore[arg-type]

    def test_no_json_at_all(self):
        assert _extract_last_json("just some plain text") is None

    def test_truncated_json(self):
        """Incomplete JSON should return None, not crash."""
        assert _extract_last_json('{"title": "Example", "url":') is None

    def test_deeply_nested(self):
        text = '{"meta": {"stats": {"html_bytes": 1000, "som_bytes": 50}}}'
        result = _extract_last_json(text)
        assert result["meta"]["stats"]["html_bytes"] == 1000

    def test_trailing_garbage(self):
        """JSON followed by non-JSON text."""
        text = '{"ok": true}\nDone in 0.3s'
        result = _extract_last_json(text)
        assert result == {"ok": True}
