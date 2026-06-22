# CSS-native Strut engine

Strut should not use SVG as the core animation target. SVG can remain as a legacy import or export path, but the default `.strut` runtime should be a CSS-native scene graph.

## Core direction

`.strut` should store DOM/CSS motion data:

- scene size and perspective
- semantic CSS layers
- state names
- timelines
- keyframes
- CSS variables
- transform3d values
- material styles such as gradients, filters, masks, shadows, and opacity

## Layer types

The CSS runtime starts with these layer kinds:

- group
- plane
- disc
- ring
- text
- image
- sprite
- shadow
- glow

## Routing

The user should not choose implementation modes.

- buttons, loaders, toggles: CSS 2D layer motion
- coins, cards, dice, cubes: CSS 3D illusion rigs
- mascots and characters: sprite or pose rigs
- logo reveals: CSS layers, masks, filters, and timing
- product UI motion: CSS state machine

## Studio cleanup

Remove the visible strategy UI from the composer:

- slash command menu
- svg/sprite/dynamic user modes
- Auto strategy badge

The user describes the animation. Strut chooses the engine internally.

## Migration plan

1. Add CSS scene runtime.
2. Add Studio CSS scene preview.
3. Route new generation results to CSS documents by default.
4. Keep SVG renderer only as a legacy fallback.
5. Export HTML/CSS, React, and Web Component first.
6. Treat Lottie export as compatibility output, not the internal model.

## Implementation start

The first runtime file is `packages/runtime-web/src/css-scene.ts`. It defines the CSS document schema, DOM renderer, generated CSS keyframes, and state switching API.
