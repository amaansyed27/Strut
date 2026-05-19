# MVP Scope

This is a maintainer-facing scope note. It is not part of the public user docs.

The MVP is one complete vertical slice: prompt or SVG to editable interactive animation to runtime export.

## User Story

As a product engineer or designer, I can create an animated login button with idle, hover, pressed, loading, success, and error states, preview it in the desktop app, export it as `.strut`, and control it in React.

## Included

- Desktop app shell on Windows, macOS, and Linux through Tauri.
- Open `.strut` format.
- Basic vector scene graph.
- SVG import.
- Artboards.
- Groups and transforms.
- Fill, stroke, and simple gradients.
- Timeline keyframes.
- Easing.
- State machine inputs: boolean, number, trigger, enum.
- Events.
- Runtime bindings.
- Web runtime.
- React runtime wrapper.
- Ollama provider support.
- OpenAI-compatible provider support.
- Adapter interface for local coding agents.
- Plan Mode sketch workflow.
- Manual review checkpoints.
- Verifier checks for export and state reachability.

## Excluded

- Collaborative editing.
- Marketplace.
- Full After Effects or Lottie parity.
- Complex text shaping.
- Bones, IK, meshes, and deformation.
- Native iOS/Android runtimes.
- Plugin marketplace.
- Cloud-hosted account system.

## Acceptance Criteria

- `npm run check` passes.
- Rust tests pass.
- The Studio opens through `npm run studio:dev`.
- A sample `.strut` file validates.
- A sample interaction can be previewed.
- Runtime code can set inputs and receive events.
- The manual review guide explains exactly what to run and inspect.
