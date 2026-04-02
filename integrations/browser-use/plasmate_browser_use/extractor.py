"""Plasmate SOM extractor for Browser Use.

Provides SOM-based content extraction that can replace or complement
Browser Use's default DOM serialization, reducing token costs by 90%+.
"""

import asyncio
import json
import subprocess
from typing import Any, Optional

from som_parser import parse_som, get_links, get_interactive_elements, get_text, to_markdown


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
        interactive = get_interactive_elements(som)
        if interactive:
            lines.append(f"## Interactive Elements ({len(interactive)})")
            for el in interactive:
                actions = ", ".join(el.actions) if el.actions else ""
                label = el.text or el.label or (el.attrs.get("placeholder", "") if el.attrs else "")
                lines.append(f'  [{el.id}] {el.role} "{label}" ({actions})')
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
