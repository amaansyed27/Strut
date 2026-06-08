# Strut Sprite-Python Replan From Phase 3

Date: 2026-06-08

Branch context: `codex/phase-1-ai-editor-shell`

## Why This Replan Exists

The original revamp plan correctly moved Strut toward a structured AI animation editor, but it did not explicitly make the Python sprite/vector engine part of the architecture. Phase 3 implemented useful groundwork: subject-aware semantic plans, validated operations, and conversion into Strut documents. That work should stay, but Phase 3 now needs a pivot pass before Phase 4 so the product matches the intended direction:

`prompt -> sprite-python authoring model -> Strut operations -> Rust validation -> .strut document -> app preview/export`

The Rust/Tauri app remains the desktop shell, native provider bridge, safety boundary, validator, and project application. The Python engine becomes the agent-friendly authoring/compiler layer for sprite-like vector animation.

## Corrected Product Positioning

Strut is an open, agent-editable animation studio and format. It should let users create Duolingo-style mascots, fluid app microinteractions, logo reveals, loaders, icons, objects, and SVG-like motion as semantic editable parts. The Codex-pet influence is style-level only: low-energy, soft idle motion, subtle bobbing, small rotations, breathable loops, and readable state changes.

Strut should not import Codex pets, a fixed atlas, or a fixed mascot body model. Mascots are one supported subject, not the whole product.

## Corrected Architecture

- **Rust/Tauri Studio**
  - Desktop app shell.
  - Native filesystem/project access.
  - Provider and local-agent bridge.
  - Validation and security boundary.
  - Preview and project workflow host.

- **React/TypeScript Studio UI**
  - AI edit rail.
  - Selectable live preview.
  - Layers, inspector, timeline controls, operation review, provider/settings UI.

- **Sprite-Python Engine**
  - Agent-friendly authoring SDK.
  - Sprite/vector object graph.
  - Motion primitives inspired by Codex-pet style.
  - Subject-aware builders for mascot, logo, loader, dice/object, icon, UI state, and abstract motion.
  - Emits Strut generation plans and operation lists.
  - Does not bypass Rust validation.

- **Strut Core And Format**
  - `.strut` remains the durable open interchange/project/runtime format.
  - Rust validates every Python or AI-emitted operation before it mutates a project.
  - Future optimized runtime artifacts can be compiled from validated `.strut`.

- **Agentic CLI**
  - Lets coding agents inspect, plan, patch, verify, render, export, and integrate Strut animations in user projects.
  - Uses the same Strut operations and validation boundary as the app.

## Updated Phase Map

Phases 1 and 2 remain complete. The previous Phase 3 is now considered **Phase 3A**: semantic generation plan and operation groundwork. The next step is **Phase 3B**, a correction pass that locks the sprite-python architecture before persistence and CLI work.

## Phase 3A: Semantic Plan And Operation Groundwork

Status: completed before this replan.

Completed scope:

- Subject-aware generation plan schema.
- Semantic part planning.
- Motion role, state, and timeline planning.
- Focused operation subset.
- Rust validation before Strut document conversion.
- Tests for dice, logo, loader, mascot, and invalid plans.
- Browser/Tauri evidence report.

Keep this work. Do not throw it away. The new Python engine should target this operation/document boundary.

## Phase 3B: Sprite-Python Architecture Lock And Engine Spike

Goal: make the Python sprite/vector engine real enough to become the authoring source for generated motion before persistence is locked.

Scope:

- Add a clear architecture decision document for sprite-python inside Strut.
- Define the Python package location, for example `packages/strut-python/` or `crates/strut-python/` only if the repo convention demands it.
- Define sprite/vector primitives:
  - `Scene`
  - `Sprite`
  - `Group`
  - `Rect`
  - `Ellipse`
  - `Path`
  - `Text`
  - `Binding`
  - `State`
  - `Timeline`
  - `Keyframe`
- Define motion primitives:
  - `idle_breathe`
  - `soft_bob`
  - `tiny_tilt`
  - `settle`
  - `reveal`
  - `pulse`
  - `progress_sweep`
  - `attention_nudge`
- Define subject builders:
  - rolling dice
  - abstract logo reveal
  - loader/progress animation
  - mascot idle animation
  - UI microinteraction
- Implement a small Python MVP that emits Strut generation plans and operation JSON, not final unchecked documents.
- Add Rust-side tests or fixtures proving Python output validates through the existing Phase 3 validator.
- Add app fixtures that display Python-generated dice/logo/loader/mascot examples.
- Keep the Rust validator authoritative.
- Do not implement persistence/undo yet.
- Do not implement the agentic CLI yet.

