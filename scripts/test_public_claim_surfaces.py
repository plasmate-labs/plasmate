from pathlib import Path
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


if __name__ == "__main__":
    unittest.main()
