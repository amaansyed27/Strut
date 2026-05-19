from __future__ import annotations

import os
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

from playwright.sync_api import expect, sync_playwright


ROOT = Path(__file__).resolve().parents[2]
PORT = int(os.environ.get("STRUT_RUNTIME_TEST_PORT", "1423"))
URL = f"http://127.0.0.1:{PORT}/examples/runtime-bot/"


def npm_command() -> str:
    return "npm.cmd" if os.name == "nt" else "npm"


def wait_for_server(process: subprocess.Popen[str], timeout_seconds: int = 30) -> None:
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None

    while time.time() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"Runtime example server exited before startup with code {process.returncode}")
        try:
            with urllib.request.urlopen(URL, timeout=1) as response:
                if response.status == 200 and process.poll() is None:
                    return
        except Exception as exc:  # noqa: BLE001 - surfaced after timeout.
            last_error = exc
        time.sleep(0.25)

    raise RuntimeError(f"Runtime example server did not start: {last_error}")


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
            page = browser.new_page(viewport={"width": 1280, "height": 820})
            page.goto(URL, wait_until="networkidle")

            expect(page.get_by_text("Runtime Bot")).to_be_visible()
            expect(page.get_by_text("Minimal Bot loaded")).to_be_visible()

            preview = page.locator("[data-strut-bot]")
            expect(preview).to_be_visible()
            expect(preview).to_have_attribute("data-state", "idle")
            expect(page.locator('[data-node-name="FacePanel"]')).to_be_visible()
            expect(page.locator('[data-node-name="GroundShadow"]')).to_be_visible()

            for state in ["Float", "Wave", "Blink", "Scan", "Celebrate", "Sleep"]:
                page.get_by_role("button", name=state).click()
                expect(preview).to_have_attribute("data-state", state.lower())

            page.screenshot(path=str(output_dir / "runtime-bot-smoke.png"), full_page=True)
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
        print(f"runtime bot smoke failed: {exc}", file=sys.stderr)
        raise
