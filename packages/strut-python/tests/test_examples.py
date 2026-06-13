from __future__ import annotations

import json
import runpy
import sys
from pathlib import Path
from contextlib import redirect_stdout
from io import StringIO

import pytest

from strut_python import abstract_logo_reveal, icon_badge, loader_progress, mascot_idle, procedural_asset, rolling_dice, ui_microinteraction
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
    assert {"Body", "Head", "LeftEye", "RightEye", "LeftWing", "RightWing", "AmbientHalo"}.issubset(names)
    assert len(names) >= 12
    notes = " ".join(envelope["plan"]["editability"]["notes"]).lower()
    purposes = " ".join(role["purpose"] for role in envelope["plan"]["motionRoles"]).lower()
    timelines = {timeline["name"] for timeline in envelope["plan"]["timelines"]}
    assert {"idle_breathe", "soft_bob", "left_soft_blink", "right_soft_blink", "curious_glance", "gentle_wave"}.issubset(
        timelines
    )
    assert "quiet companion" in purposes
    assert "companion mascot builder" in notes


def test_procedural_asset_uses_prompt_specific_semantic_parts() -> None:
    envelope = procedural_asset("make a cinematic moon portal shimmer for a launch screen").to_envelope()
    plan = envelope["plan"]
    names = {part["name"] for part in plan["parts"]}
    timeline_names = {timeline["name"] for timeline in plan["timelines"]}

    assert plan["subject"]["classification"] == "dynamic_asset"
    assert plan["subject"]["label"] == "Moon Portal Shimmer Launch"
    assert any(name.startswith("Moon Portal Shimmer") for name in names)
    assert len(names) >= 6
    assert names.isdisjoint(FORBIDDEN_MASCOT_ANATOMY)
    assert timeline_names
    assert envelope["operations"][0]["type"] == "group_nodes"


def test_procedural_bird_asset_is_not_a_mascot_template() -> None:
    envelope = envelope_for("custom", "animate a twitter bird taking flight")
    plan = envelope["plan"]
    names = {part["name"] for part in plan["parts"]}

    assert plan["subject"]["classification"] == "bird_icon"
    assert {
        "Twitter Bird Taking Flight Body",
        "Twitter Bird Taking Flight Wing",
        "Twitter Bird Taking Flight Motion Trail",
    }.issubset(names)
    assert names.isdisjoint(FORBIDDEN_MASCOT_ANATOMY)
    assert len(plan["timelines"]) >= 3


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
