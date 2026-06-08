# Strut AI Editor Revamp Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild Strut from a mostly static mascot/generation demo into a structured AI animation editor: ChatGPT-like edit mode on the left, live selectable preview on the right, dynamic semantic animation parts, consistent app theming independent of generated colors, and a CLI mode that coding agents can use inside user projects.

**Architecture:** Strut should become document-first and operation-first. The UI edits a validated Strut scene document through explicit operations. AI generation should produce semantic parts, motion roles, and patch operations instead of a fixed mascot template or a whole unstructured JSON blob. The app and CLI should share the same document schema, validation, operation log, preview renderer, and export pipeline.

**Tech Stack:** React + TypeScript studio, Tauri shell, Rust backend commands, Strut JSON document schema, Vitest/unit tests where available, Playwright/Browser visual checks, Python smoke tests, Documents plugin for phase reports, Product Design/Build Web Apps/Superpowers plugins for design and implementation support.

---

## Current Findings

- [ ] The product surface currently promises a broad animation studio, but the implementation still behaves like a mascot/chat generator.
- [ ] The generation path is too static: it uses a narrow set of states and fallback timelines instead of planning editable semantic parts for the requested subject.
- [ ] Current generated objects are not consistently treated as project source files. Chat/local state and generated payloads have too much authority compared with durable scene documents.
- [ ] The existing studio UI has useful raw pieces, but no coherent AI edit mode. Chat, preview, toolbar, project files, and inspector do not yet form one clear editing workflow.
- [ ] The editor toolbar includes selection/shape/path/bind/animate controls, but the controls are not backed by a real editable operation model.
- [ ] The current preview can be visually empty or disconnected from what the AI just generated.
- [ ] The current theme has warm paper/brown dark-mode tokens. The desired direction is off-white pastel light mode with mint accents, and blackish/dark-grey dark mode with dark navy accents.
- [ ] Fonts and spacing need to be standardized so generated animation colors do not leak into the app chrome.
- [ ] There is no agentic CLI workflow for coding agents to inspect, patch, verify, render, and export Strut animations inside another project.

## Non-Goals

- [ ] Do not add Codex pets as a product concept.
- [ ] Do not import a fixed pet atlas, fixed mascot body model, or fixed face requirement.
- [ ] Do not make another static list of pre-made animations.
- [ ] Do not let generated scene colors control the application shell theme.
- [ ] Do not ship a giant single commit for any phase.
- [ ] Do not claim a phase is complete without screenshots, tests, changed-file list, commits, and a Documents report.

## Target Product Shape

- [ ] The primary screen is an AI animation editor, not a landing page.
- [ ] Left side: AI chat/edit panel with prompt history, selected-area context, suggested operations, and apply/reject controls.
- [ ] Right side: live preview canvas/surface with direct selection, bounding boxes, labels, scrubber, state selector, and viewport controls.
- [ ] Supporting panels: layers/parts, properties, timelines, project files, provider/settings, export/CLI handoff.
- [ ] The app shell keeps one consistent design system across light and dark modes.
- [ ] Generated scenes can be colorful, but the surrounding app UI remains stable, subdued, and readable.
- [ ] The motion language should borrow the Codex pet feel only at the level of style: low-energy, soft idle motion, subtle bobbing, small rotations, breathable loops, and non-distracting micro-interactions.

## Dynamic Animation Model

- [ ] Every generated subject gets a semantic part plan, not a mascot-only anatomy.
- [ ] Example rolling dice parts: `DieBody`, `FrontFace`, `TopFace`, `Pips`, `EdgeHighlight`, `SettleShadow`.
- [ ] Example abstract logo parts: `PrimaryMark`, `Wordmark`, `AccentStroke`, `RevealMask`, `AnchorGrid`.
- [ ] Example loader parts: `Track`, `ActiveSegment`, `PulseDot`, `ProgressSweep`, `Glow`.
- [ ] Example mascot parts only when relevant: `Body`, `Head`, `Eyes`, `Arms`, `Accessory`, `GroundShadow`.
- [ ] Each part gets an editable identity: stable id, display name, role, geometry kind, visual tokens, transform, constraints, and allowed motion properties.
- [ ] Each motion is generated as a timeline with named purpose: idle, enter, emphasis, success, error, loading, hover, transition, or custom.
- [ ] AI edits are expressed as patches/operations that the user or coding agent can inspect before applying.

