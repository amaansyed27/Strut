from __future__ import annotations

import os
import subprocess
import tempfile
import time
import urllib.request
from pathlib import Path

from playwright.sync_api import expect, sync_playwright


ROOT = Path(__file__).resolve().parents[2]
CDP_PORT = int(os.environ.get("STRUT_TAURI_CDP_PORT", "9333"))
CDP_URL = f"http://127.0.0.1:{CDP_PORT}"
SCREENSHOT_DIR = Path(os.environ.get("STRUT_PHASE4_TAURI_SCREENSHOT_DIR", ROOT / "test-results" / "phase4-tauri"))


IDS = {
    "doc": "00000000-0000-0000-0000-000000000401",
    "artboard": "00000000-0000-0000-0000-000000000402",
    "rig": "00000000-0000-0000-0000-000000000403",
    "body": "00000000-0000-0000-0000-000000000404",
    "face": "00000000-0000-0000-0000-000000000405",
    "pips": "00000000-0000-0000-0000-000000000406",
    "shadow": "00000000-0000-0000-0000-000000000407",
    "timeline": "00000000-0000-0000-0000-000000000408",
    "machine": "00000000-0000-0000-0000-000000000409",
}


def npm_command() -> str:
    return "npm.cmd" if os.name == "nt" else "npm"


def wait_for_cdp(timeout_seconds: int = 180) -> None:
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None
    while time.time() < deadline:
        try:
            with urllib.request.urlopen(f"{CDP_URL}/json/version", timeout=1) as response:
                if response.status == 200:
                    return
        except Exception as exc:  # noqa: BLE001 - surfaced after timeout.
            last_error = exc
        time.sleep(0.5)
    raise RuntimeError(f"Tauri WebView2 CDP did not open: {last_error}")


def launch_tauri() -> subprocess.Popen[str]:
    env = os.environ.copy()
    env["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"] = f"--remote-debugging-port={CDP_PORT}"
    return subprocess.Popen(
        [npm_command(), "--workspace", "@strut/studio", "run", "tauri", "--", "dev"],
        cwd=ROOT,
        env=env,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )


