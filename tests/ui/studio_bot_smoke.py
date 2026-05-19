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

            expect(page.get_by_role("button", name="New chat", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="New project", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Search", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Providers", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Strut", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Strut Plan now", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Chat only", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Chat + preview", exact=True)).to_be_visible()
            expect(page.get_by_role("button", name="Editor", exact=True)).to_be_visible()
            expect(page.get_by_role("heading", name="What should we build in Strut?")).to_be_visible()
            page.screenshot(path=str(output_dir / "studio-home-redesign.png"), full_page=True)

            page.get_by_role("button", name="Search", exact=True).click()
            page.get_by_label("Search projects and chats").fill("owl")
            expect(page.get_by_role("button", name="Owl guide animation 2d", exact=True)).to_be_visible()
            page.get_by_label("Search projects and chats").fill("")

            page.get_by_role("button", name="New chat in Strut").click()
            expect(page.get_by_text("New chat ready. Tell Strut what to design or ask for a plan.")).to_be_visible()

            page.get_by_role("button", name="New project", exact=True).click()
            page.get_by_label("Project name").fill("Smoke Mascot")
            page.get_by_label("Project location").fill("D:\\StrutSmoke")
            page.get_by_role("button", name="Create project").click()

            activity = page.locator('[data-testid="activity-pill"]')
            expect(activity).to_contain_text("Browser preview project")
            expect(page.get_by_role("button", name="Smoke Mascot", exact=True)).to_be_visible()

            page.get_by_role("button", name="Providers", exact=True).click()
            expect(page.get_by_role("heading", name="Providers")).to_be_visible()
            page.get_by_role("button", name="BYOK").click()
            page.get_by_label("BYOK provider").select_option("openai")
            page.get_by_label("OpenAI API key").fill("sk-test-strut")
            page.get_by_label("OpenAI model").fill("gpt-5.2")
            page.get_by_role("button", name="Save provider").click()
            expect(activity).to_contain_text("Desktop app required for provider config")
            page.get_by_role("button", name="Test selected provider").click()
            expect(activity).to_contain_text("Desktop app required for real provider checks")

            page.get_by_role("button", name="Chat + preview", exact=True).click()
            preview = page.locator('[data-testid="character-preview"]')
            expect(preview).to_be_visible()
            expect(preview).to_have_attribute("data-character", "bot")
            expect(preview).to_have_attribute("data-state", "wave")
            page.get_by_label("Character prompt").fill("make an owl like Duo from Duolingo")
            page.get_by_role("button", name="Generate").click()
            expect(page.get_by_text("Owl Mascot preview is ready")).to_be_visible()
            expect(preview).to_have_attribute("data-character", "owl")
            expect(preview).to_have_attribute("data-state", "wave")
            page.screenshot(path=str(output_dir / "studio-owl-smoke.png"), full_page=True)

            page.get_by_role("button", name="Editor", exact=True).click()
            expect(page.get_by_text("Project files")).to_be_visible()
            expect(page.get_by_text("starter.strut.json")).to_be_visible()
            expect(page.get_by_text("OwlBody")).to_be_visible()
            expect(page.get_by_text("OwlMoods")).to_be_visible()

            page.get_by_label("Parts").uncheck()
            expect(page.get_by_text("Parts hidden")).to_be_visible()
            page.get_by_label("Parts").check()
            expect(page.get_by_text("OwlBody")).to_be_visible()

            for state in ["Idle", "Float", "Wave", "Blink", "Scan", "Celebrate", "Sleep"]:
                page.get_by_role("button", name=state).click()
                expect(preview).to_have_attribute("data-state", state.lower())

            page.get_by_role("button", name="Settings", exact=True).click()
            expect(page.get_by_role("heading", name="Settings")).to_be_visible()
            expect(page.get_by_text("Current provider")).to_be_visible()
            expect(page.get_by_label("Generation mode")).to_be_visible()

            page.get_by_role("button", name="Chat + preview", exact=True).click()
            page.get_by_label("Character prompt").fill("make a small waving robot like the reference image")
            page.get_by_role("button", name="Generate").click()
            expect(page.get_by_text("Minimal Bot preview is ready")).to_be_visible()
            expect(preview).to_have_attribute("data-character", "bot")
            expect(preview).to_have_attribute("data-state", "wave")

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
