from pathlib import Path


README = Path(__file__).parents[1] / "README.md"


def test_readme_labels_token_evidence_limits() -> None:
    readme = README.read_text()

    assert "## Token usage" in readme
    assert "does not retain a LangChain token benchmark" in readme
    assert "not token counts" in readme
    assert "## Token Comparison" not in readme
