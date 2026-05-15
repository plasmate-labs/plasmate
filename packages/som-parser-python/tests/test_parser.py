"""Tests for som-parser package."""

import json
from pathlib import Path

import pytest

from som_parser import (
    ElementAction,
    ElementRole,
    RegionRole,
    SemanticHint,
    Som,
    SomElement,
    SomShadowRoot,
    filter_elements,
    find_by_action,
    find_by_hint,
    find_by_html_id,
    find_by_id,
    find_by_role,
    find_by_text,
    from_plasmate,
    get_action_plan,
    get_action_plan_cache_key,
    get_all_elements,
    get_compression_ratio,
    get_forms,
    get_headings,
    get_inputs,
    get_interactive_elements,
    get_links,
    get_text,
    get_text_by_region,
    is_valid_som,
    parse_som,
    to_markdown,
)

FIXTURE_SOM = {
    "som_version": "0.1",
    "url": "https://example.com/",
    "title": "Example Domain",
    "lang": "en",
    "regions": [
        {
            "id": "r_nav",
            "role": "navigation",
            "elements": [
                {
                    "id": "e_1",
                    "role": "link",
                    "html_id": "home-link",
                    "text": "Home",
                    "actions": ["click"],
                    "attrs": {"href": "/"},
                },
                {
                    "id": "e_2",
                    "role": "link",
                    "text": "About",
                    "actions": ["click"],
                    "attrs": {"href": "/about"},
                },
            ],
        },
        {
            "id": "r_content",
            "role": "content",
            "elements": [
                {
                    "id": "e_3",
                    "role": "heading",
                    "text": "Welcome",
                    "attrs": {"level": 1},
                },
                {
                    "id": "e_4",
                    "role": "paragraph",
                    "text": "This is a test page.",
                },
                {
                    "id": "e_5",
                    "role": "link",
                    "text": "Learn more",
                    "actions": ["click"],
                    "attrs": {"href": "https://example.org"},
                },
                {
                    "id": "e_6",
                    "role": "image",
                    "attrs": {"src": "/logo.png", "alt": "Logo"},
                },
            ],
        },
        {
            "id": "r_form",
            "role": "form",
            "action": "/search",
            "method": "GET",
            "elements": [
                {
                    "id": "e_7",
                    "role": "text_input",
                    "label": "Search",
                    "actions": ["type", "clear"],
                    "attrs": {"input_type": "text", "name": "q", "placeholder": "Search..."},
                    "hints": ["required"],
                },
                {
                    "id": "e_8",
                    "role": "button",
                    "text": "Go",
                    "actions": ["click"],
                },
            ],
        },
    ],
    "meta": {
        "html_bytes": 5000,
        "som_bytes": 800,
        "element_count": 8,
        "interactive_count": 5,
    },
}

REPO_ROOT = Path(__file__).resolve().parents[3]


def _load_action_availability_fixture():
    fixture_dir = REPO_ROOT / "integrations" / "fixtures"
    som_payload = json.loads((fixture_dir / "action-availability.som.json").read_text())
    expected = json.loads((fixture_dir / "action-availability.expected.json").read_text())
    return parse_som(som_payload), expected["action_targets"]


@pytest.fixture
def som() -> Som:
    return parse_som(FIXTURE_SOM)


@pytest.fixture
def som_json() -> str:
    return json.dumps(FIXTURE_SOM)


# --- Parser tests ---


class TestParseSom:
    def test_parse_dict(self, som: Som):
        assert isinstance(som, Som)
        assert som.title == "Example Domain"
        assert som.url == "https://example.com/"
        assert som.som_version == "0.1"
        assert som.lang == "en"
        assert len(som.regions) == 3

    def test_parse_json_string(self, som_json: str):
        result = parse_som(som_json)
        assert isinstance(result, Som)
        assert result.title == "Example Domain"

    def test_parse_invalid_json_string(self):
        with pytest.raises(ValueError, match="Invalid JSON"):
            parse_som("not valid json {{{")

    def test_parse_invalid_schema(self):
        with pytest.raises(Exception):
            parse_som({"not": "a som"})

    def test_parse_wrong_type(self):
        with pytest.raises(TypeError):
            parse_som(42)  # type: ignore

    def test_regions_parsed(self, som: Som):
        assert som.regions[0].role == RegionRole.NAVIGATION
        assert som.regions[1].role == RegionRole.CONTENT
        assert som.regions[2].role == RegionRole.FORM

    def test_elements_parsed(self, som: Som):
        nav_elements = som.regions[0].elements
        assert len(nav_elements) == 2
        assert nav_elements[0].role == ElementRole.LINK
        assert nav_elements[0].html_id == "home-link"
        assert nav_elements[0].text == "Home"

    def test_group_role_and_legend_attr_parse(self):
        payload = {
            **FIXTURE_SOM,
            "regions": [
                {
                    "id": "r_form",
                    "role": "form",
                    "elements": [
                        {
                            "id": "e_group",
                            "role": "group",
                            "label": "Contact preference",
                            "attrs": {"legend": "Contact preference", "disabled": True},
                        }
                    ],
                }
            ],
        }
        som = parse_som(payload)
        group = som.regions[0].elements[0]
        assert group.role == ElementRole.GROUP
        assert group.attrs is not None
        assert group.attrs.legend == "Contact preference"
        assert group.attrs.disabled is True

    def test_meta_parsed(self, som: Som):
        assert som.meta.html_bytes == 5000
        assert som.meta.som_bytes == 800
        assert som.meta.element_count == 8
        assert som.meta.interactive_count == 5

    def test_form_region_attrs(self, som: Som):
        form = som.regions[2]
        assert form.action == "/search"
        assert form.method == "GET"


