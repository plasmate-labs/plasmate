"""Tests for SOM query helpers."""

import json
from pathlib import Path

import pytest

from plasmate.types import (
    ElementAttrs,
    ElementRole,
    ListItem,
    RegionRole,
    SemanticHint,
    SelectOption,
    Som,
    SomElement,
    SomMeta,
    SomRegion,
    SomShadowRoot,
    StructuredData,
)
from plasmate.query import (
    find_action_target,
    find_action_target_by_cache_key,
    find_action_target_by_html_id,
    find_action_target_by_id,
    find_action_target_by_label,
    find_action_target_by_test_id,
    find_action_targets_by_action,
    find_action_targets_by_label,
    find_action_targets_by_role,
    find_by_id,
    find_by_html_id,
    find_by_label,
    find_by_role,
    find_by_tag,
    find_by_text,
    find_interactive,
    flat_elements,
    get_action_plan,
    get_action_plan_cache_key,
    get_action_plan_index,
    get_compression_ratio,
    get_enabled_action_plan,
    get_token_estimate,
)

REPO_ROOT = Path(__file__).resolve().parents[3]


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
                        html_id="home-link",
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
            element_count=10,
            interactive_count=4,
        ),
    )


def _shadow_som() -> Som:
    """Build a small SOM with declarative shadow DOM content."""
    return Som(
        som_version="1.0",
        url="https://example.com/shadow",
        title="Shadow Page",
        lang="en",
        regions=[
            SomRegion(
                id="r_main",
                role=RegionRole.main,
                elements=[
                    SomElement(
                        id="host",
                        role=ElementRole.section,
                        shadow=SomShadowRoot(
                            mode="open",
                            elements=[
                                SomElement(
                                    id="shadow_text",
                                    role=ElementRole.paragraph,
                                    text="Inside shadow root",
                                ),
                                SomElement(
                                    id="shadow_button",
                                    role=ElementRole.button,
                                    text="Confirm",
                                    actions=["click"],
                                    attrs=ElementAttrs(aria={"pressed": False}),
                                ),
                            ],
                        ),
                    )
                ],
            )
        ],
        meta=SomMeta(html_bytes=1000, som_bytes=500, element_count=3, interactive_count=1),
    )


class TestFindByRole:
    def test_finds_matching_regions(self, sample_som: Som) -> None:
        nav = find_by_role(sample_som, "navigation")
        assert len(nav) == 1
        assert nav[0].id == "r_navigation"

    def test_returns_empty_for_no_match(self, sample_som: Som) -> None:
        result = find_by_role(sample_som, "dialog")
        assert result == []

    def test_accepts_region_selector_aliases(self, sample_som: Som) -> None:
        assert [region.id for region in find_by_role(sample_som, " MAIN-CONTENT ")] == [
            "r_main"
        ]
        assert [region.id for region in find_by_role(sample_som, '"role=navigation"')] == [
            "r_navigation"
        ]
        assert [region.id for region in find_by_role(sample_som, "role = 'navigation'")] == [
            "r_navigation"
        ]
        assert [region.id for region in find_by_role(sample_som, "content-info")] == [
            "r_footer"
        ]

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
        for alias in ("asides", "sidebar", "sidebars", "complementary"):
            assert len(find_by_role(som, alias)) == 2

    def test_preserves_extension_role_lookup(self, sample_som: Som) -> None:
        extension_region = SomRegion.model_construct(
            id="r_extension", role="extension", elements=[]
        )
        som = sample_som.model_copy(
            update={"regions": [*sample_som.regions, extension_region]}
        )

        assert [region.id for region in find_by_role(som, "extension")] == [
            "r_extension"
        ]


class TestFindById:
    def test_finds_top_level_element(self, sample_som: Som) -> None:
        el = find_by_id(sample_som, "e1")
        assert el is not None
        assert el.text == "Home"
        assert el.html_id == "home-link"

    def test_finds_nested_element(self, sample_som: Som) -> None:
        el = find_by_id(sample_som, "e8")
        assert el is not None
        assert el.text == "Nested paragraph"

    def test_returns_none_for_missing_id(self, sample_som: Som) -> None:
        assert find_by_id(sample_som, "e999") is None

    def test_finds_shadow_root_element(self) -> None:
        som = _shadow_som()
        el = find_by_id(som, "shadow_button")
        assert el is not None
        assert el.text == "Confirm"
        assert el.attrs is not None
        assert el.attrs.aria == {"pressed": False}


class TestFindByHtmlId:
    def test_finds_top_level_element(self, sample_som: Som) -> None:
        el = find_by_html_id(sample_som, "home-link")
        assert el is not None
        assert el.id == "e1"

    def test_returns_none_for_missing_html_id(self, sample_som: Som) -> None:
        assert find_by_html_id(sample_som, "missing-html-id") is None


class TestFindByLabel:
    def test_finds_label_only_controls(self, sample_som: Som) -> None:
        results = find_by_label(sample_som, "email")
        assert [el.id for el in results] == ["e9"]

    def test_exact_label_match_is_case_sensitive(self, sample_som: Som) -> None:
        assert [el.id for el in find_by_label(sample_som, "Email", exact=True)] == ["e9"]
        assert find_by_label(sample_som, "email", exact=True) == []

    def test_finds_compiled_label_fallback_attributes(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(
                id="e11",
                role=ElementRole.button,
                attrs=ElementAttrs(
                    aria={"label": "Close dialog"},
                    title="Billing details",
                    placeholder="Search products",
                    alt="Product preview",
                ),
            )
        )

        assert [el.id for el in find_by_label(sample_som, "billing")] == ["e11"]
        assert [el.id for el in find_by_label(sample_som, "search products")] == ["e11"]
        assert [el.id for el in find_by_label(sample_som, "PRODUCT PREVIEW")] == ["e11"]
        assert [el.id for el in find_by_label(sample_som, "Close dialog", exact=True)] == ["e11"]

    def test_normalizes_label_whitespace(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="e11", role=ElementRole.button, label="Billing\n details")
        )

        assert [el.id for el in find_by_label(sample_som, "billing details")] == ["e11"]
        assert [el.id for el in find_by_label(sample_som, "Billing details", exact=True)] == ["e11"]

    def test_empty_labels_do_not_match_every_labeled_element(self, sample_som: Som) -> None:
        assert find_by_label(sample_som, "") == []
        assert find_by_label(sample_som, " \n\t ") == []