## Shared Operation Vocabulary

- [ ] `create_node`
- [ ] `delete_node`
- [ ] `rename_layer`
- [ ] `group_nodes`
- [ ] `ungroup_nodes`
- [ ] `set_property`
- [ ] `bind_property`
- [ ] `add_state`
- [ ] `add_timeline`
- [ ] `add_keyframe`
- [ ] `replace_motion_track`
- [ ] `retime_timeline`
- [ ] `add_event`
- [ ] `export_asset`
- [ ] `verify_scene`

## Commit Discipline

- [ ] Start each phase with `git status --short` and record dirty files in the phase report.
- [ ] Preserve unrelated user changes. Never revert dirty files unless the user explicitly asks.
- [ ] Commit after each coherent slice. A normal phase should have 3 to 8 commits.
- [ ] Do not commit generated caches, dev-server output, screenshots outside the agreed reports folder, or temporary files.
- [ ] Use commit prefixes consistently:
  - [ ] `docs(plan): ...`
  - [ ] `style(studio): ...`
  - [ ] `feat(studio): ...`
  - [ ] `feat(cli): ...`
  - [ ] `feat(schema): ...`
  - [ ] `test(studio): ...`
  - [ ] `test(cli): ...`
  - [ ] `chore(report): ...`
- [ ] Every phase report must list commit hashes and explain what changed in each commit.

## Phase Report Requirements

At the end of every phase, create a full report using the Documents plugin.

- [ ] Save the report in `docs/reports/phase-N-<short-name>/`.
- [ ] Include a DOCX report and a rendered visual proof, such as PDF or page PNGs, depending on the Documents plugin workflow.
- [ ] Include before and after screenshots for every affected page and major feature.
- [ ] Include screenshots for both light and dark mode.
- [ ] Include desktop and mobile/narrow layouts when the phase touches responsive UI.
- [ ] Include Browser screenshots from the running studio, not only static mockups.
- [ ] Include each changed file grouped by purpose.
- [ ] Include all commits created during the phase.
- [ ] Include tests/commands run, pass/fail status, and exact remaining gaps.
- [ ] Include visual QA notes: typography, theme consistency, preview readability, overlap issues, and empty-state behavior.
- [ ] Include unresolved risks and the recommended next phase.

Recommended report structure:

- [ ] Summary
- [ ] Scope Completed
- [ ] Before Screenshots
- [ ] After Screenshots
- [ ] Page and Feature Gallery
- [ ] Changed Files
- [ ] Commit Log
- [ ] Verification Commands
- [ ] Visual QA Findings
- [ ] Remaining Risks
- [ ] Next Phase Recommendation

## Phase 0: Baseline Capture And Plan Lock

**Goal:** Freeze the current reality before the revamp begins.

- [ ] Run `git status --short`.
- [ ] Start the studio locally.
- [ ] Capture before screenshots for all current primary pages/states:
  - [ ] Home/current main workspace.
  - [ ] Chat/generate flow.
  - [ ] Preview/editor surface.
  - [ ] Project files panel.
  - [ ] Settings/provider page.
  - [ ] Light mode.
  - [ ] Dark mode.
  - [ ] Narrow/mobile layout if supported.
- [ ] Record current theme tokens, font stack, shell layout dimensions, and current preview behavior.
- [ ] Add or update baseline notes in the phase report folder.
- [ ] Commit only documentation/report artifacts if needed.

**Acceptance Criteria:**

- [ ] The repo has a clear baseline screenshot set.
- [ ] The team can compare every later UI change against this baseline.
- [ ] Dirty files are recorded before implementation starts.

**Suggested Commits:**

- [ ] `docs(report): capture strut revamp baseline`

## Phase 1: AI Editor Shell And Design System

**Goal:** Replace the current scattered UI with a coherent AI-edit-mode shell without implementing the deep dynamic generation engine yet.

- [ ] Start from a dirty-worktree review and do not clobber existing uncommitted work.
- [ ] Capture before screenshots if Phase 0 has not already done so.
- [ ] Define stable theme tokens:
  - [ ] Light mode: off-white pastel surface, not bright white.
  - [ ] Light accent: muted mint/pastel green.
  - [ ] Dark mode: blackish/dark grey surfaces.
  - [ ] Dark accent: dark navy blue, with accessible text contrast.
  - [ ] Shared semantic tokens for shell, panel, preview, border, text, muted text, selection, focus, success, warning, and danger.