def stop_process_tree(process: subprocess.Popen[str]) -> None:
    if os.name == "nt":
        subprocess.run(
            ["taskkill", "/PID", str(process.pid), "/T", "/F"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    else:
        process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=10)


def dice_document() -> dict:
    transform = {"translate_x": 0, "translate_y": 0, "rotate": 0, "scale_x": 1, "scale_y": 1}
    base_style = {"fill": "#f5f7fb", "stroke": "#111827", "stroke_width": 5, "opacity": 1, "linecap": "round", "linejoin": "round"}

    def node(node_id: str, name: str, kind: str, role: str, shape: dict, patch: dict | None = None) -> dict:
        return {
            "id": IDS[node_id],
            "name": name,
            "kind": kind,
            "role": role,
            "transform": dict(transform),
            "style": {**base_style, **(patch or {})},
            "shape": shape,
            "children": [],
        }

    return {
        "id": IDS["doc"],
        "name": "Rolling Dice Native",
        "artboards": [{
            "id": IDS["artboard"],
            "name": "Rolling Dice Native Artboard",
            "width": 960,
            "height": 540,
            "nodes": [{
                "id": IDS["rig"],
                "name": "Rolling Dice Rig",
                "kind": "group",
                "role": "scene_rig",
                "transform": dict(transform),
                "style": {"fill": None, "stroke": None, "stroke_width": 0, "opacity": 1, "linecap": "round", "linejoin": "round"},
                "shape": {"type": "none"},
                "children": [
                    node("body", "DieBody", "rect", "volume", {"type": "rect", "x": 378, "y": 174, "width": 210, "height": 210, "rx": 24}),
                    node("face", "FrontFace", "rect", "front face", {"type": "rect", "x": 402, "y": 214, "width": 168, "height": 146, "rx": 16}, {"fill": "#ffffff"}),
                    node("pips", "Pips", "path", "number marks", {"type": "path", "d": "M442 252 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0"}, {"fill": None, "stroke_width": 4}),
                    node("shadow", "SettleShadow", "ellipse", "shadow", {"type": "ellipse", "cx": 494, "cy": 414, "rx": 116, "ry": 18}, {"fill": "#1f2937", "stroke": None, "opacity": 0.22}),
                ],
            }],
        }],
        "timelines": [{
            "id": IDS["timeline"],
            "name": "settle",
            "duration_ms": 900,
            "tracks": [{
                "target": IDS["body"],
                "property": "translation.y",
                "keyframes": [
                    {"time_ms": 0, "value": {"type": "number", "value": -18}, "easing": "ease_out"},
                    {"time_ms": 900, "value": {"type": "number", "value": 0}, "easing": "ease_in_out"},
                ],
            }],
        }],
        "state_machines": [{
            "id": IDS["machine"],
            "name": "Rolling Dice Native Motion",
            "inputs": [{"name": "state", "kind": "enum"}],
            "states": ["idle", "settle"],
            "transitions": [{"from": "idle", "to": "settle", "on": "settle", "timeline": "settle"}],
        }],
        "bindings": [{"name": "edit_diebody_fill", "target": IDS["body"], "property": "style.fill"}],
        "events": [{"name": "generation_plan_validated", "description": "Seeded native dice fixture"}],
    }


def workspace_payload(project_path: str, with_document: bool = True) -> dict:
    return {
        "projects": [{
            "id": "project-native-dice",
            "name": "Native Phase 4 Dice",
            "path": project_path,
            "chats": [{
                "id": "chat-native-dice",
                "title": "Native rolling dice",
                "projectId": "project-native-dice",
                "updated": "now",
                "messages": [{"id": 1, "role": "assistant", "text": "Native dice fixture ready."}],
                "references": [],
                "document": dice_document() if with_document else None,
                "activeState": "settle",
                "selectedNodeId": IDS["body"] if with_document else None,
                "layerUi": {},
                "pendingOperation": None,
                "operationBatches": [],
                "operationHistory": [],
                "undoStack": [],
                "redoStack": [],
            }],
        }],
        "activeProjectId": "project-native-dice",
        "activeChatId": "chat-native-dice",
        "themeMode": "system",
    }


def connect_page(playwright):
    browser = playwright.chromium.connect_over_cdp(CDP_URL)
    context = browser.contexts[0]
    page = context.pages[0]
    page.wait_for_load_state("networkidle")
    return browser, page


def seed_workspace(page, payload: dict) -> None:
    page.evaluate("window.localStorage.clear()")
    page.evaluate(
        "(payload) => window.localStorage.setItem('strut-studio-workspace-v4', JSON.stringify(payload))",
        payload,
    )
    page.reload(wait_until="networkidle")


def run_smoke() -> None:
    SCREENSHOT_DIR.mkdir(parents=True, exist_ok=True)
    project_path = tempfile.mkdtemp(prefix="strut-phase4-native-")

    with sync_playwright() as playwright:
        process = launch_tauri()
        try:
            wait_for_cdp()
            browser, page = connect_page(playwright)
            seed_workspace(page, workspace_payload(project_path, with_document=True))
            expect(page.get_by_role("button", name="Editor", exact=True)).not_to_be_visible()
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            expect(page.get_by_text("Rolling Dice Native / Rolling Dice Native Motion")).to_be_visible()
            expect(page.get_by_role("button", name="Attach layer DieBody rect")).to_be_visible()
            page.get_by_role("button", name="Attach layer DieBody rect").click()
            expect(page.get_by_text("Layer: DieBody")).to_be_visible()

            expect(page.get_by_text("Ask AI to edit selection")).not_to_be_visible()
            expect(page.get_by_text("Apply operation")).not_to_be_visible()
            page.get_by_role("button", name="Save project").click()
            expect(page.get_by_test_id("activity-pill")).to_contain_text("Saved scenes/main.strut")
            page.screenshot(path=str(SCREENSHOT_DIR / "tauri-01-save-ai-first-layer.png"), full_page=True)
            browser.close()
        finally:
            stop_process_tree(process)

        process = launch_tauri()
        try:
            wait_for_cdp()
            browser, page = connect_page(playwright)
            seed_workspace(page, workspace_payload(project_path, with_document=False))
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            page.get_by_role("button", name="Reload").click()
            expect(page.get_by_test_id("activity-pill")).to_contain_text("Loaded scenes/main.strut")
            expect(page.get_by_text("Rolling Dice Native / Rolling Dice Native Motion")).to_be_visible()
            expect(page.get_by_role("button", name="Attach layer DieBody rect")).to_be_visible()
            page.screenshot(path=str(SCREENSHOT_DIR / "tauri-02-reopened-history.png"), full_page=True)

            expect(page.get_by_role("button", name="Undo")).to_be_disabled()
            expect(page.get_by_role("button", name="Redo")).to_be_disabled()
            page.screenshot(path=str(SCREENSHOT_DIR / "tauri-03-ai-first-no-manual-history.png"), full_page=True)
            browser.close()
        finally:
            stop_process_tree(process)

    assert (Path(project_path) / "scenes" / "main.strut").exists()
    assert (Path(project_path) / "operations" / "operation-batches.json").exists()


if __name__ == "__main__":
    run_smoke()
