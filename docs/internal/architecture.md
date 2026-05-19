# Architecture Notes

This is a maintainer-facing architecture note. Public user docs start at [../README.md](../README.md).

Strut is built as a desktop-first product with a Rust core and a web-based editor UI. The architecture separates the editor, file format, renderer, runtime, AI orchestration, and verifier so each piece can evolve without turning the project into a single closed application.

## System Overview

```txt
                    +------------------------+
                    |      Strut Studio      |
                    |  Tauri desktop shell   |
                    +-----------+------------+
                                |
                                v
+----------------+   +----------+-----------+   +------------------+
| Provider Router|   |      Rust Core       |   | Local Agent Hub  |
| BYOK models    |<->| document + compiler  |<->| CLIs + MCP       |
+----------------+   +----------+-----------+   +------------------+
                                |
                                v
                    +-----------+------------+
                    | Renderer + Verifier    |
                    | GPU preview + checks   |
                    +-----------+------------+
                                |
                                v
                    +-----------+------------+
                    | Runtime Artifacts      |
                    | .strut + JS runtimes   |
                    +------------------------+
```

## Layers

### Studio

The Studio is a Tauri v2 desktop app. Its UI is TypeScript and React so panels, inspectors, timelines, and agent workflows are fast to iterate. Privileged filesystem, process, provider, and compiler work goes through Rust commands with explicit permissions.

### Core

`strut-core` owns the editor-independent model:

- Documents
- Artboards
- Scene graph nodes
- Styles and paints
- Timelines
- Easing
- State machines
- Inputs
- Bindings
- Events

### Format

`strut-format` owns `.strut` read/write/validate. The early format is JSON inside a ZIP container so humans can inspect and diff project files. A compact binary runtime payload can be added after the model stabilizes.

### Renderer

The renderer starts with a clear abstraction and targets local GPU acceleration through `wgpu`. The viewport should be GPU-backed on Windows, macOS, and Linux, with CPU fallback for unsupported hardware.

### Agent Engine

The agent engine coordinates BYOK model providers and local coding agents. It does not let models write opaque output directly into the document. Agents produce typed document patches that Strut validates, previews, and verifies.

### Verifier

The verifier checks generated or edited work before it is marked done:

- state reachability
- missing inputs/bindings
- timeline validity
- export validity
- render snapshots
- performance budget
- reduced-motion behavior

## Non-Goals For The First MVP

- Browser-hosted SaaS editor
- Real-time multiplayer collaboration
- Full Lottie compatibility
- Native mobile runtimes
- Mesh deformation and bones
- Audio timelines
- Marketplace

These are not rejected forever. They are outside the first vertical slice.