- [ ] Standardize app fonts and text scale.
- [ ] Rebuild the main studio layout:
  - [ ] Left AI edit/chat rail.
  - [ ] Right live preview workspace.
  - [ ] Top project/status bar.
  - [ ] Optional lower timeline strip or compact timeline placeholder.
  - [ ] Secondary layer/inspector panel if it fits without clutter.
- [ ] Add stable empty states for no project, no generation, and no selection.
- [ ] Add visual affordances for future selected-area editing:
  - [ ] Selection outline placeholder.
  - [ ] Selected target label.
  - [ ] "Ask AI to edit selection" context area.
  - [ ] Apply/reject operation placeholder.
- [ ] Keep generation behavior working, but do not redesign the backend engine in Phase 1.
- [ ] Add or update focused tests/smokes for the shell.
- [ ] Verify with Browser screenshots across light/dark and desktop/narrow widths.
- [ ] Create the Phase 1 Documents report with before/after screenshots.

**Acceptance Criteria:**

- [ ] The first viewport clearly looks like an AI animation editor: chat/edit controls on the left and preview on the right.
- [ ] The app theme is consistent and independent of generated scene colors.
- [ ] Light mode uses off-white pastel surfaces with mint accents.
- [ ] Dark mode uses blackish/dark-grey surfaces with dark navy accents.
- [ ] Fonts, spacing, borders, and interaction states are consistent across visible pages.
- [ ] The preview area never appears like a broken blank panel; it has a useful empty or loaded state.
- [ ] No deeper generation schema or CLI engine work is mixed into this phase.

**Suggested Commits:**

- [ ] `docs(report): capture phase 1 before screenshots`
- [ ] `style(studio): add strut theme tokens`
- [ ] `feat(studio): introduce ai editor shell`
- [ ] `feat(studio): add selection-aware edit placeholders`
- [ ] `test(studio): cover ai editor shell smoke`
- [ ] `chore(report): add phase 1 ui revamp report`

**Verification Commands:**

- [ ] `npm run check`
- [ ] `cargo test --workspace`
- [ ] `python tests/ui/studio_bot_smoke.py`
- [ ] Browser visual QA for:
  - [ ] Desktop light mode.
  - [ ] Desktop dark mode.
  - [ ] Narrow light mode.
  - [ ] Narrow dark mode.
  - [ ] Empty preview.
  - [ ] Generated preview, if available.
  - [ ] Settings/provider page after theme changes.

## Phase 2: Selection, Layers, And Editable Scene Structure

**Goal:** Make the preview and side panels understand editable semantic parts.

- [ ] Introduce a scene selection model shared by preview, layers, inspector, and chat context.
- [ ] Make preview objects selectable by semantic id.
- [ ] Add layer list with part names, visibility, lock, and selection state.
- [ ] Add inspector fields for selected part identity, transform, color/style tokens, motion role, and timeline membership.
- [ ] Add operation preview UI for AI changes before applying them.
- [ ] Persist selection and operation history in the scene/project model.
- [ ] Add tests for selection state and operation preview behavior.
- [ ] Produce Phase 2 Documents report with before/after screenshots.

**Acceptance Criteria:**

- [ ] Selecting a visible part updates the layer list, inspector, and chat context.
- [ ] The user can ask AI to edit the selected area without describing the whole scene again.
- [ ] Empty, single-selection, and multi-selection states are visually clear.

**Suggested Commits:**

- [ ] `feat(studio): add scene selection state`
- [ ] `feat(studio): connect preview selection to layers`
- [ ] `feat(studio): add selected-part inspector`
- [ ] `test(studio): cover selection workflow`
- [ ] `chore(report): add phase 2 selection report`

## Phase 3: Dynamic Generation Planning And Patch Operations

**Goal:** Replace static mascot-style generation with subject-aware semantic planning and editable operations.

- [ ] Add a generation planning schema:
  - [ ] Subject classification.
  - [ ] Semantic part plan.
  - [ ] Motion role plan.
  - [ ] State/timeline plan.
  - [ ] Editability constraints.
