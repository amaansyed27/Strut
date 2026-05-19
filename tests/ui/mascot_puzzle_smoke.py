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
PORT = int(os.environ.get("STRUT_MASCOT_PUZZLE_TEST_PORT", "1426"))
URL = f"http://127.0.0.1:{PORT}/examples/mascot-puzzle/"


def npm_command() -> str:
    return "npm.cmd" if os.name == "nt" else "npm"


def wait_for_server(process: subprocess.Popen[str], timeout_seconds: int = 30) -> None:
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None

    while time.time() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"Mascot puzzle server exited before startup with code {process.returncode}")
        try:
            with urllib.request.urlopen(URL, timeout=1) as response:
                if response.status == 200 and process.poll() is None:
                    return
        except Exception as exc:  # noqa: BLE001 - surfaced after timeout.
            last_error = exc
        time.sleep(0.25)

    raise RuntimeError(f"Mascot puzzle server did not start: {last_error}")


def run_smoke() -> None:
    output_dir = ROOT / "test-results"
    output_dir.mkdir(exist_ok=True)

    process = subprocess.Popen(
        [npm_command(), "exec", "vite", "--", "--host", "127.0.0.1", "--port", str(PORT), "--strictPort"],
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

            expect(page).to_have_title("Strut Mascot Puzzle")
            expect(page.get_by_role("heading", name="Glyph Trail")).to_be_visible()
            expect(page.locator("[data-mascot-stage] svg")).to_be_visible()
            expect(page.locator("[data-mini-mascot] svg")).to_be_visible()
            expect(page.locator("[data-board] .tile")).to_have_count(9)
            expect(page.locator("[data-level-count]")).to_have_text("Level 1 / 5")

            page.screenshot(path=str(output_dir / "mascot-puzzle-start.png"), full_page=True)
            page.set_viewport_size({"width": 390, "height": 840})
            page.reload(wait_until="networkidle")
            expect(page.get_by_role("heading", name="Glyph Trail")).to_be_visible()
            expect(page.locator("[data-mascot-stage] svg")).to_be_visible()
            expect(page.locator("[data-board] .tile")).to_have_count(9)
            page.screenshot(path=str(output_dir / "mascot-puzzle-mobile.png"), full_page=True)
            page.set_viewport_size({"width": 1440, "height": 920})
            page.reload(wait_until="networkidle")

            sequences = [
                ["A", "B", "C"],
                ["O", "R", "B", "I", "T"],
                ["S", "T", "R", "U", "T"],
                ["C", "O", "D", "E", "X"],
                ["M", "A", "S", "C", "O", "T"],
            ]

            for index, sequence in enumerate(sequences):
                expect(page.locator("[data-level-count]")).to_have_text(f"Level {index + 1} / 5")
                for glyph in sequence:
                    page.locator(f'.tile[data-glyph="{glyph}"]:not(.picked)').first.click()

                expect(page.get_by_role("button", name=re.compile("Next level|Finish run"))).to_be_enabled()
                expect(page.locator("[data-screen-mascot]")).to_have_class(re.compile(r".*\brun\b.*"))
                expect(page.locator("[data-mascot-status]")).to_contain_text("Cross-screen celebration")

                if index < len(sequences) - 1:
                    page.get_by_role("button", name="Next level").click()
                else:
                    page.wait_for_timeout(260)
                    early_transform = page.locator("[data-screen-mascot]").evaluate("node => getComputedStyle(node).transform")
                    page.screenshot(path=str(output_dir / "mascot-puzzle-motion-early.png"), full_page=True)
                    page.wait_for_timeout(390)
                    mid_transform = page.locator("[data-screen-mascot]").evaluate("node => getComputedStyle(node).transform")
                    page.screenshot(path=str(output_dir / "mascot-puzzle-complete.png"), full_page=True)
                    page.wait_for_timeout(540)
                    late_transform = page.locator("[data-screen-mascot]").evaluate("node => getComputedStyle(node).transform")
                    page.screenshot(path=str(output_dir / "mascot-puzzle-motion-late.png"), full_page=True)
                    assert len({early_transform, mid_transform, late_transform}) == 3
                    expect(page.locator("[data-cleared]")).to_have_text("5")
                    expect(page.get_by_role("button", name="Finish run")).to_be_enabled()

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
        print(f"mascot puzzle smoke failed: {exc}", file=sys.stderr)
        raise
