from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Callable

from .builders import abstract_logo_reveal, icon_badge, loader_progress, mascot_idle, procedural_asset, rolling_dice, ui_microinteraction


BUILDERS: dict[str, Callable[[], object] | None] = {
    "dice": rolling_dice,
    "logo": abstract_logo_reveal,
    "loader": loader_progress,
    "mascot": mascot_idle,
    "ui": ui_microinteraction,
    "icon": icon_badge,
    "badge": icon_badge,
    "custom": None,
    "generic": None,
}


def envelope_for(name: str, instruction: str | None = None) -> dict[str, object]:
    if name in {"custom", "generic"}:
        scene = procedural_asset(instruction or "dynamic asset")
        return scene.to_envelope()
    builder = BUILDERS[name]
    assert builder is not None
    scene = builder()
    return scene.to_envelope()  # type: ignore[no-any-return]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Emit deterministic Strut generation-plan envelopes.")
    parser.add_argument("example", choices=sorted(BUILDERS))
    parser.add_argument("--instruction", help="Instruction used by the procedural custom builder.")
    parser.add_argument("--json", action="store_true", help="Print compact JSON.")
    parser.add_argument("--out", type=Path, help="Write JSON to a file.")
    args = parser.parse_args(argv)

    envelope = envelope_for(args.example, args.instruction)
    payload = json.dumps(envelope, indent=None if args.json else 2, sort_keys=True)
    if args.out:
        args.out.write_text(payload + "\n", encoding="utf-8")
    else:
        print(payload)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