- [ ] Update AI prompts so names and parts are dynamic to the subject.
- [ ] Add validation for generated plans before converting to scene documents.
- [ ] Convert AI outputs into operation lists instead of directly trusting whole documents.
- [ ] Add low-energy motion guidance as a style option, not a forced pet/face model.
- [ ] Add tests for dice, logo, loader, abstract mark, and mascot cases.
- [ ] Produce Phase 3 Documents report with generated examples and before/after comparisons.

**Acceptance Criteria:**

- [ ] Rolling dice does not produce mascot anatomy.
- [ ] Abstract logo does not require a face.
- [ ] Mascot prompts can still produce body/head/eyes only when appropriate.
- [ ] Generated scenes have editable semantic parts and named timelines.
- [ ] Invalid AI plans are rejected or repaired before becoming scene documents.

**Suggested Commits:**

- [ ] `feat(schema): add generation plan model`
- [ ] `feat(studio): generate subject-aware semantic parts`
- [ ] `feat(studio): convert generation plans into scene operations`
- [ ] `test(schema): cover dynamic subject plans`
- [ ] `chore(report): add phase 3 generation report`

## Phase 4: Document-First Persistence And Undo

**Goal:** Make Strut scene files the durable source of truth.

- [ ] Define canonical scene/project file locations.
- [ ] Add load/save behavior around validated Strut documents.
- [ ] Store operation history with undo/redo support.
- [ ] Keep chat history linked to operations, not as the only source of state.
- [ ] Add migration or compatibility support for existing generated payloads.
- [ ] Add tests for save/load/undo/redo and invalid documents.
- [ ] Produce Phase 4 Documents report.

**Acceptance Criteria:**

- [ ] A project can be closed and reopened with the same parts, timelines, selection, and operation history.
- [ ] Undo/redo works for AI-applied edits.
- [ ] Corrupt or invalid documents fail with useful errors.

**Suggested Commits:**

- [ ] `feat(schema): define canonical strut scene document`
- [ ] `feat(studio): add validated scene persistence`
- [ ] `feat(studio): add operation history undo redo`
- [ ] `test(studio): cover persistence and undo`
- [ ] `chore(report): add phase 4 persistence report`

## Phase 5: Agentic CLI Mode

**Goal:** Let coding agents use Strut from another project without hand-driving the app UI.

- [ ] Add CLI commands:
  - [ ] `strut inspect project`
  - [ ] `strut inspect scene <scene-file>`
  - [ ] `strut plan "<instruction>"`
  - [ ] `strut patch --scene <scene-file> --from <plan-file>`
  - [ ] `strut verify <scene-file>`
  - [ ] `strut render --scene <scene-file> --state <state> --out <image-file>`
  - [ ] `strut export react --scene <scene-file> --out <target-dir>`
- [ ] Add machine-readable output modes for coding agents:
  - [ ] `--json`
  - [ ] `--dry-run`
  - [ ] `--explain`
  - [ ] `--no-open`
- [ ] Add a handoff protocol between CLI and app:
  - [ ] Open scene in app.
  - [ ] Apply pending patch.
  - [ ] Preview render.
  - [ ] Return verification result.
- [ ] Add CLI docs and examples for Codex-style agents.
- [ ] Add tests for CLI patch/verify/render/export.
- [ ] Produce Phase 5 Documents report with CLI screenshots/output captures.

**Acceptance Criteria:**

- [ ] A coding agent can inspect a user's project, plan an animation, patch a scene, verify it, render proof, and export code without manually using the UI.
- [ ] CLI commands produce deterministic JSON when requested.
- [ ] CLI failures are actionable and do not silently mutate files.

**Suggested Commits:**

- [ ] `feat(cli): add scene inspect and verify commands`
- [ ] `feat(cli): add plan and patch workflow`
- [ ] `feat(cli): add render and export commands`
- [ ] `docs(cli): document agentic strut workflow`
- [ ] `test(cli): cover agentic cli mode`
- [ ] `chore(report): add phase 5 cli report`

## Phase 6: End-To-End Hardening And Release Gate

**Goal:** Prove the new architecture works across real examples and document what remains.

- [ ] Run an end-to-end gallery:
  - [ ] Rolling dice.
  - [ ] Abstract logo reveal.
  - [ ] Loader/progress animation.
  - [ ] Mascot idle animation.
  - [ ] UI microinteraction.
