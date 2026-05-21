from __future__ import annotations

import os
import re
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

from playwright.sync_api import expect, sync_playwright


ROOT = Path(__file__).resolve().parents[2]
PORT = int(os.environ.get("STRUT_STUDIO_TEST_PORT", "1421"))
URL = f"http://127.0.0.1:{PORT}"


def npm_command() -> str:
    return "npm.cmd" if os.name == "nt" else "npm"


def wait_for_server(process: subprocess.Popen[str], timeout_seconds: int = 30) -> None:
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None

    while time.time() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"Studio test server exited before startup with code {process.returncode}")
        try:
            with urllib.request.urlopen(URL, timeout=1) as response:
                if response.status == 200 and process.poll() is None:
                    return
        except Exception as exc:  # noqa: BLE001 - surfaced after timeout.
            last_error = exc
        time.sleep(0.25)

    raise RuntimeError(f"Studio test server did not start: {last_error}")


def run_smoke() -> None:
    output_dir = ROOT / "test-results"
    output_dir.mkdir(exist_ok=True)
    reference_path = output_dir / "reference-bot.svg"
    reference_path.write_text(
        """
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 200">
  <rect width="320" height="200" rx="18" fill="#59c7d7"/>
  <ellipse cx="160" cy="166" rx="56" ry="8" fill="#1f1d3b"/>
  <rect x="102" y="42" width="116" height="82" rx="24" fill="#fffaf0" stroke="#222" stroke-width="8"/>
  <rect x="126" y="66" width="68" height="36" rx="14" fill="#202038"/>
  <path d="M140 84q8-16 16 0M166 84q8-16 16 0M150 98q10 12 24 0" fill="none" stroke="#59c7d7" stroke-width="5" stroke-linecap="round"/>
</svg>
""".strip(),
        encoding="utf-8",
    )

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
            page.reload(wait_until="networkidle")

            expect(page.get_by_role("button", name="Home", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="New chat", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="New project", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Search", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Providers", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Chat only", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Chat + preview", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Editor", exact=True)).to_be_visible()
            expect(page.get_by_role("heading", name="Start a motion project")).to_be_visible()
            expect(page.get_by_role("button", name="Select folder")).to_be_visible()
            expect(page.get_by_role("button", name="Start chat")).to_be_visible()
            expect(page.get_by_text("No project selected")).to_be_visible()
            page.screenshot(path=str(output_dir / "studio-home-redesign.png"), full_page=True)

            page.get_by_role("button", name="New project", exact=True).click()
            page.get_by_label("Project name").fill("Smoke Mascot")
            page.get_by_label("Project location").fill("D:\\StrutSmoke")
            page.get_by_role("button", name="Create project").click()

            activity = page.locator('[data-testid="activity-pill"]')
            expect(activity).to_contain_text("Browser preview project")
            expect(page.get_by_role("button", name="Smoke Mascot", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Project brief now", exact=True)).to_be_visible()

            page.get_by_role("button", name="Search", exact=True).click()
            page.get_by_label("Search projects and chats").fill("smoke")
            expect(page.get_by_role("button", name="Project brief now", exact=True)).to_be_visible()
            page.get_by_label("Search projects and chats").fill("")

            page.get_by_role("button", name="New chat in Smoke Mascot").click()
            expect(page.get_by_role("button", name="New character chat now", exact=True)).to_be_visible()

            page.get_by_role("button", name="Providers", exact=True).click()
            expect(page.get_by_role("heading", name="Providers")).to_be_visible()
            gemini_cli = page.get_by_role("button").filter(has_text="Gemini CLI").first
            expect(gemini_cli).to_be_visible()
            gemini_cli.click()
            page.get_by_role("button", name="Test selected provider").click()
            expect(activity).to_contain_text("Desktop app required for real provider checks")

            page.get_by_role("button", name="BYOK").click()
            page.get_by_label("BYOK provider").select_option("openai")
            page.get_by_label("OpenAI API key").fill("sk-test-strut")
            page.get_by_label("OpenAI model").fill("gpt-5.2")
            page.get_by_role("button", name="Save provider").click()
            expect(activity).to_contain_text("Desktop app required for provider config")
            page.get_by_role("button", name="Test selected provider").click()
            expect(activity).to_contain_text("Desktop app required for real provider checks")
            page.get_by_role("button", name="Local CLI").click()
            gemini_cli.click()

            page.get_by_role("button", name="Chat + preview", exact=True).click()
            preview = page.locator('[data-testid="character-preview"]')
            expect(page.get_by_text("No scene yet").first).to_be_visible()
            expect(preview).not_to_be_visible()
            page.locator('input[aria-label="Attach reference images"]').set_input_files(str(reference_path))
            expect(page.get_by_text("reference-bot.svg")).to_be_visible()
            page.get_by_label("Character prompt").fill("make an owl like Duo from Duolingo")
            page.get_by_role("button", name="Generate").click()
            expect(page.get_by_text("Generation stopped: Error: Desktop app required")).to_be_visible()
            expect(preview).not_to_be_visible()
            page.screenshot(path=str(output_dir / "studio-real-provider-required.png"), full_page=True)

            page.get_by_role("button", name="Editor", exact=True).click()
            expect(page.get_by_text("Project files")).to_be_visible()
            expect(page.get_by_text("starter.strut.json")).to_be_visible()
            expect(page.get_by_text("No scene yet").first).to_be_visible()

            page.get_by_label("Parts").uncheck()
            expect(page.get_by_text("Parts hidden")).to_be_visible()
            page.get_by_label("Parts").check()
            page.screenshot(path=str(output_dir / "studio-editor-smoke.png"), full_page=True)

            page.get_by_role("button", name="Settings", exact=True).click()
            expect(page.get_by_role("heading", name="Settings")).to_be_visible()
            expect(page.get_by_text("Current provider")).to_be_visible()
            expect(page.get_by_label("Generation mode")).to_be_visible()
            page.screenshot(path=str(output_dir / "studio-settings-smoke.png"), full_page=True)
            page.get_by_role("radio", name="Dark").click()
            expect(page.locator("html")).to_have_attribute("data-theme", "dark")
            page.get_by_role("radio", name="Light").click()
            expect(page.locator("html")).to_have_attribute("data-theme", "light")
            page.get_by_role("radio", name="Auto").click()
            expect(page.locator("html")).to_have_attribute("data-theme", "system")

            page.reload(wait_until="networkidle")
            expect(page.get_by_role("button", name="Smoke Mascot", exact=True)).to_be_visible()
            expect(page.get_by_text("Generation stopped: Error: Desktop app required")).to_be_visible()
            expect(page.get_by_text("reference-bot.svg")).to_be_visible()
            expect(page.locator("html")).to_have_attribute("data-theme", "system")
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            expect(preview).not_to_be_visible()

            page.locator('button[aria-label^="Delete chat make an owl"]').click()
            expect(page.get_by_role("heading", name="Start a motion project")).to_be_visible()
            page.get_by_role("button", name="Remove project Smoke Mascot").click()
            expect(page.get_by_role("button", name="Smoke Mascot", exact=True)).not_to_be_visible()

            page.evaluate(
                """
                window.localStorage.setItem("strut-studio-workspace-v4", JSON.stringify({
                  projects: [{
                    id: "project-preview",
                    name: "Preview Project",
                    path: "D:\\\\StrutPreview",
                    chats: [
                      {
                        id: "chat-generated",
                        title: "Generated owl",
                        projectId: "project-preview",
                        updated: "1h",
                        messages: [{ id: 1, role: "assistant", text: "Context Owl is ready." }],
                        references: [],
                        activeState: "wave",
                        document: {
                          id: "doc-context-owl",
                          name: "Context Owl",
                          artboards: [{
                            id: "board-context",
                            name: "ContextOwl",
                            width: 960,
                            height: 540,
                            nodes: [{
                              id: "owl-rig",
                              name: "OwlRig",
                              kind: "group",
                              transform: { translate_x: 0, translate_y: 0, rotate: 0, scale_x: 1, scale_y: 1 },
                              style: { fill: null, stroke: null, stroke_width: 0, opacity: 1, linecap: null, linejoin: null },
                              shape: { type: "none" },
                              children: [
                                {
                                  id: "context-body",
                                  name: "ContextBody",
                                  kind: "ellipse",
                                  transform: { translate_x: 0, translate_y: 0, rotate: 0, scale_x: 1, scale_y: 1 },
                                  style: { fill: "#f6f0df", stroke: "#28241f", stroke_width: 8, opacity: 1, linecap: "round", linejoin: "round" },
                                  shape: { type: "ellipse", cx: 480, cy: 280, rx: 120, ry: 150 },
                                  children: []
                                },
                                {
                                  id: "context-face",
                                  name: "ContextFace",
                                  kind: "rect",
                                  transform: { translate_x: 0, translate_y: 0, rotate: 0, scale_x: 1, scale_y: 1 },
                                  style: { fill: "#29251f", stroke: "#29251f", stroke_width: 4, opacity: 1, linecap: "round", linejoin: "round" },
                                  shape: { type: "rect", x: 405, y: 210, width: 150, height: 92, rx: 28 },
                                  children: []
                                }
                              ]
                            }]
                          }],
                          timelines: [{
                            id: "wave",
                            name: "Wave",
                            duration_ms: 900,
                            tracks: [{
                              target: "owl-rig",
                              property: "translate_y",
                              keyframes: [
                                { time_ms: 0, value: { type: "number", value: 0 }, easing: "ease_in_out" },
                                { time_ms: 450, value: { type: "number", value: -16 }, easing: "ease_out" },
                                { time_ms: 900, value: { type: "number", value: 0 }, easing: "ease_in" }
                              ]
                            }]
                          }],
                          state_machines: [{
                            id: "context-moods",
                            name: "ContextMoods",
                            inputs: [{ name: "complete", kind: "trigger" }],
                            states: ["idle", "wave", "celebrate"],
                            transitions: [{ from: "idle", to: "celebrate", on: "complete", timeline: "wave" }]
                          }],
                          bindings: [{ name: "complete" }],
                          events: [{ name: "level_complete" }]
                        }
                      },
                      {
                        id: "chat-empty",
                        title: "Empty follow-up",
                        projectId: "project-preview",
                        updated: "now",
                        messages: [],
                        references: [],
                        document: null,
                        activeState: "idle"
                      }
                    ]
                  }],
                  activeProjectId: "project-preview",
                  activeChatId: "chat-empty",
                  themeMode: "system"
                }));
                """
            )
            page.reload(wait_until="networkidle")
            page.get_by_role("button", name="Editor", exact=True).click()
            expect(page.get_by_role("button", name="Empty follow-up now")).to_be_visible()
            expect(page.get_by_text("Context Owl / ContextMoods")).to_be_visible()
            expect(page.locator('[data-testid="character-preview"]')).to_be_visible()
            expect(page.get_by_text("ContextBody")).to_be_visible()
            page.get_by_role("button", name="Celebrate", exact=True).click()
            expect(page.get_by_role("button", name="Celebrate", exact=True)).to_have_class(
                re.compile("active")
            )

            page.screenshot(path=str(output_dir / "studio-bot-smoke.png"), full_page=True)
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
    try:
        run_smoke()
    except Exception as exc:  # noqa: BLE001 - command line test should be explicit.
        print(f"studio bot smoke failed: {exc}", file=sys.stderr)
        raise
