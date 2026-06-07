# Phase 1 AI Editor Shell Report

Phase 1 is a UI shell and design-system phase only. It replaces the scattered studio surface with a coherent AI editor shell, keeps generation behavior working, and deliberately does not start Phase 2 selection/operation work.

## Summary

- Phase 1 scope: AI editor shell, design tokens, stable preview states, selection-aware placeholders, provider visibility, readable chat/preview layout, smoke coverage, and final visual evidence.
- Phase 2 not started: no real selection hit-testing, no apply/reject operation patches, no dynamic generation planning engine, and no agentic CLI mode were implemented.
- Final evidence refresh: official after screenshots were refreshed from current HEAD after the later UI commits `bfe03ca` and `a38c354`.
- Documents deliverable: DOCX report was recreated. Rendered page proof is attempted via the Documents renderer; if blocked locally, `rendered/README.md` documents the exact blocker and the screenshot-gallery fallback proof.

## Dirty Worktree Recorded Before Final Fix Pass

Recorded with `git status --short --branch` on 2026-06-07 before refreshing the report:

```text
## codex/phase-1-ai-editor-shell
 M README.md
 M apps/studio/src-tauri/src/lib.rs
 M docs/README.md
 M docs/guides/generate-a-character.md
 M docs/learn/first-animation.md
 M docs/learn/quick-start.md
 M docs/learn/what-is-strut.md
?? docs/learn/motion-language.md
?? docs/superpowers/
```

These unrelated dirty and untracked files were preserved and not reverted.

## Scope Completed

- Built the Phase 1 AI editor shell with a left AI edit/chat rail, right preview workspace, top project/status bar, provider/status chips, project files context, and scene layer placeholders.
- Added selection-aware placeholders: selected target label, visual selection outline for generated preview proof, "Ask AI to edit selection" placeholder, and disabled apply/reject operation placeholder.
- Added stable empty preview and generated preview states without making the preview look broken when generation is unavailable in browser preview.
- Added design-system tokens for off-white/mint light mode and blackish/navy dark mode. App chrome stays independent of generated scene colors.
- Updated chat rendering: user messages render as chat bubbles; assistant/system responses render as direct markdown.
- Redesigned provider visibility: selected provider is visible in the provider page, top chrome, composer, settings, and assistant responses.
- Added a final readability pass for chat + preview: removed the canvas-grid dominance, reduced prompt-chip clutter, widened the transcript, narrowed the preview rail, and aligned the surface closer to a Codex-like task app.
- Added/updated focused smoke coverage for shell, theme behavior, provider visibility, markdown response rendering, user bubble rendering, empty preview, generated preview proof, selection placeholder, and responsive layouts.

## Changed Files

### Studio Shell And Theme

- `apps/studio/src/App.tsx`
- `apps/studio/src/App.css`

### Focused Smoke Coverage

- `tests/ui/studio_bot_smoke.py`

### Phase Report And Evidence

- `docs/reports/phase-1-ai-editor-shell/baseline.md`
- `docs/reports/phase-1-ai-editor-shell/phase-1-ai-editor-shell-report.md`
- `docs/reports/phase-1-ai-editor-shell/phase-1-ai-editor-shell-report.docx`
- `docs/reports/phase-1-ai-editor-shell/rendered/README.md`
- `docs/reports/phase-1-ai-editor-shell/screenshots/before/*`
- `docs/reports/phase-1-ai-editor-shell/screenshots/after/*`

## Commit Log

- `a0ae012` docs(report): capture phase 1 before screenshots
  Recorded initial dirty tree and Browser baseline screenshots for the studio surfaces.
- `6f909ec` style(studio): add strut theme tokens
  Added off-white/mint light tokens, blackish/navy dark tokens, focus states, and shell/preview/panel variables.
- `96f45e0` feat(studio): introduce ai editor shell
  Added AI edit rail, live preview workspace, top status bar, inspector/context panels, and local placeholder selection state.
- `4ac2b31` test(studio): cover ai editor shell smoke
  Expanded the studio smoke around the AI shell, selected target, operation placeholder, provider pages, theme toggles, and generated preview state.
- `bb024cd` feat(studio): add selection-aware edit placeholders
  Tightened selection-aware preview affordances, responsive shell behavior, and placeholder states.
- `09d3a59` chore(report): add phase 1 ui revamp report
  Added the first Phase 1 report package and screenshot inventory.
- `bfe03ca` fix(studio): clarify chat and provider UI
  Added user chat bubbles, direct markdown assistant responses, visible selected-provider summaries, provider-in-response output, and clearer editor shell structure.
