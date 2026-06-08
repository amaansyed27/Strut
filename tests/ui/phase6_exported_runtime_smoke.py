from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

from playwright.sync_api import sync_playwright


ROOT = Path(__file__).resolve().parents[2]
OUTPUT_DIR = Path(os.environ.get("STRUT_PHASE6_OUTPUT_DIR", ROOT / "test-results" / "phase-6-exported-runtime"))
STRUT_EXE = ROOT / "target" / "debug" / ("strut.exe" if os.name == "nt" else "strut")
CASES = [
    ("dice", "make rolling dice settle softly", "dice", "Rolling Dice Motion", "settle"),
    ("logo", "make an abstract logo reveal", "logo", "Abstract Logo Motion", "reveal"),
    ("loader", "make a calm progress loader animation", "loader", "Progress Loader Motion", "loading"),
    ("mascot", "make a low energy companion mascot idle animation", "mascot", "Helpful Mascot Motion", "idle"),
    ("ui", "make a button UI microinteraction", "ui", "Button Microinteraction Motion", "hover"),
    ("icon-badge", "make a success icon badge animation", "badge", "Icon Badge Motion", "success"),
]


def run_command(args: list[str], cwd: Path = ROOT) -> dict:
    completed = subprocess.run(args, cwd=cwd, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed: {' '.join(args)}\nstdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    return json.loads(completed.stdout)


def ensure_cli() -> None:
    if STRUT_EXE.exists():
        return
    subprocess.run(["cargo", "build", "-p", "strut-cli"], cwd=ROOT, check=True)


def render_harness(scene: dict) -> str:
    return f"""
<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <title>Phase 6 Exported Runtime Smoke</title>
  <style>
    body {{ margin: 0; min-height: 100vh; display: grid; place-items: center; background: #f8fafc; font-family: Arial, sans-serif; }}
    main {{ width: min(920px, 92vw); border: 1px solid #cbd5e1; background: white; padding: 18px; }}
    svg {{ width: 100%; height: auto; display: block; }}
  </style>
</head>
<body>
  <main id="app"></main>
  <script>
    const scene = {json.dumps(scene)};
    const ns = "http://www.w3.org/2000/svg";
    const app = document.getElementById("app");
    const artboard = scene.artboards[0];
    const svg = document.createElementNS(ns, "svg");
    svg.setAttribute("viewBox", `0 0 ${{artboard.width}} ${{artboard.height}}`);
    svg.setAttribute("role", "img");
    svg.setAttribute("aria-label", scene.name);
    svg.dataset.strutExportSmoke = "true";
    function paint(value) {{ return value ?? "none"; }}
    function appendNode(parent, node) {{
      const shape = node.shape || {{ type: "none" }};
      const tag = shape.type === "rect" ? "rect" : shape.type === "ellipse" ? "ellipse" : shape.type === "path" ? "path" : shape.type === "text" ? "text" : "g";
      const el = document.createElementNS(ns, tag);
      el.dataset.nodeName = node.name;
      el.dataset.nodeRole = node.role || "";
      const style = node.style || {{}};
      el.setAttribute("fill", paint(style.fill));
      el.setAttribute("stroke", paint(style.stroke));
      el.setAttribute("stroke-width", String(style.stroke_width ?? 0));
      el.setAttribute("opacity", String(style.opacity ?? 1));
      if (shape.type === "rect") {{
        for (const key of ["x", "y", "width", "height", "rx"]) el.setAttribute(key, String(shape[key]));
      }} else if (shape.type === "ellipse") {{
        for (const key of ["cx", "cy", "rx", "ry"]) el.setAttribute(key, String(shape[key]));
      }} else if (shape.type === "path") {{
        el.setAttribute("d", shape.d);
      }} else if (shape.type === "text") {{
        el.setAttribute("x", String(shape.x));
        el.setAttribute("y", String(shape.y));
        el.setAttribute("font-size", String(shape.size));
        el.textContent = shape.value;
      }}
      parent.append(el);
      for (const child of node.children || []) appendNode(el, child);
    }}
    for (const node of artboard.nodes) appendNode(svg, node);
    app.append(svg);
  </script>
</body>
</html>
"""


def run_smoke() -> None:
    ensure_cli()
    if OUTPUT_DIR.exists():
        shutil.rmtree(OUTPUT_DIR)
    (OUTPUT_DIR / "plans").mkdir(parents=True)
    (OUTPUT_DIR / "scenes").mkdir()
    (OUTPUT_DIR / "renders").mkdir()
    (OUTPUT_DIR / "exports").mkdir()
    (OUTPUT_DIR / "screenshots").mkdir()

    transcript: list[dict[str, object]] = []
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True)
        for slug, instruction, classification, document_name, state in CASES:
            case_dir = OUTPUT_DIR / slug
            case_dir.mkdir()
            scene = case_dir / "scene.strut"
            shutil.copy(ROOT / "samples" / "login-button.strut", scene)

            plan = run_command([str(STRUT_EXE), "sprite", "plan", instruction, "--json", "--dry-run", "--explain"])
            assert plan["planSummary"]["subjectClassification"] == classification
            assert plan["document"]["name"] == document_name
            assert "document" not in plan["envelope"]
            plan_path = OUTPUT_DIR / "plans" / f"{slug}.plan.json"
            plan_path.write_text(json.dumps(plan, indent=2), encoding="utf-8")

            before = scene.read_bytes()
            dry_patch = run_command([str(STRUT_EXE), "patch", "--scene", str(scene), "--from", str(plan_path), "--dry-run", "--json"])
            assert dry_patch["dryRun"] is True
            assert scene.read_bytes() == before

            patch = run_command([str(STRUT_EXE), "patch", "--scene", str(scene), "--from", str(plan_path), "--json"])
            verify = run_command([str(STRUT_EXE), "verify", str(scene), "--json"])
            render = run_command([
                str(STRUT_EXE),
                "render",
                "--scene",
                str(scene),
                "--state",
                state,
                "--out",
                str(OUTPUT_DIR / "renders" / f"{slug}.svg"),
                "--json",
                "--no-open",
            ])
            export_dir = OUTPUT_DIR / "exports" / slug
            export = run_command([str(STRUT_EXE), "export", "react", "--scene", str(scene), "--out", str(export_dir), "--json"])
            exported_scene = json.loads((export_dir / "scene.json").read_text(encoding="utf-8"))
            assert exported_scene["name"] == document_name
            assert "export function StrutAnimation" in (export_dir / "StrutAnimation.tsx").read_text(encoding="utf-8")

            harness = case_dir / "runtime-harness.html"
            harness.write_text(render_harness(exported_scene), encoding="utf-8")
            page = browser.new_page(viewport={"width": 1100, "height": 760})
            errors: list[str] = []
            page.on("console", lambda msg: errors.append(msg.text) if msg.type == "error" else None)
            page.on("pageerror", lambda exc: errors.append(str(exc)))
            page.goto(harness.resolve().as_uri(), wait_until="networkidle")
            page.locator("[data-strut-export-smoke='true']").wait_for()
            page.screenshot(path=str(OUTPUT_DIR / "screenshots" / f"runtime-{slug}.png"), full_page=True)
            page.close()
            assert not errors, f"{slug} console errors: {errors}"
            transcript.append(
                {
                    "slug": slug,
                    "planSubject": plan["planSummary"]["subjectClassification"],
                    "patch": patch["message"],
                    "verify": verify["validation"]["message"],
                    "render": render["out"],
                    "exportFiles": export["files"],
                }
            )
        browser.close()

    (OUTPUT_DIR / "command-transcript.json").write_text(json.dumps(transcript, indent=2), encoding="utf-8")


if __name__ == "__main__":
    try:
        run_smoke()
    except Exception as exc:  # noqa: BLE001 - command line test should be explicit.
        print(f"phase 6 exported runtime smoke failed: {exc}", file=sys.stderr)
        raise
