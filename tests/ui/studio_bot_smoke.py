from __future__ import annotations

import os
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


def wait_for_server(timeout_seconds: int = 30) -> None:
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None

    while time.time() < deadline:
        try:
            with urllib.request.urlopen(URL, timeout=1) as response:
                if response.status == 200:
                    return
        except Exception as exc:  # noqa: BLE001 - surfaced after timeout.
            last_error = exc
        time.sleep(0.25)

    raise RuntimeError(f"Studio test server did not start: {last_error}")


def run_smoke() -> None:
    output_dir = ROOT / "test-results"
    output_dir.mkdir(exist_ok=True)

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
        wait_for_server()
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(headless=True)
            page = browser.new_page(viewport={"width": 1440, "height": 920})
            page.goto(URL, wait_until="networkidle")

            expect(page.get_by_text("Strut Studio")).to_be_visible()
            expect(page.get_by_text("Minimal Bot.strut")).to_be_visible()
            expect(page.get_by_text("HelmetShell")).to_be_visible()
            expect(page.get_by_text("BotMoods")).to_be_visible()

            preview = page.locator('[data-testid="character-preview"]')
            expect(preview).to_be_visible()
            expect(preview).to_have_attribute("data-character", "bot")
            expect(preview).to_have_attribute("data-state", "wave")

            page.get_by_label("Character prompt").fill("make an owl like Duo from Duolingo")
            page.get_by_role("button", name="Generate Character").click()
            expect(page.get_by_text("Owl Mascot.strut")).to_be_visible()
            expect(page.get_by_text("Beak")).to_be_visible()
            expect(page.get_by_text("OwlMoods")).to_be_visible()
            expect(preview).to_have_attribute("data-character", "owl")
            expect(preview).to_have_attribute("data-state", "wave")
            page.screenshot(path=str(output_dir / "studio-owl-smoke.png"), full_page=True)

            page.get_by_label("Character prompt").fill("make a small waving robot like the reference image")
            page.get_by_role("button", name="Generate Character").click()
            expect(page.get_by_text("Minimal Bot.strut")).to_be_visible()
            expect(preview).to_have_attribute("data-character", "bot")
            expect(preview).to_have_attribute("data-state", "wave")

            page.get_by_role("button", name="Generate Character").click()
            expect(page.locator('[data-testid="plan-sketches"]')).to_be_visible()
            expect(page.get_by_text("Floating Helper")).to_be_visible()
            page.get_by_text("Scanner Bot").click()
            page.get_by_role("button", name="Build Character").click()
            expect(preview).to_have_attribute("data-state", "scan")

            for state in ["Idle", "Float", "Wave", "Blink", "Scan", "Celebrate", "Sleep"]:
                page.locator(f'[data-state-button="{state.lower()}"]').click()
                expect(preview).to_have_attribute("data-state", state.lower())

            page.screenshot(path=str(output_dir / "studio-bot-smoke.png"), full_page=True)
            browser.close()
    finally:
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
