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
            expect(page.get_by_test_id("workspace-title")).to_contain_text("Home")
            page.screenshot(path=str(output_dir / "studio-home-redesign.png"), full_page=True)

            page.get_by_role("button", name="New project", exact=True).click()
            page.get_by_label("Project name").fill("Smoke Mascot")
            page.get_by_label("Project location").fill("D:\\StrutSmoke")
            page.get_by_role("button", name="Create project").click()

            activity = page.locator('[data-testid="activity-pill"]')
            expect(activity).to_contain_text("Browser preview project")
            expect(page.get_by_role("button", name="Smoke Mascot", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Project brief now", exact=True)).to_be_visible()
            expect(page.locator(".workspace-top")).not_to_contain_text("No scene")
            expect(page.locator(".workspace-top")).not_to_contain_text("batches")
            page.get_by_role("button", name="Project options for Smoke Mascot").click()
            expect(page.get_by_role("menuitem", name="Pin project")).to_be_visible()
            expect(page.get_by_role("menuitem", name="Rename project")).to_be_visible()
            expect(page.get_by_role("menuitem", name="Delete project")).to_be_visible()
            page.get_by_role("menuitem", name="Open in Explorer").click()
            expect(activity).to_contain_text("Desktop app required to open project folder")
            page.get_by_role("button", name="Project options for Smoke Mascot").click()
            page.get_by_role("menuitem", name="Pin project").click()
            expect(page.get_by_text("Pinned")).to_be_visible()
            expect(page.get_by_role("button", name="Pinned project Smoke Mascot")).to_be_visible()

            page.get_by_role("button", name="Search", exact=True).click()
            page.get_by_label("Search projects and chats").fill("smoke")
            expect(page.get_by_role("button", name="Project brief now", exact=True)).to_be_visible()
            page.get_by_label("Search projects and chats").fill("")

            page.get_by_role("button", name="New chat in Smoke Mascot").click()
            expect(page.get_by_role("button", name="New motion chat now", exact=True)).to_be_visible()
            page.get_by_role("button", name="Chat options for New motion chat").click()
            expect(page.get_by_role("menuitem", name="Pin chat")).to_be_visible()
            expect(page.get_by_role("menuitem", name="Rename chat")).to_be_visible()
            expect(page.get_by_role("menuitem", name="Delete chat")).to_be_visible()
            page.get_by_role("menuitem", name="Pin chat").click()
            expect(page.get_by_role("button", name="Pinned chat New motion chat")).to_be_visible()

            page.get_by_role("button", name="Providers", exact=True).click()
            expect(page.get_by_role("heading", name="Providers")).to_be_visible()
            gemini_cli = page.get_by_role("button").filter(has_text="Gemini CLI").first
            expect(gemini_cli).to_be_visible()
            gemini_cli.click()
            expect(page.get_by_test_id("selected-provider-summary")).to_contain_text("Gemini CLI")
            expect(page.get_by_test_id("selected-provider-summary")).to_contain_text("Selected provider")
            page.get_by_role("button", name="Test selected provider").click()
            expect(activity).to_contain_text("Desktop app required for real provider checks")

            page.get_by_role("button", name="BYOK").click()
            page.get_by_label("BYOK provider").select_option("openai")
            expect(page.get_by_test_id("selected-provider-summary")).to_contain_text("OpenAI")
            page.get_by_label("OpenAI API key").fill("sk-test-strut")
            page.get_by_label("OpenAI model").fill("gpt-5.2")
            page.get_by_role("button", name="Save provider").click()
            expect(activity).to_contain_text("Desktop app required for provider config")
            page.get_by_role("button", name="Test selected provider").click()
            expect(activity).to_contain_text("Desktop app required for real provider checks")
            page.get_by_role("button", name="Local CLI").click()
            gemini_cli.click()
            expect(page.get_by_test_id("selected-provider-summary")).to_contain_text("Gemini CLI")

            page.get_by_role("button", name="Chat + preview", exact=True).click()
            expect(page.get_by_test_id("workspace-title")).to_contain_text("New motion chat")
            expect(page.get_by_role("button", name="Title options for New motion chat")).to_be_visible()
            expect(page.get_by_role("button", name="Provider Gemini CLI", exact=True)).to_be_visible()
            expect(page.get_by_label("Composer tools")).to_be_visible()
            expect(page.get_by_role("button", name="Reload")).to_be_visible()
            expect(page.get_by_role("button", name="Undo")).to_be_visible()
            expect(page.get_by_role("button", name="Redo")).to_be_visible()
            expect(page.get_by_text("Provider: Gemini CLI")).not_to_be_visible()
            preview = page.locator('[data-testid="character-preview"]')
            expect(page.get_by_text("No scene yet").first).to_be_visible()
            expect(preview).not_to_be_visible()
            page.get_by_label("Motion prompt").fill("who are you?")
            page.get_by_role("button", name="Generate").click()
            expect(page.locator(".message.assistant .markdown-response").filter(has_text="I'm Strut's animation design assistant")).to_be_visible()
            expect(page.get_by_text("Assistant Identity Badge")).not_to_be_visible()
            page.locator('input[aria-label="Attach reference images"]').set_input_files(str(reference_path))
            expect(page.get_by_text("reference-bot.svg")).to_be_visible()
            page.get_by_label("Motion prompt").fill("make an owl like Duo from Duolingo")
            page.get_by_role("button", name="Generate").click()
            expect(page.locator(".message.user .message-text").filter(has_text="make an owl like Duo")).to_be_visible()
            expect(page.locator(".message.assistant .markdown-response").filter(has_text="Generation stopped")).to_be_visible()
            expect(page.get_by_text("Generation stopped").first).to_be_visible()
            expect(page.get_by_text("Provider: Gemini CLI").first).to_be_visible()
            expect(page.get_by_text("Error: Desktop app required").first).to_be_visible()
            expect(preview).not_to_be_visible()
            page.screenshot(path=str(output_dir / "studio-real-provider-required.png"), full_page=True)

            page.get_by_role("button", name="Editor", exact=True).click()
            expect(page.get_by_label("AI editor shell")).to_be_visible()
            expect(page.get_by_label("AI edit rail")).to_be_visible()
            expect(page.get_by_test_id("selection-context")).to_contain_text("No selection")
            expect(page.get_by_role("button", name="Ask AI to edit selection")).to_be_disabled()
            expect(page.get_by_text("No operation staged")).to_be_visible()
            expect(page.get_by_role("button", name="Apply operation")).to_be_disabled()
            expect(page.get_by_role("button", name="Reject")).to_be_disabled()
            expect(page.get_by_label("Project files and scene layers")).to_be_visible()
            expect(page.get_by_text("Scene layers")).to_be_visible()
            expect(page.get_by_text("main.strut")).to_be_visible()
            expect(page.get_by_text("operation-batches.json")).to_be_visible()
            expect(page.get_by_text("No scene yet").first).to_be_visible()
            expect(page.get_by_text("No editable layers yet.")).to_be_visible()

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
            expect(page.get_by_text("Generation stopped").first).to_be_visible()
            expect(page.get_by_text("Provider: Gemini CLI").first).to_be_visible()
            expect(page.get_by_text("reference-bot.svg")).to_be_visible()
            expect(page.locator("html")).to_have_attribute("data-theme", "system")
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            expect(preview).not_to_be_visible()

            page.get_by_role("button", name=re.compile(r"Title options for")).click()
            page.get_by_role("menuitem", name="Delete chat").click()
            expect(page.get_by_role("heading", name="Start a motion project")).to_be_visible()
            page.get_by_role("button", name="Project options for Smoke Mascot").click()
            page.get_by_role("menuitem", name="Delete project").click()
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
            page.get_by_role("button", name="ContextBody ellipse").click()
            expect(page.get_by_test_id("selection-context")).to_contain_text("ContextBody")
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("context-body")
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("Translate X")
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("Fill")
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("No timeline tracks target this part.")
            expect(page.get_by_text("Preview selection is bound to the semantic scene node.")).to_be_visible()
            expect(page.get_by_role("button", name="Ask AI to edit selection")).to_be_enabled()

            page.locator('[data-node-id="context-face"]').click()
            expect(page.get_by_test_id("selection-context")).to_contain_text("ContextFace")
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("context-face")
            page.get_by_role("button", name="ContextBody ellipse").click()
            expect(page.get_by_test_id("selection-context")).to_contain_text("ContextBody")
            page.get_by_role("button", name="Hide ContextBody").click()
            expect(page.get_by_test_id("selection-context")).to_contain_text("No selection")
            expect(page.locator('[data-node-id="context-body"]')).not_to_be_visible()
            page.get_by_role("button", name="Show ContextBody").click()
            page.get_by_role("button", name="Lock ContextBody").click()
            page.get_by_role("button", name="ContextBody ellipse").click()
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("Locked")
            page.get_by_role("button", name="Unlock ContextBody").click()
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("Unlocked")
            expect(page.get_by_test_id("selection-context")).to_contain_text("ContextBody")
            page.get_by_label("Motion prompt").fill("make the selected body fill warmer and slightly bigger")
            page.get_by_role("button", name="Ask AI to edit selection").click()
            expect(page.get_by_test_id("operation-preview")).to_contain_text("ContextBody")
            expect(page.get_by_test_id("operation-preview")).to_contain_text("transform.patch")
            expect(page.get_by_test_id("operation-preview")).to_contain_text("style.fill")
            expect(page.get_by_role("button", name="Apply operation")).to_be_enabled()
            expect(page.get_by_role("button", name="Reject")).to_be_enabled()
            page.get_by_role("button", name="Celebrate", exact=True).click()
            expect(page.get_by_role("button", name="Celebrate", exact=True)).to_have_class(
                re.compile("active")
            )

            page.reload(wait_until="networkidle")
            page.get_by_role("button", name="Editor", exact=True).click()
            expect(page.get_by_test_id("selection-context")).to_contain_text("ContextBody")
            expect(page.get_by_test_id("operation-preview")).to_contain_text("ContextBody")
            expect(page.get_by_test_id("operation-preview")).to_contain_text("Operation history")

            page.evaluate(
                """
                const transform = { translate_x: 0, translate_y: 0, rotate: 0, scale_x: 1, scale_y: 1 };
                const baseStyle = { fill: "#f6f0df", stroke: "#25221d", stroke_width: 5, opacity: 1, linecap: "round", linejoin: "round" };
                const node = (id, name, kind, role, shape, style = {}) => ({
                  id,
                  name,
                  kind,
                  role,
                  transform,
                  style: { ...baseStyle, ...style },
                  shape,
                  children: []
                });
                const documentFor = (id, name, state, target, children) => ({
                  id: `${id}-document`,
                  name,
                  artboards: [{
                    id: `${id}-artboard`,
                    name: `${name} Artboard`,
                    width: 960,
                    height: 540,
                    nodes: [{
                      id: `${id}-rig`,
                      name: `${name} Rig`,
                      kind: "group",
                      role: "scene_rig",
                      transform,
                      style: { fill: null, stroke: null, stroke_width: 0, opacity: 1, linecap: "round", linejoin: "round" },
                      shape: { type: "none" },
                      children
                    }]
                  }],
                  timelines: [{
                    id: `${id}-${state}`,
                    name: state,
                    duration_ms: 1200,
                    tracks: [{
                      target,
                      property: "translation.y",
                      keyframes: [
                        { time_ms: 0, value: { type: "number", value: 0 }, easing: "ease_in_out" },
                        { time_ms: 600, value: { type: "number", value: -8 }, easing: "ease_out" },
                        { time_ms: 1200, value: { type: "number", value: 0 }, easing: "ease_in_out" }
                      ]
                    }]
                  }],
                  state_machines: [{
                    id: `${id}-machine`,
                    name: `${name} Motion`,
                    inputs: [{ name: "state", kind: "enum" }],
                    states: ["idle", state],
                    transitions: [{ from: "idle", to: state, on: state, timeline: state }]
                  }],
                  bindings: [{ name: `${target}_fill`, target, property: "style.fill" }],
                  events: [{ name: "generation_plan_validated", description: "Seeded Phase 3B sprite-python fixture after Rust validation" }]
                });
                const dice = documentFor("dice", "Rolling Dice", "settle", "DieBody", [
                  node("DieBody", "DieBody", "rect", "volume", { type: "rect", x: 378, y: 174, width: 210, height: 210, rx: 24 }, { fill: "#f5f7fb" }),
                  node("FrontFace", "FrontFace", "rect", "front face", { type: "rect", x: 402, y: 214, width: 168, height: 146, rx: 16 }, { fill: "#ffffff" }),
                  node("TopFace", "TopFace", "path", "top face", { type: "path", d: "M402 214 L454 168 L618 184 L570 214 Z" }, { fill: "#e6edf7" }),
                  node("Pips", "Pips", "path", "number marks", { type: "path", d: "M442 252 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0 M530 320 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0" }, { fill: null, stroke: "#111827", stroke_width: 4 }),
                  node("EdgeHighlight", "EdgeHighlight", "path", "edge light", { type: "path", d: "M414 228 L454 188 L604 202" }, { fill: null, stroke: "#b7c7db", stroke_width: 7 }),
                  node("SettleShadow", "SettleShadow", "ellipse", "grounding shadow", { type: "ellipse", cx: 494, cy: 414, rx: 116, ry: 18 }, { fill: "#1f2937", stroke: null, opacity: 0.22 })
                ]);
                const logo = documentFor("logo", "Abstract Logo", "reveal", "PrimaryMark", [
                  node("PrimaryMark", "PrimaryMark", "path", "main vector mark", { type: "path", d: "M382 180 C450 120 540 146 582 222 C520 206 470 234 432 306 C398 266 370 226 382 180 Z" }, { fill: "#6ee7b7" }),
                  node("Wordmark", "Wordmark", "text", "brand text", { type: "text", x: 396, y: 384, value: "STRUT", size: 42 }, { fill: "#172033", stroke: null, stroke_width: 0 }),
                  node("AccentStroke", "AccentStroke", "path", "accent line", { type: "path", d: "M392 326 C452 352 528 348 596 312" }, { fill: null, stroke: "#2563eb", stroke_width: 8 }),
                  node("RevealMask", "RevealMask", "rect", "reveal mask", { type: "rect", x: 360, y: 154, width: 280, height: 250, rx: 20 }, { fill: "transparent", opacity: 0.08 }),
                  node("AnchorGrid", "AnchorGrid", "path", "alignment grid", { type: "path", d: "M360 270 L640 270 M500 150 L500 410" }, { fill: null, stroke: "#94a3b8", stroke_width: 2, opacity: 0.38 }),
                  node("Glow", "Glow", "ellipse", "soft emphasis", { type: "ellipse", cx: 498, cy: 266, rx: 118, ry: 76 }, { fill: "#dbeafe", stroke: null, opacity: 0.28 })
                ]);
                const loader = documentFor("loader", "Progress Loader", "loading", "ActiveSegment", [
                  node("Track", "Track", "ellipse", "background track", { type: "ellipse", cx: 480, cy: 270, rx: 120, ry: 120 }, { fill: "transparent", stroke: "#cbd5e1", stroke_width: 14 }),
                  node("ActiveSegment", "ActiveSegment", "path", "active arc", { type: "path", d: "M480 150 A120 120 0 0 1 600 270" }, { fill: null, stroke: "#14b8a6", stroke_width: 16 }),
                  node("PulseDot", "PulseDot", "ellipse", "pulse marker", { type: "ellipse", cx: 600, cy: 270, rx: 14, ry: 14 }, { fill: "#0f766e" }),
                  node("ProgressSweep", "ProgressSweep", "path", "sweep indicator", { type: "path", d: "M480 270 L600 270" }, { fill: null, stroke: "#99f6e4", stroke_width: 6 }),
                  node("Glow", "Glow", "ellipse", "soft glow", { type: "ellipse", cx: 480, cy: 270, rx: 144, ry: 144 }, { fill: "#ccfbf1", stroke: null, opacity: 0.25 }),
                  node("CenterLabel", "CenterLabel", "text", "progress label", { type: "text", x: 454, y: 282, value: "42%", size: 24 }, { fill: "#134e4a", stroke: null, stroke_width: 0 })
                ]);
                const mascot = documentFor("mascot", "Helpful Mascot", "wave", "Body", [
                  node("Body", "Body", "ellipse", "body", { type: "ellipse", cx: 480, cy: 306, rx: 92, ry: 118 }, { fill: "#a7f3d0" }),
                  node("Head", "Head", "ellipse", "head", { type: "ellipse", cx: 480, cy: 190, rx: 82, ry: 68 }, { fill: "#ecfdf5" }),
                  node("Eyes", "Eyes", "path", "eyes", { type: "path", d: "M446 186 q10 -16 20 0 M494 186 q10 -16 20 0" }, { fill: null, stroke: "#064e3b", stroke_width: 8 }),
                  node("Arms", "Arms", "path", "arms", { type: "path", d: "M394 292 C350 310 344 352 382 364 M566 292 C610 310 616 352 578 364" }, { fill: null, stroke: "#047857", stroke_width: 10 }),
                  node("AccentBadge", "AccentBadge", "ellipse", "accent", { type: "ellipse", cx: 512, cy: 316, rx: 16, ry: 16 }, { fill: "#34d399" }),
                  node("GroundShadow", "GroundShadow", "ellipse", "shadow", { type: "ellipse", cx: 480, cy: 438, rx: 108, ry: 16 }, { fill: "#064e3b", stroke: null, opacity: 0.2 })
                ]);
                const chat = (id, title, document, activeState) => ({
                  id,
                  title,
                  projectId: "project-phase-3",
                  updated: "now",
                    messages: [{ id: Date.now() + Math.random(), role: "assistant", text: `${title} generated through sprite-python and validated Strut operations.` }],
                  references: [],
                  document,
                  activeState,
                  selectedNodeId: null,
                  layerUi: {},
                  pendingOperation: null,
                  operationHistory: []
                });
                window.localStorage.setItem("strut-studio-workspace-v4", JSON.stringify({
                  projects: [{
                    id: "project-phase-3",
                    name: "Phase 3B Sprite-Python Fixtures",
                    path: "D:\\\\StrutPhase3",
                    chats: [
                      chat("chat-dice", "Rolling dice", dice, "settle"),
                      chat("chat-logo", "Abstract logo", logo, "reveal"),
                      chat("chat-loader", "Loader", loader, "loading"),
                      chat("chat-mascot", "Mascot", mascot, "wave")
                    ]
                  }],
                  activeProjectId: "project-phase-3",
                  activeChatId: "chat-dice",
                  themeMode: "system"
                }));
                """
            )
            page.reload(wait_until="networkidle")
            page.get_by_role("button", name="Editor", exact=True).click()

            expect(page.get_by_text("Rolling Dice / Rolling Dice Motion")).to_be_visible()
            expect(page.get_by_role("button", name="DieBody rect")).to_be_visible()
            expect(page.get_by_role("button", name="Pips path")).to_be_visible()
            expect(page.get_by_role("button", name="Head ellipse")).not_to_be_visible()
            page.get_by_role("button", name="DieBody rect").click()
            expect(page.get_by_test_id("selection-context")).to_contain_text("DieBody")
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("volume")

            page.get_by_role("button", name="Abstract logo now").click()
            expect(page.get_by_text("Abstract Logo / Abstract Logo Motion")).to_be_visible()
            expect(page.get_by_role("button", name="PrimaryMark path")).to_be_visible()
            expect(page.get_by_role("button", name="Wordmark text")).to_be_visible()
            expect(page.get_by_role("button", name="Face ellipse")).not_to_be_visible()

            page.get_by_role("button", name="Loader now").click()
            expect(page.get_by_text("Progress Loader / Progress Loader Motion")).to_be_visible()
            expect(page.get_by_role("button", name="ActiveSegment path")).to_be_visible()
            expect(page.get_by_role("button", name="Track ellipse")).to_be_visible()
            expect(page.get_by_role("button", name="Body ellipse")).not_to_be_visible()
            page.get_by_role("button", name="ActiveSegment path").click()
            expect(page.get_by_test_id("selected-part-inspector")).to_contain_text("active arc")

            page.get_by_role("button", name="Mascot now").click()
            expect(page.get_by_text("Helpful Mascot / Helpful Mascot Motion")).to_be_visible()
            expect(page.get_by_role("button", name="Body ellipse")).to_be_visible()
            expect(page.get_by_role("button", name="Head ellipse")).to_be_visible()
            expect(page.get_by_role("button", name="Eyes path")).to_be_visible()

            page.screenshot(path=str(output_dir / "studio-phase-3b-sprite-python-smoke.png"), full_page=True)
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
