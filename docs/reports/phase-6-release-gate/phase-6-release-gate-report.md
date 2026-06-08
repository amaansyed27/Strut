# Phase 6 Release Gate Report

Date: 2026-06-09

Branch: `codex/phase-1-ai-editor-shell`

## Summary

Phase 6 is complete as an end-to-end hardening and evidence pass. The corrected Strut architecture was verified across rolling dice, abstract logo reveal, loader/progress animation, mascot idle animation, UI microinteraction, and icon/badge animation examples.

The gate proves the intended flow:

```text
sprite-python or deterministic CLI plan -> generation plan + operations -> Rust validation -> .strut document -> CLI inspect/patch/verify/render/export -> Studio/native load where feasible
```

No new product phase beyond Phase 6 was started. The Rust/Tauri/React app architecture remains in place, and sprite-python still emits generation plans and operation lists rather than unchecked final documents.

## Initial Dirty Files

Captured before Phase 6 edits with `git status --short --branch`:

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

These files were pre-existing dirty work and were not reverted or folded into Phase 6 commits.

## End-To-End Gallery Matrix

| Example | Producer | Emits plan + operations only | Rust validation | `.strut` patch | CLI inspect/verify | Render proof | React export/runtime proof | Studio selectable | Native load |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Rolling dice | sprite-python/CLI | PASS | PASS | PASS | PASS | PASS | PASS | PASS | Covered by native persistence smoke |
| Abstract logo reveal | sprite-python/CLI | PASS | PASS | PASS | PASS | PASS | PASS | PASS | Not separately native-loaded |
| Loader/progress | sprite-python/CLI | PASS | PASS | PASS | PASS | PASS | PASS | PASS | Not separately native-loaded |
| Mascot idle | sprite-python/CLI | PASS | PASS | PASS | PASS | PASS | PASS | PASS | Not separately native-loaded |
| UI microinteraction | sprite-python/CLI | PASS | PASS | PASS | PASS | PASS | PASS | PASS | Not separately native-loaded |
| Icon/badge | sprite-python/CLI | PASS | PASS | PASS | PASS | PASS | PASS | PASS | PASS |

Native loading for every gallery file would be slow and repetitive in the current test harness, so Phase 6 uses the native persistence smoke for dice and a new native smoke for a CLI-generated icon/badge `.strut` project. Browser/Playwright covers all six examples.

## Example Evidence

### Rolling Dice

- Plan subject: `dice`.
- Semantic parts include `DieBody`, `FrontFace`, `TopFace`, `Pips`, `EdgeHighlight`, and `SettleShadow`.
- Non-mascot anatomy check: PASS.
- CLI project flow: `inspect project -> sprite plan -> patch --dry-run -> patch -> inspect scene -> verify -> render -> export react`.
- Studio screenshot: `screenshots/studio/studio-phase6-dice.png`.
- Runtime screenshot: `screenshots/runtime/screenshots/runtime-dice.png`.
- Render proof: `screenshots/runtime/renders/dice.svg`.

### Abstract Logo Reveal

- Plan subject: `logo`.
- Semantic parts include `PrimaryMark`, `Wordmark`, `AccentStroke`, `RevealMask`, `AnchorGrid`, and `Glow`.
- Non-mascot anatomy check: PASS.
- Studio screenshot: `screenshots/studio/studio-phase6-logo.png`.
- Runtime screenshot: `screenshots/runtime/screenshots/runtime-logo.png`.
- Render proof: `screenshots/runtime/renders/logo.svg`.

### Loader/Progress

- Plan subject: `loader`.
- Semantic parts include `Track`, `ActiveSegment`, `PulseDot`, `ProgressSweep`, `Glow`, and `CenterLabel`.
- Non-mascot anatomy check: PASS.
- Studio screenshot: `screenshots/studio/studio-phase6-loader.png`.
- Runtime screenshot: `screenshots/runtime/screenshots/runtime-loader.png`.
- Render proof: `screenshots/runtime/renders/loader.svg`.

### Mascot Idle

