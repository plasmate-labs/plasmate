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
            assert "[disabled]" in line
        if target.get("blocked_reason"):
            assert f'[blocked_reason={target["blocked_reason"]}]' in line
        if target.get("required"):
            assert "[required]" in line
        if target.get("group"):
            assert f'[group={target["group"]}]' in line
        if target.get("input_type"):
            assert f'[type={target["input_type"]}]' in line
        if target.get("description"):
            assert target["description"] in line

    cache_keys = [target["cache_key"] for target in expected_targets]
    assert len(cache_keys) == len(set(cache_keys))
