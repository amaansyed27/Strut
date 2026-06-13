from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

os.environ.setdefault("STRUT_STUDIO_TEST_PORT", "1424")

from playwright.sync_api import expect, sync_playwright
from studio_bot_smoke import ROOT, URL, npm_command, stop_process_tree, wait_for_server


OUTPUT_DIR = Path(os.environ.get("STRUT_PHASE6_OUTPUT_DIR", ROOT / "test-results" / "phase-6-gallery"))


def run_smoke() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
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
        os.environ["STRUT_STUDIO_TEST_PORT"],
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
            page = browser.new_page(viewport={"width": 1440, "height": 960})
            errors: list[str] = []
            page.on("console", lambda msg: errors.append(msg.text) if msg.type == "error" else None)
            page.on("pageerror", lambda exc: errors.append(str(exc)))
            page.goto(URL, wait_until="networkidle")
            page.evaluate("window.localStorage.clear()")
            seed_phase6_workspace(page)
            page.reload(wait_until="networkidle")
            expect(page.get_by_role("button", name="Editor", exact=True)).not_to_be_visible()
            page.get_by_role("button", name="Chat + preview", exact=True).click()

            gallery = [
                ("Rolling dice", "Rolling Dice Motion / Rolling Dice Motion", "DieBody", "rect", "volume", "dice"),
                ("Abstract logo", "Abstract Logo Motion / Abstract Logo Motion", "PrimaryMark", "path", "main vector mark", "logo"),
                ("Loader", "Progress Loader Motion / Progress Loader Motion", "ActiveSegment", "path", "active arc", "loader"),
                ("Mascot", "Helpful Mascot Motion / Helpful Mascot Motion", "Body", "ellipse", "body", "mascot"),
                ("UI microinteraction", "Button Microinteraction Motion / Button Microinteraction Motion", "ButtonSurface", "rect", "control surface", "ui"),
                ("Icon badge", "Icon Badge Motion / Icon Badge Motion", "BadgePlate", "ellipse", "badge base", "icon-badge"),
            ]

            for title, heading, layer_name, layer_kind, role, slug in gallery:
                page.get_by_role("button", name=f"{title} now").click()
                expect(page.get_by_text(heading)).to_be_visible()
                layer_button = page.get_by_role("button", name=f"Attach layer {layer_name} {layer_kind}")
                expect(layer_button).to_be_visible()
                expect(layer_button).to_contain_text(role)
                layer_button.click()
                expect(page.get_by_text(f"Layer: {layer_name}")).to_be_visible()
                expect(page.get_by_role("button", name="Attach layer Head ellipse")).not_to_be_visible() if slug != "mascot" else expect(page.get_by_role("button", name="Attach layer Head ellipse")).to_be_visible()
                page.screenshot(path=str(OUTPUT_DIR / f"studio-phase6-{slug}.png"), full_page=True)

            expect(page.get_by_text("Layer: BadgePlate")).to_be_visible()
            expect(page.get_by_label("Scene layers rail")).to_be_visible()
            page.screenshot(path=str(OUTPUT_DIR / "studio-phase6-selection-layers-inspector.png"), full_page=True)

            page.get_by_role("button", name="Settings", exact=True).click()
            page.get_by_role("radio", name="Light").click()
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            expect(page.locator("html")).to_have_attribute("data-theme", "light")
            page.screenshot(path=str(OUTPUT_DIR / "studio-phase6-light-theme.png"), full_page=True)

            page.get_by_role("button", name="Settings", exact=True).click()
            page.get_by_role("radio", name="Dark").click()
            page.get_by_role("button", name="Chat + preview", exact=True).click()
            expect(page.locator("html")).to_have_attribute("data-theme", "dark")
            page.screenshot(path=str(OUTPUT_DIR / "studio-phase6-dark-theme.png"), full_page=True)

            assert not errors, f"console errors observed: {errors}"
            browser.close()
    finally:
        stop_process_tree(process)