- Plan subject: `mascot`.
- Semantic parts include `Body`, `Head`, `Eyes`, `Arms`, `AccentBadge`, and `GroundShadow`.
- Low-energy companion style check: PASS. The mascot builder uses quiet idle/bob/nudge motion and explicitly allows anatomy only because the subject is mascot.
- Studio screenshot: `screenshots/studio/studio-phase6-mascot.png`.
- Runtime screenshot: `screenshots/runtime/screenshots/runtime-mascot.png`.
- Render proof: `screenshots/runtime/renders/mascot.svg`.

### UI Microinteraction

- Plan subject: `ui`.
- Semantic parts include `ButtonSurface`, `ButtonLabel`, `FocusRing`, `CheckMark`, and `HoverGlow`.
- Non-mascot anatomy check: PASS.
- Studio screenshot: `screenshots/studio/studio-phase6-ui.png`.
- Runtime screenshot: `screenshots/runtime/screenshots/runtime-ui.png`.
- Render proof: `screenshots/runtime/renders/ui.svg`.

### Icon/Badge

- Plan subject: `badge`.
- Semantic parts include `BadgePlate`, `InnerShield`, `SparkGlyph`, `OrbitStroke`, `StatusDot`, and `BadgeLabel`.
- Non-mascot anatomy check: PASS.
- Studio screenshot: `screenshots/studio/studio-phase6-icon-badge.png`.
- Runtime screenshot: `screenshots/runtime/screenshots/runtime-icon-badge.png`.
- Native Tauri screenshot: `screenshots/tauri/tauri-phase6-cli-icon-badge.png`.
- Render proof: `screenshots/runtime/renders/icon-badge.svg`.

## CLI Transcript Summary

The full JSON transcript from `tests/ui/phase6_exported_runtime_smoke.py` is saved at:

- `screenshots/runtime/command-transcript.json`

For each gallery example, the transcript records:

- `sprite plan ... --json --dry-run --explain`
- `patch --dry-run --json`, with byte-for-byte no-mutation check
- `patch --json`
- `verify --json`
- `render --json --no-open`
- `export react --json`

All six examples reported `validated patch and wrote scene`, `scene document is valid`, a deterministic SVG render path, and three export files: `scene.json`, `StrutAnimation.tsx`, and `README.md`.

## Browser And Playwright Evidence

Browser plugin controls were not exposed in this thread, so browser QA used Playwright/Chromium.

Result: PASS.

Coverage:

- Export/runtime smoke rendered all six exported scenes through a local HTML harness with no console or page errors.
- Studio gallery smoke seeded all six gallery examples, switched chats, selected representative layers, and verified the selected-part inspector.
- Studio gallery smoke captured light and dark theme screenshots and verified `html[data-theme]` switched correctly.
- Generated scene colors stayed inside the preview/runtime content and did not alter the app chrome/theme.

Screenshots:

- `screenshots/runtime/screenshots/runtime-dice.png`
- `screenshots/runtime/screenshots/runtime-logo.png`
- `screenshots/runtime/screenshots/runtime-loader.png`
- `screenshots/runtime/screenshots/runtime-mascot.png`
- `screenshots/runtime/screenshots/runtime-ui.png`
- `screenshots/runtime/screenshots/runtime-icon-badge.png`
- `screenshots/studio/studio-phase6-dice.png`
- `screenshots/studio/studio-phase6-logo.png`
- `screenshots/studio/studio-phase6-loader.png`
- `screenshots/studio/studio-phase6-mascot.png`
- `screenshots/studio/studio-phase6-ui.png`
- `screenshots/studio/studio-phase6-icon-badge.png`
- `screenshots/studio/studio-phase6-selection-layers-inspector.png`
- `screenshots/studio/studio-phase6-light-theme.png`
- `screenshots/studio/studio-phase6-dark-theme.png`

## Computer/Tauri Evidence

Computer Use tools were not exposed by tool discovery in this thread. Native QA used the repo's Tauri dev launch with WebView2 remote debugging and Playwright CDP, matching the existing Phase 4 native QA approach.

