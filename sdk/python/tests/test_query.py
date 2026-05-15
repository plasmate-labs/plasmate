"""Tests for SOM query helpers."""

import json
from pathlib import Path

import pytest

from plasmate.types import (
    ElementAttrs,
    ElementRole,
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
    find_action_target_by_cache_key,
    find_action_target_by_html_id,
    find_action_target_by_id,
    find_by_id,
    find_by_html_id,
    find_by_role,
    find_by_tag,
    find_by_text,
    find_interactive,
    flat_elements,
    get_action_plan,
    get_action_plan_cache_key,
    get_action_plan_fingerprint,
    get_action_plan_index,
    get_action_plan_summary,
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
                        html_id="main-copy",
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
    def test_finds_source_html_id(self, sample_som: Som) -> None:
        el = find_by_html_id(sample_som, "main-copy")
        assert el is not None
        assert el.id == "e4"

    def test_returns_none_for_missing_html_id(self, sample_som: Som) -> None:
        assert find_by_html_id(sample_som, "missing") is None


class TestFindByTag:
    def test_finds_all_links(self, sample_som: Som) -> None:
        links = find_by_tag(sample_som, "link")
        assert len(links) == 2

    def test_finds_paragraphs_including_nested(self, sample_som: Som) -> None:
        paras = find_by_tag(sample_som, "paragraph")
        assert len(paras) == 3  # e4, e8, e10

    def test_finds_none_for_missing_tag(self, sample_som: Som) -> None:
        assert find_by_tag(sample_som, "table") == []

    def test_finds_shadow_root_roles(self) -> None:
        assert [el.id for el in find_by_tag(_shadow_som(), "paragraph")] == ["shadow_text"]


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
    def test_returns_compact_action_targets(self, sample_som: Som) -> None:
        plan = get_action_plan(sample_som)

        assert plan[0] == {
            "id": "e1",
            "role": "link",
            "actions": ["click"],
            "enabled": True,
            "label": "Home",
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

    def test_finds_action_target_by_cache_key(self, sample_som: Som) -> None:
        target = find_action_target_by_cache_key(
            sample_som, "plasmate-action:v1:5b218ab1"
        )
        assert target is not None
        assert target["id"] == "e9"
        assert find_action_target_by_cache_key(sample_som, "missing") is None

        fixture_dir = REPO_ROOT / "integrations" / "fixtures"
        som = Som.model_validate(
            json.loads((fixture_dir / "action-availability.som.json").read_text())
        )
        expected = json.loads(
            (fixture_dir / "action-availability.expected.json").read_text()
        )["action_targets"]
        disabled_target = expected[2]
        assert disabled_target["id"] == "e_save"
        assert (
            find_action_target_by_cache_key(som, disabled_target["cache_key"])
            == disabled_target
        )
        assert (
            find_action_target_by_cache_key(
                som, disabled_target["cache_key"], enabled_only=True
            )
            is None
        )

    def test_finds_action_targets_by_ids(self, sample_som: Som) -> None:
        target = find_action_target_by_id(sample_som, "e9")
        assert target is not None
        assert target["cache_key"] == "plasmate-action:v1:5b218ab1"
        assert find_action_target_by_id(sample_som, "e3") is None

        fixture_dir = REPO_ROOT / "integrations" / "fixtures"
        som = Som.model_validate(
            json.loads((fixture_dir / "action-availability.som.json").read_text())
        )
        html_target = find_action_target_by_html_id(som, "save-settings")
        assert html_target is not None
        assert html_target["id"] == "e_save"
        assert find_action_target_by_html_id(sample_som, "main-copy") is None
        assert find_action_target_by_id(som, "e_save", enabled_only=True) is None
        assert (
            find_action_target_by_html_id(som, "save-settings", enabled_only=True)
            is None
        )

    def test_returns_enabled_action_plan(self) -> None:
        fixture_dir = REPO_ROOT / "integrations" / "fixtures"
        som = Som.model_validate(
            json.loads((fixture_dir / "action-availability.som.json").read_text())
        )
        expected = json.loads(
            (fixture_dir / "action-availability.expected.json").read_text()
        )["action_targets"]

        assert get_enabled_action_plan(som) == [
            target for target in expected if target["enabled"]
        ]

    def test_returns_action_plan_summary_for_replay_validation(self) -> None:
        fixture_dir = REPO_ROOT / "integrations" / "fixtures"
        som = Som.model_validate(
            json.loads((fixture_dir / "action-availability.som.json").read_text())
        )

        summary = get_action_plan_summary(som)
        assert summary["fingerprint"] == get_action_plan_fingerprint(som)
        assert summary["enabled_fingerprint"] == get_action_plan_fingerprint(
            som, enabled_only=True
        )
        assert summary["fingerprint"] != summary["enabled_fingerprint"]
        assert summary["total"] == 10
        assert summary["enabled"] == 7
        assert summary["disabled"] == 3
        assert summary["with_cache_key"] == 10
        assert summary["unique_cache_keys"] == 10
        assert summary["duplicate_cache_keys"] == []
        assert summary["with_html_id"] == 3
        assert summary["by_role"] == {
            "button": 3,
            "checkbox": 1,
            "link": 1,
            "radio": 1,
            "select": 1,
            "text_input": 3,
        }
        assert summary["blocked_reasons"] == {
            "disabled": 1,
            "inert": 1,
            "readonly": 1,
        }

    def test_indexes_action_plan_for_replay_lookups(self) -> None:
        fixture_dir = REPO_ROOT / "integrations" / "fixtures"
        som = Som.model_validate(
            json.loads((fixture_dir / "action-availability.som.json").read_text())
        )
        expected = json.loads(
            (fixture_dir / "action-availability.expected.json").read_text()
        )["action_targets"]
        index = get_action_plan_index(som)

        assert index["by_id"]["e_save"] == find_action_target_by_id(som, "e_save")
        assert index["by_cache_key"][expected[0]["cache_key"]] == expected[0]
        assert index["by_html_id"]["save-settings"]["id"] == "e_save"

        enabled_index = get_action_plan_index(som, enabled_only=True)
        assert "e_disabled" not in enabled_index["by_id"]
        assert "disabled-control" not in enabled_index["by_html_id"]


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

    def test_finds_shadow_root_text(self) -> None:
        results = find_by_text(_shadow_som(), "inside shadow")
        assert len(results) == 1
        assert results[0].id == "shadow_text"

    def test_finds_by_label_text(self, sample_som: Som) -> None:
        results = find_by_text(sample_som, "email")
        assert [result.id for result in results] == ["e9"]

    def test_exact_match_is_case_sensitive_and_checks_labels(self, sample_som: Som) -> None:
        assert [result.id for result in find_by_text(sample_som, "Email", exact=True)] == [
            "e9"
        ]
        assert find_by_text(sample_som, "email", exact=True) == []

    def test_no_match_returns_empty(self, sample_som: Som) -> None:
        assert find_by_text(sample_som, "zzz_no_match") == []


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
    def test_returns_positive_int(self, sample_som: Som) -> None:
        estimate = get_token_estimate(sample_som)
        assert isinstance(estimate, int)
        assert estimate > 0

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
