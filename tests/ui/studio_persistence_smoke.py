from __future__ import annotations

import json
import os
import subprocess
import time
import urllib.request
from pathlib import Path

from playwright.sync_api import expect, sync_playwright


ROOT = Path(__file__).resolve().parents[2]
PORT = int(os.environ.get("STRUT_STUDIO_PERSISTENCE_PORT", "1422"))
URL = f"http://127.0.0.1:{PORT}"
SCREENSHOT_DIR = Path(os.environ.get("STRUT_PHASE4_SCREENSHOT_DIR", ROOT / "test-results" / "phase4-persistence"))


def npm_command() -> str:
    return "npm.cmd" if os.name == "nt" else "npm"


def wait_for_server(process: subprocess.Popen[str], timeout_seconds: int = 30) -> None:
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None
    while time.time() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"Studio server exited early with code {process.returncode}")
        try:
            with urllib.request.urlopen(URL, timeout=1) as response:
                if response.status == 200:
                    return
        except Exception as exc:  # noqa: BLE001 - surfaced after timeout.
            last_error = exc
        time.sleep(0.25)
    raise RuntimeError(f"Studio server did not start: {last_error}")


def dice_document() -> dict:
    transform = {"translate_x": 0, "translate_y": 0, "rotate": 0, "scale_x": 1, "scale_y": 1}
    style = {"fill": "#f5f7fb", "stroke": "#111827", "stroke_width": 5, "opacity": 1, "linecap": "round", "linejoin": "round"}

    def node(node_id: str, name: str, kind: str, role: str, shape: dict, patch: dict | None = None) -> dict:
        return {
            "id": node_id,
            "name": name,
            "kind": kind,
            "role": role,
            "transform": dict(transform),
            "style": {**style, **(patch or {})},
            "shape": shape,
            "children": [],
        }

    return {
        "id": "dice-document",
        "name": "Rolling Dice",
        "artboards": [{
            "id": "dice-artboard",
            "name": "Rolling Dice Artboard",
            "width": 960,
            "height": 540,
            "nodes": [{
                "id": "SceneRig",
                "name": "Rolling Dice Rig",
                "kind": "group",
                "role": "scene_rig",
                "transform": dict(transform),
                "style": {"fill": None, "stroke": None, "stroke_width": 0, "opacity": 1, "linecap": "round", "linejoin": "round"},
                "shape": {"type": "none"},
                "children": [
                    node("DieBody", "DieBody", "rect", "volume", {"type": "rect", "x": 378, "y": 174, "width": 210, "height": 210, "rx": 24}),
                    node("FrontFace", "FrontFace", "rect", "front face", {"type": "rect", "x": 402, "y": 214, "width": 168, "height": 146, "rx": 16}, {"fill": "#ffffff"}),
                    node("Pips", "Pips", "path", "number marks", {"type": "path", "d": "M442 252 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0"}, {"fill": None, "stroke_width": 4}),
                    node("SettleShadow", "SettleShadow", "ellipse", "shadow", {"type": "ellipse", "cx": 494, "cy": 414, "rx": 116, "ry": 18}, {"fill": "#1f2937", "stroke": None, "opacity": 0.22}),
                ],
            }],
        }],
        "timelines": [{
            "id": "settle",
            "name": "settle",
            "duration_ms": 900,
            "tracks": [{
                "target": "DieBody",
                "property": "translation.y",
                "keyframes": [
                    {"time_ms": 0, "value": {"type": "number", "value": -18}, "easing": "ease_out"},
                    {"time_ms": 900, "value": {"type": "number", "value": 0}, "easing": "ease_in_out"},
                ],
            }],
        }],
        "state_machines": [{
            "id": "dice-machine",
            "name": "Rolling Dice Motion",
            "inputs": [{"name": "state", "kind": "enum"}],
            "states": ["idle", "settle"],
            "transitions": [{"from": "idle", "to": "settle", "on": "settle", "timeline": "settle"}],
        }],
        "bindings": [{"name": "edit_diebody_fill", "target": "DieBody", "property": "style.fill"}],
        "events": [{"name": "generation_plan_validated", "description": "Seeded validated dice fixture"}],
    }


