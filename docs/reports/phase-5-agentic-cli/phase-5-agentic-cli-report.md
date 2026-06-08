# Phase 5 Agentic CLI Report

Date: 2026-06-09

Branch: `codex/phase-1-ai-editor-shell`

## Summary

Phase 5 is implemented for the core agentic workflow. Strut now has a Rust-backed `strut` CLI binary in `crates/strut-cli` that can inspect projects and scenes, generate validated operation plans, use sprite-python as a planning backend, patch `.strut` scene packages through Rust validation, verify scenes and operation batches, write deterministic SVG render proofs, and export React integration files.

Phase 6 end-to-end gallery hardening was not started.

## Initial Dirty Files

Captured before edits:

```text
## codex/phase-1-ai-editor-shell
 M README.md
 M docs/README.md
 M docs/guides/generate-a-character.md
 M docs/learn/first-animation.md
 M docs/learn/quick-start.md
 M docs/learn/what-is-strut.md
?? docs/learn/motion-language.md
```

These files were pre-existing dirty work and were not reverted or folded into Phase 5 commits.

## Scope Completed

- Added `strut inspect project` with canonical project file checks, main scene detection, operation batch summaries, document summaries, timelines, states, and warnings.
- Added `strut inspect scene <scene-file>` with artboards, nodes, timelines, states, events, semantic roles, and validation status.
- Added `strut plan "<instruction>"` using deterministic local fixtures and Rust validation.
- Added `strut sprite plan "<instruction>"` using sprite-python when available, with checked fixture fallback.
- Added deterministic sprite-python UI microinteraction fixture coverage alongside dice, logo, loader, and mascot.
- Added `strut patch --scene <scene-file> --from <plan-file>` with validation before writing and `--dry-run` immutability.
- Added `strut verify <scene-file>` with optional `--batch` validation.
- Added `strut render --scene <scene-file> --state <state> --out <image-file>` as deterministic SVG proof output.
- Added `strut export react --scene <scene-file> --out <target-dir>` with safe overwrite behavior, `--dry-run`, and generated integration files.
- Added binary-level CLI integration test coverage.
- Added CLI reference and integration guide docs.

## Validation Boundary

The CLI writes only after Rust reads and validates the current `.strut` package, validates the planned replacement `Document` through `strut-format`, validates the pending operation batch shape, and writes a new `.strut` package through `strut_format::write_strut_file`.

Sprite-python output remains a generation-plan envelope plus operation list. It does not bypass Rust validation and does not write scene files directly.

## Render Limitation

`crates/strut-renderer` currently exposes render planning only. Phase 5 therefore implements the strongest useful deterministic render proof available in this repo: an SVG structural proof generated from the validated Strut document. The JSON output explicitly reports `cpu-fallback-svg-proof` and lists this limitation.

## App Handoff Limitation

The core CLI workflow does not require the Studio UI. The app handoff protocol is documented, but a command to open Studio, apply a pending patch, preview render, and return verification status remains future work.

## Changed Files

- `Cargo.toml`
- `Cargo.lock`
- `crates/strut-cli/Cargo.toml`
- `crates/strut-cli/src/main.rs`
- `crates/strut-cli/tests/agentic_cli.rs`
- `packages/strut-python/examples/ui.py`
- `packages/strut-python/fixtures/ui.plan.json`
- `packages/strut-python/tests/test_examples.py`
- `docs/reference/agentic-cli.md`
- `docs/guides/agentic-cli-integration.md`
- `docs/reports/phase-5-agentic-cli/phase-5-agentic-cli-report.md`
- `docs/reports/phase-5-agentic-cli/phase-5-agentic-cli-report.docx`
- `docs/reports/phase-5-agentic-cli/screenshots/exported-runtime-demo.png`

## Commits

- `ea8ed8e feat(cli): add agentic strut commands`
  - Added the `strut-cli` crate and binary.
  - Added inspect, plan, sprite plan, patch, verify, render, and React export commands.
  - Added sprite-python UI fixture coverage.
- `b410d7b test(cli): cover agentic cli mode`
  - Added an integration test that runs the built `strut` binary through inspect, plan, dry-run patch, patch, verify, render, and export dry-run.
- `docs(cli): document agentic strut workflow`
  - Adds CLI reference docs, integration recipes, and this Phase 5 report.

## Verification Commands

Focused commands run during implementation:

| Command | Result |
|---|---|
| `cargo test -p strut-cli` | PASS, 3 unit tests and 1 integration test. |
| `$env:PYTHONPATH='src'; python -m pytest tests` from `packages/strut-python` | PASS, 22 tests. |
| `target\debug\strut.exe sprite plan "make a calm loader microinteraction" --json --dry-run --explain` | PASS, emitted validated `strut.cli.plan.v1` JSON. |
| `target\debug\strut.exe patch --scene <temp>\scene.strut --from <temp>\plan.json --dry-run --json` | PASS, no mutation. |
| `target\debug\strut.exe patch --scene <temp>\scene.strut --from <temp>\plan.json --json` | PASS, wrote validated scene. |
| `target\debug\strut.exe verify <temp>\scene.strut --json` | PASS. |
| `target\debug\strut.exe render --scene <temp>\scene.strut --state loading --out <temp>\proof.svg --json --no-open` | PASS, wrote deterministic SVG proof. |
| `target\debug\strut.exe export react --scene <temp>\scene.strut --out <temp>\react-export --dry-run --json` | PASS. |

Final broad verification:

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, workspace tests green with 1 ignored authenticated Gemini CLI test and existing Studio dead-code warnings. |
| `npm --workspace @strut/studio run check` | PASS. |
| `npm run check` | PASS, with existing Rust dead-code warnings during `cargo check`. |
| `git diff --check` | PASS. |
| Playwright exported-runtime demo harness | PASS, rendered exported scene data in Chromium and captured `screenshots/exported-runtime-demo.png`. |

## Browser QA

Browser plugin controls were not exposed in this thread, so browser QA used Playwright/Chromium. Result: PASS.

Coverage:

- Generated a sprite-python logo plan through the CLI.
- Patched a copied `.strut` scene after Rust validation.
- Rendered a deterministic SVG proof.
- Exported React integration files.
- Loaded exported `scene.json` in a local HTML harness and rendered the scene as SVG in Chromium.
- Captured `screenshots/exported-runtime-demo.png`.

## Computer And Tauri QA

Computer/Tauri QA was not required for core CLI commands. Phase 5 did not change Tauri UI behavior. App handoff remains documented but not implemented, so no native app handoff smoke was run.

## Remaining Risks

- Render output is a deterministic SVG proof, not full runtime/raster animation rendering.
- React export renders static SVG structure and includes enough code for integration, but timeline playback remains future runtime work.
- App handoff is documented but not wired as a CLI command.
- The CLI duplicates the generation-plan validation/conversion shape from the Tauri crate because Phase 3/4 validation helpers are currently private to the app crate. A future cleanup should move that shared operation validation into a reusable Rust crate.

## Phase 6 Recommendation

Proceed to Phase 6 only after review. The recommended next step is end-to-end gallery hardening across dice, logo, loader, mascot, UI microinteraction, and icon/badge examples using the CLI and Studio together.