class TestIsValidSom:
    def test_valid(self):
        assert is_valid_som(FIXTURE_SOM) is True

    def test_valid_string(self):
        assert is_valid_som(json.dumps(FIXTURE_SOM)) is True

    def test_invalid_dict(self):
        assert is_valid_som({"bad": "data"}) is False

    def test_invalid_string(self):
        assert is_valid_som("nope") is False

    def test_invalid_type(self):
        assert is_valid_som(123) is False


class TestFromPlasmate:
    def test_direct_som(self):
        result = from_plasmate(json.dumps(FIXTURE_SOM))
        assert result.title == "Example Domain"

    def test_wrapped_som(self):
        wrapped = json.dumps({"som": FIXTURE_SOM})
        result = from_plasmate(wrapped)
        assert result.title == "Example Domain"

    def test_mixed_output(self):
        output = f"Fetching https://example.com/...\n{json.dumps(FIXTURE_SOM)}\nDone."
        result = from_plasmate(output)
        assert result.title == "Example Domain"

    def test_wrapped_mixed_output(self):
        output = f"Fetching...\n{json.dumps({'som': FIXTURE_SOM})}\nDone."
        result = from_plasmate(output)
        assert result.title == "Example Domain"

    def test_invalid_json(self):
        with pytest.raises(ValueError, match="No JSON object"):
            from_plasmate("not json")


# --- Query tests ---


class TestGetAllElements:
    def test_count(self, som: Som):
        elements = get_all_elements(som)
        assert len(elements) == 8

    def test_all_have_ids(self, som: Som):
        elements = get_all_elements(som)
        ids = [el.id for el in elements]
        assert ids == ["e_1", "e_2", "e_3", "e_4", "e_5", "e_6", "e_7", "e_8"]

    def test_includes_shadow_root_elements(self):
        som = parse_som(
            {
                **FIXTURE_SOM,
                "regions": [
                    {
                        "id": "r_content",
                        "role": "content",
                        "elements": [
                            {
                                "id": "host",
                                "role": "section",
                                "shadow": {
                                    "mode": "open",
                                    "elements": [
                                        {
                                            "id": "shadow_action",
                                            "role": "button",
                                            "text": "Shadow Save",
                                            "actions": ["click"],
                                        }
                                    ],
                                },
                            }
                        ],
                    }
                ],
            }
        )
        assert isinstance(som.regions[0].elements[0].shadow, SomShadowRoot)
        assert [el.id for el in get_all_elements(som)] == ["host", "shadow_action"]


class TestFindByRole:
    def test_links(self, som: Som):
        links = find_by_role(som, ElementRole.LINK)
        assert len(links) == 3

    def test_string_role(self, som: Som):
        links = find_by_role(som, "link")
        assert len(links) == 3

    def test_headings(self, som: Som):
        headings = find_by_role(som, ElementRole.HEADING)
        assert len(headings) == 1
        assert headings[0].text == "Welcome"

    def test_buttons(self, som: Som):
        buttons = find_by_role(som, ElementRole.BUTTON)
        assert len(buttons) == 1
        assert buttons[0].text == "Go"

    def test_no_results(self, som: Som):
        tables = find_by_role(som, ElementRole.TABLE)
        assert tables == []


class TestFindById:
    def test_found(self, som: Som):
        el = find_by_id(som, "e_3")
        assert el is not None
        assert el.text == "Welcome"
        assert el.role == ElementRole.HEADING

    def test_not_found(self, som: Som):
        assert find_by_id(som, "nonexistent") is None

    def test_finds_shadow_root_element(self):
        som = parse_som(
            {
                **FIXTURE_SOM,
                "regions": [
                    {
                        "id": "r_content",
                        "role": "content",
                        "elements": [
                            {
                                "id": "host",
                                "role": "section",
                                "shadow": {
                                    "mode": "open",
                                    "elements": [
                                        {
                                            "id": "shadow_link",
                                            "role": "link",
                                            "text": "Shadow Docs",
                                            "actions": ["click"],
                                            "attrs": {"href": "/shadow-docs"},
                                        }
                                    ],
                                },
                            }
                        ],
                    }
                ],
            }
        )
        assert find_by_id(som, "shadow_link").text == "Shadow Docs"


