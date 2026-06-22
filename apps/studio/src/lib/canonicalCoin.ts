import type { StrutDocument, StrutNode, Timeline } from "../types";

const id = (index: number) => `00000000-0000-4000-8000-${index.toString(16).padStart(12, "0")}`;

const IDs = {
  document: id(1),
  artboard: id(2),
  machine: id(3),
  root: id(4),
  shadow: id(5),
  rig: id(6),
  rimBack: id(7),
  rimSide: id(8),
  front: id(9),
  frontSurface: id(10),
  frontOuter: id(11),
  frontInner: id(12),
  frontEmblem: id(13),
  back: id(14),
  backSurface: id(15),
  backOuter: id(16),
  backEmblem: id(17),
  glint: id(18),
  spark: id(19),
  idle: id(20),
  anticipation: id(21),
  flip: id(22),
  settle: id(23),
};

function style(fill: string | null, stroke: string | null, strokeWidth = 0, opacity = 1) {
  return { fill, stroke, stroke_width: strokeWidth, opacity, linecap: "round", linejoin: "round" };
}

function transform(translate_x = 0, translate_y = 0, scale_x = 1, scale_y = 1) {
  return { translate_x, translate_y, rotate: 0, rotate_x: 0, rotate_y: 0, scale_x, scale_y };
}

function group(idValue: string, name: string, role: string, children: StrutNode[], x = 0, y = 0, opacity = 1): StrutNode {
  return {
    id: idValue,
    name,
    kind: "group",
    role,
    transform: transform(x, y),
    style: style(null, null, 0, opacity),
    shape: { type: "none" },
    children,
  };
}

function ellipse(idValue: string, name: string, role: string, rx: number, ry: number, fill: string | null, stroke: string | null, strokeWidth = 0, x = 0, y = 0, opacity = 1): StrutNode {
  return {
    id: idValue,
    name,
    kind: "ellipse",
    role,
    transform: transform(x, y),
    style: style(fill, stroke, strokeWidth, opacity),
    shape: { type: "ellipse", cx: 0, cy: 0, rx, ry },
    children: [],
  };
}

function path(idValue: string, name: string, role: string, d: string, fill: string | null, stroke: string | null, strokeWidth = 0, opacity = 1): StrutNode {
  return {
    id: idValue,
    name,
    kind: "path",
    role,
    transform: transform(),
    style: style(fill, stroke, strokeWidth, opacity),
    shape: { type: "path", d },
    children: [],
  };
}

function value(v: number) {
  return { type: "number", value: v } as const;
}

function track(target: string, property: string, frames: Array<[number, number, Timeline["tracks"][number]["keyframes"][number]["easing"]]>) {
  return {
    target,
    property,
    keyframes: frames.map(([time_ms, v, easing]) => ({ time_ms, value: value(v), easing })),
  };
}

function timeline(idValue: string, name: string, duration_ms: number, loops: boolean, tracks: NonNullable<Timeline["tracks"]>): Timeline {
  return { id: idValue, name, duration_ms, loops, tracks };
}

export function shouldUseCanonicalCoin(prompt: string): boolean {
  const lower = prompt.toLowerCase();
  return /\b(coin|medallion|medal|heads|tails)\b/.test(lower) && /\b(flip|spin|front|back|rim|glint|settle|anticipation|2\.5d|3d)\b/.test(lower);
}