- [ ] Verify dynamic naming and editable parts in every example.
- [ ] Verify app UI does not inherit generated colors.
- [ ] Verify selection and AI patch workflow.
- [ ] Verify CLI inspect/plan/patch/render/export.
- [ ] Verify final visual quality with Browser and, if needed, Computer Use for the native app.
- [ ] Produce final Documents report with the complete before/after gallery.

**Acceptance Criteria:**

- [ ] Strut can generate and edit multiple non-mascot animation types.
- [ ] The UI reads as a coherent AI editor.
- [ ] The CLI is useful to coding agents in real user projects.
- [ ] Tests, screenshots, reports, and commits provide enough evidence to continue product work safely.

**Suggested Commits:**

- [ ] `test(e2e): add dynamic animation gallery coverage`
- [ ] `docs(report): add final revamp evidence report`
- [ ] `chore(release): document remaining risks and next steps`

## Phase 1 Worker Prompt

Use the prompt below to start Phase 1 in a fresh Codex worker. Keep the worker constrained to Phase 1 only.

```text
You are Codex working in D:\TheDawnlightGroup\DawnlightLabs\Strut.

Implement Phase 1 only from docs/superpowers/plans/2026-06-05-strut-ai-editor-revamp.md: AI Editor Shell And Design System.

You have full local repo access. You may spawn focused subagents for design audit, code audit, visual QA, and test/report support, but you remain responsible for reviewing their outputs and integrating only scoped changes. You may use these plugins/tools as needed: Computer, Browser, build-web-apps, superpowers, documents, and product-design.

Hard boundaries:
- Do not implement the dynamic generation planning engine in Phase 1.
- Do not implement the agentic CLI mode in Phase 1.
- Do not import Codex pets, a pet atlas, or a fixed mascot/face model.
- Do not make one giant commit.
- Do not revert or overwrite existing dirty work unless the user explicitly approves it.
- Keep generation behavior working, but limit backend changes to what is strictly needed for UI compatibility.

Required workflow:
1. Read the plan doc and use superpowers:subagent-driven-development or superpowers:executing-plans.
2. Run git status --short and record the dirty files before editing.
3. Start the current studio locally and capture BEFORE screenshots with Browser for the main workspace, current chat/generation state, preview/editor surface, project/settings/provider pages, light mode, dark mode, and narrow layout where possible.
4. Implement a coherent AI edit-mode shell:
   - Left AI chat/edit rail.
   - Right live preview workspace.
   - Top project/status bar.
   - Stable empty/generated preview states.
   - Selection-aware placeholders: selected target label, selection outline, "ask AI to edit selection", apply/reject operation placeholder.
5. Implement a consistent design system:
   - Light mode: off-white pastel surfaces, not bright white, with muted mint/pastel green accents.
   - Dark mode: blackish/dark-grey surfaces with dark navy accents.
   - Consistent font stack, spacing, borders, focus states, shell/panel/preview tokens.
   - App chrome must stay consistent independent of generated scene colors.
6. Add or update focused tests/smokes for the shell and theme behavior.
7. Verify with:
   - npm run check
   - cargo test --workspace
   - python tests/ui/studio_bot_smoke.py
   - Browser visual QA for desktop light/dark, narrow light/dark, empty preview, generated preview if available, and settings/provider page.
8. Make small commits after coherent slices. Suggested commits:
   - docs(report): capture phase 1 before screenshots
   - style(studio): add strut theme tokens
   - feat(studio): introduce ai editor shell
   - feat(studio): add selection-aware edit placeholders
   - test(studio): cover ai editor shell smoke
   - chore(report): add phase 1 ui revamp report
9. At the end, use the Documents plugin to create a full Phase 1 report in docs/reports/phase-1-ai-editor-shell/ with:
   - DOCX report and rendered visual proof.
   - Before and after screenshots for each affected page and feature.
   - Light/dark screenshots.
   - Desktop/narrow screenshots.
   - Changed files grouped by purpose.
   - Commit hashes and explanations.
   - Commands/tests run and pass/fail results.
   - Visual QA notes.
   - Remaining risks and recommended Phase 2 next steps.

Final response must include:
- Phase 1 summary.
- Commits made.
- Files changed.
- Verification commands and results.
- Link to the Documents report.
- Any remaining risks.
```