class TestFindByHtmlId:
    def test_found(self, som: Som):
        el = find_by_html_id(som, "home-link")
        assert el is not None
        assert el.id == "e_1"

    def test_not_found(self, som: Som):
        assert find_by_html_id(som, "missing-id") is None


class TestFindByText:
    def test_substring(self, som: Som):
        results = find_by_text(som, "home")
        assert len(results) == 1
        assert results[0].id == "e_1"

    def test_case_insensitive(self, som: Som):
        results = find_by_text(som, "WELCOME")
        assert len(results) == 1

    def test_exact_match(self, som: Som):
        results = find_by_text(som, "Home", exact=True)
        assert len(results) == 1

    def test_exact_no_match(self, som: Som):
        results = find_by_text(som, "home", exact=True)
        assert len(results) == 0

    def test_label_match(self, som: Som):
        results = find_by_text(som, "search")
        assert len(results) == 1
        assert results[0].id == "e_7"

    def test_no_match(self, som: Som):
        results = find_by_text(som, "xyznonexistent")
        assert len(results) == 0

    def test_finds_shadow_root_text(self):
        som = parse_som(
            {
                **FIXTURE_SOM,
                "regions": [
                    {
                        "id": "r_content",
                        "role": "content",
                        "elements": [
                            {
                                "id": "host",
                                "role": "section",
                                "shadow": {
                                    "mode": "open",
                                    "elements": [
                                        {
                                            "id": "shadow_text",
                                            "role": "paragraph",
                                            "text": "Inside shadow root",
                                        }
                                    ],
                                },
                            }
                        ],
                    }
                ],
            }
        )
        assert find_by_text(som, "inside shadow")[0].id == "shadow_text"


class TestGetInteractiveElements:
    def test_count(self, som: Som):
        interactive = get_interactive_elements(som)
        assert len(interactive) == 5

    def test_all_have_actions(self, som: Som):
        interactive = get_interactive_elements(som)
        for el in interactive:
            assert el.actions is not None
            assert len(el.actions) > 0

    def test_includes_shadow_root_actions(self):
        som = parse_som(
            {
                **FIXTURE_SOM,
                "regions": [
                    {
                        "id": "r_content",
                        "role": "content",
                        "elements": [
                            {
                                "id": "host",
                                "role": "section",
                                "shadow": {
                                    "mode": "open",
                                    "elements": [
                                        {
                                            "id": "shadow_button",
                                            "role": "button",
                                            "text": "Confirm",
                                            "actions": ["click"],
                                        }
                                    ],
                                },
                            }
                        ],
                    }
                ],
            }
        )
        assert [el.id for el in get_interactive_elements(som)] == ["shadow_button"]


class TestFindByAction:
    def test_finds_clickable_elements(self, som: Som):
        clickable = find_by_action(som, "click")
        assert [el.id for el in clickable] == ["e_1", "e_2", "e_5", "e_8"]

    def test_finds_typed_elements(self, som: Som):
        typed = find_by_action(som, ElementAction.TYPE)
        assert [el.id for el in typed] == ["e_7"]


class TestFindByHint:
    def test_finds_required_elements(self, som: Som):
        required = find_by_hint(som, "required")
        assert [el.id for el in required] == ["e_7"]

    def test_returns_empty_for_missing_hint(self, som: Som):
        assert find_by_hint(som, SemanticHint.DANGER) == []


