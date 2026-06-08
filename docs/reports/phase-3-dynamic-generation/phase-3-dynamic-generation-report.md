# Phase 3 Dynamic Generation Planning And Patch Operations Report

Date: 2026-06-08

Branch: `codex/phase-1-ai-editor-shell`

## Summary

Phase 3 replaces the previous static mascot-style generation path with a subject-aware generation planning pipeline. Provider responses now prefer a structured `{ plan, operations }` payload, validate the semantic plan and operation list, and only then convert the result into a Strut document. Whole-document parsing remains available only as an explicit validated fallback.

The implementation keeps Phase 1 and Phase 2 surfaces intact. Generated documents still render in Studio, expose semantic layer names, populate the selected-part inspector, and work with the existing preview-only operation log. Phase 4 persistence/undo and agentic CLI mode were not started.

## Commits

- `d87ee4d feat(schema): add generation plan operations`
- `53cea83 test(studio): cover dynamic generation subjects`
- report commit: `chore(report): add phase 3 generation report`

## Changed Files

- `apps/studio/src-tauri/src/lib.rs`
  - Added generation plan, semantic part, motion role, timeline, editability, and scene operation schemas.
  - Added plan validation and operation validation before document conversion.
  - Updated provider prompts to request subject-aware plans and explicit operation lists.
  - Added validated fallback handling for legacy whole-document responses.
  - Added focused Rust tests for dice, logo, loader, mascot, invalid references, invalid geometry, and mascot-only anatomy rejection.
- `crates/strut-core/src/lib.rs`
  - Added optional node role metadata for semantic editability.
- `apps/studio/src/App.tsx`
  - Displayed generation plan metadata in the existing assistant activity surface without redesigning the Phase 1/2 UI.
- `tests/ui/studio_bot_smoke.py`
  - Added deterministic Phase 3 fixture coverage for dice, abstract logo, loader, and mascot documents.
  - Verified selection, layers, and inspector behavior after generated documents are created.
- `docs/reports/phase-3-dynamic-generation/`
  - Added Markdown and DOCX evidence report plus screenshots.

## Schema And Operation Model

The new generation schema represents:

- Subject classification, such as `dice`, `logo`, `loader`, or `mascot`.
- Semantic part plans, including stable part ids, labels, geometry, styles, and editability roles.
- Motion role plans that map semantic parts to animation responsibilities.
- State and timeline plans with named tracks and keyframes.
- Editability constraints describing what can be selected, grouped, locked, or safely exposed to the inspector.

The supported operation subset is intentionally focused:

- `create_node`
- `group_nodes`
- `set_property`
- `add_state`
- `add_timeline`
- `add_keyframe`
- `bind_property`
- `emit_event`

The pipeline now prefers:

`prompt -> generation plan -> validated operations -> Strut document`

over:

`prompt -> arbitrary whole document`

The fallback whole-document path remains for compatibility, but it is parsed explicitly and validated through existing Strut document repair/validation helpers rather than silently bypassing the new plan path.

## Validation Coverage

Validation now rejects or blocks conversion for:

- Duplicate semantic part ids, motion role ids, timeline ids, state ids, and operation ids.
- Missing part references from motion roles and editability constraints.
- Timeline tracks targeting unknown nodes.
- Timeline/keyframe references to unknown states or properties.
- Invalid geometry, including non-positive sizes and non-finite coordinates.
- Invalid parent/group references in operation lists.
- Mascot-only anatomy used for non-mascot subjects, such as `Head`, `Eyes`, `Arms`, or `Body` in dice/logo/loader plans.

## Subject Diversity Evidence

Required subject cases were covered in Rust and UI smoke tests:

- Rolling dice produces semantic dice parts such as `DieBody`, `FrontFace`, `TopFace`, `Pips`, `EdgeHighlight`, and `SettleShadow`; it does not produce mascot anatomy.
- Abstract logo produces `PrimaryMark`, `Wordmark`, `AccentStroke`, and `RevealMask`; it does not require a face.
- Loader produces `Track`, `ActiveSegment`, `PulseDot`, and `ProgressSweep`; it does not require a face or body.
- Mascot prompts can still produce mascot anatomy when requested.
- Generated scenes expose editable semantic parts and named timelines.
- Invalid plans are rejected before becoming Strut documents.

