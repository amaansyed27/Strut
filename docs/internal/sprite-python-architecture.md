# Sprite-Python Architecture Boundary

Date: 2026-06-08

## Decision

Strut keeps Rust/Tauri as the application shell, provider bridge, native project host, and validation boundary. The new sprite-python package is an authoring and compiler layer for agent-friendly vector animation construction.

The locked flow is:

```text
prompt -> sprite-python authoring model -> Strut generation plan + operations -> Rust validation -> Strut document -> Studio preview/export
```

Python does not emit unchecked final `.strut` documents and does not mutate projects directly. Every Python-authored scene must pass through the same Rust generation-plan and operation validator introduced in Phase 3A before it can become a Strut document.

## Responsibilities

Rust/Tauri Studio owns:

- desktop app shell and native filesystem access
- provider/local-agent bridge
- validation and safety boundary
- conversion from validated operations into Strut documents
- preview, project workflow, and future persistence

React/TypeScript Studio owns:

- AI edit rail
- live selectable preview
- layers, inspector, timeline controls, operation review, provider/settings UI

Sprite-python owns:

- ergonomic vector/sprite authoring primitives
- deterministic subject builders for common animation classes
- low-energy motion helpers such as breathe, bob, tilt, settle, reveal, pulse, sweep, and nudge
- emission of Strut generation plans plus inspectable operation lists

## Python Package Location

The package lives at `packages/strut-python/` because the repository already groups language/runtime packages under `packages/`. The importable module is `strut_python`.

## Authoring Primitives

The Phase 3B spike defines these minimal primitives:

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

These primitives compile into generation-plan parts, timelines, and operations. They are not a replacement schema for Strut Core.

## Validation Boundary

The Rust validator remains authoritative for:

- part count and stable part ids
- subject classification rules
- mascot-only anatomy rejection for non-mascot subjects
- geometry sanity
- state and timeline references
- operation references and ordering constraints
- conversion into `strut_core::Document`

Python tests can check local structure and determinism, but acceptance requires Rust tests to parse the Python examples and convert them through the existing Phase 3A path.

## Non-Goals For Phase 3B

- no Phase 4 persistence or undo/redo
- no agentic CLI mode
- no UI redesign
- no Codex pet import, pet atlas, fixed mascot, or fixed face model
- no Python-produced final document bypass

