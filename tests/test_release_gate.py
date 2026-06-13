from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def load_release_gate():
    spec = importlib.util.spec_from_file_location("release_gate", ROOT / "scripts" / "release_gate.py")
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_release_gate_reports_required_product_surfaces() -> None:
    release_gate = load_release_gate()

    report = release_gate.collect_release_gate(ROOT)
    check_ids = {check["id"] for check in report["checks"]}

    assert {
        "version_consistency",
        "studio_bundle",
        "site_workspace",
        "ci_release_matrix",
        "launch_smokes",
        "download_surface",
    }.issubset(check_ids)
    assert report["targetVersion"] == "1.0.0"
    assert report["currentVersion"]
    assert isinstance(report["ready"], bool)


def test_release_gate_has_release_scripts_and_ci_matrix() -> None:
    release_gate = load_release_gate()

    report = release_gate.collect_release_gate(ROOT)
    checks = {check["id"]: check for check in report["checks"]}

    assert checks["launch_smokes"]["ok"]
    assert checks["ci_release_matrix"]["ok"]


def test_release_gate_blocks_until_version_is_release_ready() -> None:
    release_gate = load_release_gate()

    report = release_gate.collect_release_gate(ROOT)
    blockers = "\n".join(report["blockers"])

    assert "version" in blockers.lower()
