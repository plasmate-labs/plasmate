import json
from pathlib import Path

from langchain_plasmate.som_output import som_to_text


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


def test_som_to_text_surfaces_interactive_state():
    som = load_action_availability_fixture()
    expected_targets = load_action_availability_expectations()

    text = som_to_text(som)

    assert '[e_email] input(email) "Work email"' in text

    for target in expected_targets:
        line = next(line for line in text.splitlines() if f'[{target["id"]}]' in line)

        assert f'[cache_key={target["cache_key"]}]' in line
        if target["enabled"]:
            assert "[enabled]" in line
        else:
            assert "[disabled]" in line
        if target.get("blocked_reason"):
            assert f'[blocked_reason={target["blocked_reason"]}]' in line
        if target.get("required"):
            assert "[required]" in line
        if target.get("value"):
            assert f'[value="{target["value"]}"]' in line
        if "checked" in target:
            assert f'[checked="{target["checked"]}"]' in line
        for state_key in ("expanded", "pressed", "selected"):
            if state_key in target:
                assert f'[{state_key}="{target[state_key]}"]' in line
        if target.get("group"):
            assert f'[group="{target["group"]}"]' in line
        if target.get("description"):
            assert f'[description="{target["description"]}"]' in line

    cache_keys = [target["cache_key"] for target in expected_targets]
    assert len(cache_keys) == len(set(cache_keys))
