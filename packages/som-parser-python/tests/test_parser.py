"""Tests for som-parser package."""

import json

import pytest

from som_parser import (
    ElementRole,
    RegionRole,
    Som,
    SomElement,
    filter_elements,
    find_by_id,
    find_by_role,
    find_by_text,
    from_plasmate,
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
                    "attrs": {"input_type": "text", "placeholder": "Search..."},
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
        assert nav_elements[0].text == "Home"

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

    def test_invalid_json(self):
        with pytest.raises(ValueError, match="Invalid JSON"):
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


class TestGetInteractiveElements:
    def test_count(self, som: Som):
        interactive = get_interactive_elements(som)
        assert len(interactive) == 5

    def test_all_have_actions(self, som: Som):
        interactive = get_interactive_elements(som)
        for el in interactive:
            assert el.actions is not None
            assert len(el.actions) > 0


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