Result: PASS.

Coverage:

- Native app launched cleanly.
- Native persistence smoke saved, reopened, and verified a dice scene with selection/layers/inspector and undo/redo.
- New Phase 6 native smoke created a real CLI-patched icon/badge `.strut` project, loaded it in native Studio through `Reopen`, selected `BadgePlate`, verified the inspector role text, and confirmed no mascot-only `Head` layer was present.
- Tauri processes were stopped by the smoke scripts after completion.

Screenshots:

- `screenshots/tauri/tauri-phase6-cli-icon-badge.png`
- `screenshots/tauri-persistence/tauri-01-save-applied.png`
- `screenshots/tauri-persistence/tauri-02-reopened-history.png`
- `screenshots/tauri-persistence/tauri-03-undo-redo.png`

## Changed Files

Sprite-python gallery:

- `packages/strut-python/src/strut_python/builders.py`
- `packages/strut-python/src/strut_python/cli.py`
- `packages/strut-python/src/strut_python/__init__.py`
- `packages/strut-python/examples/icon.py`
- `packages/strut-python/fixtures/icon.plan.json`
- `packages/strut-python/tests/test_examples.py`

Rust CLI and validation:

- `crates/strut-cli/src/main.rs`
- `crates/strut-cli/tests/agentic_cli.rs`
- `apps/studio/src-tauri/src/lib.rs`

Phase 6 QA scripts:

- `tests/ui/phase6_exported_runtime_smoke.py`
- `tests/ui/studio_phase6_gallery_smoke.py`
- `tests/ui/studio_phase6_tauri_gallery_smoke.py`

Report and evidence:

- `docs/reports/phase-6-release-gate/phase-6-release-gate-report.md`
- `docs/reports/phase-6-release-gate/phase-6-release-gate-report.docx`
- `docs/reports/phase-6-release-gate/screenshots/`

## Commits

- `ad461d3 test(e2e): add sprite python gallery coverage`
  - Added the icon/badge sprite-python builder, fixture, and example script.
  - Expanded Python regression coverage to all six gallery classes.
  - Expanded Rust CLI integration coverage to run the full project flow for all six examples.
  - Expanded Tauri fixture validation to include UI and icon/badge plans.
  - Added Phase 6 Playwright smokes for exported runtime proof, Studio gallery proof, and native Tauri loading of a CLI-generated icon/badge scene.

The report/evidence commit is expected to be a separate `docs(report): add final sprite python revamp evidence` commit after this document is written.

## Verification Commands

| Command | Result |
|---|---|
| `python -m pytest packages/strut-python/tests` | PASS, 32 tests. |
| From `packages/strut-python`: `$env:PYTHONPATH='src'; python -m strut_python.cli loader --json --out $env:TEMP\strut-loader-plan.json` | PASS. |
| `cargo fmt --all --check` | PASS. |
| `cargo test -p strut-cli` | PASS, 3 unit tests and 5 integration tests. |
| `cargo test -p strut-studio` | PASS, 34 passed, 1 ignored authenticated Gemini CLI test. Existing dead-code warnings remain. |
| `cargo test --workspace` | PASS, workspace tests green, same ignored authenticated Gemini CLI test and existing Studio warnings. |
| `npm --workspace @strut/studio run check` | PASS. |
| `npm run check` | PASS, including runtime-web, runtime-react, runtime-next, and cargo check. Existing Studio warnings remain. |
| `python tests/ui/studio_bot_smoke.py` | PASS. |
| `python tests/ui/studio_persistence_smoke.py` | PASS. |
| `python tests/ui/studio_tauri_persistence_smoke.py` | PASS. |
| `python tests/ui/phase6_exported_runtime_smoke.py` | PASS. |
| `python tests/ui/studio_phase6_gallery_smoke.py` | PASS. |
| `python tests/ui/studio_phase6_tauri_gallery_smoke.py` | PASS. |
| `git diff --check` | PASS. |

## Quality Checks