class TestFindByTag:
    def test_finds_all_links(self, sample_som: Som) -> None:
        links = find_by_tag(sample_som, "link")
        assert len(links) == 2

    def test_accepts_common_html_tag_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="e11", role=ElementRole.list, actions=["click"])
        )
        assert [el.id for el in find_by_tag(sample_som, "a")] == ["e1", "e2"]
        assert [el.id for el in find_by_tag(sample_som, "p")] == ["e4", "e8", "e10"]
        for alias in ("menu", "dl", "definition-list", "unordered-list", "ordered_list"):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["e11"]
        assert [item["id"] for item in find_action_targets_by_role(sample_som, "dl")] == [
            "e11"
        ]

    def test_accepts_structural_html_tag_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(id="e12", role=ElementRole.section),
                SomElement(id="e13", role=ElementRole.group),
            ]
        )
        assert [el.id for el in find_by_tag(sample_som, "article")] == ["e7", "e12"]
        assert [el.id for el in find_by_tag(sample_som, "div")] == ["e13"]

    def test_accepts_group_backed_structural_tag_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(SomElement(id="group", role=ElementRole.group))
        for alias in (
            "figure",
            "figures",
            "radiogroup",
            "radiogroups",
            "progress",
            "progressbar",
            "progressbars",
            "meter",
            "meters",
            "output",
            "outputs",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["group"], alias

    def test_narrows_progress_and_meter_aliases_using_source_roles(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(
                    id="progress",
                    role=ElementRole.group,
                    attrs=ElementAttrs(source_role="progress-bar"),
                ),
                SomElement(
                    id="meter",
                    role=ElementRole.group,
                    attrs=ElementAttrs(source_role="meter"),
                ),
                SomElement(id="other-group", role=ElementRole.group),
            ]
        )

        for alias in (
            "progress",
            "progressbar",
            "progress-bar",
            "progress bar",
            "progressbars",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["progress"], alias
        for alias in ("meter", "meters"):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["meter"], alias

    def test_accepts_image_iframe_and_details_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(id="image", role=ElementRole.image, actions=["click"]),
                SomElement(id="frame", role=ElementRole.iframe, actions=["click"]),
                SomElement(id="details", role=ElementRole.details, actions=["click"]),
            ]
        )

        for alias in ("image", "images"):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["e6", "image"]
        for alias in ("iframe", "iframes"):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["frame"]
        for alias in ("details", "detail", "summary", "summaries"):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["details"]

        for alias, expected in (
            ("images", "image"),
            ("iframes", "frame"),
            ("summaries", "details"),
        ):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == [
                expected
            ]

    def test_accepts_plural_fieldset_alias(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="fieldset", role=ElementRole.group, actions=["click"])
        )

        assert [el.id for el in find_by_tag(sample_som, "fieldsets")] == ["fieldset"]
        assert [item["id"] for item in find_action_targets_by_role(sample_som, "fieldsets")] == [
            "fieldset"
        ]

    def test_accepts_blockquote_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="quote", role=ElementRole.paragraph, text="A cited thought")
        )

        for alias in ("blockquote", "blockquotes", "quote", "quotes"):
            assert [el.id for el in find_by_tag(sample_som, alias)] == [
                "e4",
                "e8",
                "quote",
                "e10",
            ]

    def test_accepts_media_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(
                    id="audio",
                    role=ElementRole.group,
                    attrs=ElementAttrs(source_role="audio"),
                ),
                SomElement(
                    id="video",
                    role=ElementRole.group,
                    attrs=ElementAttrs(source_role="video"),
                ),
                SomElement(
                    id="canvas",
                    role=ElementRole.group,
                    attrs=ElementAttrs(source_role="canvas"),
                ),
            ]
        )
        for alias in (
            "audio",
            "audios",
            "audio-item",
            "audioitem",
            "audio_items",
            "audio-items",
            "audioitems",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["audio"]
        for alias in (
            "video",
            "videos",
            "video-item",
            "videoitem",
            "video_items",
            "video-items",
            "videoitems",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["video"]
        for alias in (
            "canvas",
            "canvases",
            "canvas-item",
            "canvasitem",
            "canvas_items",
            "canvas-items",
            "canvasitems",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["canvas"]
        for alias in ("media", "media-item", "mediaitem", "media_items", "mediaitems"):
            assert [el.id for el in find_by_tag(sample_som, alias)] == [
                "e6",
                "audio",
                "video",
                "canvas",
            ]

    def test_accepts_aria_control_aliases(self, sample_som: Som) -> None:
        assert [el.id for el in find_by_tag(sample_som, "textbox")] == ["e9"]
        assert [el.id for el in find_by_tag(sample_som, "searchbox")] == ["e9"]
        assert [el.id for el in find_by_tag(sample_som, "search-input")] == ["e9"]
        assert find_by_tag(sample_som, "listbox") == []

    def test_accepts_aria_button_role_aliases(self, sample_som: Som) -> None:
        for alias in (
            "menuitem",
            "menuitems",
            "option",
            "options",
            "tab",
            "tabs",
            "treeitem",
            "treeitems",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["e5"]

    def test_accepts_checkbox_and_radio_role_aliases(self, sample_som: Som) -> None:
        checkbox = SomElement(id="checkbox", role=ElementRole.checkbox, actions=["toggle"])
        radio = SomElement(id="radio", role=ElementRole.radio, actions=["click"])
        som = Som(
            som_version=sample_som.som_version,
            url=sample_som.url,
            title=sample_som.title,
            lang=sample_som.lang,
            meta=sample_som.meta,
            regions=[SomRegion(id="r_controls", role=RegionRole.main, elements=[checkbox, radio])],
        )
        for alias in ("check box", "check-box", "menu item checkboxes", "switches"):
            assert [element.id for element in find_by_tag(som, alias)] == ["checkbox"]
        for alias in ("radio button", "radio-button", "menu item radios"):
            assert [element.id for element in find_by_tag(som, alias)] == ["radio"]
        for alias in ("check box", "menu item checkboxes", "switches"):
            assert [item["id"] for item in find_action_targets_by_role(som, alias)] == ["checkbox"]
        for alias in ("radio button", "menu item radios"):
            assert [item["id"] for item in find_action_targets_by_role(som, alias)] == ["radio"]

    def test_accepts_plural_text_control_aliases(self, sample_som: Som) -> None:
        for alias in (
            "text",
            "email",
            "number",
            "password",
            "tel",
            "url",
            "date",
            "date-time",
            "datetime-local",
            "date time local",
            "month",
            "time",
            "week",
            "color",
            "range",
            "hidden",
            "file",
            "text inputs",
            "text_box",
            "text_boxes",
            "textboxes",
            "search boxes",
            "search inputs",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["e9"]

    def test_accepts_search_and_query_control_aliases(self, sample_som: Som) -> None:
        for alias in (
            "search-control",
            "search field",
            "query",
            "query-box",
            "query input",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["e9"]

    def test_accepts_plural_control_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(id="e11", role=ElementRole.select, actions=["select"]),
                SomElement(id="e12", role=ElementRole.textarea, actions=["type"]),
                SomElement(id="e13", role=ElementRole.checkbox, actions=["toggle"]),
                SomElement(id="e14", role=ElementRole.radio, actions=["toggle"]),
            ]
        )
        for alias, expected in (
            ("buttons", ["e5"]),
            ("links", ["e1", "e2"]),
            ("selects", ["e11"]),
            ("combobox", ["e11"]),
            ("comboboxes", ["e11"]),
            ("combo_box", ["e11"]),
            ("combo boxes", ["e11"]),
            ("combo-boxes", ["e11"]),
            ("listbox", ["e11"]),
            ("listboxes", ["e11"]),
            ("list_box", ["e11"]),
            ("list boxes", ["e11"]),
            ("textareas", ["e12"]),
            ("text_area", ["e12"]),
            ("text areas", ["e12"]),
            ("slider", ["e9"]),
            ("spin buttons", ["e9"]),
            ("checkboxes", ["e13"]),
            ("radios", ["e14"]),
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == expected

    def test_accepts_spaced_aria_widget_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="e12", role=ElementRole.radio)
        )
        assert [el.id for el in find_by_tag(sample_som, "menu-item-radio")] == ["e12"]
        assert [el.id for el in find_by_tag(sample_som, "menu item radio")] == ["e12"]

    def test_accepts_heading_level_aliases_and_trimmed_input(self, sample_som: Som) -> None:
        assert [el.id for el in find_by_tag(sample_som, " H1 ")] == ["e3"]
        assert [el.id for el in find_by_tag(sample_som, "heading1")] == ["e3"]
        assert [el.id for el in find_by_tag(sample_som, "heading-1")] == ["e3"]
        assert find_by_tag(sample_som, "h2") == []

    def test_finds_paragraphs_including_nested(self, sample_som: Som) -> None:
        paras = find_by_tag(sample_som, "paragraph")
        assert len(paras) == 3  # e4, e8, e10

    def test_finds_none_for_missing_tag(self, sample_som: Som) -> None:
        assert find_by_tag(sample_som, "table") == []

    def test_accepts_table_grid_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="table", role=ElementRole.table, text="Orders")
        )
        for alias in (
            "data_table",
            "data-tables",
            "tables",
            "data grid",
            "data-grids",
            "grid",
            "grids",
            "table grid",
            "table-grids",
            "treegrid",
            "treegrids",
        ):
            assert [el.id for el in find_by_tag(sample_som, alias)] == ["table"]

    def test_finds_shadow_root_roles(self) -> None:
        assert [el.id for el in find_by_tag(_shadow_som(), "paragraph")] == ["shadow_text"]


class TestFindActionTargetsByRole:
    def test_accepts_html_control_aliases(self, sample_som: Som) -> None:
        assert [item["id"] for item in find_action_targets_by_role(sample_som, "INPUT")] == ["e9"]
        assert [item["id"] for item in find_action_targets_by_role(sample_som, "textbox")] == ["e9"]
        for alias in (
            "text",
            "email",
            "number",
            "password",
            "tel",
            "url",
            "date",
            "date-time",
            "datetime-local",
            "date time local",
            "month",
            "time",
            "week",
            "color",
            "range",
            "hidden",
            "file",
            "search-box",
            "search box",
        ):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == ["e9"]

    def test_accepts_plural_text_control_aliases(self, sample_som: Som) -> None:
        for alias in (
            "text inputs",
            "text_box",
            "text_boxes",
            "textboxes",
            "search boxes",
            "search inputs",
            "slider",
            "sliders",
            "spinbutton",
            "spin buttons",
        ):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == ["e9"]

    def test_accepts_plural_control_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(id="e11", role=ElementRole.select, actions=["select"]),
                SomElement(id="e12", role=ElementRole.textarea, actions=["type"]),
                SomElement(id="e13", role=ElementRole.checkbox, actions=["toggle"]),
                SomElement(id="e14", role=ElementRole.radio, actions=["toggle"]),
            ]
        )
        for alias, expected in (
            ("buttons", ["e5"]),
            ("links", ["e1", "e2"]),
            ("selects", ["e11"]),
            ("comboboxes", ["e11"]),
            ("combo boxes", ["e11"]),
            ("combo-boxes", ["e11"]),
            ("combobox", ["e11"]),
            ("listboxes", ["e11"]),
            ("listbox", ["e11"]),
            ("list_box", ["e11"]),
            ("list boxes", ["e11"]),
            ("textareas", ["e12"]),
            ("text_area", ["e12"]),
            ("text areas", ["e12"]),
            ("checkboxes", ["e13"]),
            ("radios", ["e14"]),
        ):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == expected

    def test_accepts_media_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(
                    id="audio",
                    role=ElementRole.group,
                    actions=["click"],
                    attrs=ElementAttrs(source_role="audio"),
                ),
                SomElement(
                    id="video",
                    role=ElementRole.group,
                    actions=["click"],
                    attrs=ElementAttrs(source_role="video"),
                ),
                SomElement(
                    id="canvas",
                    role=ElementRole.group,
                    actions=["click"],
                    attrs=ElementAttrs(source_role="canvas"),
                ),
            ]
        )
        for aliases, expected in (
            (("audio-item", "audioitem", "audio-items", "audioitems"), "audio"),
            (("video-item", "videoitem", "video-items", "videoitems"), "video"),
            (("canvas-item", "canvasitem", "canvas-items", "canvasitems"), "canvas"),
        ):
            for alias in aliases:
                assert [
                    item["id"] for item in find_action_targets_by_role(sample_som, alias)
                ] == [expected]
        assert [item["id"] for item in find_action_targets_by_role(sample_som, "media")] == [
            "audio",
            "video",
            "canvas",
        ]

    def test_media_action_alias_normalizes_source_role_case(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(
                id="audio-upper",
                role=ElementRole.group,
                actions=["click"],
                attrs=ElementAttrs(source_role="Audio"),
            )
        )

        assert [item["id"] for item in find_action_targets_by_role(sample_som, "media")] == [
            "audio-upper",
        ]

    def test_trims_role_aliases(self, sample_som: Som) -> None:
        assert [item["id"] for item in find_action_targets_by_role(sample_som, "  a  ")] == ["e1", "e2"]

    def test_accepts_structural_tag_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="e12", role=ElementRole.section, actions=["click"])
        )
        assert [item["id"] for item in find_action_targets_by_role(sample_som, "article")] == ["e12"]

    def test_accepts_group_backed_structural_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="e12", role=ElementRole.group, actions=["click"])
        )
        for alias in (
            "figure",
            "figures",
            "radiogroup",
            "radiogroups",
            "progress",
            "progressbar",
            "progressbars",
            "meter",
            "meters",
            "output",
            "outputs",
        ):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == [
                "e12"
            ], alias

    def test_narrows_progress_and_meter_action_aliases_using_source_roles(
        self, sample_som: Som
    ) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(
                    id="progress",
                    role=ElementRole.group,
                    actions=["click"],
                    attrs=ElementAttrs(source_role="progress bar"),
                ),
                SomElement(
                    id="meter",
                    role=ElementRole.group,
                    actions=["click"],
                    attrs=ElementAttrs(source_role="meter"),
                ),
                SomElement(id="other-group", role=ElementRole.group, actions=["click"]),
            ]
        )

        for alias in (
            "progress",
            "progressbar",
            "progress-bar",
            "progress bar",
            "progressbars",
        ):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == [
                "progress"
            ], alias
        for alias in ("meter", "meters"):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == [
                "meter"
            ], alias

    def test_accepts_blockquote_action_role_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="quote", role=ElementRole.paragraph, actions=["click"])
        )
        for alias in ("blockquote", "blockquotes", "quote", "quotes"):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == [
                "quote"
            ], alias

    def test_accepts_table_grid_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="e12", role=ElementRole.table, actions=["click"])
        )
        for alias in (
            "tables",
            "data_table",
            "data-grids",
            "grid",
            "table grid",
            "treegrid",
            "treegrids",
        ):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == [
                "e12"
            ]

    def test_accepts_aria_widget_aliases(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.extend(
            [
                SomElement(id="e12", role=ElementRole.button, actions=["click"]),
                SomElement(id="e13", role=ElementRole.checkbox, actions=["toggle"]),
                SomElement(id="e14", role=ElementRole.radio, actions=["toggle"]),
            ]
        )
        assert "e12" in [item["id"] for item in find_action_targets_by_role(sample_som, "tab")]
        assert "e13" in [item["id"] for item in find_action_targets_by_role(sample_som, "switch")]
        assert "e14" in [item["id"] for item in find_action_targets_by_role(sample_som, "menuitemradio")]

    def test_accepts_aria_button_role_aliases(self, sample_som: Som) -> None:
        for alias in (
            "menuitem",
            "menuitems",
            "option",
            "options",
            "tab",
            "tabs",
            "treeitem",
            "treeitems",
        ):
            assert [item["id"] for item in find_action_targets_by_role(sample_som, alias)] == ["e5"]

    def test_accepts_hyphenated_and_spaced_aria_widget_aliases(
        self, sample_som: Som
    ) -> None:
        sample_som.regions[1].elements.append(
            SomElement(id="e15", role=ElementRole.radio, actions=["toggle"])
        )
        for alias in ("menu-item-radio", "menu item radio"):
            assert [
                item["id"] for item in find_action_targets_by_role(sample_som, alias)
            ] == ["e15"]


class TestFindActionTargetsByAction:
    def test_accepts_quoted_action_aliases(self, sample_som: Som) -> None:
        assert [
            item["id"]
            for item in find_action_targets_by_action(sample_som, '"action:type_text"')
        ] == ["e9"]
        assert [
            item["id"] for item in find_action_targets_by_action(sample_som, "'action:clear'")
        ] == ["e9"]

    def test_accepts_clear_text_action_aliases(self, sample_som: Som) -> None:
        for alias in ("clear_text", "clear-text", "clear text", "action:clear_text"):
            assert [
                item["id"] for item in find_action_targets_by_action(sample_som, alias)
            ] == ["e9"], alias


class TestFindInteractive:
    def test_finds_all_interactive_elements(self, sample_som: Som) -> None:
        interactive = find_interactive(sample_som)
        ids = [e.id for e in interactive]
        assert "e1" in ids  # link
        assert "e2" in ids  # link
        assert "e5" in ids  # button
        assert "e9" in ids  # text_input (nested)
        assert len(ids) == 4

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

    def test_finds_shadow_root_interactive_elements(self) -> None:
        assert [el.id for el in find_interactive(_shadow_som())] == ["shadow_button"]


class TestGetActionPlan:
    def test_action_target_by_label_normalizes_whitespace(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(
                id="e11",
                role=ElementRole.button,
                label="Account\n settings",
                actions=["click"],
            )
        )

        target = find_action_target_by_label(sample_som, "Account settings")

        assert target is not None
        assert target["id"] == "e11"
        assert find_action_target(sample_som, "Account settings")["id"] == "e11"
        assert find_action_target(sample_som, "account settings")["id"] == "e11"
        assert find_action_target(sample_som, "Account settings", by="label")["id"] == "e11"
        assert find_action_target(sample_som, "  \n\t") is None

    def test_uses_label_fallback_for_unlabelled_controls(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(
                id="e11",
                role=ElementRole.button,
                actions=["click"],
                attrs=ElementAttrs(title="Account settings"),
            )
        )

        target = find_action_targets_by_label(sample_som, "settings")[0]

        assert target["id"] == "e11"
        assert target["label"] == "Account settings"

    def test_ignores_whitespace_only_labels_when_selecting_fallback(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(
                id="e12",
                role=ElementRole.button,
                label="  \n\t",
                text="Visible action",
                actions=["click"],
            )
        )

        target = find_action_target(sample_som, "Visible action")

        assert target is not None
        assert target["id"] == "e12"
        assert target["label"] == "Visible action"

    def test_returns_compact_action_targets(self, sample_som: Som) -> None:
        plan = get_action_plan(sample_som)

        assert plan[0] == {
            "id": "e1",
            "role": "link",
            "actions": ["click"],
            "enabled": True,
            "label": "Home",
            "html_id": "home-link",
            "href": "/",
            "cache_key": "plasmate-action:v1:04ca84bb",
        }
        assert plan[-1] == {
            "id": "e9",
            "role": "text_input",
            "actions": ["click", "type", "clear"],
            "enabled": True,
            "label": "Email",
            "input_type": "email",
            "placeholder": "you@example.com",
            "required": True,
            "cache_key": "plasmate-action:v1:5b218ab1",
        }

    def test_matches_shared_action_availability_manifest(self) -> None:
        fixture_dir = REPO_ROOT / "integrations" / "fixtures"
        som = Som.model_validate(
            json.loads((fixture_dir / "action-availability.som.json").read_text())
        )
        expected = json.loads(
            (fixture_dir / "action-availability.expected.json").read_text()
        )["action_targets"]

        assert get_action_plan(som) == expected

    def test_returns_deterministic_cache_keys(self) -> None:
        assert (
            get_action_plan_cache_key(
                {
                    "id": "e9",
                    "role": "text_input",
                    "actions": ["click", "type", "clear"],
                    "enabled": True,
                    "label": "Email",
                    "input_type": "email",
                    "placeholder": "you@example.com",
                    "required": True,
                }
            )
            == "plasmate-action:v1:5b218ab1"
        )

    def test_cache_keys_normalize_label_whitespace(self) -> None:
        base = {
            "id": "e_settings",
            "role": "button",
            "actions": ["click"],
            "label": "Account settings",
        }

        assert get_action_plan_cache_key(base) == get_action_plan_cache_key(
            {**base, "label": "Account\n settings"}
        )

    def test_indexes_action_targets_for_replay(self, sample_som: Som) -> None:
        fixture_dir = REPO_ROOT / "integrations" / "fixtures"
        som = Som.model_validate(
            json.loads((fixture_dir / "action-availability.som.json").read_text())
        )
        expected = json.loads(
            (fixture_dir / "action-availability.expected.json").read_text()
        )["action_targets"]
        save = next(target for target in expected if target["id"] == "e_save")

        index = get_action_plan_index(som)

        assert index["by_id"]["e_save"] == save
        assert index["by_cache_key"][save["cache_key"]] == save
        assert index["by_html_id"]["save-button"] == save
        assert index["by_test_id"]["settings-save"] == save
        assert index["by_label"]["Save"] == save
        assert [target["id"] for target in index["by_role"]["button"]] == [
            "e_save",
            "e_preview",
        ]
        assert [target["id"] for target in index["by_action"]["click"]] == [
            "e_save",
            "e_preview",
            "e_billing",
        ]
        assert find_action_target(som, "e_save") == save
        assert find_action_target(som, save["cache_key"]) == save
        assert find_action_target(som, "save-button") == save
        assert find_action_target(som, "settings-save") == save
        assert find_action_target(som, "Save", by="label") == save
        assert find_action_target(som, "settings-save", by="test_id") == save
        assert find_action_target_by_id(som, "e_save") == save
        assert find_action_target_by_cache_key(som, save["cache_key"]) == save
        assert find_action_target_by_html_id(som, "save-button") == save
        assert find_action_target_by_test_id(som, "settings-save") == save
        assert find_action_target_by_label(som, "Save") == save
        assert find_action_targets_by_label(som, "save") == [save]
        assert find_action_targets_by_label(som, "  Save  ", exact=True) == [save]
        assert find_action_targets_by_label(som, "") == []
        assert find_action_targets_by_label(som, " \n\t ", exact=True) == []
        assert find_action_targets_by_role(som, "button") == [expected[2], expected[3]]
        assert [target["id"] for target in find_action_targets_by_role(som, " A ")] == [
            "e_billing",
        ]
        assert [target["id"] for target in find_action_targets_by_role(som, " COMBOBOX ")] == [
            "e_plan",
        ]
        assert [target["id"] for target in find_action_targets_by_role(som, "listbox")] == [
            "e_plan",
        ]
        for alias in (
            "search-control",
            "search field",
            "query",
            "query-box",
            "query input",
        ):
            assert [target["id"] for target in find_action_targets_by_role(som, alias)] == [
                "e_email",
                "e_upload",
                "e_quota",
            ]
        assert find_action_targets_by_action(som, "click") == [
            expected[2],
            expected[3],
            expected[-1],
        ]
        assert [target["id"] for target in find_action_targets_by_action(som, " type-text ")] == [
            "e_email",
            "e_upload",
            "e_quota",
        ]
        assert [target["id"] for target in find_action_targets_by_action(som, "select-option")] == [
            "e_plan",
            "e_annual",
        ]
        assert [target["id"] for target in find_action_targets_by_action(som, "type text")] == [
            "e_email",
            "e_upload",
            "e_quota",
        ]
        assert [target["id"] for target in find_action_targets_by_action(som, "select option")] == [
            "e_plan",
            "e_annual",
        ]
        assert [target["id"] for target in find_action_targets_by_action(som, "action : type-text")] == [
            "e_email",
            "e_upload",
            "e_quota",
        ]
        assert [target["id"] for target in find_action_targets_by_action(som, "ACTION = SELECT OPTION")] == [
            "e_plan",
            "e_annual",
        ]
        assert [target["id"] for target in find_action_targets_by_action(som, "action :  type   text")] == [
            "e_email",
            "e_upload",
            "e_quota",
        ]

        reset = sample_som.regions[1].elements[0].model_copy(
            update={
                "id": "e_reset",
                "role": ElementRole.button,
                "actions": ["click"],
                "attrs": ElementAttrs(button_type="reset"),
            }
        )
        image_submit = sample_som.regions[1].elements[0].model_copy(
            update={
                "id": "e_image_submit",
                "role": ElementRole.button,
                "actions": ["click"],
                "attrs": ElementAttrs(button_type="image"),
            }
        )
        sample_som.regions[1].elements.extend([reset, image_submit])
        assert [target["id"] for target in find_action_targets_by_action(sample_som, "reset")] == ["e_reset"]
        assert [target["id"] for target in find_action_targets_by_action(sample_som, "action:submit")] == [
            "e_image_submit",
        ]
        assert find_action_target(som, "settings-save", enabled_only=True) is None
        assert find_action_target(som, "Save", by="label", enabled_only=True) is None
        assert find_action_targets_by_role(som, "button", enabled_only=True) == []
        assert find_action_targets_by_action(som, "click", enabled_only=True) == [expected[-1]]

    def test_enabled_action_plan_index_filters_blocked_targets(self) -> None:
        fixture_dir = REPO_ROOT / "integrations" / "fixtures"
        som = Som.model_validate(
            json.loads((fixture_dir / "action-availability.som.json").read_text())
        )

        enabled = get_enabled_action_plan(som)
        index = get_action_plan_index(som, enabled_only=True)

        assert all(target["enabled"] for target in enabled)
        assert "e_save" not in index["by_id"]
        assert "settings-save" not in index["by_test_id"]
        assert "Save" not in index["by_label"]
        assert "button" not in index["by_role"]
        assert [target["id"] for target in index["by_action"]["click"]] == ["e_billing"]
        assert "e_plan" in index["by_id"]

    def test_action_index_normalizes_label_whitespace(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(
                id="e11",
                role=ElementRole.button,
                label="Account\n settings",
                actions=["click"],
            )
        )

        target = get_action_plan_index(sample_som)["by_label"]["Account settings"]

        assert target["id"] == "e11"


class TestFindByText:
    def test_finds_by_exact_text(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "Click Me")
        assert len(results) == 1
        assert results[0].id == "e5"

    def test_case_insensitive(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "click me")
        assert len(results) == 1

    def test_exact_is_case_sensitive_and_not_a_substring(self, sample_som: Som) -> None:
        assert [element.id for element in find_by_text(sample_som, "Click Me", exact=True)] == [
            "e5"
        ]
        assert find_by_text(sample_som, "click me", exact=True) == []
        assert find_by_text(sample_som, "Click", exact=True) == []

    def test_normalizes_whitespace_and_zero_width_separators(self, sample_som: Som) -> None:
        sample_som.regions[1].elements.append(
            SomElement(
                id="e_split_text",
                role=ElementRole.button,
                text="Save\u2060\n changes",
            )
        )

        results = find_by_text(sample_som, "save changes")
        assert [element.id for element in results] == ["e_split_text"]

    def test_partial_match(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "Example")
        texts = [e.text for e in results]
        assert "Welcome to Example" in texts

    def test_finds_label_only_controls(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "email")
        assert len(results) == 1
        assert results[0].id == "e9"

    def test_finds_nested_text(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "Nested")
        assert len(results) == 1
        assert results[0].id == "e8"

    def test_finds_shadow_root_text(self) -> None:
        results = find_by_text(_shadow_som(), "inside shadow")
        assert len(results) == 1
        assert results[0].id == "shadow_text"

    def test_finds_compiled_element_descriptions(self) -> None:
        som = Som(
            som_version="1.0",
            url="https://example.com/descriptions",
            title="Descriptions",
            lang="en",
            regions=[
                SomRegion(
                    id="r_main",
                    role=RegionRole.main,
                    elements=[
                        SomElement(
                            id="e_email",
                            role=ElementRole.text_input,
                            attrs=ElementAttrs(
                                description="Use your work address for alerts"
                            ),
                        )
                    ],
                )
            ],
            meta=SomMeta(
                html_bytes=100,
                som_bytes=50,
                element_count=1,
                interactive_count=1,
            ),
        )

        results = find_by_text(som, "work address")
        assert [element.id for element in results] == ["e_email"]

    def test_finds_user_facing_attribute_text(self) -> None:
        som = Som(
            som_version="1.0",
            url="https://example.com/attribute-text",
            title="Attribute Text",
            lang="en",
            regions=[
                SomRegion(
                    id="r_main",
                    role=RegionRole.main,
                    elements=[
                        SomElement(
                            id="e_logo",
                            role=ElementRole.image,
                            attrs=ElementAttrs(alt="Product wordmark"),
                        ),
                        SomElement(
                            id="e_search",
                            role=ElementRole.text_input,
                            attrs=ElementAttrs(placeholder="Search documentation"),
                        ),
                        SomElement(
                            id="e_submit",
                            role=ElementRole.button,
                            attrs=ElementAttrs(value="Search documentation"),
                        ),
                        SomElement(
                            id="e_progress",
                            role=ElementRole.section,
                            attrs=ElementAttrs(
                                aria={
                                    "valuetext": "50 percent complete",
                                    "roledescription": "progress indicator",
                                    "valuenow": "50",
                                }
                            ),
                        ),
                    ],
                )
            ],
            meta=SomMeta(
                html_bytes=100,
                som_bytes=50,
                element_count=2,
                interactive_count=1,
            ),
        )

        assert [element.id for element in find_by_text(som, "wordmark")] == ["e_logo"]
        assert [element.id for element in find_by_text(som, "documentation")] == [
            "e_search",
            "e_submit",
        ]
        assert [element.id for element in find_by_text(som, "50 percent")] == [
            "e_progress"
        ]
        assert [element.id for element in find_by_text(som, "progress indicator")] == [
            "e_progress"
        ]

    def test_no_match_returns_empty(self, sample_som: Som) -> None:
        assert find_by_text(sample_som, "zzz_no_match") == []

    def test_empty_query_does_not_match_every_element(self, sample_som: Som) -> None:
        assert find_by_text(sample_som, "") == []

    def test_finds_compiled_list_items(self) -> None:
        som = Som(
            som_version="1.0",
            url="https://example.com/list",
            title="List Page",
            lang="en",
            regions=[
                SomRegion(
                    id="r_main",
                    role=RegionRole.main,
                    elements=[
                        SomElement(
                            id="e_list",
                            role=ElementRole.list,
                            attrs=ElementAttrs(
                                ordered=False,
                                items=[
                                    ListItem(text="Install"),
                                    ListItem(text="Configure"),
                                ],
                            ),
                        )
                    ],
                )
            ],
            meta=SomMeta(
                html_bytes=100,
                som_bytes=50,
                element_count=1,
                interactive_count=0,
            ),
        )

        results = find_by_text(som, "install")
        assert [el.id for el in results] == ["e_list"]
        assert find_by_text(som, "Configure")[0].id == "e_list"
        assert find_by_text(som, "zzz_no_match") == []

    def test_finds_compiled_description_list_items(self) -> None:
        som = Som.model_validate(
            {
                "som_version": "1.0",
                "url": "https://example.test/glossary",
                "title": "Glossary",
                "lang": "en",
                "regions": [
                    {
                        "id": "r_main",
                        "role": "main",
                        "elements": [
                            {
                                "id": "e_glossary",
                                "role": "list",
                                "html_id": "glossary",
                                "attrs": {
                                    "items": [
                                        {
                                            "term": "SOM",
                                            "description": "Semantic Object Model",
                                        },
                                        {
                                            "term": "AWP",
                                            "description": "Agent Web Protocol",
                                        },
                                    ]
                                },
                            },
                            {
                                "id": "e_bullets",
                                "role": "list",
                                "html_id": "bullets",
                                "attrs": {"items": [{"text": "Dot"}]},
                            },
                        ],
                    }
                ],
                "meta": {
                    "html_bytes": 100,
                    "som_bytes": 50,
                    "element_count": 2,
                    "interactive_count": 0,
                },
            }
        )

        glossary = som.regions[0].elements[0]
        items = glossary.attrs.items if glossary.attrs else None
        assert items is not None
        assert items[0].term == "SOM"
        assert items[0].description == "Semantic Object Model"
        assert items[0].text is None
        assert find_by_text(som, "SOM")[0].id == "e_glossary"
        assert find_by_text(som, "Semantic Object Model")[0].id == "e_glossary"
        assert find_by_text(som, "Dot")[0].id == "e_bullets"
        assert find_by_text(som, "zzz_no_match") == []

    def test_finds_compiled_table_captions(self) -> None:
        som = Som(
            som_version="1.0",
            url="https://example.com/table",
            title="Table Page",
            lang="en",
            regions=[
                SomRegion(
                    id="r_main",
                    role=RegionRole.main,
                    elements=[
                        SomElement(
                            id="e_table",
                            role=ElementRole.table,
                            attrs=ElementAttrs(
                                caption="Plans",
                                headers=["Plan", "Price"],
                                rows=[
                                    ["Starter", "$9"],
                                    ["Pro", "$29"],
                                ],
                            ),
                        )
                    ],
                )
            ],
            meta=SomMeta(
                html_bytes=100,
                som_bytes=50,
                element_count=1,
                interactive_count=0,
            ),
        )

        results = find_by_text(som, "plans")
        assert [el.id for el in results] == ["e_table"]
        assert find_by_text(som, "Plans")[0].id == "e_table"
        assert find_by_text(som, "zzz_no_match") == []

    def test_finds_compiled_select_options(self) -> None:
        som = Som(
            som_version="1.0",
            url="https://example.com/form",
            title="Form Page",
            lang="en",
            regions=[
                SomRegion(
                    id="r_main",
                    role=RegionRole.main,
                    elements=[
                        SomElement(
                            id="e_select",
                            role=ElementRole.select,
                            attrs=ElementAttrs(
                                options=[
                                    SelectOption(value="us", text="United States"),
                                    SelectOption(value="ca", text="Canada"),
                                ],
                            ),
                        )
                    ],
                )
            ],
            meta=SomMeta(
                html_bytes=100,
                som_bytes=50,
                element_count=1,
                interactive_count=1,
            ),
        )

        results = find_by_text(som, "united states")
        assert [el.id for el in results] == ["e_select"]
        assert find_by_text(som, "Canada")[0].id == "e_select"
        assert find_by_text(som, "zzz_no_match") == []

    def test_finds_compiled_table_rows(self) -> None:
        som = Som(
            som_version="1.0",
            url="https://example.com/table",
            title="Table Page",
            lang="en",
            regions=[
                SomRegion(
                    id="r_main",
                    role=RegionRole.main,
                    elements=[
                        SomElement(
                            id="e_table",
                            role=ElementRole.table,
                            attrs=ElementAttrs(
                                headers=["Plan", "Price"],
                                rows=[
                                    ["Starter", "$9"],
                                    ["Pro", "$29"],
                                ],
                            ),
                        )
                    ],
                )
            ],
            meta=SomMeta(
                html_bytes=100,
                som_bytes=50,
                element_count=1,
                interactive_count=0,
            ),
        )

        results = find_by_text(som, "starter")
        assert [el.id for el in results] == ["e_table"]
        assert find_by_text(som, "Price")[0].id == "e_table"
        assert find_by_text(som, "zzz_no_match") == []


class TestFlatElements:
    def test_flattens_all_elements(self, sample_som: Som) -> None:
        elements = flat_elements(sample_som)
        ids = [e.id for e in elements]
        assert "e1" in ids
        assert "e8" in ids  # nested
        assert "e9" in ids  # nested
        assert len(ids) == 10

    def test_preserves_order(self, sample_som: Som) -> None:
        elements = flat_elements(sample_som)
        ids = [e.id for e in elements]
        assert ids.index("e7") < ids.index("e8")
        assert ids.index("e8") < ids.index("e9")

    def test_includes_shadow_root_elements_in_order(self) -> None:
        ids = [e.id for e in flat_elements(_shadow_som())]
        assert ids == ["host", "shadow_text", "shadow_button"]


class TestGetTokenEstimate:
    def test_uses_canonical_som_bytes_for_dict(self) -> None:
        som = {
            "som_version": "1.0",
            "url": "fixture",
            "title": "Fixture",
            "regions": [],
            "meta": {
                "html_bytes": 20,
                "som_bytes": 5,
                "element_count": 0,
                "interactive_count": 0,
            },
        }
        assert get_token_estimate(som) == 2

    def test_returns_positive_int(self, sample_som: Som) -> None:
        estimate = get_token_estimate(sample_som)
        assert isinstance(estimate, int)
        assert estimate > 0

    def test_accepts_dict(self) -> None:
        d = {"som_version": "1.0", "title": "Test"}
        estimate = get_token_estimate(d)
        assert estimate > 0

    def test_rounds_legacy_dict_estimates_up(self) -> None:
        # Nine serialized bytes should estimate three tokens, matching metadata-backed SOMs.
        assert get_token_estimate({"a": ""}) == 3

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


class TestGetCompressionRatio:
    def test_uses_source_and_som_bytes(self, sample_som: Som) -> None:
        assert get_compression_ratio(sample_som) == 2.5

    def test_accepts_dict_and_returns_infinity_for_empty_som(self) -> None:
        som = {
            "meta": {"html_bytes": 20, "som_bytes": 0},
        }
        assert get_compression_ratio(som) == float("inf")


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

    def test_element_attrs_preserve_unmodeled_compiled_fields(self) -> None:
        som = Som.model_validate(
            {
                "som_version": "0.1",
                "url": "https://example.test/fr",
                "title": "Accueil",
                "lang": "fr",
                "regions": [
                    {
                        "id": "r_main",
                        "role": "main",
                        "elements": [
                            {
                                "id": "e1",
                                "role": "link",
                                "text": "Accueil",
                                "actions": ["click"],
                                "attrs": {
                                    "href": "/fr",
                                    "lang": "fr",
                                    "autofocus": True,
                                },
                            }
                        ],
                    }
                ],
                "meta": {
                    "html_bytes": 32,
                    "som_bytes": 16,
                    "element_count": 1,
                    "interactive_count": 1,
                },
            }
        )
        attrs = som.regions[0].elements[0].attrs
        assert attrs is not None
        dumped = attrs.model_dump(exclude_none=True)
        assert dumped["href"] == "/fr"
        assert dumped["lang"] == "fr"
        assert dumped["autofocus"] is True

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

    def test_list_item_accepts_compiled_description_list_fields(self) -> None:
        item = ListItem.model_validate(
            {"term": "SOM", "description": "Semantic Object Model"}
        )
        assert item.term == "SOM"
        assert item.description == "Semantic Object Model"
        assert item.text is None
        bullet = ListItem.model_validate({"text": "Dot"})
        assert bullet.text == "Dot"
        assert bullet.term is None
        assert bullet.description is None
        with pytest.raises(Exception):
            ListItem.model_validate({"text": "Dot", "unknown_field": "oops"})
