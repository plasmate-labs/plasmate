"""Plasmate SOM extractor for Browser Use.

Provides SOM-based content extraction that can replace or complement
Browser Use's default DOM serialization, reducing token costs by 90%+.
"""

import asyncio
import json
import subprocess
from typing import Any, Optional

from som_parser import parse_som, get_action_plan, get_links, get_text, to_markdown


def _extract_last_json(text: str) -> Any:
    """Extract the last complete JSON object from potentially mixed output.

    Handles cases where Plasmate emits progress/log lines alongside the
    JSON payload.  Returns None if no valid JSON object is found.
    """
    if not text:
        return None

    stripped = text.strip()

    # Fast path: clean output
    try:
        return json.loads(stripped)
    except (json.JSONDecodeError, ValueError):
        pass

    # Line scan: JSON on its own line (progress line before payload)
    for line in reversed(stripped.splitlines()):
        line = line.strip()
        if line.startswith(("{", "[")):
            try:
                return json.loads(line)
            except (json.JSONDecodeError, ValueError):
                pass

    # Brace walk: JSON embedded in a longer string
    decoder = json.JSONDecoder()
    for pos in reversed([i for i, ch in enumerate(stripped) if ch == "{"]):
        try:
            value, _ = decoder.raw_decode(stripped, pos)
            return value
        except (json.JSONDecodeError, ValueError):
            continue

    return None


