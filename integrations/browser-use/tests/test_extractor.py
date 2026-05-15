import json
from pathlib import Path

from plasmate_browser_use.extractor import PlasmateExtractor


FIXTURE_PATH = (
    Path(__file__).resolve().parents[2]
    / "fixtures"
    / "action-availability.som.json"
)
EXPECTATIONS_PATH = (
    Path(__file__).resolve().parents[2]
    / "fixtures"
    / "action-availability.expected.json"
)


def load_action_availability_fixture():
    return json.loads(FIXTURE_PATH.read_text())


def load_action_availability_expectations():
    return json.loads(EXPECTATIONS_PATH.read_text())["action_targets"]


def test_build_context_surfaces_action_availability():
    extractor = PlasmateExtractor.__new__(PlasmateExtractor)
    som = load_action_availability_fixture()
    expected_targets = load_action_availability_expectations()

    context = extractor._build_context(som)
    context_lines = context.splitlines()
    summary_line = next(line for line in context_lines if line.startswith("Action plan:"))

    assert "fingerprint=plasmate-plan:v1:" in summary_line
    assert "enabled_fingerprint=plasmate-plan:v1:" in summary_line
    assert "enabled=7 disabled=3" in summary_line

    for target in expected_targets:
        line = next(line for line in context_lines if f'[{target["id"]}]' in line)

        assert f'{target["role"]} "{target["label"]}"' in line
        assert f'[cache_key={target["cache_key"]}]' in line
        for provenance_key in ("test_id", "data_action", "data_state"):
            if target.get(provenance_key):
                assert f'[{provenance_key}={target[provenance_key]}]' in line
        if target["enabled"]:
            assert "[enabled]" in line
        else:
            assert f'[{target.get("blocked_reason", "blocked")}]' in line
        if target.get("blocked_reason"):
            assert f'[blocked_reason={target["blocked_reason"]}]' in line
        if target.get("html_id"):
            assert f'[html_id={target["html_id"]}]' in line
        if target.get("required"):
            assert "[required]" in line
        if target.get("readonly"):
            assert "[readonly]" in line
        if target.get("inert"):
            assert "[inert]" in line
        if target.get("group"):
            assert f'[group={target["group"]}]' in line
        if target.get("input_type"):
            assert f'[type={target["input_type"]}]' in line
        if target.get("name"):
            assert f'[name={target["name"]}]' in line
        if target.get("accept"):
            assert f'[accept={target["accept"]}]' in line
        if "capture" in target:
            assert f'[capture={target["capture"]}]' in line
        if "multiple" in target:
            assert f'[multiple={target["multiple"]}]' in line
        if target.get("autocomplete"):
            assert f'[autocomplete={target["autocomplete"]}]' in line
        if target.get("inputmode"):
            assert f'[inputmode={target["inputmode"]}]' in line
        if target.get("enterkeyhint"):
            assert f'[enterkeyhint={target["enterkeyhint"]}]' in line
        if target.get("autocapitalize"):
            assert f'[autocapitalize={target["autocapitalize"]}]' in line
        if target.get("dirname"):
            assert f'[dirname={target["dirname"]}]' in line
        if target.get("dir"):
            assert f'[dir={target["dir"]}]' in line
        if target.get("lang"):
            assert f'[lang={target["lang"]}]' in line
        if target.get("form"):
            assert f'[form={target["form"]}]' in line
        if target.get("list"):
            assert f'[list={target["list"]}]' in line
        for link_key in ("target", "rel", "download"):
            if link_key in target:
                assert f'[{link_key}={target[link_key]}]' in line
        for media_key in ("alt", "src"):
            if media_key in target:
                assert f'[{media_key}={target[media_key]}]' in line
        for command_key in (
            "popovertarget",
            "popovertargetaction",
            "commandfor",
            "command",
            "popover",
            "button_type",
            "formaction",
            "formmethod",
            "formenctype",
            "formtarget",
        ):
            if target.get(command_key):
                assert f'[{command_key}={target[command_key]}]' in line
        if "formnovalidate" in target:
            assert f'[formnovalidate={target["formnovalidate"]}]' in line
        if target.get("accesskey"):
            assert f'[accesskey={target["accesskey"]}]' in line
        for relation_key in ("title", "aria_label", "aria_description", "labelledby", "describedby"):
            if target.get(relation_key):
                assert f'[{relation_key}={target[relation_key]}]' in line
        if "spellcheck" in target:
            assert f'[spellcheck={target["spellcheck"]}]' in line
        if "draggable" in target:
            assert f'[draggable={target["draggable"]}]' in line
        if target.get("value"):
            assert f'[value={target["value"]}]' in line
        for constraint_key in ("minlength", "maxlength", "min", "max", "step", "pattern"):
            if constraint_key in target:
                assert f'[{constraint_key}={target[constraint_key]}]' in line
        if "checked" in target:
            assert f'[checked={target["checked"]}]' in line
        for state_key in (
            "expanded",
            "pressed",
            "selected",
            "current",
            "controls",
            "haspopup",
            "invalid",
            "aria_placeholder",
            "aria_autocomplete",
            "active_descendant",
            "errormessage",
            "keyshortcuts",
            "roledescription",
            "busy",
            "live",
            "modal",
            "atomic",
            "relevant",
            "owns",
            "flowto",
            "details",
            "multiline",
            "multiselectable",
            "orientation",
            "sort",
            "level",
            "posinset",
            "setsize",
            "rowindex",
            "colindex",
            "rowcount",
            "colcount",
            "grabbed",
            "dropeffect",
            "valuemin",
            "valuemax",
            "valuenow",
            "valuetext",
        ):
            if state_key in target:
                assert f'[{state_key}={target[state_key]}]' in line
        if target.get("description"):
            assert target["description"] in line

    cache_keys = [target["cache_key"] for target in expected_targets]
    assert len(cache_keys) == len(set(cache_keys))


