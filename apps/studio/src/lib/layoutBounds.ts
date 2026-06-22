import type { StrutDocument } from "../types";

export function ensureVisibleGeneratedDocument(document: StrutDocument): StrutDocument {
  const next = JSON.parse(JSON.stringify(document)) as StrutDocument;
  const artboard = next.artboards[0];
  if (!artboard) return next;

  const width = artboard.width || 960;
  const height = artboard.height || 540;
  let left = Number.POSITIVE_INFINITY;
  let top = Number.POSITIVE_INFINITY;
  let right = Number.NEGATIVE_INFINITY;
  let bottom = Number.NEGATIVE_INFINITY;

  for (const root of artboard.nodes) {
    const rootX = root.transform?.translate_x ?? 0;
    const rootY = root.transform?.translate_y ?? 0;
    for (const part of root.children ?? [root]) {
      const x = rootX + (part.transform?.translate_x ?? 0);
      const y = rootY + (part.transform?.translate_y ?? 0);
      const shape = part.shape;
      if (shape?.type === "ellipse") {
        left = Math.min(left, x + shape.cx - shape.rx);
        top = Math.min(top, y + shape.cy - shape.ry);
        right = Math.max(right, x + shape.cx + shape.rx);
        bottom = Math.max(bottom, y + shape.cy + shape.ry);
      }
      if (shape?.type === "rect") {
        left = Math.min(left, x + shape.x);
        top = Math.min(top, y + shape.y);
        right = Math.max(right, x + shape.x + shape.width);
        bottom = Math.max(bottom, y + shape.y + shape.height);
      }
    }
  }

  if (!Number.isFinite(left + top + right + bottom)) return next;
  const unsafe = left < width * 0.08 || right > width * 0.92 || top < height * 0.1 || bottom > height * 0.86;
  if (!unsafe) return next;

  const dx = width / 2 - (left + right) / 2;
  const dy = height * 0.46 - (top + bottom) / 2;
  for (const root of artboard.nodes) {
    root.transform = root.transform ?? {};
    root.transform.translate_x = (root.transform.translate_x ?? 0) + dx;
    root.transform.translate_y = (root.transform.translate_y ?? 0) + dy;
  }
  return next;
}
