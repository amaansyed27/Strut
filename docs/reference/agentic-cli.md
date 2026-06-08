# Agentic CLI

Phase 5 adds a Rust-backed `strut` command for coding agents that need to inspect, plan, patch, verify, render, and export Strut animations without hand-driving the Studio UI.

The CLI keeps the same boundary as the desktop app:

```text
instruction -> generation plan + operations -> Rust validation -> .strut package -> export or integration
```

Sprite-python remains an authoring backend. It emits generation-plan envelopes and operation lists; it does not write unchecked `.strut` documents or mutate projects directly.

## Commands

Inspect a project folder:

```powershell
cargo run -p strut-cli -- inspect project D:\path\to\project --json
```

The project inspector reports canonical Phase 4 files:

- `strut.project.json`
- `scenes/main.strut`
- `operations/operation-batches.json`
- `ui/studio-state.json`

Inspect a scene:

```powershell
cargo run -p strut-cli -- inspect scene scenes\main.strut --json
```

Plan with the local deterministic Rust path:

```powershell
cargo run -p strut-cli -- plan "make a calm dice animation" --json --dry-run --explain > plan.json
```

Plan with sprite-python:

```powershell
cargo run -p strut-cli -- sprite plan "make a loader microinteraction" --json --dry-run --explain > plan.json
```

The sprite command invokes `packages/strut-python` when available and falls back to checked deterministic fixtures. Supported deterministic subjects are dice, logo, loader, mascot, and UI microinteraction.

Patch a scene from a saved plan:

```powershell
cargo run -p strut-cli -- patch --scene scenes\main.strut --from plan.json --dry-run --json
cargo run -p strut-cli -- patch --scene scenes\main.strut --from plan.json --json
```

`--dry-run` validates the plan and current scene but never writes. A real patch writes only after Rust validates the replacement document and operation batch payload.

Verify a scene and optional operation batches:

```powershell
cargo run -p strut-cli -- verify scenes\main.strut --json
cargo run -p strut-cli -- verify scenes\main.strut --batch operations\operation-batches.json --json
```

Render a deterministic proof:

```powershell
cargo run -p strut-cli -- render --scene scenes\main.strut --state idle --out proof.svg --json --no-open
```

The current renderer crate exposes render planning but not full raster output. The CLI therefore writes a deterministic SVG proof that reflects the validated document structure and clearly reports the limitation in JSON.

Export React integration code:

```powershell
cargo run -p strut-cli -- export react --scene scenes\main.strut --out src\strut-animation --json
```

The exporter writes:

- `scene.json`
- `StrutAnimation.tsx`
- `README.md`

Existing files are not overwritten unless `--force` is passed. `--dry-run` reports the planned files without writing.

## JSON Contract

All `--json` output is deterministic and parseable. Plan output uses:

```json
{
  "format": "strut.cli.plan.v1",
  "instruction": "...",
  "backend": "sprite-python",
  "planSummary": {},
  "envelope": {},
  "document": {},
  "batch": {},
  "warnings": []
}
```

The `document` is a Rust-validated Strut document. The `batch` is a pending operation batch containing a validated `replace_document` operation for the patch workflow.

## Handoff Status

Core inspect, plan, patch, verify, render, and export workflows do not require the Studio UI. App handoff remains a documented protocol for a future shell command: open a scene in Studio, apply a pending validated patch, preview render, and return verification status.

