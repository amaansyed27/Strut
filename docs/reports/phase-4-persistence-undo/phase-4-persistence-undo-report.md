# Phase 4 Persistence And Undo Report

Date: 2026-06-08

Branch: `codex/phase-1-ai-editor-shell`

## Summary

Phase 4 is implemented, review-fixed, and verified. Strut Studio now treats validated scene/project documents and operation batches as the durable source of truth for the editor workflow. Native Studio projects use canonical files:

- `strut.project.json`
- `scenes/main.strut`
- `operations/operation-batches.json`
- `ui/studio-state.json`

The React Studio keeps the Phase 1/2 editor shell intact while adding minimal controls for Save, Reopen, Apply, Reject, Undo, Redo, and operation history. Python and AI generated operation envelopes still pass through Rust validation before becoming persisted documents or applied batches. Phase 5 agentic CLI mode was not started.

## Phase 4 Review Fix Pass

This review pass addressed two Phase 4 persistence hardening findings without starting Phase 5 or agentic CLI mode.

- Rust native persistence now validates operation batch payloads against the current Strut document before saving or accepting loaded operation logs.
- Persisted operation payload validation rejects unsupported operation types, missing `set_property` node ids, unsafe or unsupported property paths, invalid value shapes, empty applied/pending/undone batches, invalid replacement documents, and malformed generated plan operations.
- Sprite-python/generated operation batches still pass through Rust generation-plan validation, then their raw plan operations are cross-checked against the converted validated document before persistence.
- Native project load now confines `mainScene` to a safe relative project-root path. Absolute paths, `..` traversal, root/prefix components, and canonicalized paths outside the project root are rejected before reading.
- Focused Rust tests cover malformed operation payloads, missing targets, unsupported properties/types, empty applied batches, invalid replacement documents, valid sprite-python/generated batches, absolute/traversal `mainScene` paths, valid relative scene paths, and legacy missing-scene fallback.

## Initial Dirty Files

Captured before edits with `git status --short --branch`:

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

These files were pre-existing dirty work and were not reverted or folded into Phase 4.

## Scope Completed

- Defined operation batch persistence records with stable ids, source type, validation result, document revision id, optional prompt/source metadata, operations, and timestamps.
- Added native save/load commands for validated `.strut` packages and operation batch logs.
- Added compatibility loading for legacy `scenes/starter.strut.json` project scenes.
- Added Rust validation command support for generation-plan operation batches, including sprite-python fixtures.
- Converted the Studio operation preview into a validated pending operation batch.
- Implemented real Apply and Reject for validated operation batches.
- Implemented undo/redo around applied operation batches.
- Linked chat messages to operation batch ids.
- Added browser-local snapshot save/reopen fallback for Playwright/browser QA.
- Preserved Phase 1/2 UI structure and only added compact persistence/history controls.

## Canonical Persistence Model

Native projects now use:

| File | Purpose |
|---|---|
| `strut.project.json` | Project manifest with `mainScene`, operation batch log, and UI state paths. |
| `scenes/main.strut` | Validated Strut package written through `strut_format::write_strut_file`. |
| `operations/operation-batches.json` | Durable operation batch log with source, validation, revision, metadata, operation payloads, and timestamps. |
| `ui/studio-state.json` | Active state, selected node id, and layer UI persistence. |

## Changed Files

Schema and native persistence:

- `apps/studio/src-tauri/src/lib.rs`
  - Review fix: Rust-side operation payload validation against the current document.
  - Review fix: safe `mainScene` resolution confined to the project root.

Studio UI:

- `apps/studio/src/App.tsx`
- `apps/studio/src/App.css`

Focused verification:

- `tests/ui/studio_bot_smoke.py`
- `tests/ui/studio_persistence_smoke.py`
- `tests/ui/studio_tauri_persistence_smoke.py`
  - Review pass also added focused Rust unit tests in `apps/studio/src-tauri/src/lib.rs` for payload validation and path safety.

Report artifacts:

- `docs/reports/phase-4-persistence-undo/phase-4-persistence-undo-report.md`
- `docs/reports/phase-4-persistence-undo/phase-4-persistence-undo-report.docx`
- `docs/reports/phase-4-persistence-undo/screenshots/`

## Commits

- `deb98c9 feat(schema): define operation batch persistence`
  - Added native operation batch schema, canonical project files, `.strut` save/load, compatibility legacy scene loading, Rust validation commands, and backend tests.
- `0f743f1 feat(studio): add validated persistence and operation history`
  - Added Save/Reopen, Apply/Reject, Undo/Redo, operation batch state, chat-to-batch links, browser snapshot fallback, and minimal styling.
- `5fe55b3 test(studio): cover persistence undo and invalid documents`
  - Added Phase 4 browser smoke coverage and updated the general smoke for `main.strut` and enabled Apply/Reject.
- `6b09521 test(studio): cover native persistence restart smoke`
  - Added native Tauri/WebView2 persistence smoke that saves, restarts, reloads, and verifies history plus undo/redo.
- `4a62db1 test(studio): refresh operation history smoke label`
  - Updated the general smoke after the operation history label became explicit.
- `chore(report): add phase 4 persistence evidence`
  - Adds this report, DOCX, and screenshot evidence.
- `3735ff7 fix(studio): harden phase 4 native persistence`
  - Added Rust-side persisted operation payload validation against the current document.
  - Added replacement document validation for `replace_document` operation batches.
  - Added generated plan operation validation for sprite-python/raw generated batches.
  - Added safe project-root confinement for manifest `mainScene` paths.
  - Added focused Rust tests for malformed payloads, missing targets, unsupported properties/types, empty applied batches, invalid replacement documents, valid sprite-python/generated batches, absolute/traversal paths, valid relative paths, and legacy fallback.
- `chore(report): refresh phase 4 review evidence`
  - Refreshes this Markdown/DOCX report and screenshot evidence after the review fix pass.

## Verification Commands

| Command | Result |
|---|---|
| `python -m pytest packages/strut-python/tests` | PASS, 18 tests. |
| `$env:PYTHONPATH='src'; python -m strut_python.cli loader --json --out $env:TEMP\strut-loader-plan.json` from `packages/strut-python` | PASS. |
| `cargo fmt --all --check` | PASS. |
| `cargo test -p strut-studio` | PASS, 32 passed, 1 ignored authenticated Gemini CLI test. |
| `cargo test --workspace` | PASS, workspace tests green, same 1 ignored authenticated Gemini CLI test. |
| `npm --workspace @strut/studio run check` | PASS. |
| `npm run check` | PASS, with existing Rust dead-code warnings during `cargo check`. |
| `python tests/ui/studio_bot_smoke.py` | PASS. |
| `python tests/ui/studio_persistence_smoke.py` | PASS. |
| `python tests/ui/studio_tauri_persistence_smoke.py` | PASS. |
| `git diff --check` | PASS. |

Review-pass focused coverage in `cargo test -p strut-studio`:

- Persisted operation payloads reject malformed operations, unsupported operation types, missing `set_property` target ids, unsafe property paths, invalid value shapes, and empty applied batches.
- Replacement operation batches validate both `nextDocument` and non-null `previousDocument` as full Strut documents.
- Sprite-python/generated operation batches persist only after Rust generation-plan validation and Rust operation payload validation.
- Project manifest `mainScene` rejects absolute and traversal paths, accepts valid relative paths, and preserves the legacy missing-scene fallback.

Known warning note: `strut-studio` still emits existing warnings around legacy fallback helpers/fields such as `CHARACTER_DOCUMENT_SYSTEM_PROMPT`, `EditabilityPlan.notes`, `SceneOperation.parent`, and `document_repair_prompt`. These warnings predate Phase 4 behavior and do not fail verification.

## Browser QA

Browser plugin controls were not exposed by tool discovery in this thread, so browser visual QA used Playwright/Chromium against the running Studio.

Result: PASS.

Coverage:

- Dice fixture saved and reopened through browser snapshot fallback.
- Apply/Reject visible behavior works for validated operation batches.
- Undo/redo works after an applied operation.
- Invalid operation is rejected with a useful message: `Operation targets missing node MissingPart`.
- Selection, layers, and inspector still work after reload.

Review-pass note: the Playwright browser smoke was rerun after the Rust persistence hardening, and the browser screenshots in this report were refreshed from that run.

Screenshots:

- `screenshots/browser/browser-01-dice-reopened.png`
- `screenshots/browser/browser-02-apply-ready.png`
- `screenshots/browser/browser-03-applied-history.png`
- `screenshots/browser/browser-04-undo-redo.png`
- `screenshots/browser/browser-05-save-reopen.png`
- `screenshots/browser/browser-06-rejected-history.png`
- `screenshots/browser/browser-07-invalid-rejected.png`

## Computer And Tauri QA

Computer Use controls were not exposed by tool discovery in this thread. Native QA used a Tauri launch with WebView2 remote debugging and Playwright CDP fallback.

Result: PASS.

Coverage:

- Native app launched.
- Native save wrote `scenes/main.strut` and `operations/operation-batches.json` to a temp project.
- Native app restarted and reloaded the saved project from disk.
- Operation history survived restart via the persisted operation batch log.
- Undo/redo worked in the native app after reload.
- No native-only clipping or broken layout was observed in captured screenshots.

Review-pass note: the native Tauri/WebView2 smoke was rerun after the Rust persistence hardening, and the Tauri screenshots in this report were refreshed from that run.

Screenshots:

- `screenshots/tauri/tauri-01-save-applied.png`
- `screenshots/tauri/tauri-02-reopened-history.png`
- `screenshots/tauri/tauri-03-undo-redo.png`

## Before Screenshots

Copied forward from the Phase 3B final state:

- `screenshots/before-phase-3b/browser-phase-3b-sprite-python-smoke.png`
- `screenshots/before-phase-3b/tauri-01-dice-selection.png`
- `screenshots/before-phase-3b/tauri-02-loader-selection.png`

## Visual QA Notes

- The Studio shell remains the same Phase 1/2 structure: sidebar, top project/status bar, AI edit rail, preview, layers, and inspector.
- New persistence controls are compact and do not change the primary layout.
- Apply is disabled for invalid/unvalidated batches and enabled for validated pending batches.
- Operation history records status and source type clearly.
- Browser and Tauri screenshots show no obvious overlapping text, clipped controls, or blank preview regression.
- DOCX render QA could not be completed because `soffice`/LibreOffice was not available on PATH in this environment. The DOCX was generated structurally from the same report content and screenshot list.

## Remaining Risks

- Operation execution is intentionally narrow in Phase 4: manual UI batches currently apply validated `set_property` operations and generated batches are recorded as validated document replacement batches.
- Browser mode uses local validation and browser snapshot fallback because it cannot call native Tauri filesystem commands.
- Native save/load currently requires an active project path; a richer file picker/open-project flow remains future UI work.
- Rust now checks persisted revision ids are present and use the expected `rev-` marker, but it does not enforce exact equality with a Rust-computed revision because the React UI and Rust backend currently compute different Phase 4 revision shapes. A shared content-hash revision should replace this limitation before stricter multi-writer workflows.
- Raw sprite-python/generated plan operations use pre-document semantic ids, so Rust validates those generated operation targets against the converted document's node names as well as stable ids. The converted `.strut` document itself remains the authoritative validated scene artifact.
- Existing Rust dead-code warnings remain from earlier fallback/provider code.

## Phase 5 Recommendation

Proceed to Phase 5 only after review. The recommended next step is agentic CLI mode built on the same validated operation and `.strut` persistence boundary:

- inspect project and scene files
- validate and patch scene files through Rust
- render proof outputs
- export integration code
- never mutate a project from Python or CLI output unless Rust validation accepts the operation batch

Phase 5 CLI mode was not started in this phase.