def test_extract_action_plan_helpers_support_replay_indexes():
    extractor = PlasmateExtractor.__new__(PlasmateExtractor)
    som = load_action_availability_fixture()
    extractor.extract = lambda _url: som

    enabled_plan = extractor.extract_enabled_action_plan("fixture://actions")
    assert [target["id"] for target in enabled_plan] == [
        "e_upload",
        "e_image_submit",
        "e_plan",
        "e_compact",
        "e_annual",
        "e_quota",
        "e_billing",
    ]

    index = extractor.extract_action_plan_index("fixture://actions")
    assert index["by_id"]["e_save"]["blocked_reason"] == "disabled"
    assert index["by_html_id"]["save-settings"]["id"] == "e_save"

    enabled_index = extractor.extract_action_plan_index(
        "fixture://actions", enabled_only=True
    )
    assert "e_save" not in enabled_index["by_id"]
    assert "save-settings" not in enabled_index["by_html_id"]

    summary = extractor.extract_action_plan_summary("fixture://actions")
    assert summary["total"] == 10
    assert summary["enabled"] == 7
    assert summary["disabled"] == 3
    assert summary["with_cache_key"] == 10
    assert summary["unique_cache_keys"] == 10
    assert summary["duplicate_cache_keys"] == []
    assert summary["with_html_id"] == 3
    assert summary["duplicate_html_ids"] == []
    assert summary["with_test_id"] == 2
    assert summary["with_data_action"] == 1
    assert summary["with_data_state"] == 2
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
    assert summary["fingerprint"].startswith("plasmate-plan:v1:")
    assert extractor.extract_action_plan_fingerprint("fixture://actions") == summary["fingerprint"]
    assert (
        extractor.extract_action_plan_fingerprint(
            "fixture://actions", enabled_only=True
        )
        == summary["enabled_fingerprint"]
    )
