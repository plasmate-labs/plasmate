import json
from pathlib import Path

from plasmate_browser_use.extractor import PlasmateExtractor


FIXTURE_PATH = (
    Path(__file__).resolve().parents[2]
    / "fixtures"
    / "action-availability.som.json"
)


def load_action_availability_fixture():
    return json.loads(FIXTURE_PATH.read_text())


def test_build_context_surfaces_action_availability():
    extractor = PlasmateExtractor.__new__(PlasmateExtractor)
    som = load_action_availability_fixture()

    context = extractor._build_context(som)

    assert '[e_email] text_input "Work email" (type) [enabled] [required] [group=Account] [type=email]' in context
    assert '[e_save] button "Save" (click) [disabled] [blocked_reason=disabled]' in context
    assert '[e_plan] select "Plan" (select) [enabled] [required] [group=Billing]' in context
    assert "Unavailable until required fields are complete" in context