def workspace_payload(pending_invalid: bool = False) -> dict:
    pending = None
    batches: list[dict] = []
    if pending_invalid:
        pending = {
            "id": "batch-invalid-missing-node",
            "targetId": "MissingPart",
            "targetName": "MissingPart",
            "intent": "invalid missing node test",
            "operationType": "style.patch",
            "affectedProperties": ["style.fill"],
            "createdAt": "2026-06-08T00:00:00.000Z",
            "sourceType": "manual",
            "status": "pending",
            "validationResult": {"ok": True, "message": "stale validation", "validator": "test", "validatedAt": "old"},
            "documentRevisionId": "rev-stale",
            "previousDocumentRevisionId": "rev-stale",
            "prompt": "invalid missing node test",
            "sourceMetadata": {"test": "invalid"},
            "operations": [{
                "id": "op-invalid",
                "type": "set_property",
                "targetId": "MissingPart",
                "targetName": "MissingPart",
                "property": "style.fill",
                "previousValue": "#fff",
                "value": "#000",
            }],
            "updatedAt": "2026-06-08T00:00:00.000Z",
        }
        batches.append(pending)

    return {
        "projects": [{
            "id": "project-dice",
            "name": "Phase 4 Dice Fixture",
            "path": "D:\\StrutPhase4",
            "chats": [{
                "id": "chat-dice",
                "title": "Rolling dice",
                "projectId": "project-dice",
                "updated": "now",
                "messages": [{"id": 1, "role": "assistant", "text": "Dice fixture from validated operations."}],
                "references": [],
                "document": dice_document(),
                "activeState": "settle",
                "selectedNodeId": "DieBody",
                "layerUi": {},
                "pendingOperation": pending,
                "operationBatches": batches,
                "operationHistory": batches,
                "undoStack": [],
                "redoStack": [],
            }],
        }],
        "activeProjectId": "project-dice",
        "activeChatId": "chat-dice",
        "themeMode": "system",
    }


def run_smoke() -> None:
    SCREENSHOT_DIR.mkdir(parents=True, exist_ok=True)
    command = [
        npm_command(),
        "--workspace",
        "@strut/studio",
        "run",
        "dev",
        "--",
        "--host",
        "127.0.0.1",
        "--port",
        str(PORT),
        "--strictPort",
    ]
    process = subprocess.Popen(
        command,
        cwd=ROOT,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )

    try:
        wait_for_server(process)
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(headless=True)
            page = browser.new_page(viewport={"width": 1440, "height": 920})
            page.goto(URL, wait_until="networkidle")
            page.evaluate("window.localStorage.clear()")
            page.evaluate(
                "(payload) => window.localStorage.setItem('strut-studio-workspace-v4', JSON.stringify(payload))",
                workspace_payload(),
            )
            page.reload(wait_until="networkidle")
            expect(page.get_by_role("button", name="Editor", exact=True)).not_to_be_visible()
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            expect(page.get_by_text("Rolling Dice / Rolling Dice Motion")).to_be_visible()
            expect(page.get_by_label("Scene layers rail")).to_be_visible()
            expect(page.get_by_role("button", name="Attach layer DieBody rect")).to_be_visible()
            page.get_by_role("button", name="Attach layer DieBody rect").click()
            expect(page.get_by_text("Layer: DieBody")).to_be_visible()
            page.screenshot(path=str(SCREENSHOT_DIR / "browser-01-dice-reopened.png"), full_page=True)

            expect(page.get_by_text("Ask AI to edit selection")).not_to_be_visible()
            expect(page.get_by_text("Apply operation")).not_to_be_visible()
            expect(page.get_by_text("Reject")).not_to_be_visible()
            page.screenshot(path=str(SCREENSHOT_DIR / "browser-02-ai-first-layers.png"), full_page=True)

            page.get_by_role("button", name="Save project").click()
            expect(page.get_by_test_id("activity-pill")).to_contain_text("Saved browser snapshot")
            page.get_by_role("button", name="Reload").click()
            expect(page.get_by_test_id("activity-pill")).to_contain_text("Reopened browser snapshot")
            expect(page.get_by_role("button", name="Attach layer DieBody rect")).to_be_visible()
            page.screenshot(path=str(SCREENSHOT_DIR / "browser-03-save-reopen.png"), full_page=True)

            page.evaluate(
                "(payload) => window.localStorage.setItem('strut-studio-workspace-v4', JSON.stringify(payload))",
                workspace_payload(pending_invalid=True),
            )
            page.reload(wait_until="networkidle")
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            expect(page.get_by_text("stale validation")).not_to_be_visible()
            expect(page.get_by_role("button", name="Attach layer DieBody rect")).to_be_visible()
            page.screenshot(path=str(SCREENSHOT_DIR / "browser-04-invalid-hidden-from-ai-first-ui.png"), full_page=True)
            browser.close()
    finally:
        stop_process_tree(process)


def stop_process_tree(process: subprocess.Popen[str]) -> None:
    if os.name == "nt":
        subprocess.run(
            ["taskkill", "/PID", str(process.pid), "/T", "/F"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=8)
        return
    process.terminate()
    try:
        process.wait(timeout=8)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=8)


if __name__ == "__main__":
    run_smoke()
