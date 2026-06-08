from __future__ import annotations

import json
import runpy
import sys
from pathlib import Path
from contextlib import redirect_stdout
from io import StringIO

import pytest

from strut_python import abstract_logo_reveal, loader_progress, mascot_idle, rolling_dice
from strut_python.cli import envelope_for


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"


@pytest.mark.parametrize("example", ["dice", "logo", "loader", "mascot"])
def test_examples_emit_generation_plan_and_operations(example: str) -> None:
    envelope = envelope_for(example)

    assert set(envelope) == {"plan", "operations"}
    assert "document" not in envelope
    assert envelope["operations"]
    assert envelope["operations"][0]["type"] == "group_nodes"
    assert envelope["operations"][-1]["type"] == "emit_event"


@pytest.mark.parametrize("example", ["dice", "logo", "loader", "mascot"])
def test_examples_are_deterministic_against_fixtures(example: str) -> None:
    fixture = json.loads((FIXTURES / f"{example}.plan.json").read_text(encoding="utf-8"))

    assert envelope_for(example) == fixture


def test_non_mascot_examples_do_not_emit_mascot_anatomy() -> None:
    forbidden = {"Body", "Head", "Eyes", "Arms", "Legs", "Face", "Smile"}

    for example in ["dice", "logo", "loader"]:
        names = {part["name"] for part in envelope_for(example)["plan"]["parts"]}
        assert names.isdisjoint(forbidden)


def test_mascot_example_uses_anatomy_only_when_subject_is_mascot() -> None:
    envelope = envelope_for("mascot")
    names = {part["name"] for part in envelope["plan"]["parts"]}

    assert envelope["plan"]["subject"]["classification"] == "mascot"
    assert {"Body", "Head", "Eyes"}.issubset(names)


@pytest.mark.parametrize(
    ("builder", "expected"),
    [
        (rolling_dice, "dice"),
        (abstract_logo_reveal, "logo"),
        (loader_progress, "loader"),
        (mascot_idle, "mascot"),
    ],
)
def test_builder_subject_classification(builder, expected: str) -> None:
    assert builder().to_envelope()["plan"]["subject"]["classification"] == expected


@pytest.mark.parametrize("example", ["dice", "logo", "loader", "mascot"])
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
