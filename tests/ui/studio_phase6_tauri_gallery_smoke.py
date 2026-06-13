from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

os.environ.setdefault("STRUT_TAURI_CDP_PORT", "9334")

from playwright.sync_api import expect, sync_playwright
from studio_tauri_persistence_smoke import ROOT, connect_page, launch_tauri, stop_process_tree, wait_for_cdp


OUTPUT_DIR = Path(os.environ.get("STRUT_PHASE6_TAURI_SCREENSHOT_DIR", ROOT / "test-results" / "phase-6-tauri"))
STRUT_EXE = ROOT / "target" / "debug" / ("strut.exe" if os.name == "nt" else "strut")


def run_json(args: list[str]) -> dict:
    completed = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed: {' '.join(args)}\nstdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    return json.loads(completed.stdout)


def ensure_cli() -> None:
    if STRUT_EXE.exists():
        return
    subprocess.run(["cargo", "build", "-p", "strut-cli"], cwd=ROOT, check=True)


def create_cli_generated_project() -> Path:
    ensure_cli()
    root = Path(tempfile.mkdtemp(prefix="strut-phase6-tauri-"))
    (root / "scenes").mkdir()
    (root / "operations").mkdir()
    (root / "ui").mkdir()
    shutil.copy(ROOT / "samples" / "login-button.strut", root / "scenes" / "main.strut")
    (root / "strut.project.json").write_text(
        json.dumps({"name": "Phase 6 Native Icon Badge", "mainScene": "scenes/main.strut"}, indent=2),
        encoding="utf-8",
    )
    (root / "operations" / "operation-batches.json").write_text("[]", encoding="utf-8")
    (root / "ui" / "studio-state.json").write_text(
        json.dumps({"activeState": "success", "selectedNodeId": None, "layerUi": {}}, indent=2),
        encoding="utf-8",
    )
    plan = run_json([str(STRUT_EXE), "sprite", "plan", "make a success icon badge animation", "--json", "--dry-run", "--explain"])
    plan_path = root / "icon-badge.plan.json"
    plan_path.write_text(json.dumps(plan, indent=2), encoding="utf-8")
    run_json([str(STRUT_EXE), "patch", "--scene", str(root / "scenes" / "main.strut"), "--from", str(plan_path), "--json"])
    run_json([str(STRUT_EXE), "verify", str(root / "scenes" / "main.strut"), "--json"])
    return root


def workspace_payload(project_path: Path) -> dict:
    return {
        "projects": [
            {
                "id": "project-phase6-native",
                "name": "Phase 6 Native Icon Badge",
                "path": str(project_path),
                "chats": [
                    {
                        "id": "chat-phase6-native",
                        "title": "Native icon badge",
                        "projectId": "project-phase6-native",
                        "updated": "now",
                        "messages": [{"id": 1, "role": "assistant", "text": "CLI-generated icon badge project ready."}],
                        "references": [],
                        "document": None,
                        "activeState": "success",
                        "selectedNodeId": None,
                        "layerUi": {},
                        "pendingOperation": None,
                        "operationBatches": [],
                        "operationHistory": [],
                        "undoStack": [],
                        "redoStack": [],
                    }
                ],
            }
        ],
        "activeProjectId": "project-phase6-native",
        "activeChatId": "chat-phase6-native",
        "themeMode": "light",
    }


def seed_workspace(page, payload: dict) -> None:
    page.evaluate("window.localStorage.clear()")
    page.evaluate(
        "(payload) => window.localStorage.setItem('strut-studio-workspace-v4', JSON.stringify(payload))",
        payload,
    )
    page.reload(wait_until="networkidle")


def run_smoke() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    project_path = create_cli_generated_project()
    with sync_playwright() as playwright:
        process = launch_tauri()
        try:
            wait_for_cdp()
            browser, page = connect_page(playwright)
            errors: list[str] = []
            page.on("console", lambda msg: errors.append(msg.text) if msg.type == "error" else None)
            page.on("pageerror", lambda exc: errors.append(str(exc)))
            seed_workspace(page, workspace_payload(project_path))
            expect(page.get_by_role("button", name="Editor", exact=True)).not_to_be_visible()
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            page.get_by_role("button", name="Reload").click()
            expect(page.get_by_test_id("activity-pill")).to_contain_text("Loaded scenes/main.strut")
            expect(page.get_by_role("button", name="Attach layer BadgePlate ellipse")).to_be_visible()
            page.get_by_role("button", name="Attach layer BadgePlate ellipse").click()
            expect(page.get_by_text("badge base")).to_be_visible()
            expect(page.get_by_text("Layer: BadgePlate")).to_be_visible()
            expect(page.get_by_role("button", name="Attach layer Head ellipse")).not_to_be_visible()
            page.screenshot(path=str(OUTPUT_DIR / "tauri-phase6-cli-icon-badge.png"), full_page=True)
            assert not errors, f"native console errors observed: {errors}"
            browser.close()
        finally:
            stop_process_tree(process)


if __name__ == "__main__":
    try:
        run_smoke()
    except Exception as exc:  # noqa: BLE001 - command line test should be explicit.
        print(f"studio phase 6 tauri gallery smoke failed: {exc}", file=sys.stderr)
        raise