def _format_action_plan_item(item: dict[str, object]) -> str:
    """Render one compact action target for LLM page context."""
    actions = ", ".join(str(action) for action in item.get("actions", []))
    role = str(item.get("role", ""))
    element_id = str(item.get("id", ""))
    label = str(
        item.get("label")
        or item.get("placeholder")
        or item.get("name")
        or item.get("href")
        or ""
    )

    parts = [f'  [{element_id}] {role} "{label}"']
    if actions:
        parts.append(f"({actions})")

    flags: list[str] = []
    if item.get("enabled") is False:
        reason = item.get("blocked_reason")
        flags.append(str(reason or "blocked"))
        if reason:
            flags.append(f"blocked_reason={reason}")
    elif item.get("enabled") is True:
        flags.append("enabled")
    if item.get("cache_key"):
        flags.append(f"cache_key={item['cache_key']}")
    if item.get("required") is True:
        flags.append("required")
    if item.get("readonly") is True:
        flags.append("readonly")
    if item.get("inert") is True:
        flags.append("inert")
    if item.get("group"):
        flags.append(f"group={item['group']}")
    if item.get("target"):
        flags.append(f"target={item['target']}")
    if item.get("rel"):
        flags.append(f"rel={item['rel']}")
    if "download" in item:
        flags.append(f"download={item['download']}")
    if item.get("input_type"):
        flags.append(f"type={item['input_type']}")
    if item.get("autofocus") is True:
        flags.append("autofocus")
    if item.get("name"):
        flags.append(f"name={item['name']}")
    for test_key in ("test_id", "test", "qa"):
        if item.get(test_key):
            flags.append(f"{test_key}={item[test_key]}")
    if item.get("accept"):
        flags.append(f"accept={item['accept']}")
    if "capture" in item:
        flags.append(f"capture={item['capture']}")
    if "multiple" in item:
        flags.append(f"multiple={item['multiple']}")
    if item.get("selected_values"):
        flags.append(f"selected_values={','.join(item['selected_values'])}")
    if "size" in item:
        flags.append(f"size={item['size']}")
    if item.get("autocomplete"):
        flags.append(f"autocomplete={item['autocomplete']}")
    if item.get("inputmode"):
        flags.append(f"inputmode={item['inputmode']}")
    if item.get("enterkeyhint"):
        flags.append(f"enterkeyhint={item['enterkeyhint']}")
    if item.get("autocapitalize"):
        flags.append(f"autocapitalize={item['autocapitalize']}")
    if item.get("dirname"):
        flags.append(f"dirname={item['dirname']}")
    if item.get("form"):
        flags.append(f"form={item['form']}")
    for form_key in (
        "form_action",
        "form_method",
        "form_target",
        "form_enctype",
        "form_accept_charset",
        "form_autocomplete",
    ):
        if item.get(form_key):
            flags.append(f"{form_key}={item[form_key]}")
    if "form_novalidate" in item:
        flags.append(f"form_novalidate={item['form_novalidate']}")
    if item.get("list"):
        flags.append(f"list={item['list']}")
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
        if item.get(command_key):
            flags.append(f"{command_key}={item[command_key]}")
    if "formnovalidate" in item:
        flags.append(f"formnovalidate={item['formnovalidate']}")
    if item.get("accesskey"):
        flags.append(f"accesskey={item['accesskey']}")
    if "spellcheck" in item:
        flags.append(f"spellcheck={item['spellcheck']}")
    if item.get("value"):
        flags.append(f"value={item['value']}")
    for constraint_key in ("minlength", "maxlength", "min", "max", "step", "pattern"):
        if constraint_key in item:
            flags.append(f"{constraint_key}={item[constraint_key]}")
    if "checked" in item:
        flags.append(f"checked={item['checked']}")
    for state_key in (
        "expanded",
        "pressed",
        "selected",
        "current",
        "controls",
        "haspopup",
        "invalid",
        "aria_placeholder",
        "aria_autocomplete",
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
        if state_key in item:
            flags.append(f"{state_key}={item[state_key]}")

    if flags:
        parts.append(" ".join(f"[{flag}]" for flag in flags))
    if item.get("description"):
        parts.append(f'- {item["description"]}')

    return " ".join(parts)


class PlasmateExtractor:
    """Extract web page content using Plasmate's SOM format.

    Use this alongside Browser Use to get structured, token-efficient
    page representations instead of raw DOM serialization.
    """

    def __init__(self, plasmate_bin: str = "plasmate"):
        self.plasmate_bin = plasmate_bin
        self._verify_binary()

    def _verify_binary(self):
        """Check that plasmate binary is available."""
        try:
            result = subprocess.run(
                [self.plasmate_bin, "--version"],
                capture_output=True, text=True, timeout=5
            )
            if result.returncode != 0:
                raise RuntimeError(f"plasmate binary not working: {result.stderr}")
        except FileNotFoundError:
            raise RuntimeError(
                "plasmate binary not found. Install with: cargo install plasmate\n"
                "Or: curl -fsSL https://plasmate.app/install.sh | sh"
            )

    def extract(self, url: str) -> dict:
        """Fetch a URL and return parsed SOM output.

        Returns the full SOM dict with regions, elements, meta, etc.
        """
        result = subprocess.run(
            [self.plasmate_bin, "fetch", url],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode != 0:
            raise RuntimeError(f"plasmate fetch failed: {result.stderr}")
        som = _extract_last_json(result.stdout)
        if som is None:
            raise RuntimeError(
                f"plasmate returned no valid JSON for {url}: {result.stdout[:200]}"
            )
        return som

    async def extract_async(self, url: str) -> dict:
        """Async version of extract."""
        proc = await asyncio.create_subprocess_exec(
            self.plasmate_bin, "fetch", url,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )
        stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=30)
        if proc.returncode != 0:
            raise RuntimeError(f"plasmate fetch failed: {stderr.decode()}")
        som = _extract_last_json(stdout.decode())
        if som is None:
            raise RuntimeError(
                f"plasmate returned no valid JSON for {url}: {stdout.decode()[:200]}"
            )
        return som

    def extract_markdown(self, url: str) -> str:
        """Fetch a URL and return SOM content as markdown.

        This is the simplest integration -- just get readable text
        with structure preserved.
        """
        som_data = self.extract(url)
        som = parse_som(som_data)
        return to_markdown(som)

    async def extract_markdown_async(self, url: str) -> str:
        """Async version of extract_markdown."""
        som_data = await self.extract_async(url)
        som = parse_som(som_data)
        return to_markdown(som)

    def extract_action_plan(self, url: str) -> list[dict[str, object]]:
        """Fetch a URL and return compact action targets.

        Each target includes the SOM id, role, actions, label/context fields,
        and availability fields such as ``enabled`` and ``blocked_reason``.
        """
        som_data = self.extract(url)
        som = parse_som(som_data)
        return get_action_plan(som)

    async def extract_action_plan_async(self, url: str) -> list[dict[str, object]]:
        """Async version of extract_action_plan."""
        som_data = await self.extract_async(url)
        som = parse_som(som_data)
        return get_action_plan(som)

    def get_page_context(self, url: str) -> str:
        """Get a token-efficient page context string for LLM consumption.

        Returns a formatted string with:
        - Page title and URL
        - Interactive elements (what the agent can do)
        - Content summary
        - Compression stats
        """
        som_data = self.extract(url)
        return self._build_context(som_data)

    async def get_page_context_async(self, url: str) -> str:
        """Async version of get_page_context."""
        som_data = await self.extract_async(url)
        return self._build_context(som_data)

    def _build_context(self, som_data: dict) -> str:
        """Build the LLM context string from raw SOM data."""
        som = parse_som(som_data)

        lines = []
        lines.append(f"# {som.title}")
        lines.append(f"URL: {som.url}")
        lines.append(f"Language: {som.lang}")
        lines.append("")

        # Interactive elements
        action_plan = get_action_plan(som)
        if action_plan:
            lines.append(f"## Interactive Elements ({len(action_plan)})")
            for item in action_plan:
                lines.append(_format_action_plan_item(item))
            lines.append("")

        # Links
        links = get_links(som)
        if links:
            lines.append(f"## Links ({len(links)})")
            for link in links[:20]:  # Cap at 20
                lines.append(f"  [{link['id']}] {link['text']} -> {link['href']}")
            if len(links) > 20:
                lines.append(f"  ... and {len(links) - 20} more")
            lines.append("")

        # Content
        text = get_text(som)
        if text:
            lines.append("## Content")
            lines.append(text[:2000])  # Cap content
            if len(text) > 2000:
                lines.append(f"... ({len(text) - 2000} more characters)")

        # Stats
        meta = som.meta
        ratio = meta.html_bytes / max(meta.som_bytes, 1)
        lines.append("")
        lines.append("---")
        lines.append(f"Compression: {ratio:.1f}x ({meta.html_bytes} HTML bytes -> {meta.som_bytes} SOM bytes)")
        lines.append(f"Elements: {meta.element_count} ({meta.interactive_count} interactive)")

        return "\n".join(lines)