def seed_phase6_workspace(page) -> None:
    page.evaluate(
        """
        () => {
          const style = (extra = {}) => ({ fill: "#f8fafc", stroke: "#111827", stroke_width: 4, opacity: 1, linecap: "round", linejoin: "round", ...extra });
          const node = (id, name, kind, role, shape, stylePatch = {}) => ({
            id, name, kind, role,
            transform: { translate_x: 0, translate_y: 0, rotate: 0, scale_x: 1, scale_y: 1 },
            style: style(stylePatch),
            shape,
            children: []
          });
          const documentFor = (id, name, state, target, children) => ({
            id: `${id}-document`,
            name: `${name} Motion`,
            artboards: [{
              id: `${id}-artboard`,
              name,
              width: 960,
              height: 540,
              nodes: [{
                id: `${id}-rig`,
                name: `${name} Rig`,
                kind: "group",
                role: "scene_rig",
                transform: { translate_x: 0, translate_y: 0, rotate: 0, scale_x: 1, scale_y: 1 },
                style: style({ fill: null, stroke: null, stroke_width: 0 }),
                shape: { type: "none" },
                children
              }]
            }],
            timelines: [{
              id: `${id}-${state}`,
              name: state,
              duration_ms: 1200,
              tracks: [{ target, property: "translation.y", keyframes: [
                { time_ms: 0, value: { type: "number", value: 0 }, easing: "ease_in_out" },
                { time_ms: 600, value: { type: "number", value: -8 }, easing: "ease_out" },
                { time_ms: 1200, value: { type: "number", value: 0 }, easing: "ease_in_out" }
              ] }]
            }],
            state_machines: [{
              id: `${id}-machine`,
              name: `${name} Motion`,
              inputs: [{ name: "state", kind: "enum" }],
              states: ["idle", state],
              transitions: [{ from: "idle", to: state, on: state, timeline: state }]
            }],
            bindings: [{ name: `${target}_fill`, target, property: "style.fill" }],
            events: [{ name: "generation_plan_validated", description: "Seeded Phase 6 gallery fixture after Rust validation" }]
          });
          const docs = {
            dice: documentFor("dice", "Rolling Dice", "settle", "DieBody", [
              node("DieBody", "DieBody", "rect", "volume", { type: "rect", x: 378, y: 174, width: 210, height: 210, rx: 24 }, { fill: "#f5f7fb" }),
              node("FrontFace", "FrontFace", "rect", "front face", { type: "rect", x: 402, y: 214, width: 168, height: 146, rx: 16 }, { fill: "#ffffff" }),
              node("TopFace", "TopFace", "path", "top face", { type: "path", d: "M402 214 L454 168 L618 184 L570 214 Z" }, { fill: "#e6edf7" }),
              node("Pips", "Pips", "path", "number marks", { type: "path", d: "M442 252 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0 M530 320 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0" }, { fill: null, stroke: "#111827" }),
              node("EdgeHighlight", "EdgeHighlight", "path", "edge light", { type: "path", d: "M414 228 L454 188 L604 202" }, { fill: null, stroke: "#b7c7db" }),
              node("SettleShadow", "SettleShadow", "ellipse", "grounding shadow", { type: "ellipse", cx: 494, cy: 414, rx: 116, ry: 18 }, { fill: "#1f2937", stroke: null, opacity: 0.22 })
            ]),
            logo: documentFor("logo", "Abstract Logo", "reveal", "PrimaryMark", [
              node("PrimaryMark", "PrimaryMark", "path", "main vector mark", { type: "path", d: "M382 180 C450 120 540 146 582 222 C520 206 470 234 432 306 C398 266 370 226 382 180 Z" }, { fill: "#6ee7b7" }),
              node("Wordmark", "Wordmark", "text", "brand text", { type: "text", x: 396, y: 384, value: "STRUT", size: 42 }, { fill: "#172033", stroke: null }),
              node("AccentStroke", "AccentStroke", "path", "accent line", { type: "path", d: "M392 326 C452 352 528 348 596 312" }, { fill: null, stroke: "#2563eb" }),
              node("RevealMask", "RevealMask", "rect", "reveal mask", { type: "rect", x: 360, y: 154, width: 280, height: 250, rx: 20 }, { fill: "transparent", stroke: null, opacity: 0.08 }),
              node("AnchorGrid", "AnchorGrid", "path", "alignment grid", { type: "path", d: "M360 270 L640 270 M500 150 L500 410" }, { fill: null, stroke: "#94a3b8", opacity: 0.38 }),
              node("Glow", "Glow", "ellipse", "soft emphasis", { type: "ellipse", cx: 498, cy: 266, rx: 118, ry: 76 }, { fill: "#dbeafe", stroke: null, opacity: 0.28 })
            ]),
            loader: documentFor("loader", "Progress Loader", "loading", "ActiveSegment", [
              node("Track", "Track", "ellipse", "background track", { type: "ellipse", cx: 480, cy: 270, rx: 120, ry: 120 }, { fill: "transparent", stroke: "#cbd5e1" }),
              node("ActiveSegment", "ActiveSegment", "path", "active arc", { type: "path", d: "M480 150 A120 120 0 0 1 600 270" }, { fill: null, stroke: "#14b8a6" }),
              node("PulseDot", "PulseDot", "ellipse", "pulse marker", { type: "ellipse", cx: 600, cy: 270, rx: 14, ry: 14 }, { fill: "#0f766e" }),
              node("ProgressSweep", "ProgressSweep", "path", "sweep indicator", { type: "path", d: "M480 270 L600 270" }, { fill: null, stroke: "#99f6e4" }),
              node("Glow", "Glow", "ellipse", "soft glow", { type: "ellipse", cx: 480, cy: 270, rx: 144, ry: 144 }, { fill: "#ccfbf1", stroke: null, opacity: 0.25 }),
              node("CenterLabel", "CenterLabel", "text", "progress label", { type: "text", x: 454, y: 282, value: "42%", size: 24 }, { fill: "#134e4a", stroke: null })
            ]),
            mascot: documentFor("mascot", "Helpful Mascot", "hover", "Body", [
              node("Body", "Body", "ellipse", "body", { type: "ellipse", cx: 480, cy: 306, rx: 92, ry: 118 }, { fill: "#a7f3d0" }),
              node("Head", "Head", "ellipse", "head", { type: "ellipse", cx: 480, cy: 190, rx: 82, ry: 68 }, { fill: "#ecfdf5" }),
              node("Eyes", "Eyes", "path", "eyes", { type: "path", d: "M446 186 q10 -16 20 0 M494 186 q10 -16 20 0" }, { fill: null, stroke: "#064e3b" }),
              node("Arms", "Arms", "path", "arms", { type: "path", d: "M394 292 C350 310 344 352 382 364 M566 292 C610 310 616 352 578 364" }, { fill: null, stroke: "#047857" }),
              node("AccentBadge", "AccentBadge", "ellipse", "accent", { type: "ellipse", cx: 512, cy: 316, rx: 16, ry: 16 }, { fill: "#34d399" }),
              node("GroundShadow", "GroundShadow", "ellipse", "shadow", { type: "ellipse", cx: 480, cy: 438, rx: 108, ry: 16 }, { fill: "#064e3b", stroke: null, opacity: 0.2 })
            ]),
            ui: documentFor("ui", "Button Microinteraction", "hover", "ButtonSurface", [
              node("ButtonSurface", "ButtonSurface", "rect", "control surface", { type: "rect", x: 370, y: 254, width: 220, height: 72, rx: 12 }, { fill: "#e0f2fe" }),
              node("ButtonLabel", "ButtonLabel", "text", "label", { type: "text", x: 426, y: 297, value: "Continue", size: 22 }, { fill: "#0f172a", stroke: null }),
              node("FocusRing", "FocusRing", "path", "focus ring", { type: "path", d: "M364 248 L596 248 L596 332 L364 332 Z" }, { fill: null, stroke: "#38bdf8", opacity: 0.65 }),
              node("CheckMark", "CheckMark", "path", "success mark", { type: "path", d: "M444 294 L470 316 L520 264" }, { fill: null, stroke: "#16a34a" }),
              node("HoverGlow", "HoverGlow", "ellipse", "hover glow", { type: "ellipse", cx: 480, cy: 290, rx: 138, ry: 52 }, { fill: "#bae6fd", stroke: null, opacity: 0.22 })
            ]),
            icon: documentFor("icon", "Icon Badge", "success", "BadgePlate", [
              node("BadgePlate", "BadgePlate", "ellipse", "badge base", { type: "ellipse", cx: 480, cy: 270, rx: 106, ry: 106 }, { fill: "#eef2ff" }),
              node("InnerShield", "InnerShield", "path", "inner shield", { type: "path", d: "M480 188 L552 220 L540 308 C526 350 504 374 480 386 C456 374 434 350 420 308 L408 220 Z" }, { fill: "#c7d2fe" }),
              node("SparkGlyph", "SparkGlyph", "path", "spark glyph", { type: "path", d: "M480 222 L494 260 L534 270 L494 280 L480 318 L466 280 L426 270 L466 260 Z" }, { fill: "#fef3c7", stroke: "#92400e" }),
              node("OrbitStroke", "OrbitStroke", "path", "orbit stroke", { type: "path", d: "M386 286 C430 224 534 214 584 260" }, { fill: null, stroke: "#38bdf8", opacity: 0.82 }),
              node("StatusDot", "StatusDot", "ellipse", "status dot", { type: "ellipse", cx: 566, cy: 206, rx: 18, ry: 18 }, { fill: "#22c55e", stroke: "#14532d" }),
              node("BadgeLabel", "BadgeLabel", "text", "short label", { type: "text", x: 434, y: 410, value: "VERIFIED", size: 24 }, { fill: "#1e1b4b", stroke: null })
            ])
          };
          const chat = (id, title, document, activeState) => ({
            id, title, projectId: "project-phase-6", updated: "now",
            messages: [{ id: Math.floor(Math.random() * 100000), role: "assistant", text: `${title} generated through sprite-python plans and validated Strut operations.` }],
            references: [], document, activeState, selectedNodeId: null, layerUi: {}, pendingOperation: null, operationHistory: []
          });
          window.localStorage.setItem("strut-studio-workspace-v4", JSON.stringify({
            projects: [{
              id: "project-phase-6",
              name: "Phase 6 Release Gallery",
              path: "D:\\\\StrutPhase6",
              chats: [
                chat("chat-dice", "Rolling dice", docs.dice, "settle"),
                chat("chat-logo", "Abstract logo", docs.logo, "reveal"),
                chat("chat-loader", "Loader", docs.loader, "loading"),
                chat("chat-mascot", "Mascot", docs.mascot, "hover"),
                chat("chat-ui", "UI microinteraction", docs.ui, "hover"),
                chat("chat-icon", "Icon badge", docs.icon, "success")
              ]
            }],
            activeProjectId: "project-phase-6",
            activeChatId: "chat-dice",
            themeMode: "light"
          }));
        }
        """
    )


if __name__ == "__main__":
    try:
        run_smoke()
    except Exception as exc:  # noqa: BLE001 - command line test should be explicit.
        print(f"studio phase 6 gallery smoke failed: {exc}", file=sys.stderr)
        raise
