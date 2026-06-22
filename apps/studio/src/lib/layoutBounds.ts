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

  const addEllipse = (x: number, y: number, cx: number, cy: number, rx: number, ry: number) => {
    left = Math.min(left, x + cx - rx);
    top = Math.min(top, y + cy - ry);
    right = Math.max(right, x + cx + rx);
    bottom = Math.max(bottom, y + cy + ry);
  };
  const addRect = (x: number, y: number, rx: number, ry: number, w: number, h: number) => {
    left = Math.min(left, x + rx);
    top = Math.min(top, y + ry);
    right = Math.max(right, x + rx + w);
    bottom = Math.max(bottom, y + ry + h);
  };

  for (const root of artboard.nodes) {
    const rootX = root.transform?.translate_x ?? 0;
    const rootY = root.transform?.translate_y ?? 0;
    for (const group of root.children ?? [root]) {
      const groupX = rootX + (group.transform?.translate_x ?? 0);
      const groupY = rootY + (group.transform?.translate_y ?? 0);
      for (const part of group.children ?? [group]) {
        const x = groupX + (part.transform?.translate_x ?? 0);
        const y = groupY + (part.transform?.translate_y ?? 0);
        const shape = part.shape;
        if (shape?.type === "ellipse") addEllipse(x, y, shape.cx, shape.cy, shape.rx, shape.ry);
        if (shape?.type === "rect") addRect(x, y, shape.x, shape.y, shape.width, shape.height);
        if (shape?.type === "text") addRect(x, y, shape.x, shape.y - shape.size, Math.max(24, shape.value.length * shape.size * 0.62), shape.size);
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