export function createCanonicalCoinDocument(name = "Premium 2.5D Coin Flip"): StrutDocument {
  const frontFace = group(IDs.front, "Front Face Group", "front face", [
    ellipse(IDs.frontSurface, "Front Face Surface", "front face surface", 72, 72, "#f8c70a", "#5f3b00", 4),
    ellipse(IDs.frontOuter, "Front Outer Rim", "rim", 64, 64, null, "#fff1a8", 7, 0, 0, 0.94),
    ellipse(IDs.frontInner, "Front Inner Bezel", "inner bezel", 48, 48, null, "#ad7609", 4, 0, 0, 0.9),
    path(IDs.frontEmblem, "Front Emblem Star", "front emblem detail", "M0 -34 L10 -10 L36 -10 L15 5 L24 32 L0 16 L-24 32 L-15 5 L-36 -10 L-10 -10 Z", "#7c4a00", "#fff0a6", 3, 0.95),
  ]);

  const backFace = group(IDs.back, "Back Face Group", "back face", [
    ellipse(IDs.backSurface, "Back Face Surface", "back face surface", 72, 72, "#e39a08", "#5f3b00", 4),
    ellipse(IDs.backOuter, "Back Outer Rim", "back rim", 64, 64, null, "#ffe29a", 7, 0, 0, 0.9),
    path(IDs.backEmblem, "Back Emblem Orbit", "back emblem detail", "M-38 0 C-20 -26 20 -26 38 0 C20 26 -20 26 -38 0 M0 -38 C26 -20 26 20 0 38 C-26 20 -26 -20 0 -38", null, "#7b4c05", 5, 0.95),
  ], 0, 0, 0);

  const coinRig = group(IDs.rig, "Coin Rig", "coin rig", [
    ellipse(IDs.rimBack, "Rim Depth Back Plate", "rim depth edge", 78, 73, "#7c4a06", "#3b2300", 3, 9, 10),
    ellipse(IDs.rimSide, "Warm Side Thickness", "side edge depth", 76, 72, "#b97809", "#5b3500", 2, 5, 6),
    frontFace,
    backFace,
    path(IDs.glint, "Moving Glint Highlight", "glint highlight polish", "M-42 -40 C-18 -58 18 -58 42 -40", null, "#fff8d2", 6, 0.82),
    path(IDs.spark, "Settle Spark", "spark polish", "M86 -54 L86 -22 M70 -38 L102 -38", null, "#fff7bf", 5, 0),
  ], 480, 258);

  return {
    id: IDs.document,
    name,
    artboards: [{
      id: IDs.artboard,
      name: "Coin Artboard",
      width: 960,
      height: 540,
      nodes: [group(IDs.root, "Root", "root", [
        ellipse(IDs.shadow, "Reactive Ground Shadow", "reactive ground shadow", 96, 17, "#1f2937", null, 0, 488, 392, 0.18),
        coinRig,
      ])],
    }],
    timelines: [
      timeline(IDs.idle, "idle", 1600, true, [
        track(IDs.rig, "translation.y", [[0, 0, "ease_in_out"], [800, -8, "ease_in_out"], [1600, 0, "ease_in_out"]]),
        track(IDs.rig, "rotation", [[0, -2, "ease_in_out"], [800, 2, "ease_in_out"], [1600, -2, "ease_in_out"]]),
        track(IDs.shadow, "scale.x", [[0, 1, "ease_in_out"], [800, 0.86, "ease_in_out"], [1600, 1, "ease_in_out"]]),
      ]),
      timeline(IDs.anticipation, "anticipation", 520, false, [
        track(IDs.rig, "translation.y", [[0, 0, "ease_out"], [220, 10, "ease_in"], [520, -18, "ease_out"]]),
        track(IDs.rig, "scale.x", [[0, 1, "ease_out"], [220, 1.12, "ease_in"], [520, 0.92, "ease_out"]]),
        track(IDs.shadow, "scale.x", [[0, 1, "ease_out"], [220, 1.18, "ease_in"], [520, 0.74, "ease_out"]]),
      ]),
      timeline(IDs.flip, "flip", 1250, false, [
        track(IDs.rig, "translation.y", [[0, -16, "ease_out"], [420, -96, "ease_out"], [900, -20, "ease_in_out"], [1250, 0, "ease_out"]]),
        track(IDs.rig, "scale.x", [[0, 1, "ease_in_out"], [260, 0.14, "ease_in_out"], [520, -1, "ease_in_out"], [780, -0.16, "ease_in_out"], [1020, 1.05, "ease_out"], [1250, 1, "ease_out"]]),
        track(IDs.rig, "rotation", [[0, 0, "ease_in_out"], [520, 210, "linear"], [1250, 360, "ease_out"]]),
        track(IDs.front, "opacity", [[0, 1, "linear"], [510, 1, "linear"], [540, 0, "linear"], [1250, 0, "linear"]]),
        track(IDs.back, "opacity", [[0, 0, "linear"], [510, 0, "linear"], [540, 1, "linear"], [1250, 1, "linear"]]),
        track(IDs.glint, "translation.x", [[0, -42, "ease_out"], [680, 42, "ease_in_out"], [1250, 8, "ease_out"]]),
        track(IDs.shadow, "opacity", [[0, 0.18, "ease_out"], [420, 0.04, "ease_out"], [1250, 0.2, "ease_out"]]),
        track(IDs.shadow, "scale.x", [[0, 0.76, "ease_out"], [420, 0.46, "ease_out"], [1250, 1.08, "ease_out"]]),
      ]),
      timeline(IDs.settle, "settle", 720, false, [
        track(IDs.rig, "translation.y", [[0, -24, "ease_out"], [280, 8, "ease_in_out"], [720, 0, "ease_out"]]),
        track(IDs.rig, "scale", [[0, 0.94, "ease_out"], [280, 1.08, "ease_in_out"], [720, 1, "ease_out"]]),
        track(IDs.front, "opacity", [[0, 0, "linear"], [720, 0, "linear"]]),
        track(IDs.back, "opacity", [[0, 1, "linear"], [720, 1, "linear"]]),
        track(IDs.spark, "opacity", [[0, 0, "linear"], [160, 1, "ease_out"], [720, 0, "ease_in"]]),
        track(IDs.shadow, "scale.x", [[0, 0.82, "ease_out"], [280, 1.18, "ease_in_out"], [720, 1, "ease_out"]]),
      ]),
    ],
    state_machines: [{ id: IDs.machine, name: "Controller", states: ["idle", "anticipation", "flip", "settle"], inputs: [], transitions: [] }],
    bindings: [],
    events: [],
  };
}
