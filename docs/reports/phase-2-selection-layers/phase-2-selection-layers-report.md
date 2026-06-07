# Phase 2 Selection, Layers, And Editable Scene Structure Report

Phase 2 makes the Strut Studio preview and side panels understand editable semantic parts. It adds real shared selection state, selectable SVG preview nodes, layer metadata, a read-only selected-part inspector, and inspectable preview-only operation records.

## Summary

- Phase 2 scope completed: semantic node selection, preview hit selection, layer selection, layer visibility/lock UI state, selected-part inspector, AI edit context, preview-only pending operation records, persisted selected node id, persisted layer UI state, and persisted operation history.
- Phase boundary preserved: no Phase 3 dynamic generation planning, no full patch execution, no agentic CLI mode, and no Apply/Reject mutation path were started.
- Operation Apply/Reject remain disabled intentionally because there is no safe tested operation application model in Phase 2.
- Browser plugin controls were not exposed in this thread, so deterministic Browser QA used Playwright against localhost as a fallback.
- Computer Use controls were not exposed in this thread, so native Tauri QA used the actual Tauri WebView2 shell with remote debugging enabled for deterministic inspection and screenshots.

## Commits

- `2c47162` feat(studio): add shared scene selection surfaces
- `c8ec236` test(studio): cover phase 2 selection workflow
- `e427599` fix(studio): separate layer row hit targets

## Changed Files

- `apps/studio/src/App.tsx`
- `apps/studio/src/App.css`
- `tests/ui/studio_bot_smoke.py`
- `docs/reports/phase-2-selection-layers/**`

## Implementation Notes

- Selection is stored per active chat as `selectedNodeId` and shared by preview, layer list, selected-part inspector, and AI edit context.
- Layer visibility and lock state are stored as lightweight UI state. Visibility hides nodes in preview and clears selection for the hidden node; lock is visible in the inspector and prevents preview click selection while still allowing layer inspection.
- Preview nodes render semantic `data-node-id` wrappers and selected-node halos. Geometry is truthful for rect, ellipse, text, and group bounds; path halos reuse the path as an approximation.
- The inspector is read-only and shows id, name, kind, role, visibility, lock, transform, style, opacity, and timeline membership.
- The AI rail can stage a preview-only operation with target id/name, intent, inferred operation type, affected properties, and recent history.

## Screenshot Evidence

### Before Screenshots From Phase 1 Final State

- `screenshots/before-phase-1/after-01-main-workspace-light.png`
- `screenshots/before-phase-1/after-04-ai-editor-shell-light.png`
- `screenshots/before-phase-1/after-09-ai-editor-shell-dark.png`
- `screenshots/before-phase-1/after-11-narrow-editor-dark.png`
- `screenshots/before-phase-1/after-12-generated-preview-smoke.png`

### Browser / Localhost After Screenshots

- `screenshots/browser/browser-01-desktop-light-preview-selection.png`
- `screenshots/browser/browser-02-desktop-light-operation-preview.png`
- `screenshots/browser/browser-03-settings-light.png`
- `screenshots/browser/browser-04-desktop-dark-selection.png`
- `screenshots/browser/browser-05-providers-dark.png`
- `screenshots/browser/browser-06-empty-scene-dark.png`
- `screenshots/browser/browser-07-narrow-light-selection.png`

### Native Tauri After Screenshots

- `screenshots/tauri/tauri-01-native-preview-selection.png`
- `screenshots/tauri/tauri-02-native-operation-preview.png`

## Verification Results

- `npm --workspace @strut/studio run check`: PASS.
- `npm run check`: PASS.
- `cargo test --workspace`: PASS. The authenticated Gemini CLI end-to-end test remains intentionally ignored because it requires authenticated local credentials.
- `python tests/ui/studio_bot_smoke.py`: PASS.
- `git diff --check`: PASS.

## Browser QA Result

- Desktop light: PASS. Visible preview part selection updates AI context, selected halo, layer row, and inspector.
- Desktop light operation preview: PASS. Staged operation shows target id/name, intent, operation type, affected properties, and disabled Apply/Reject.
- Desktop dark: PASS. Selection, layer state, inspector, and operation preview remain readable.
- Narrow layout: PASS. Selection flow remains readable and no horizontal overflow was detected.
- Empty scene: PASS. No-scene and no-selection states remain clear.
- Provider/settings pages: PASS. Existing Phase 1 theme consistency remains intact.

## Computer / Tauri QA Result

- Native app launch: PASS with `npm --workspace @strut/studio run tauri dev`.
- Native inspection method: actual Tauri WebView2 shell with remote debugging enabled for deterministic seeding and inspection.
- Native Phase 2 selection UI: PASS. Preview selection, layer selection, inspector update, and operation preview were verified in the native shell.
- Native screenshots: captured under `screenshots/tauri/`.
- Native provider generation: not exercised because provider credentials/auth were not available; seeded semantic document state was used instead.

## Still Placeholder Or Deferred

- Apply/Reject are intentionally disabled and do not mutate scene documents.
- Operation previews are inspectable local records, not real AI patch execution.
- Dynamic generation planning, validated operation patches, and operation replay belong to Phase 3.
- Multi-selection was not implemented; Phase 2 exposes clear empty and single-selection states only.
- Preview path bounds use a practical halo approximation where exact path geometry is not available.

## Phase 3 Recommendation

Phase 3 should introduce a validated operation schema, patch planning, safe operation application/rejection, undoable operation history, and tests that prove document mutation is explicit, reversible, and schema-validated. CLI mode should remain out of scope until the app and shared operation model are stable.
