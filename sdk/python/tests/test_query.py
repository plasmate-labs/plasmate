"""Tests for SOM query helpers."""

import pytest

from plasmate.types import (
    ElementAttrs,
    ElementRole,
    RegionRole,
    SemanticHint,
    SelectOption,
    ShadowRoot,
    Som,
    SomElement,
    SomMeta,
    SomRegion,
    StructuredData,
)
from plasmate.query import (
    find_by_id,
    find_by_role,
    find_by_tag,
    find_by_text,
    find_interactive,
    flat_elements,
    get_token_estimate,
)


@pytest.fixture
def sample_som() -> Som:
    """Build a realistic SOM fixture."""
    return Som(
        som_version="1.0",
        url="https://example.com",
        title="Example Page",
        lang="en",
        regions=[
            SomRegion(
                id="r_navigation",
                role=RegionRole.navigation,
                label="Main Nav",
                elements=[
                    SomElement(
                        id="e1",
                        role=ElementRole.link,
                        text="Home",
                        actions=["click"],
                        attrs=ElementAttrs(href="/"),
                    ),
                    SomElement(
                        id="e2",
                        role=ElementRole.link,
                        text="About",
                        actions=["click"],
                        attrs=ElementAttrs(href="/about"),
                    ),
                ],
            ),
            SomRegion(
                id="r_main",
                role=RegionRole.main,
                elements=[
                    SomElement(
                        id="e3",
                        role=ElementRole.heading,
                        text="Welcome to Example",
                        attrs=ElementAttrs(level=1),
                    ),
                    SomElement(
                        id="e4",
                        role=ElementRole.paragraph,
                        text="This is the main content.",
                    ),
                    SomElement(
                        id="e5",
                        role=ElementRole.button,
                        text="Click Me",
                        actions=["click"],
                        hints=[SemanticHint.primary],
                    ),
                    SomElement(
                        id="e6",
                        role=ElementRole.image,
                        attrs=ElementAttrs(src="/logo.png", alt="Logo"),
                    ),
                    SomElement(
                        id="e7",
                        role=ElementRole.section,
                        shadow=ShadowRoot(
                            mode="open",
                            elements=[
                                SomElement(
                                    id="e_shadow",
                                    role=ElementRole.button,
                                    text="Shadow Action",
                                    actions=["click"],
                                    html_id="shadow-button",
                                )
                            ],
                        ),
                        children=[
                            SomElement(
                                id="e8",
                                role=ElementRole.paragraph,
                                text="Nested paragraph",
                            ),
                            SomElement(
                                id="e9",
                                role=ElementRole.text_input,
                                label="Email",
                                actions=["click", "type", "clear"],
                                attrs=ElementAttrs(
                                    input_type="email",
                                    placeholder="you@example.com",
                                    required=True,
                                ),
                            ),
                        ],
                    ),
                ],
            ),
            SomRegion(
                id="r_footer",
                role=RegionRole.footer,
                elements=[
                    SomElement(
                        id="e10",
                        role=ElementRole.paragraph,
                        text="Copyright 2025",
                    ),
                ],
            ),
        ],
        meta=SomMeta(
        html_bytes=5000,
        som_bytes=2000,
        element_count=11,
        interactive_count=5,
        ),
    )


class TestFindByRole:
    def test_finds_matching_regions(self, sample_som: Som) -> None:
        nav = find_by_role(sample_som, "navigation")
        assert len(nav) == 1
        assert nav[0].id == "r_navigation"

    def test_returns_empty_for_no_match(self, sample_som: Som) -> None:
        result = find_by_role(sample_som, "dialog")
        assert result == []

    def test_finds_multiple_regions(self) -> None:
        som = Som(
            som_version="1.0",
            url="https://example.com",
            title="T",
            lang="en",
            regions=[
                SomRegion(id="r_aside", role=RegionRole.aside, elements=[]),
                SomRegion(id="r_aside_2", role=RegionRole.aside, elements=[]),
            ],
            meta=SomMeta(html_bytes=0, som_bytes=0, element_count=0, interactive_count=0),
        )
        assert len(find_by_role(som, "aside")) == 2


class TestFindById:
    def test_finds_top_level_element(self, sample_som: Som) -> None:
        el = find_by_id(sample_som, "e1")
        assert el is not None
        assert el.text == "Home"

    def test_finds_nested_element(self, sample_som: Som) -> None:
        el = find_by_id(sample_som, "e8")
        assert el is not None
        assert el.text == "Nested paragraph"

    def test_finds_shadow_element(self, sample_som: Som) -> None:
        el = find_by_id(sample_som, "e_shadow")
        assert el is not None
        assert el.html_id == "shadow-button"

    def test_returns_none_for_missing_id(self, sample_som: Som) -> None:
        assert find_by_id(sample_som, "e999") is None