- Non-mascot examples do not use mascot-only anatomy: PASS.
- Mascot example can reach the intended low-energy companion style: PASS.
- Dynamic naming is present and subject-specific: PASS.
- Generated examples have semantic editable parts and named timelines: PASS.
- CLI JSON outputs are deterministic and parseable: PASS.
- Failure paths are actionable and do not mutate files: PASS, covered by patch dry-run/no-mutation, tampered top-level plan mismatch, invalid replacement document, and export preflight tests.
- Export output is usable by a coding agent in a sample app: PASS, exported `scene.json`, `StrutAnimation.tsx`, and README are written for all six examples and rendered through a local browser harness.
- UI still reads as a coherent AI editor: PASS in Studio screenshots.
- Generated scene colors do not alter app chrome/theme: PASS in light/dark Studio screenshots.
- Artifacts remain agent-editable and human-readable: PASS, plans, exported scene JSON, React TSX, README files, SVG proofs, and HTML harnesses are saved under the report evidence folder.

## DOCX Render QA

DOCX report path:

- `docs/reports/phase-6-release-gate/phase-6-release-gate-report.docx`

Renderers/checks attempted:

```text
where.exe soffice
C:\Users\Amaan\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe C:\Users\Amaan\.codex\plugins\cache\openai-primary-runtime\documents\26.601.10930\skills\documents\render_docx.py docs\reports\phase-6-release-gate\phase-6-release-gate-report.docx --output_dir docs\reports\phase-6-release-gate\docx-render
```

Blocker:

```text
INFO: Could not find files for the given pattern(s).
FileNotFoundError: [WinError 2] The system cannot find the file specified
```

LibreOffice/`soffice` is not available on PATH in this environment, so DOCX-to-PDF/page-PNG rendering could not be completed. The screenshot gallery is the fallback visual proof, specifically:

- all six runtime screenshots under `screenshots/runtime/screenshots/`
- all six Studio screenshots under `screenshots/studio/`
- `screenshots/studio/studio-phase6-selection-layers-inspector.png`
- `screenshots/studio/studio-phase6-light-theme.png`
- `screenshots/studio/studio-phase6-dark-theme.png`
- native Tauri screenshots under `screenshots/tauri/` and `screenshots/tauri-persistence/`

DOCX media structural check: PASS. The generated DOCX contains five embedded screenshot media files under `word/media/`.

## Visual QA Notes

- The Studio shell remains the Phase 1/2 AI editor: left edit rail, preview workspace, layers, inspector, timeline controls, and operation preview.
- Scene palettes are varied by example but do not bleed into the app chrome.
- Layer rows and inspector fields retain readable semantic names.
- Selection halos and layer selection remain coherent across dice, logo, loader, mascot, UI, and icon/badge.
- Light and dark theme screenshots show stable chrome independent of generated scene colors.
- No console errors were observed in the tested browser paths.
- No native-only clipping or broken layout was observed in the captured Tauri icon/badge and persistence screenshots.

## Remaining Risks And Deferred Work

- Render proof remains deterministic SVG structure, not full runtime/raster animation rendering.
- React export remains static SVG structure; timeline playback is still future runtime work.
- CLI/app handoff remains documented but not wired as an automatic command.
- The native QA pass loaded a CLI-generated icon/badge and exercised dice persistence, but did not natively open every gallery example.
- Existing Studio Rust dead-code warnings remain from earlier fallback/provider code.
- Browser QA used Playwright/Chromium because Browser plugin controls were not exposed.
- Computer Use controls were not exposed; native QA used Tauri/WebView2 CDP.

## Recommended Next Product Steps

1. Promote the deterministic SVG proof into a real render/runtime pipeline with timeline playback.
2. Wire CLI app handoff so `strut` can open Studio with a pending validated patch and report back verification.
3. Move shared generation-plan/operation validation into a reusable Rust crate to reduce CLI/Tauri duplication.
4. Add a gallery browser inside Studio backed by validated `.strut` fixtures rather than localStorage seeding.
5. Expand native QA to load all gallery examples once native open-project ergonomics are smoother.

Phase 6 did not start a new feature phase beyond the release gate.
