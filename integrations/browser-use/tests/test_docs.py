from pathlib import Path


def test_published_docs_import_shipped_extractor_not_missing_browser():
    root = Path(__file__).resolve().parents[3]
    surfaces = [
        root / "website" / "docs" / "src" / "integration-browser-use.md",
        root / "website" / "docs" / "integration-browser-use.html",
    ]
    for path in surfaces:
        content = path.read_text()
        assert "from plasmate_browser_use import PlasmateBrowser" not in content
        assert "PlasmateBrowser(" not in content
        assert "from plasmate_browser_use import PlasmateExtractor" in content
        assert "get_page_context" in content
