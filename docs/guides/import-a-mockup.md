# Import A Mockup

Strut can start from a mockup, image, or SVG.

## SVG

SVG is the best starting point because it already contains vector geometry. Strut can parse paths, groups, fills, strokes, and names without guessing.

Recommended workflow:

1. Export a clean SVG from your design tool.
2. Import it into Strut.
3. Review the layer names.
4. Group related parts.
5. Add motion and states.

## Raster Images

PNG, JPG, and WebP files need interpretation. Strut combines image processing and vision models to estimate layers and shapes.

Raster import is useful for:

- screenshots
- rough mockups
- sketches
- references from another app

Raster import may need cleanup because pixels do not contain real layer information.

## What Happens After Import

Strut converts the mockup into an editable scene:

```txt
mockup
  -> layers
  -> groups
  -> named parts
  -> artboard
  -> optional motion plan
```