Acceptance criteria:

- Python can generate at least four deterministic examples: dice, logo, loader, mascot.
- Each Python example emits a plan and operations accepted by Rust validation.
- Non-mascot examples do not contain mascot-only anatomy.
- Mascot examples can use body/head/eyes when requested.
- The generated scenes still work with Phase 2 selection/layers/inspector.
- The report clearly states Python is an authoring/compiler layer, not a replacement for Rust/Tauri.

Suggested commits:

- `docs(arch): define sprite python engine boundary`
- `feat(sprite-python): add authoring model prototype`
- `feat(sprite-python): emit strut plans and operations`
- `test(sprite-python): validate generated subject fixtures`
- `chore(report): add phase 3b sprite python evidence`

Verification:

- `python -m pytest packages/strut-python/tests`
- `python packages/strut-python/examples/dice.py --json`
- `cargo test -p strut-studio`
- `cargo test --workspace`
- `npm --workspace @strut/studio run check`
- `python tests/ui/studio_bot_smoke.py`
- Browser QA for Python-generated dice/logo/loader/mascot.
- Computer/Tauri QA for at least two Python-generated examples.

Report:

- Create `docs/reports/phase-3b-sprite-python-engine/`.
- Include Markdown, DOCX, screenshots, command results, changed files, commits, and remaining risks.
- Include before screenshots from Phase 3A and after screenshots from Python-generated examples.

## Phase 4: Document-First Persistence Around Validated Operations

Goal: make validated Strut scene files and operation batches the durable source of truth, now with sprite-python output feeding the same model.

Scope:

- Define canonical project and scene file locations.
- Persist `.strut` documents produced from validated operation batches.
- Store operation batches with:
  - stable ids
  - source type: `ai`, `sprite-python`, `manual`, or `cli`
  - validation result
  - document revision id
  - optional prompt/source metadata
- Implement undo/redo around validated operation batches.
- Link chat history to operation batches instead of making chat/local state the source of truth.
- Add migration/compatibility support for existing generated local-state payloads.
- Make Apply/Reject perform real mutation only after validation.
- Keep Python-generated output passing through Rust validation.

Acceptance criteria:

- A project can be closed and reopened with the same parts, timelines, selection, operation history, and Python-generated metadata.
- Undo/redo works for applied operation batches.
- Corrupt documents fail with useful errors.
- Python output cannot mutate a project unless the Rust validator accepts it.

Suggested commits:

- `feat(schema): define operation batch persistence`
- `feat(studio): save validated strut scenes`
- `feat(studio): add apply reject for validated operations`
- `feat(studio): add undo redo operation history`
- `test(studio): cover persistence undo and invalid documents`
- `chore(report): add phase 4 persistence evidence`

Verification:

- `npm --workspace @strut/studio run check`
- `npm run check`
- `cargo test --workspace`
- `python tests/ui/studio_bot_smoke.py`
- focused persistence/undo tests
- Browser QA for close/reopen, apply/reject, undo/redo.
- Computer/Tauri QA for native project persistence.

## Phase 5: Agentic CLI Mode And Project Integration

Goal: let coding agents use Strut from another project without hand-driving the app UI.

Scope:

- Add CLI commands:
  - `strut inspect project`
  - `strut inspect scene <scene-file>`
  - `strut plan "<instruction>"`
  - `strut sprite plan "<instruction>"`
  - `strut patch --scene <scene-file> --from <plan-file>`
  - `strut verify <scene-file>`
  - `strut render --scene <scene-file> --state <state> --out <image-file>`
  - `strut export react --scene <scene-file> --out <target-dir>`
- Add machine-readable modes:
  - `--json`
  - `--dry-run`
  - `--explain`
  - `--no-open`
- Add handoff between CLI and app:
  - open scene in app
  - apply pending validated patch
  - preview render
  - return verification result
- Add integration recipes for common targets:
  - React
  - Next.js
  - plain web component or runtime-web
- Add examples showing a coding agent integrating a Strut animation into a sample app.
- Use sprite-python as one planning backend, but keep CLI output as validated Strut operations/documents.

Acceptance criteria:

- A coding agent can inspect a user project, generate a sprite-python-backed animation plan, patch a scene, verify it, render proof, export integration code, and report what changed.
- CLI commands produce deterministic JSON when requested.
- CLI failures are actionable and do not silently mutate files.
- The CLI does not need the app UI for core inspect/plan/patch/verify/render/export workflows.

Suggested commits:

