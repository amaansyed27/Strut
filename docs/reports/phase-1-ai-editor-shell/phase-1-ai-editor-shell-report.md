# Phase 1 AI Editor Shell Report

Phase 1 replaced the scattered studio surface with a coherent AI edit-mode shell while keeping generation behavior intact. The new first viewport now has a top project/status bar, left AI edit rail, right preview workspace, stable empty/generated preview states, selection-aware placeholder affordances, and a consistent design system across light and dark modes.

## Scope Completed
- AI edit rail, live preview workspace, top status bar, stable empty/generated preview states.
- Selection-aware placeholder UI: selected target label, preview outline, ask AI to edit selection, apply/reject placeholder.
- Design tokens: off-white pastel light mode with muted mint accents; blackish/dark-grey dark mode with dark navy accents.
- Focused smoke coverage for shell/theme/selection behavior.

## Changed Files
### Studio shell and theme
- `apps/studio/src/App.tsx`
- `apps/studio/src/App.css`

### Focused smoke coverage
- `tests/ui/studio_bot_smoke.py`

### Phase report evidence
- `docs/reports/phase-1-ai-editor-shell/baseline.md`
- `docs/reports/phase-1-ai-editor-shell/screenshots/before/*`
- `docs/reports/phase-1-ai-editor-shell/screenshots/after/*`
- `docs/reports/phase-1-ai-editor-shell/phase-1-ai-editor-shell-report.docx`

## Commits
- `a0ae012` docs(report): capture phase 1 before screenshots: Recorded the initial dirty tree and Browser baseline screenshots.
- `6f909ec` style(studio): add strut theme tokens: Introduced the off-white/mint light theme, blackish/navy dark theme, focus states, and shell tokens.
- `96f45e0` feat(studio): introduce ai editor shell: Added the AI edit rail, preview workspace, status chips, inspector, and local selection placeholder state.
- `4ac2b31` test(studio): cover ai editor shell smoke: Expanded the studio smoke around the AI shell, selected target, operation placeholder, and generated preview selection.
- `bb024cd` feat(studio): add selection-aware edit placeholders: Tightened responsive shell behavior, selection outline styling, and themed scroll/root chrome.

## Verification
- `npm run check`: PASS. TypeScript checks for studio/runtime packages and cargo check completed.
- `cargo test --workspace`: PASS. All Rust workspace tests passed: 1 + 4 + 5 + 1 + 14 passed; one authenticated Gemini CLI test remained intentionally ignored.
- `python tests/ui/studio_bot_smoke.py`: PASS. Studio browser smoke passed, including Phase 1 shell assertions and seeded generated preview selection.
- `Browser visual QA`: PASS. Captured desktop light/dark, narrow light/dark, empty preview, providers/settings, and editor shell screenshots. Generated preview proof used the seeded smoke because browser-only generation stops at the expected desktop-runtime fallback.
- `DOCX render export`: BLOCKED. LibreOffice/`soffice` was not available, and Microsoft Word COM PDF export hung twice before producing a PDF. The DOCX package passed structural inspection and contains 13 embedded media files; Browser screenshots are the visual proof artifacts for this phase.

## Screenshot Evidence
### Before
- `screenshots/before/before-01-main-workspace-light.png`
- `screenshots/before/before-02-chat-generation-state-light.png`
- `screenshots/before/before-03-empty-preview-light.png`
- `screenshots/before/before-04-editor-surface-light.png`
- `screenshots/before/before-05-provider-page-light.png`
- `screenshots/before/before-06-settings-page-light.png`
- `screenshots/before/before-07-settings-page-dark.png`
- `screenshots/before/before-08-empty-preview-dark.png`
- `screenshots/before/before-09-narrow-chat-light.png`
- `screenshots/before/before-10-narrow-editor-dark.png`
### After
- `screenshots/after/after-01-main-workspace-light.png`
- `screenshots/after/after-02-chat-generation-state-light.png`
- `screenshots/after/after-03-empty-preview-light.png`
- `screenshots/after/after-04-ai-editor-shell-light.png`
- `screenshots/after/after-05-provider-page-light.png`
- `screenshots/after/after-06-settings-page-light.png`
- `screenshots/after/after-07-settings-page-dark.png`
- `screenshots/after/after-08-empty-preview-dark.png`
- `screenshots/after/after-09-ai-editor-shell-dark.png`
- `screenshots/after/after-10-narrow-chat-light.png`
- `screenshots/after/after-11-narrow-editor-dark.png`
- `screenshots/after/after-12-generated-preview-smoke.png`

## Visual QA Notes
- Desktop light/dark: app chrome remains subdued and independent of generated scene colors.
- Narrow light/dark: shell stacks into top navigation, toolbar, AI rail, preview, and inspector without overlapping primary controls.
- Empty preview: no broken blank panel; dashed selection placeholder and empty guidance remain visible.
- Generated preview: seeded smoke proof shows selected ContextBody label, outline, layer active state, and apply/reject placeholders.

## Remaining Risks
- Selection is visual/contextual only. It does not yet produce validated document operations or operation history; that belongs in Phase 2.
- Browser-only generation still cannot create a generated document because real generation requires the Tauri desktop runtime/provider path.
- Some unrelated dirty docs/backend files from the initial worktree remain intentionally preserved and uncommitted.
- The existing App.tsx remains large; deeper Phase 2 work should consider extracting shell, rail, preview, and inspector components before adding real operation state.

## Recommended Phase 2
- Phase 2 should add a real scene selection model shared by preview, layers, inspector, and AI context.
- Add inspectable operation preview data before enabling Apply/Reject.
- Keep dynamic generation planning and CLI work out of Phase 2 unless the phase boundary is explicitly reopened.
