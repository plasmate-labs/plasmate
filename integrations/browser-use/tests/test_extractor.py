from plasmate_browser_use.extractor import PlasmateExtractor


def test_build_context_surfaces_action_availability():
    extractor = PlasmateExtractor.__new__(PlasmateExtractor)
    som = {
        "som_version": "1.0",
        "url": "https://example.com/settings",
        "title": "Settings",
        "lang": "en",
        "meta": {
            "html_bytes": 400,
            "som_bytes": 100,
            "element_count": 1,
            "interactive_count": 1,
        },
        "regions": [
            {
                "id": "r1",
                "role": "form",
                "elements": [
                    {
                        "id": "e_save",
                        "role": "button",
                        "text": "Save",
                        "actions": ["click"],
                        "attrs": {
                            "disabled": True,
                            "description": "Unavailable until required fields are complete",
                        },
                    }
                ],
            }
        ],
    }

    context = extractor._build_context(som)

    assert '[e_save] button "Save" (click) [disabled] [blocked_reason=disabled]' in context
    assert "Unavailable until required fields are complete" in context