class TestFindByTag:
    def test_finds_all_links(self, sample_som: Som) -> None:
        links = find_by_tag(sample_som, "link")
        assert len(links) == 2

    def test_finds_paragraphs_including_nested(self, sample_som: Som) -> None:
        paras = find_by_tag(sample_som, "paragraph")
        assert len(paras) == 3  # e4, e8, e10

    def test_finds_none_for_missing_tag(self, sample_som: Som) -> None:
        assert find_by_tag(sample_som, "table") == []


class TestFindInteractive:
    def test_finds_all_interactive_elements(self, sample_som: Som) -> None:
        interactive = find_interactive(sample_som)
        ids = [e.id for e in interactive]
        assert "e1" in ids  # link
        assert "e2" in ids  # link
        assert "e5" in ids  # button
        assert "e9" in ids  # text_input (nested)
        assert "e_shadow" in ids  # shadow DOM button
        assert len(ids) == 5

    def test_empty_som_returns_empty(self) -> None:
        som = Som(
            som_version="1.0",
            url="https://example.com",
            title="T",
            lang="en",
            regions=[],
            meta=SomMeta(html_bytes=0, som_bytes=0, element_count=0, interactive_count=0),
        )
        assert find_interactive(som) == []


class TestFindByText:
    def test_finds_by_exact_text(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "Click Me")
        assert len(results) == 1
        assert results[0].id == "e5"

    def test_case_insensitive(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "click me")
        assert len(results) == 1

    def test_partial_match(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "Example")
        texts = [e.text for e in results]
        assert "Welcome to Example" in texts

    def test_finds_nested_text(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "Nested")
        assert len(results) == 1
        assert results[0].id == "e8"

    def test_finds_shadow_text(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "Shadow")
        assert len(results) == 1
        assert results[0].id == "e_shadow"

    def test_no_match_returns_empty(self, sample_som: Som) -> None:
        assert find_by_text(sample_som, "zzz_no_match") == []


class TestFlatElements:
    def test_flattens_all_elements(self, sample_som: Som) -> None:
        elements = flat_elements(sample_som)
        ids = [e.id for e in elements]
        assert "e1" in ids
        assert "e8" in ids  # nested
        assert "e9" in ids  # nested
        assert "e_shadow" in ids  # shadow DOM
        assert len(ids) == 11

    def test_preserves_order(self, sample_som: Som) -> None:
        elements = flat_elements(sample_som)
        ids = [e.id for e in elements]
        assert ids.index("e7") < ids.index("e8")
        assert ids.index("e8") < ids.index("e9")
        assert ids.index("e9") < ids.index("e_shadow")


class TestGetTokenEstimate:
    def test_returns_positive_int(self, sample_som: Som) -> None:
        estimate = get_token_estimate(sample_som)
        assert isinstance(estimate, int)
        assert estimate == 500

    def test_accepts_dict(self) -> None:
        d = {"som_version": "1.0", "title": "Test"}
        estimate = get_token_estimate(d)
        assert estimate > 0

    def test_larger_som_has_more_tokens(self, sample_som: Som) -> None:
        small_som = Som(
            som_version="1.0",
            url="https://example.com",
            title="T",
            lang="en",
            regions=[],
            meta=SomMeta(html_bytes=0, som_bytes=0, element_count=0, interactive_count=0),
        )
        assert get_token_estimate(sample_som) > get_token_estimate(small_som)


class TestPydanticModels:
    def test_som_roundtrip(self, sample_som: Som) -> None:
        data = sample_som.model_dump()
        restored = Som(**data)
        assert restored == sample_som

    def test_som_json_roundtrip(self, sample_som: Som) -> None:
        json_str = sample_som.model_dump_json()
        restored = Som.model_validate_json(json_str)
        assert restored == sample_som

    def test_rejects_extra_fields(self) -> None:
        with pytest.raises(Exception):
            SomMeta(
                html_bytes=0,
                som_bytes=0,
                element_count=0,
                interactive_count=0,
                unknown_field="oops",
            )

    def test_validates_level_range(self) -> None:
        attrs = ElementAttrs(level=3)
        assert attrs.level == 3
        with pytest.raises(Exception):
            ElementAttrs(level=0)
        with pytest.raises(Exception):
            ElementAttrs(level=7)

    def test_validates_non_negative_meta(self) -> None:
        with pytest.raises(Exception):
            SomMeta(html_bytes=-1, som_bytes=0, element_count=0, interactive_count=0)

    def test_structured_data(self) -> None:
        sd = StructuredData(
            open_graph={"title": "Test", "description": "A page"},
            meta={"viewport": "width=device-width"},
        )
        assert sd.open_graph["title"] == "Test"

    def test_select_option(self) -> None:
        opt = SelectOption(value="us", text="United States", selected=True)
        assert opt.selected is True
