from html import unescape
from pathlib import Path
import json
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]

SURFACES = (
    (
        ("website/docs/src/why-som.md", "website/docs/why-som.html"),
        "lower token costs, faster inference",
        "measure output size and token use for the target workflow",
    ),
    (
        ("website/docs/src/som-first-sites.md", "website/docs/som-first-sites.html"),
        "Reduce token costs by avoiding boilerplate and duplicated content",
        "measure token use with the target tokenizer",
    ),
)


GO_SDK_SURFACES = (
    "website/docs/src/sdk-go.md",
    "website/docs/sdk-go.html",
)


SOM_REFERENCE_SURFACES = (
    "website/docs/src/som.md",
    "website/docs/som.html",
)


def first_json_example(text: str) -> dict:
    markdown = re.search(r"```json\n(\{.*?\n\})\n```", text, re.S)
    if markdown:
        return json.loads(markdown.group(1))
    html = re.search(r'<code class="language-json">(.*?)</code>', text, re.S)
    if not html:
        raise AssertionError("published SOM reference is missing a JSON example")
    return json.loads(unescape(html.group(1)))


class PublicClaimSurfaceTests(unittest.TestCase):
    def test_public_guidance_uses_evidence_aligned_wording(self) -> None:
        for paths, stale_wording, aligned_wording in SURFACES:
            for relative_path in paths:
                with self.subTest(path=relative_path):
                    text = (ROOT / relative_path).read_text(encoding="utf-8")
                    self.assertNotIn(stale_wording, text)
                    self.assertIn(aligned_wording, text)

    def test_go_sdk_docs_use_published_module_and_options_type(self) -> None:
        for relative_path in GO_SDK_SURFACES:
            with self.subTest(path=relative_path):
                text = (ROOT / relative_path).read_text(encoding="utf-8")
                self.assertIn("github.com/plasmate-labs/plasmate/sdk/go", text)
                self.assertIn("FetchPageOptions", text)
                self.assertNotIn("github.com/nickel-org/plasmate-go", text)
                self.assertNotIn("plasmate.FetchOptions{", text)

    def test_som_reference_example_matches_compiled_contract(self) -> None:
        for relative_path in SOM_REFERENCE_SURFACES:
            with self.subTest(path=relative_path):
                text = (ROOT / relative_path).read_text(encoding="utf-8")
                self.assertNotIn('"compression_ratio"', text)
                self.assertNotIn('"version": "0.1"', text)
                self.assertNotIn("&quot;version&quot;: &quot;0.1&quot;", text)
                self.assertNotIn('"role": "Navigation"', text)
                self.assertNotIn("r_aside_0", text)
                self.assertIn("text_input", text)
                example = first_json_example(text)
                self.assertEqual(example["som_version"], "0.1")
                self.assertEqual(example["lang"], "en")
                self.assertEqual(example["meta"]["interactive_count"], 20)
                self.assertNotIn("compression_ratio", example["meta"])
                self.assertEqual(example["regions"][0]["role"], "navigation")
                self.assertIn("open_graph", example["structured_data"])
                self.assertIn("twitter_card", example["structured_data"])


if __name__ == "__main__":
    unittest.main()