Low-energy motion remains a style option, but the prompt now frames it as subtle, calm, breathable motion rather than implying a face, pet, mascot, or fixed body model.

## Before Screenshots

Phase 2 final-state screenshots were copied forward as the before baseline:

- `screenshots/before-phase-2/before-browser-selection.png`
- `screenshots/before-phase-2/before-browser-operation-preview.png`
- `screenshots/before-phase-2/before-tauri-selection.png`

## Browser QA

Browser opened `http://127.0.0.1:1421`, confirmed the Strut Studio shell loaded, and reported no console warnings or errors. The Browser runtime could not seed localStorage fixtures because the in-page sandbox exposed `localStorage` as unavailable/read-only, so deterministic fixture QA was completed with Playwright against the same localhost app.

Browser and Playwright evidence:

- `screenshots/browser/browser-00-in-app-home-loaded.png`
- `screenshots/browser/browser-01-dice-selection.png`
- `screenshots/browser/browser-02-logo-fixture.png`
- `screenshots/browser/browser-03-loader-selection.png`
- `screenshots/browser/browser-04-mascot-selection.png`

Playwright verified:

- Dice generation fixture renders and selects semantic dice layers.
- Logo fixture renders without face requirements.
- Loader fixture renders with loader roles and inspector metadata.
- Mascot fixture still supports mascot anatomy when requested.
- Selection, layers, and inspector continue to work after generated documents are loaded.

## Computer And Tauri QA

Native QA used `npm --workspace @strut/studio run tauri -- dev` with WebView2 remote debugging enabled for fixture seeding and screenshots, plus Computer Use for native-window inspection and interaction.

Evidence:

- `screenshots/tauri/tauri-01-dice-selection.png`
- `screenshots/tauri/tauri-02-loader-fixture.png`
- `screenshots/tauri/tauri-03-mascot-fixture.png`
- `screenshots/tauri/tauri-04-native-body-selected.png`

Computer Use verified:

- The native Strut Studio app launched.
- Phase 3 seeded examples rendered in the native WebView.
- Layers and inspector remained interactive.
- Clicking a native layer row selected the expected semantic node and updated the inspector.
- No native-only clipping was observed in the checked windows.

Provider generation was not exercised live because authenticated provider credentials were not available in this run. Deterministic fixtures and Rust parser tests covered the provider response shape and conversion path.

## Verification Commands

- `npm --workspace @strut/studio run check` - passed.
- `npm run check` - passed.
- `cargo test --workspace` - passed.
- `cargo test -p strut-studio -- --nocapture` - passed, with one ignored authenticated Gemini CLI end-to-end test.
- `python tests/ui/studio_bot_smoke.py` - passed.
- `python -m py_compile tests\ui\studio_bot_smoke.py` - passed.
- `cargo fmt --all --check` - passed.
- `git diff --check` - passed.

Non-fatal Rust warnings remain for unused fallback helpers/fields in `strut-studio`, including the legacy whole-document prompt and repair prompt. They are intentionally still present for compatibility but should be cleaned up when the fallback path is revisited.

## Placeholder Or Deferred Work

- Production Apply/Reject, persistence, and undo/redo were not implemented.
- Agentic CLI mode was not started.
- Live provider authentication was unavailable, so provider execution was not verified beyond prompt shape, parser tests, and deterministic fixtures.
- Operation preview remains preview-only from Phase 2; Phase 3 validates operation conversion but does not turn the preview log into persistent document history.
- Whole-document parsing still exists as a validated compatibility fallback.

## Phase 4 Recommendation

Phase 4 should build persistent operation application and undo/redo on top of the validated operation model introduced here. Recommended next steps:

- Store operation batches with stable ids, source plan metadata, validation result, and document revision references.
- Implement Apply/Reject behavior only for validated operations.
- Add undo/redo around operation batches rather than raw document snapshots.
- Keep provider output behind the plan/operation validator so persistence never stores unvalidated AI documents.