- `a38c354` fix(studio): make chat preview readable
  Reworked the chat + preview layout toward a Codex-like readable task surface: wider transcript, narrower preview rail, cleaner topbar, no dominant grid background, and fewer visual distractions.

## Screenshot Evidence

### Before Screenshots

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

### Final After Screenshots Refreshed From Current HEAD

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

## Before And After Comparison Notes

- Main workspace: the baseline had a landing-like/scattered workspace; the final UI has consistent app chrome, clearer project/status context, and no generated-scene colors leaking into the shell.
- Chat + preview: the stale after gallery still showed a large canvas grid, floating composer card, prompt chips, and a wide empty preview region. The final after screenshot now shows a readable transcript, user bubble, direct markdown assistant response, bottom composer, and narrower preview rail.
- Editor shell: the final editor screenshot keeps Phase 1 placeholders visible while avoiding the earlier side-card clutter. It shows selected target context, operation placeholder, preview workspace, project files, and scene layers without implementing real Phase 2 hit-testing.
- Provider page: selected provider is explicit in a summary, selected row state, top chrome, composer label, settings, and assistant response text.
- Settings: light and dark theme screenshots confirm the final token set and dark-mode navy/blackish surfaces.
- Narrow layout: the final narrow chat and editor screenshots show responsive stacking without horizontal overflow.
- Generated preview proof: the seeded smoke screenshot shows a generated document, selected `ContextBody`, selection outline, active layer row, and state buttons. This remains visual/contextual only; real selectable preview hit-testing is Phase 2.

## Verification Commands

Final verification for this closure pass:

- `npm --workspace @strut/studio run check`: PASS.
- `npm run check`: PASS.
- `cargo test --workspace`: PASS. The authenticated Gemini CLI end-to-end test remains intentionally ignored because it requires an authenticated local CLI.
- `python tests/ui/studio_bot_smoke.py`: PASS.
- `git diff --check`: PASS.
- Browser visual QA: PASS. Captured fresh desktop light, desktop dark, narrow light, narrow dark, empty preview, generated preview, provider page, settings page, and editor shell screenshots from current HEAD.

## Documents Deliverable And Rendered Proof

- DOCX report: `phase-1-ai-editor-shell-report.docx`.
- Markdown source: `phase-1-ai-editor-shell-report.md`.
- Rendered proof directory: `rendered/`.
- If `rendered/page-*.png` or a PDF exists, it is the direct DOCX render proof.
- If local rendering remains blocked, `rendered/README.md` records the renderer attempted, exact blocker, and the screenshot-gallery fallback proof.
- Screenshot-gallery fallback proof is the full `screenshots/before/` and refreshed `screenshots/after/` inventory listed above. These are Browser/Playwright screenshots from the running studio and constitute the visual proof when DOCX page rendering is unavailable.

## Visual QA Findings

- Typography: control labels, chat roles, markdown responses, provider labels, and sidebar text now use a consistent readable scale.
- Theme consistency: app chrome stays stable across light/dark modes and does not inherit generated scene colors.
- Preview readability: empty previews have clear guidance; generated preview proof shows a readable scene and selected target placeholder.
- Chat readability: user messages are bubbles and assistant/system messages render as direct markdown.
- Provider state: selected provider is visible across provider page, top chrome, composer, settings, activity, and assistant response.
- Responsive behavior: narrow chat and editor layouts are usable with no horizontal overflow in the captured viewport.
- Overlap/occlusion: final Browser screenshots show no primary text/control overlap in the refreshed Phase 1 surfaces.

## Remaining Risks

- Selection remains a visual/context placeholder. Real preview hit-testing, shared selection state, and selected-part inspector behavior belong to Phase 2.
- Apply/reject operation controls are intentionally disabled placeholders. Real operation patches and operation history belong to later phases.
- Browser-only generation still stops at the expected desktop-runtime/provider fallback. Real generation requires the Tauri desktop runtime/provider path.
- The generated-preview proof is seeded through the smoke test rather than produced by browser-only generation.
- The studio `App.tsx` remains large; Phase 2 should consider component extraction before adding real operation state.
- Unrelated dirty docs/Tauri files were preserved and remain outside this Phase 1 report-refresh commit.

## Recommended Phase 2

- Add a real scene selection model shared by preview, layers, inspector, and AI context.
- Connect visible preview objects to selectable semantic IDs.
- Add inspectable operation preview data before enabling Apply/Reject.
- Keep dynamic generation planning and agentic CLI mode out of Phase 2 unless the phase boundary is explicitly reopened.

## Phase Boundary

Phase 2 has not started in this pass. This closure pass only refreshed Phase 1 evidence, report files, screenshots, and document deliverables.
