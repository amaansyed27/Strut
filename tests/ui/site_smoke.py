from __future__ import annotations

import subprocess
import sys
import time
from pathlib import Path

from playwright.sync_api import expect, sync_playwright


ROOT = Path(__file__).resolve().parents[2]
PORT = 4187


def npm_command() -> str:
    return "npm.cmd" if sys.platform.startswith("win") else "npm"


def wait_for_server(process: subprocess.Popen[bytes]) -> None:
    for _ in range(80):
        if process.poll() is not None:
            raise RuntimeError("site dev server exited early")
        try:
            import urllib.request

            with urllib.request.urlopen(f"http://127.0.0.1:{PORT}", timeout=0.25) as response:
                if response.status < 500:
                    return
        except Exception:
            time.sleep(0.25)
    raise TimeoutError("site dev server did not become ready")


def run_smoke() -> None:
    output_dir = ROOT / "test-results" / "site"
    output_dir.mkdir(parents=True, exist_ok=True)
    process = subprocess.Popen(
        [
            npm_command(),
            "--workspace",
            "@strut/site",
            "run",
            "dev",
            "--",
            "--host",
            "127.0.0.1",
            "--port",
            str(PORT),
            "--strictPort",
        ],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        wait_for_server(process)
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(headless=True)
            page = browser.new_page(viewport={"width": 1440, "height": 1000})
            console_errors: list[str] = []
            page.on("console", lambda message: console_errors.append(message.text) if message.type == "error" else None)
            page.goto(f"http://127.0.0.1:{PORT}", wait_until="networkidle")

            expect(page.get_by_role("heading", name="Strut", exact=True)).to_be_visible()
            expect(page.get_by_text("AI-native asset studio for coding agents")).to_be_visible()
            expect(page.get_by_role("link", name="Download for Windows")).to_be_visible()
            expect(page.get_by_role("link", name="Download for macOS")).to_be_visible()
            expect(page.get_by_role("link", name="Download for Linux")).to_be_visible()
            expect(page.get_by_text("Strut Sprite", exact=True).first).to_be_visible()
            expect(page.get_by_text("Agentic CLI", exact=True)).to_be_visible()
            expect(page.get_by_text("Open-source runtime", exact=True)).to_be_visible()
            expect(page.locator("#examples [data-demo-card]")).to_have_count(6)
            expect(page.locator("[data-strut='runtime']")).to_have_count(9)
            expect(page.locator("[data-proof='release-checklist']")).to_contain_text("v1.0 release gate")

            page.screenshot(path=str(output_dir / "site-home.png"), full_page=True)
            browser.close()
            assert not console_errors, "\n".join(console_errors)
    finally:
        process.terminate()
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()


if __name__ == "__main__":
    try:
        run_smoke()
    except Exception as exc:
        print(f"site smoke failed: {exc}", file=sys.stderr)
        raise