- `feat(cli): add scene inspect and verify`
- `feat(cli): add sprite backed plan command`
- `feat(cli): add validated patch workflow`
- `feat(cli): add render and export commands`
- `docs(cli): document agentic strut workflow`
- `test(cli): cover agentic cli mode`
- `chore(report): add phase 5 cli evidence`

Verification:

- `cargo test --workspace`
- CLI unit/integration tests.
- Sample-project integration test.
- Render proof output comparison.
- Browser QA for exported runtime demo.
- Computer/Tauri QA for app handoff.

## Phase 6: End-To-End Hardening And Release Gate

Goal: prove the corrected architecture works across real examples and document remaining gaps.

Scope:

- End-to-end gallery:
  - rolling dice
  - abstract logo reveal
  - loader/progress animation
  - mascot idle animation
  - UI microinteraction
  - icon or badge animation
- Verify each example can be produced by sprite-python, validated by Rust, opened in Tauri, selected in the editor, patched through operations, rendered, exported, and integrated into a sample app.
- Verify app UI does not inherit generated colors.
- Verify CLI inspect/plan/patch/render/export.
- Verify generated artifacts are agent-editable and human-readable.
- Create final Documents report with complete before/after gallery.

Acceptance criteria:

- Strut can generate and edit multiple non-mascot animation types.
- Mascot quality can reach the intended low-energy companion style when requested.
- The UI reads as a coherent AI editor.
- The sprite-python engine is documented and tested as the authoring/compiler layer.
- The CLI is useful to coding agents in real user projects.
- Tests, screenshots, reports, and commits provide enough evidence to continue product work safely.

Suggested commits:

- `test(e2e): add sprite python gallery coverage`
- `test(e2e): cover cli project integration`
- `docs(report): add final sprite python revamp evidence`
- `chore(release): document remaining risks and next steps`

Verification:

- all package checks
- all Rust tests
- all Python tests
- all UI smoke tests
- CLI integration tests
- Browser visual QA
- Computer/Tauri native QA
- final screenshot/report generation

## Updated Next Worker Prompt: Phase 3B

Use this prompt for the next implementation chat before starting Phase 4.

```text
You are Codex working in D:\TheDawnlightGroup\DawnlightLabs\Strut.

Implement Phase 3B only from docs/superpowers/plans/2026-06-08-strut-sprite-python-replan.md: Sprite-Python Architecture Lock And Engine Spike.

Context:
- Phase 1 and Phase 2 are complete and reviewed.
- Phase 3A is complete and reviewed: it added subject-aware generation plans, validated operation schemas, and conversion to Strut documents.
- We are correcting the architecture before Phase 4 persistence so Strut uses a Python sprite/vector authoring engine while keeping the Rust/Tauri app and Rust validation boundary.

Hard boundaries:
- Do not start Phase 4 persistence/undo.
- Do not start agentic CLI mode.
- Do not redesign the UI.
- Do not import Codex pets, a pet atlas, or a fixed mascot/face model.
- Do not make Python bypass Rust validation.
- Do not make one giant commit.
- Do not revert unrelated dirty work.

Required workflow:
1. Run `git status --short --branch` and record dirty files.
2. Read:
   - docs/superpowers/plans/2026-06-08-strut-sprite-python-replan.md
   - docs/superpowers/plans/2026-06-05-strut-ai-editor-revamp.md
   - docs/reports/phase-3-dynamic-generation/phase-3-dynamic-generation-report.md
   - apps/studio/src-tauri/src/lib.rs
   - tests/ui/studio_bot_smoke.py
3. Add architecture docs explaining the Rust/Tauri + sprite-python split.
4. Add a minimal Python package/prototype for sprite/vector authoring.
5. Implement deterministic Python examples for dice, abstract logo, loader, and mascot.
6. Make Python emit Strut generation plans and operations, not unchecked final documents.
7. Add tests proving the Python examples validate through the existing Rust validation/document conversion path.
8. Add or update UI fixture smoke coverage only as needed to prove Python-generated examples still work with selection/layers/inspector.
9. Use Browser for visual QA and Computer/Tauri for native QA.
10. Create `docs/reports/phase-3b-sprite-python-engine/` with Markdown, DOCX, screenshots, commits, changed files, commands, results, risks, and Phase 4 recommendation.

Verification:
- python -m pytest packages/strut-python/tests
- cargo test -p strut-studio
- cargo test --workspace
- npm --workspace @strut/studio run check
- npm run check
- python tests/ui/studio_bot_smoke.py
- git diff --check

Final response must include:
- whether Phase 3B is complete
- commits made
- files changed
- Browser QA result
- Computer/Tauri QA result
- verification commands and results
- report path
- remaining risks
- explicit statement that Phase 4 and CLI mode were not started
```
