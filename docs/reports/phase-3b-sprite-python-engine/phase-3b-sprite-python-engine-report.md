# Phase 3B Sprite-Python Architecture Lock And Engine Spike Report

Date: 2026-06-08

Branch: `codex/phase-1-ai-editor-shell`

## Summary

Phase 3B is closed after the completion fix pass. Strut now has a documented Rust/Tauri plus sprite-python split, a minimal deterministic Python sprite/vector authoring package, generated `{ plan, operations }` fixtures for dice/logo/loader/mascot, committed source roadmap docs, and Rust tests proving those Python envelopes validate through the existing Phase 3A generation-plan and operation conversion path.

Python is an authoring/compiler layer only. It does not emit unchecked final `.strut` documents and does not bypass Rust validation.

Phase 4 persistence/undo and agentic CLI mode were not started.

## Initial Dirty Files

Captured with `git status --short --branch` before edits:

```text
## codex/phase-1-ai-editor-shell
 M README.md
 M docs/README.md
 M docs/guides/generate-a-character.md
 M docs/learn/first-animation.md
 M docs/learn/quick-start.md
 M docs/learn/what-is-strut.md
?? docs/learn/motion-language.md
?? docs/superpowers/
```

These pre-existing dirty files were preserved and not reverted.

## Completion Fix Pass Dirty Files

Captured with `git status --short --branch` at the start of the completion fix pass:

```text
## codex/phase-1-ai-editor-shell
 M README.md
 M docs/README.md
 M docs/guides/generate-a-character.md
 M docs/learn/first-animation.md
 M docs/learn/quick-start.md
 M docs/learn/what-is-strut.md
?? docs/learn/motion-language.md
?? docs/superpowers/
```

The fix pass committed only the Phase 3B plan docs from `docs/superpowers/` and left the unrelated dirty learn/readme files untouched.

## Scope Completed

- Added `docs/internal/sprite-python-architecture.md` to lock the architecture boundary.
- Added `packages/strut-python/` with sprite/vector primitives:
  - `Scene`, `Sprite`, `Group`, `Rect`, `Ellipse`, `Path`, `Text`
  - `Binding`, `State`, `Timeline`, `Keyframe`
- Added motion primitives:
  - `idle_breathe`, `soft_bob`, `tiny_tilt`, `settle`, `reveal`, `pulse`, `progress_sweep`, `attention_nudge`
- Added subject builders:
  - rolling dice
  - abstract logo reveal
  - loader/progress animation
  - mascot idle animation
  - UI microinteraction prototype
- Added deterministic examples and committed generated plan fixtures for dice, logo, loader, and mascot.
- Added Python tests for determinism, JSON examples, subject classification, mascot anatomy rules, and the no-final-document invariant.
- Added Rust-side validation tests that read Python-generated fixtures and convert them through `document_from_generation_plan_text`.
- Updated the existing UI smoke fixture labels for Phase 3B sprite-python coverage without redesigning the UI.
- Committed the roadmap and Phase 3B source plan docs so a clean checkout contains the plan source used for this phase.
- Applied `cargo fmt --all` to the Phase 3B Rust fixture test formatting.
- Clarified the reproducible Python module command as a PowerShell command run from `packages/strut-python` with `PYTHONPATH=src`.

## Architecture Boundary

Locked flow:

```text
prompt -> sprite-python authoring model -> Strut generation plan + operations -> Rust validation -> Strut document -> Studio preview/export
```

Rust/Tauri remains:

- desktop shell
- native filesystem and provider bridge
- validation and security boundary
- conversion from validated operations into Strut documents
- project workflow host

Sprite-python is:

- an agent-friendly authoring SDK
- a deterministic sprite/vector planning layer
- an emitter of inspectable generation plans and operations

Sprite-python is not:

- a persistence layer
- a CLI mode
- a final document writer
- a replacement for Rust validation
- a Codex pet or fixed mascot atlas importer

## Changed Files

Architecture docs:

- `docs/internal/sprite-python-architecture.md`
- `docs/superpowers/plans/2026-06-05-strut-ai-editor-revamp.md`
- `docs/superpowers/plans/2026-06-08-strut-sprite-python-replan.md`

Python package:

- `.gitignore`
- `packages/strut-python/pyproject.toml`
- `packages/strut-python/src/strut_python/__init__.py`
- `packages/strut-python/src/strut_python/model.py`
- `packages/strut-python/src/strut_python/motion.py`
- `packages/strut-python/src/strut_python/builders.py`
- `packages/strut-python/src/strut_python/cli.py`
- `packages/strut-python/examples/dice.py`
- `packages/strut-python/examples/logo.py`
- `packages/strut-python/examples/loader.py`
- `packages/strut-python/examples/mascot.py`
- `packages/strut-python/fixtures/dice.plan.json`
- `packages/strut-python/fixtures/logo.plan.json`
- `packages/strut-python/fixtures/loader.plan.json`
- `packages/strut-python/fixtures/mascot.plan.json`

Validation and smoke tests:

- `apps/studio/src-tauri/src/lib.rs`
- `packages/strut-python/tests/test_examples.py`
- `tests/ui/studio_bot_smoke.py`

Report artifacts:

- `docs/reports/phase-3b-sprite-python-engine/phase-3b-sprite-python-engine-report.md`
- `docs/reports/phase-3b-sprite-python-engine/phase-3b-sprite-python-engine-report.docx`
- `docs/reports/phase-3b-sprite-python-engine/screenshots/`
- `docs/reports/phase-3b-sprite-python-engine/rendered/README.md`

## Commits

- `f00297f docs(arch): define sprite python engine boundary`
- `475899a feat(sprite-python): add authoring model prototype`
- `acc2967 test(sprite-python): validate generated subject fixtures`
- `c3926af test(studio): cover sprite python fixture smoke`
- `73ae9b1 chore(report): add phase 3b sprite python evidence`
- `c49d8e1 docs(plan): commit strut sprite python roadmap`
- `4709313 chore(format): apply rustfmt to phase 3b tests`
- report refresh commit: `chore(report): refresh phase 3b completion evidence` (current report commit)

## Verification Commands

- `python -m pytest packages/strut-python/tests` - passed, 18 tests.
- From `packages/strut-python`: `$env:PYTHONPATH='src'; python -m strut_python.cli loader --json --out $env:TEMP\strut-loader-plan.json` - passed.
- `cargo fmt --all --check` - passed.
- `cargo test -p strut-studio` - passed, 21 passed, 1 ignored authenticated Gemini CLI test.
- `cargo test --workspace` - passed, workspace tests green, same 1 ignored authenticated Gemini CLI test.
- `npm --workspace @strut/studio run check` - passed.
- `npm run check` - passed.
- `python tests/ui/studio_bot_smoke.py` - passed.
- `git diff --check` - passed.

The Python module command is reproducible from `packages/strut-python` only when `PYTHONPATH=src` is set, unless the package is installed in editable or normal mode. This phase used the explicit PowerShell form above rather than adding an installed console-script workflow.

Rust warning note: `strut-studio` still emits existing unused-code warnings around legacy fallback helpers/fields, including `CHARACTER_DOCUMENT_SYSTEM_PROMPT`, `EditabilityPlan.notes`, `SceneOperation.parent`, and `document_repair_prompt`. These warnings predate Phase 3B behavior and do not fail verification.

## Browser QA

The Codex in-app Browser control tools were not exposed by tool discovery in this thread, so browser visual QA used Playwright/Chromium through `python tests/ui/studio_bot_smoke.py`, followed by direct visual inspection of the resulting screenshot.

Evidence:

- `screenshots/browser/browser-phase-3b-sprite-python-smoke.png`

Result:

- Passed.
- Phase 3B sprite-python fixture project rendered in the Studio browser surface.
- Mascot preview was nonblank.
- Layers, state buttons, left AI rail, and inspector area were visible with no observed overlap.
- The fixture message states it was generated through sprite-python and validated Strut operations.

## Computer And Tauri QA

Computer Use desktop controls were not exposed by tool discovery in this thread. Native QA was completed by launching Tauri with WebView2 remote debugging, seeding deterministic sprite-python fixture documents into the native WebView, and capturing screenshots from the running Tauri app.

Evidence:

- `screenshots/tauri/tauri-01-dice-selection.png`
- `screenshots/tauri/tauri-02-loader-selection.png`

Result:

- Passed.
- Native Tauri app launched.
- Dice fixture rendered, `DieBody` could be selected, and the inspector showed `volume`.
- Loader fixture rendered, `ActiveSegment` could be selected, and the inspector showed `active arc`.
- Layers and selection stayed synchronized in the native app.

## Screenshot Gallery

Before evidence copied forward from Phase 3A:

- `screenshots/before-phase-3a/browser-01-dice-selection.png`
- `screenshots/before-phase-3a/browser-03-loader-selection.png`
- `screenshots/before-phase-3a/tauri-01-dice-selection.png`

After evidence from Phase 3B:

- `screenshots/browser/browser-phase-3b-sprite-python-smoke.png`
- `screenshots/tauri/tauri-01-dice-selection.png`
- `screenshots/tauri/tauri-02-loader-selection.png`

## Remaining Risks

- The Python engine is a minimal spike, not a full production sprite compiler.
- UI microinteraction builder exists in the package but was not part of the four required deterministic fixture assertions.
- Rust validation functions are still private to `strut-studio`; a later phase may want a shared validation crate API if CLI work is explicitly started.
- Browser and Computer Use plugin controls were unavailable in this thread, so Browser/Computer QA used Playwright and Tauri WebView2 CDP fallbacks.
- Existing Rust warnings remain around legacy fallback helpers and unused fields.

## Phase 4 Recommendation

Start Phase 4 only after this report is reviewed. Build persistence around validated operation batches, not raw Python output or unvalidated documents.

Recommended Phase 4 shape:

- persist validated Strut documents created from operation batches
- store operation source metadata such as `sprite-python`, `ai`, or `manual`
- keep validation result and document revision ids with each batch
- implement Apply/Reject and undo/redo only for accepted operation batches
- keep Python output unable to mutate projects unless Rust validation accepts it

## Boundary Confirmation

Phase 4 persistence/undo was not started. Agentic CLI mode was not started. The UI was not redesigned. Codex pets, pet atlas imports, and fixed mascot/face models were not introduced.
