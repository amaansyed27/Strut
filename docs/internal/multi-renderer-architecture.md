# Multi-renderer motion architecture

Strut should not force every animation into a `StrutDocument` SVG scene. That path is correct for editable vector motion, but it is the wrong renderer for CSS 3D objects, sprite-based characters, canvas effects, and complex product/UI components.

## Current problem

The current Studio generation path assumes:

```txt
provider output -> AssistantResult.document -> StrutDocument -> SVG preview -> SVG/runtime export
```

This makes dice, coins, cards, mascots, particles, and UI effects compete for the same limited rect/ellipse/path/text scene graph. The result is usually a flat 2.5D approximation, even when the browser has a better native renderer for the requested animation.

## Target path

```txt
prompt
-> motion intent router
-> renderer selection
-> MotionSpec
-> deterministic recipe/compiler
-> renderer-specific preview
-> verifier
-> export target
```

## Renderer map

| Prompt family | Preferred renderer |
| --- | --- |
| Dice, coin, card, cube, product spin | `dom-css3d` |
| Buttons, toggles, UI microinteractions | `dom-css` or `svg-css` |
| Icons, logos, loaders, vector reveals | `svg-css` |
| Mascots, pets, game-like character reactions | `sprite-css` or rigged SVG |
| Particles, liquid, smoke, physics effects | `canvas2d` or `webgl` |

## Output union

```ts
type MotionArtifact =
  | { kind: "strut_document"; renderer: "svg-css"; document: StrutDocument }
  | { kind: "runtime_component"; renderer: "dom-css3d" | "dom-css" | "sprite-css" | "canvas2d" | "webgl"; component: RuntimeComponent };
```

A runtime component owns its HTML, CSS, JS, states, inputs, and assets. It is previewed in an isolated iframe, not converted back into SVG nodes.

## First implementation slice

1. Add `MotionArtifact`, `RuntimeComponent`, `MotionSpec`, and renderer types.
2. Add an intent router so prompts route before generation.
3. Add `HtmlComponentPreview` for iframe-based DOM/CSS/JS preview.
4. Add deterministic recipes for `dom-css3d.die.roll`, then `dom-css3d.coin.flip` and `dom-css3d.card.flip`.
5. Add component verification: perspective, preserve-3d, translateZ, visible primary object, and non-empty states.
6. Store component artifacts beside `.strut` documents instead of forcing them into the `.strut` scene graph.

## Rule

If a renderer can solve the motion directly, Strut should use that renderer. SVG is one renderer, not the entire product.
