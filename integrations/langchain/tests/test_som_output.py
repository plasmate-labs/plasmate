from langchain_plasmate.som_output import som_to_text


def test_som_to_text_surfaces_interactive_state():
    som = {
        "title": "Signup",
        "url": "https://example.com/signup",
        "meta": {
            "html_bytes": 800,
            "som_bytes": 200,
            "element_count": 1,
            "interactive_count": 1,
        },
        "regions": [
            {
                "id": "r1",
                "role": "form",
                "elements": [
                    {
                        "id": "e_email",
                        "role": "text_input",
                        "label": "Email",
                        "actions": ["type"],
                        "attrs": {
                            "input_type": "email",
                            "required": True,
                            "disabled": True,
                            "description": "Use your work email",
                            "group": "Account",
                        },
                    }
                ],
            }
        ],
    }

    text = som_to_text(som)

    assert '[e_email] input(email) "Email"' in text
    assert "[disabled]" in text
    assert "[required]" in text
    assert '[group="Account"]' in text
    assert '[description="Use your work email"]' in text