class TestGetActionPlan:
    def test_returns_compact_action_targets(self, som: Som):
        plan = get_action_plan(som)
        assert plan[0] == {
            "id": "e_1",
            "role": "link",
            "actions": ["click"],
            "enabled": True,
            "label": "Home",
            "html_id": "home-link",
            "href": "/",
            "cache_key": "plasmate-action:v1:4f5af432",
        }
        assert plan[-2] == {
            "id": "e_7",
            "role": "text_input",
            "actions": ["type", "clear"],
            "enabled": True,
            "label": "Search",
            "name": "q",
            "input_type": "text",
            "placeholder": "Search...",
            "form_action": "/search",
            "form_method": "GET",
            "cache_key": "plasmate-action:v1:0b6b537f",
        }

    def test_marks_disabled_targets_unavailable(self):
        disabled_som = parse_som(
            {
                **FIXTURE_SOM,
                "regions": [
                    {
                        "id": "r_form",
                        "role": "form",
                        "elements": [
                            {
                                "id": "locked",
                                "role": "button",
                                "text": "Archive",
                                "actions": ["click"],
                                "attrs": {"disabled": True},
                            }
                        ],
                    }
                ],
                "meta": {
                    "html_bytes": 100,
                    "som_bytes": 50,
                    "element_count": 1,
                    "interactive_count": 1,
                },
            }
        )

        assert get_action_plan(disabled_som) == [
            {
                "id": "locked",
                "role": "button",
                "actions": ["click"],
                "enabled": False,
                "label": "Archive",
                "disabled": True,
                "blocked_reason": "disabled",
                "cache_key": "plasmate-action:v1:2de92b9a",
            }
        ]

    def test_returns_deterministic_cache_keys(self):
        assert (
            get_action_plan_cache_key(
                {
                    "id": "e_7",
                    "role": "text_input",
                    "actions": ["type", "clear"],
                    "enabled": True,
                    "label": "Search",
                    "name": "q",
                    "input_type": "text",
                    "placeholder": "Search...",
                }
            )
            == "plasmate-action:v1:0b6b537f"
        )

    def test_matches_shared_action_availability_manifest(self):
        som, expected_targets = _load_action_availability_fixture()

        assert get_action_plan(som) == expected_targets


class TestGetLinks:
    def test_links(self, som: Som):
        links = get_links(som)
        assert len(links) == 3
        assert links[0] == {"text": "Home", "href": "/", "id": "e_1"}
        assert links[1] == {"text": "About", "href": "/about", "id": "e_2"}
        assert links[2] == {
            "text": "Learn more",
            "href": "https://example.org",
            "id": "e_5",
        }


class TestGetForms:
    def test_forms(self, som: Som):
        forms = get_forms(som)
        assert len(forms) == 1
        assert forms[0].id == "r_form"
        assert forms[0].action == "/search"


class TestGetInputs:
    def test_inputs(self, som: Som):
        inputs = get_inputs(som)
        assert len(inputs) == 1
        assert inputs[0].id == "e_7"
        assert inputs[0].role == ElementRole.TEXT_INPUT


class TestGetHeadings:
    def test_headings(self, som: Som):
        headings = get_headings(som)
        assert len(headings) == 1
        assert headings[0] == {"level": 1, "text": "Welcome", "id": "e_3"}


class TestGetText:
    def test_text(self, som: Som):
        text = get_text(som)
        assert "Home" in text
        assert "About" in text
        assert "Welcome" in text
        assert "This is a test page." in text
        assert "Learn more" in text
        assert "Search" in text
        assert "Go" in text


class TestGetTextByRegion:
    def test_regions(self, som: Som):
        regions = get_text_by_region(som)
        assert len(regions) == 3
        assert regions[0]["region_id"] == "r_nav"
        assert regions[0]["role"] == "navigation"
        assert "Home" in regions[0]["text"]

    def test_content_region(self, som: Som):
        regions = get_text_by_region(som)
        content = regions[1]
        assert content["role"] == "content"
        assert "Welcome" in content["text"]
        assert "This is a test page." in content["text"]


class TestGetCompressionRatio:
    def test_ratio(self, som: Som):
        ratio = get_compression_ratio(som)
        assert ratio == 5000 / 800
        assert ratio == 6.25


class TestToMarkdown:
    def test_contains_title(self, som: Som):
        md = to_markdown(som)
        assert "# Example Domain" in md

    def test_contains_url(self, som: Som):
        md = to_markdown(som)
        assert "URL: https://example.com/" in md

    def test_contains_regions(self, som: Som):
        md = to_markdown(som)
        assert "## Navigation" in md
        assert "## Content" in md
        assert "## Form" in md

    def test_contains_links(self, som: Som):
        md = to_markdown(som)
        assert "[Home](/)" in md
        assert "[About](/about)" in md

    def test_contains_heading(self, som: Som):
        md = to_markdown(som)
        assert "### Welcome" in md

    def test_contains_paragraph(self, som: Som):
        md = to_markdown(som)
        assert "This is a test page." in md

    def test_contains_image(self, som: Som):
        md = to_markdown(som)
        assert "![Logo](/logo.png)" in md

    def test_contains_button(self, som: Som):
        md = to_markdown(som)
        assert "[Button: Go]" in md

    def test_contains_input(self, som: Som):
        md = to_markdown(som)
        assert "Input: Search" in md


class TestFilterElements:
    def test_filter_by_actions(self, som: Som):
        clickable = filter_elements(
            som, lambda el: el.actions is not None and "click" in [a.value for a in el.actions]
        )
        assert len(clickable) == 4  # 3 links + 1 button

    def test_filter_by_text(self, som: Som):
        with_text = filter_elements(som, lambda el: el.text is not None)
        assert len(with_text) == 6  # all except image and text_input
