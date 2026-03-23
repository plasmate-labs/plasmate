"""Example: Using Plasmate SOM with Browser Use for cheaper AI browsing."""
import asyncio
from plasmate_browser_use import PlasmateExtractor


async def main():
    extractor = PlasmateExtractor()

    # Get SOM-formatted page context (10x fewer tokens than raw HTML)
    context = extractor.get_page_context("https://news.ycombinator.com")
    print(context)

    # Or get structured SOM data
    som = extractor.extract("https://news.ycombinator.com")
    print(f"Compression: {som['meta']['html_bytes'] / som['meta']['som_bytes']:.1f}x")
    print(f"Elements: {som['meta']['element_count']}")

    # Or just markdown
    md = extractor.extract_markdown("https://news.ycombinator.com")
    print(md[:500])


if __name__ == "__main__":
    asyncio.run(main())
