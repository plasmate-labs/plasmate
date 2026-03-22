"""Example: Using Browser Use with Plasmate backend.

This demonstrates how to use Plasmate as an alternative browser backend for
Browser Use, replacing Playwright + Chrome with Plasmate's SOM output.

The key advantage: Plasmate returns Semantic Object Model (SOM) output that
uses ~10x fewer tokens than raw DOM, while preserving all interactive elements
and content structure.

Requirements:
    pip install plasmate-browser-use langchain-openai

    # Ensure `plasmate` binary is on your PATH:
    # npm install -g plasmate
    # -- or --
    # cargo install plasmate
"""

from __future__ import annotations

import asyncio
import os

from plasmate_browser_use import PlasmateBrowser, token_count_comparison


async def main():
    """Navigate to Hacker News and find the top story using Plasmate."""

    print("=" * 60)
    print("Browser Use + Plasmate Example")
    print("=" * 60)
    print()

    async with PlasmateBrowser() as browser:
        # Step 1: Navigate to Hacker News
        print("[1] Navigating to Hacker News...")
        state = await browser.navigate("https://news.ycombinator.com")

        print(f"    Title: {state.title}")
        print(f"    URL:   {state.url}")
        print(f"    Interactive elements: {len(state.interactive_elements)}")
        print()

        # Step 2: Show the SOM output (what the LLM agent would see)
        print("[2] SOM output (what the LLM sees):")
        print("-" * 60)
        # Show first 50 lines to keep output manageable
        lines = state.text.split("\n")
        for line in lines[:50]:
            print(f"    {line}")
        if len(lines) > 50:
            print(f"    ... ({len(lines) - 50} more lines)")
        print("-" * 60)
        print()

        # Step 3: Token comparison
        print("[3] Token usage comparison:")
        comparison = token_count_comparison(state.som, state.text)
        print(f"    HTML size:           {comparison['html_bytes']:>8,} bytes")
        print(f"    SOM (JSON) size:     {comparison['som_bytes']:>8,} bytes  ({comparison['byte_ratio']}x smaller)")
        print(f"    SOM (text) size:     {comparison['som_text_bytes']:>8,} bytes")
        print(f"    Est. HTML tokens:    {comparison['html_tokens_est']:>8,}")
        print(f"    Est. SOM tokens:     {comparison['som_tokens_est']:>8,}  ({comparison['token_savings_pct']}% savings)")
        print()

        # Step 4: Find interactive elements (links on HN = stories)
        print("[4] Top stories (first 5 links in main region):")
        story_links = [
            el for el in state.interactive_elements
            if el.role == "link" and el.attrs.get("href", "").startswith("http")
        ]
        for i, link in enumerate(story_links[:5], 1):
            print(f"    {i}. [{link.index}] {link.text}")
            print(f"       -> {link.attrs.get('href', '')}")
        print()

        # Step 5: Click the top story
        if story_links:
            top_story = story_links[0]
            print(f"[5] Clicking top story: [{top_story.index}] {top_story.text}...")
            state = await browser.click(top_story.index)
            print(f"    Now at: {state.title}")
            print(f"    URL:    {state.url}")

            # Show token comparison for the new page
            comparison = token_count_comparison(state.som, state.text)
            print(f"    SOM tokens: {comparison['som_tokens_est']:,} ({comparison['token_savings_pct']}% savings)")
        else:
            print("[5] No external story links found to click.")

    print()
    print("Done!")


async def compare_with_browser_use_default():
    """Side-by-side comparison: Plasmate SOM vs typical Browser Use DOM output.

    This demonstrates the token savings without requiring a full Browser Use
    + Playwright setup. It shows what each backend would feed to the LLM.
    """

    print("=" * 60)
    print("Token Comparison: Plasmate SOM vs Browser Use DOM")
    print("=" * 60)
    print()

    test_urls = [
        "https://news.ycombinator.com",
        "https://example.com",
    ]

    async with PlasmateBrowser() as browser:
        for url in test_urls:
            state = await browser.navigate(url)
            comparison = token_count_comparison(state.som, state.text)

            print(f"  {state.title} ({url})")
            print(f"  {'─' * 50}")
            print(f"  Browser Use (Playwright DOM):  ~{comparison['html_tokens_est']:>6,} tokens")
            print(f"  Plasmate (SOM):                ~{comparison['som_tokens_est']:>6,} tokens")
            print(f"  Savings:                        {comparison['token_ratio']:>5}x fewer tokens")
            print()


if __name__ == "__main__":
    asyncio.run(main())
