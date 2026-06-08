from __future__ import annotations

import json
import runpy
import sys
from pathlib import Path
from contextlib import redirect_stdout
from io import StringIO

import pytest

from strut_python import abstract_logo_reveal, icon_badge, loader_progress, mascot_idle, rolling_dice, ui_microinteraction
from strut_python.cli import envelope_for


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
GALLERY_EXAMPLES = ["dice", "logo", "loader", "mascot", "ui", "icon"]
NON_MASCOT_EXAMPLES = ["dice", "logo", "loader", "ui", "icon"]
FORBIDDEN_MASCOT_ANATOMY = {"Body", "Head", "Eyes", "Arms", "Legs", "Face", "Smile"}


@pytest.mark.parametrize("example", GALLERY_EXAMPLES)
def test_examples_emit_generation_plan_and_operations(example: str) -> None:
    envelope = envelope_for(example)

    assert set(envelope) == {"plan", "operations"}
    assert "document" not in envelope
    assert envelope["operations"]
    assert envelope["operations"][0]["type"] == "group_nodes"
    assert envelope["operations"][-1]["type"] == "emit_event"


@pytest.mark.parametrize("example", GALLERY_EXAMPLES)
def test_examples_are_deterministic_against_fixtures(example: str) -> None:
    fixture = json.loads((FIXTURES / f"{example}.plan.json").read_text(encoding="utf-8"))

    assert envelope_for(example) == fixture


def test_non_mascot_examples_do_not_emit_mascot_anatomy() -> None:
    for example in NON_MASCOT_EXAMPLES:
        names = {part["name"] for part in envelope_for(example)["plan"]["parts"]}
        assert names.isdisjoint(FORBIDDEN_MASCOT_ANATOMY)


def test_mascot_example_uses_anatomy_only_when_subject_is_mascot() -> None:
    envelope = envelope_for("mascot")
    names = {part["name"] for part in envelope["plan"]["parts"]}

    assert envelope["plan"]["subject"]["classification"] == "mascot"
    assert {"Body", "Head", "Eyes"}.issubset(names)
    notes = " ".join(envelope["plan"]["editability"]["notes"]).lower()
    purposes = " ".join(role["purpose"] for role in envelope["plan"]["motionRoles"]).lower()
    assert "quiet" in purposes
    assert "anatomy is present because subject is mascot" in notes


@pytest.mark.parametrize(
    ("builder", "expected"),
    [
        (rolling_dice, "dice"),
        (abstract_logo_reveal, "logo"),
        (loader_progress, "loader"),
        (mascot_idle, "mascot"),
        (ui_microinteraction, "ui"),
        (icon_badge, "badge"),
    ],
)
def test_builder_subject_classification(builder, expected: str) -> None:
    assert builder().to_envelope()["plan"]["subject"]["classification"] == expected


@pytest.mark.parametrize("example", GALLERY_EXAMPLES)
def test_examples_have_semantic_editable_parts_and_named_timelines(example: str) -> None:
    envelope = envelope_for(example)
    plan = envelope["plan"]
    part_names = [part["name"] for part in plan["parts"]]
    timeline_names = [timeline["name"] for timeline in plan["timelines"]]

    assert len(part_names) >= 5
    assert len(set(part_names)) == len(part_names)
    assert all(name and not name.startswith("Part ") for name in part_names)
    assert all(timeline_names)
    assert set(plan["editability"]["editableParts"]) == {part["id"] for part in plan["parts"]}


@pytest.mark.parametrize("example", GALLERY_EXAMPLES)
def test_example_scripts_print_json(example: str) -> None:
    script = ROOT / "examples" / f"{example}.py"
    stdout = StringIO()
    previous_argv = sys.argv
    sys.argv = [str(script), "--json"]
    try:
        with pytest.raises(SystemExit) as exit_info, redirect_stdout(stdout):
            runpy.run_path(str(script), run_name="__main__")
    finally:
        sys.argv = previous_argv

    assert exit_info.value.code == 0
    assert json.loads(stdout.getvalue()) == envelope_for(example)
