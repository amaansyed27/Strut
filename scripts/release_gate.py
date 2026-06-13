from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


TARGET_VERSION = "1.0.0"


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def status(ok: bool, detail: str) -> dict[str, Any]:
    return {"ok": ok, "detail": detail}


def check_version_consistency(root: Path) -> dict[str, Any]:
    package_version = read_json(root / "package.json").get("version", "")
    cargo_text = read_text(root / "Cargo.toml")
    tauri_version = read_json(root / "apps/studio/src-tauri/tauri.conf.json").get("version", "")
    python_text = read_text(root / "packages/strut-python/pyproject.toml")
    cargo_match = re.search(r"(?m)^version\s*=\s*\"([^\"]+)\"", cargo_text)
    python_match = re.search(r"(?m)^version\s*=\s*\"([^\"]+)\"", python_text)
    versions = {
        "package": package_version,
        "cargo": cargo_match.group(1) if cargo_match else "",
        "tauri": tauri_version,
        "strutPython": python_match.group(1) if python_match else "",
    }
    consistent = len(set(versions.values())) == 1 and all(versions.values())
    target_ready = consistent and package_version == TARGET_VERSION
    detail = f"versions={versions}"
    if consistent and not target_ready:
        detail += f"; version must be bumped to {TARGET_VERSION} for v1.0"
    return {"id": "version_consistency", **status(target_ready, detail), "versions": versions}


def check_studio_bundle(root: Path) -> dict[str, Any]:
    config = read_json(root / "apps/studio/src-tauri/tauri.conf.json")
    bundle = config.get("bundle", {})
    icons = bundle.get("icon", [])
    missing_icons = [icon for icon in icons if not (root / "apps/studio/src-tauri" / icon).exists()]
    ok = bool(bundle.get("active")) and bundle.get("targets") == "all" and not missing_icons
    detail = "Tauri bundle active for all targets with icons present" if ok else f"bundle={bundle}; missing_icons={missing_icons}"
    return {"id": "studio_bundle", **status(ok, detail)}


def check_site_workspace(root: Path) -> dict[str, Any]:
    package = read_json(root / "package.json")
    site_package = root / "apps/site/package.json"
    site_main = root / "apps/site/src/main.tsx"
    site_styles = root / "apps/site/src/styles.css"
    workspaces = package.get("workspaces", [])
    ok = "apps/site" in workspaces and site_package.exists() and site_main.exists() and site_styles.exists()
    detail = "site workspace is wired and has a launch page" if ok else "apps/site workspace or launch files missing"
    return {"id": "site_workspace", **status(ok, detail)}


def check_ci_release_matrix(root: Path) -> dict[str, Any]:
    workflow_text = read_text(root / ".github/workflows/ci.yml")
    has_site = "@strut/site" in workflow_text or "test:site" in workflow_text
    has_os_matrix = all(token in workflow_text for token in ["windows-latest", "macos-latest", "ubuntu-latest"])
    has_tauri_build = "studio:build" in workflow_text or "tauri" in workflow_text.lower()
    ok = has_site and has_os_matrix and has_tauri_build
    missing = []
    if not has_site:
        missing.append("site check/smoke")
    if not has_os_matrix:
        missing.append("Windows/macOS/Linux release matrix")
    if not has_tauri_build:
        missing.append("Tauri bundle build")
    return {
        "id": "ci_release_matrix",
        **status(ok, "CI covers release matrix" if ok else f"missing: {', '.join(missing)}"),
    }


def check_launch_smokes(root: Path) -> dict[str, Any]:
    package = read_json(root / "package.json")
    scripts = package.get("scripts", {})
    required = ["check", "test", "test:ui", "test:site", "test:runtime-ui", "test:all", "studio:build"]
    missing = [script for script in required if script not in scripts]
    test_all = scripts.get("test:all", "")
    ok = not missing and all(script in test_all for script in ["test:site", "test:ui", "test:runtime-ui"])
    detail = "launch smoke scripts are wired" if ok else f"missing={missing}; test:all={test_all}"
    return {"id": "launch_smokes", **status(ok, detail)}


def check_download_surface(root: Path) -> dict[str, Any]:
    site = read_text(root / "apps/site/src/main.tsx")
    labels = ["Download for Windows", "Download for macOS", "Download for Linux"]
    missing = [label for label in labels if label not in site]
    ok = not missing
    detail = "download links exist for Windows, macOS, and Linux" if ok else f"missing download labels: {missing}"
    return {"id": "download_surface", **status(ok, detail)}


def collect_release_gate(root: Path) -> dict[str, Any]:
    checks = [
        check_version_consistency(root),
        check_studio_bundle(root),
        check_site_workspace(root),
        check_ci_release_matrix(root),
        check_launch_smokes(root),
        check_download_surface(root),
    ]
    blockers = [f"{check['id']}: {check['detail']}" for check in checks if not check["ok"]]
    version_check = next(check for check in checks if check["id"] == "version_consistency")
    return {
        "targetVersion": TARGET_VERSION,
        "currentVersion": version_check["versions"]["package"],
        "ready": not blockers,
        "checks": checks,
        "blockers": blockers,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Inspect whether Strut is ready for a v1.0 release.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--json", action="store_true", help="Print machine-readable JSON.")
    args = parser.parse_args(argv)

    report = collect_release_gate(args.root.resolve())
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(f"Strut release gate: {'READY' if report['ready'] else 'BLOCKED'}")
        for check in report["checks"]:
            marker = "PASS" if check["ok"] else "FAIL"
            print(f"- {marker} {check['id']}: {check['detail']}")
    return 0 if report["ready"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
