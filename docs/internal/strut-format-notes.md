# Strut Format Notes

This is a maintainer-facing technical note. The public file reference lives at [../reference/strut-files.md](../reference/strut-files.md).

`.strut` is the open project/runtime container for Strut animation components.

## Goals

- Open and documented.
- Local-first.
- Versioned.
- Inspectable during early development.
- Friendly to source control.
- Efficient enough for runtime export.
- Strictly validated before playback or AI edits are accepted.

## Container

The initial `.strut` container is a ZIP archive:

```txt
component.strut
  manifest.json
  document.json
  assets/
    image-*.png
  previews/
    poster.png
```

The editable format starts as JSON for transparency. A future optimized runtime payload can be added:

```txt
runtime.strutb
```

## Manifest

```json
{
  "format": "strut",
  "schemaVersion": "0.1.0",
  "document": "document.json",
  "createdBy": "strut-studio",
  "minimumRuntime": "0.1.0"
}
```

## Document Model

```txt
Document
  Artboard[]
  Assets
  Animations
  StateMachines
  Bindings
  Events
```

## Scene Graph

```txt
Node
  Group
  Path
  Rect
  Ellipse
  Text
  Image
  HitArea
```

Each node has:

- stable id
- human-readable name
- transform
- visibility
- optional style
- optional children

## Animation

```txt
Timeline
  Track[]
    target node id
    property path
    Keyframe[]
```

Keyframes include:

- time
- value
- easing

## State Machines

```txt
StateMachine
  Input[]
  State[]
  Transition[]
```

Inputs:

- boolean
- number
- trigger
- enum

Transitions:

- source
- target
- condition
- animation
- duration

## Bindings

Bindings expose runtime-controllable properties:

```txt
label -> text node content
theme.accent -> fill color
progress -> timeline seek value
```

## Compatibility

Strut runtimes must reject unsupported major versions and report a clear diagnostic for unsupported features. Minor version behavior should be additive where possible.
