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

    for target in expected_targets:
        line = next(line for line in context_lines if f'[{target["id"]}]' in line)

        assert f'{target["role"]} "{target["label"]}"' in line
        assert f'[cache_key={target["cache_key"]}]' in line
        if target["enabled"]:
            assert "[enabled]" in line
        else:
            assert f'[{target.get("blocked_reason", "blocked")}]' in line
        if target.get("blocked_reason"):
            assert f'[blocked_reason={target["blocked_reason"]}]' in line
        if target.get("required"):
            assert "[required]" in line
        if target.get("readonly"):
            assert "[readonly]" in line
        if target.get("group"):
            assert f'[group={target["group"]}]' in line
        if target.get("input_type"):
            assert f'[type={target["input_type"]}]' in line
        if target.get("autocomplete"):
            assert f'[autocomplete={target["autocomplete"]}]' in line
        if target.get("inputmode"):
            assert f'[inputmode={target["inputmode"]}]' in line
        if target.get("enterkeyhint"):
            assert f'[enterkeyhint={target["enterkeyhint"]}]' in line
        if target.get("form"):
            assert f'[form={target["form"]}]' in line
        if target.get("list"):
            assert f'[list={target["list"]}]' in line
        for link_key in ("target", "rel", "download"):
            if link_key in target:
                assert f'[{link_key}={target[link_key]}]' in line
        for command_key in (
            "popovertarget",
            "popovertargetaction",
            "commandfor",
            "command",
            "popover",
        ):
            if target.get(command_key):
                assert f'[{command_key}={target[command_key]}]' in line
        if target.get("accesskey"):
            assert f'[accesskey={target["accesskey"]}]' in line
        if target.get("value"):
            assert f'[value={target["value"]}]' in line
        for constraint_key in ("minlength", "maxlength", "pattern"):
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
            "aria_autocomplete",
            "active_descendant",
            "errormessage",
            "keyshortcuts",
            "roledescription",
            "busy",
            "live",
            "atomic",
            "relevant",
            "owns",
            "flowto",
            "details",
        ):
            if state_key in target:
                assert f'[{state_key}={target[state_key]}]' in line
        if target.get("description"):
            assert target["description"] in line

    cache_keys = [target["cache_key"] for target in expected_targets]
    assert len(cache_keys) == len(set(cache_keys))
