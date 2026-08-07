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


class PublicClaimSurfaceTests(unittest.TestCase):
    def test_public_guidance_uses_evidence_aligned_wording(self) -> None:
        for paths, stale_wording, aligned_wording in SURFACES:
            for relative_path in paths:
                with self.subTest(path=relative_path):
                    text = (ROOT / relative_path).read_text(encoding="utf-8")
                    self.assertNotIn(stale_wording, text)
                    self.assertIn(aligned_wording, text)


if __name__ == "__main__":
    unittest.main()
