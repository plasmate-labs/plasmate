import json
from pathlib import Path

from langchain_plasmate.som_output import som_to_text


FIXTURE_PATH = (
    Path(__file__).resolve().parents[2]
    / "fixtures"
    / "action-availability.som.json"
)


def load_action_availability_fixture():
    return json.loads(FIXTURE_PATH.read_text())


def test_som_to_text_surfaces_interactive_state():
    som = load_action_availability_fixture()

    text = som_to_text(som)

    assert '[e_email] input(email) "Work email"' in text
    assert "[enabled]" in text
    assert "[disabled]" in text
    assert "[blocked_reason=disabled]" in text
    assert "[required]" in text
    assert '[group="Account"]' in text
    assert '[description="Use your work email"]' in text
