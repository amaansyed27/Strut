import type { StrutDocument, StrutNode, Timeline } from "../types";

type Easing = "linear" | "ease_in" | "ease_out" | "ease_in_out" | "steps";
type NumericFrame = [timeMs: number, value: number, easing?: Easing];

const stableUuid = (index: number) => `00000000-0000-4000-8000-${index.toString(16).padStart(12, "0")}`;

const ID = {
  document: stableUuid(1),
  artboard: stableUuid(2),
  controller: stableUuid(3),
  root: stableUuid(4),
  groundShadow: stableUuid(5),
  contactSpark: stableUuid(6),
  coinRig: stableUuid(7),
  edgeStack: stableUuid(8),
  depth5: stableUuid(9),
  depth4: stableUuid(10),
  depth3: stableUuid(11),
  depth2: stableUuid(12),
  edgeLines: stableUuid(13),
  backLayers: stableUuid(14),
  backFace: stableUuid(15),
  backInner: stableUuid(16),
  backRim: stableUuid(17),
  backStar: stableUuid(18),
  backOrbitA: stableUuid(19),
  backOrbitB: stableUuid(20),
  backGlint: stableUuid(21),
  frontLayers: stableUuid(22),
  frontFace: stableUuid(23),
  frontInner: stableUuid(24),
  frontRim: stableUuid(25),
  frontRimLight: stableUuid(26),
  frontHead: stableUuid(27),
  frontLaurelLeft: stableUuid(28),
  frontLaurelRight: stableUuid(29),
  frontGlint: stableUuid(30),
  idleTimeline: stableUuid(31),
  flipHeadsTimeline: stableUuid(32),
  flipTailsTimeline: stableUuid(33),
  hoverTimeline: stableUuid(34),
} as const;

const num = (time_ms: number, value: number, easing: Easing = "ease_in_out") => ({
  time_ms,
  value: { type: "number" as const, value },
  easing,
});

const track = (target: string, property: string, frames: NumericFrame[]): NonNullable<Timeline["tracks"]>[number] => ({
  target,
  property,
  keyframes: frames.map(([timeMs, value, easing]) => num(timeMs, value, easing)),
});

const style = (fill?: string | null, stroke?: string | null, stroke_width = 0, opacity = 1): NonNullable<StrutNode["style"]> => ({
  fill: fill ?? null,
  stroke: stroke ?? null,
  stroke_width,
  opacity,
  linecap: "round",
  linejoin: "round",
});

const transform = (translate_x = 0, translate_y = 0, rotate = 0, scale_x = 1, scale_y = 1): NonNullable<StrutNode["transform"]> => ({
  translate_x,
  translate_y,
  rotate,
  scale_x,
  scale_y,
});

const node = (
  id: string,
  name: string,
  kind: string,
  role: string,
  shape: NonNullable<StrutNode["shape"]>,
  nodeStyle: NonNullable<StrutNode["style"]>,
  nodeTransform: NonNullable<StrutNode["transform"]> = transform(),
  children: StrutNode[] = [],
): StrutNode => ({
  id,
  name,
  kind,
  role,
  shape,
  transform: nodeTransform,
  style: nodeStyle,
  children,
});

const group = (
  id: string,
  name: string,
  role: string,
  children: StrutNode[],
  nodeTransform: NonNullable<StrutNode["transform"]> = transform(),
  opacity = 1,
): StrutNode => node(id, name, "group", role, { type: "none" }, style(null, null, 0, opacity), nodeTransform, children);

const ellipse = (cx: number, cy: number, rx: number, ry: number): NonNullable<StrutNode["shape"]> => ({
  type: "ellipse",
  cx,
  cy,
  rx,
  ry,
});

const path = (d: string): NonNullable<StrutNode["shape"]> => ({ type: "path", d });

function collectPartNames(nodes: StrutNode[]): string[] {
  return nodes.flatMap((item) => [item.name, ...collectPartNames(item.children ?? [])]);
}

export function shouldUseCanonicalCoin(prompt: string, document?: StrutDocument): boolean {
  const timelineNames = document?.timelines.map((timeline) => timeline.name).join(" ") ?? "";
  const stateNames = document?.state_machines.flatMap((machine) => machine.states).join(" ") ?? "";
  const text = `${prompt} ${document?.name ?? ""} ${timelineNames} ${stateNames}`.toLowerCase();
  const hasCoinSubject = /\b(coin|medallion|medal|doubloon)\b/.test(text)
    || (/\btoken\b/.test(text) && /\b(flip|heads|tails|spin|rotate|2\.5d|3d)\b/.test(text));
  const hasFlipIntent = /\b(flip|heads|tails|spin|rotate|2\.5d|3d|rim|edge|depth)\b/.test(text);
  return hasCoinSubject && hasFlipIntent;
}

export function canonicalCoinPlanSummary(document: StrutDocument) {
  return {
    subjectClassification: "object",
    subjectLabel: "premium 2.5D coin flip",
    partNames: collectPartNames(document.artboards[0]?.nodes ?? []).filter((name) => name !== "Coin Scene Root"),
    timelineNames: document.timelines.map((timeline) => timeline.name),
  };
}

export function createCanonicalCoinDocument(name = "Premium 2.5D Coin Flip"): StrutDocument {
  const frontLayers = group(ID.frontLayers, "Heads Face Layers", "front face", [
    node(ID.frontFace, "Heads Gold Surface", "ellipse", "face", ellipse(0, 0, 92, 92), style("#f7c948", "#7c4a03", 3)),
    node(ID.frontInner, "Inset Warm Surface", "ellipse", "face detail", ellipse(0, 0, 72, 72), style("#fbd45d", "#b7791f", 4)),
    node(ID.frontRim, "Raised Outer Rim", "ellipse", "rim", ellipse(0, 0, 98, 98), style(null, "#8a5200", 9)),
    node(ID.frontRimLight, "Rim Highlight Arc", "path", "highlight", path("M-64 -62 C-22 -94 48 -88 74 -42"), style(null, "#fff3a3", 8, 0.88)),
    node(ID.frontHead, "Embossed Heads Profile", "path", "heads symbol", path("M-8 -30 C9 -30 20 -16 17 2 C15 15 8 24 0 28 C-8 24 -15 15 -17 2 C-20 -16 -8 -30 -8 -30 Z M-30 50 C-16 33 16 33 31 50 L31 61 L-30 61 Z"), style("#8a5200", "#5f3700", 2, 0.92)),
    node(ID.frontLaurelLeft, "Left Laurel Cut", "path", "engraving", path("M-48 -18 C-62 -4 -62 22 -46 38 M-53 1 L-67 -1 M-55 16 L-68 22 M-50 31 L-61 42"), style(null, "#a76503", 4, 0.8)),
    node(ID.frontLaurelRight, "Right Laurel Cut", "path", "engraving", path("M48 -18 C62 -4 62 22 46 38 M53 1 L67 -1 M55 16 L68 22 M50 31 L61 42"), style(null, "#a76503", 4, 0.8)),
    node(ID.frontGlint, "Moving Face Glint", "ellipse", "glint", ellipse(-34, -46, 16, 7), style("#fff8c7", null, 0, 0.82), transform(0, 0, -24)),
  ]);

  const backLayers = group(ID.backLayers, "Tails Face Layers", "back face", [
    node(ID.backFace, "Tails Amber Surface", "ellipse", "face", ellipse(0, 0, 92, 92), style("#d69e2e", "#713f12", 3)),
    node(ID.backInner, "Inset Tails Surface", "ellipse", "face detail", ellipse(0, 0, 72, 72), style("#ecc94b", "#975a16", 4)),
    node(ID.backRim, "Raised Tails Rim", "ellipse", "rim", ellipse(0, 0, 98, 98), style(null, "#7c2d12", 9)),
    node(ID.backStar, "Embossed Tails Star", "path", "tails symbol", path("M0 -52 L13 -17 L50 -17 L20 6 L31 43 L0 21 L-31 43 L-20 6 L-50 -17 L-13 -17 Z"), style("#7c2d12", "#451a03", 2, 0.94)),
    node(ID.backOrbitA, "Tails Orbit Ring", "ellipse", "engraving", ellipse(0, 0, 54, 22), style(null, "#9a5a0a", 4, 0.82), transform(0, 0, -18)),
    node(ID.backOrbitB, "Tails Cross Orbit", "ellipse", "engraving", ellipse(0, 0, 54, 22), style(null, "#9a5a0a", 4, 0.7), transform(0, 0, 34)),
    node(ID.backGlint, "Back Face Glint", "ellipse", "glint", ellipse(38, -42, 14, 6), style("#fff3a3", null, 0, 0.72), transform(0, 0, 20)),
  ], transform(), 0);

  const edgeStack = group(ID.edgeStack, "Visible Rim Depth Stack", "rim depth", [
    node(ID.depth5, "Deep Bronze Thickness", "ellipse", "depth", ellipse(12, 14, 96, 94), style("#7c2d12", "#5f3700", 2, 0.96)),
    node(ID.depth4, "Warm Side Thickness", "ellipse", "depth", ellipse(9, 11, 96, 94), style("#9a5a0a", null, 0, 0.96)),
    node(ID.depth3, "Mid Rim Thickness", "ellipse", "depth", ellipse(6, 8, 96, 94), style("#b7791f", null, 0, 0.94)),
    node(ID.depth2, "Lit Rim Thickness", "ellipse", "depth", ellipse(3, 4, 96, 94), style("#d69e2e", null, 0, 0.92)),
    node(ID.edgeLines, "Grooved Edge Lines", "path", "edge detail", path("M88 -48 L108 -38 M93 -18 L115 -10 M95 16 L116 24 M84 50 L104 62"), style(null, "#fff0a3", 4, 0.8)),
  ], transform(0, 0), 0.92);

  const coinRig = group(ID.coinRig, "Coin 2.5D Flip Rig", "primary motion", [edgeStack, backLayers, frontLayers], transform(480, 250));

  const root = group(ID.root, "Coin Scene Root", "root", [
    node(ID.groundShadow, "Reactive Ground Shadow", "ellipse", "shadow", ellipse(0, 0, 118, 18), style("#1f2937", null, 0, 0.2), transform(492, 388)),
    node(ID.contactSpark, "Settle Spark", "path", "polish", path("M-14 0 L-28 0 M14 0 L29 0 M0 -8 L0 -20"), style(null, "#f6c84c", 5, 0), transform(480, 382)),
    coinRig,
  ]);

  const timelines: Timeline[] = [
    {
      id: ID.idleTimeline,
      name: "idle",
      duration_ms: 1600,
      loops: true,
      tracks: [
        track(ID.coinRig, "translation.y", [[0, 0], [800, -9], [1600, 0]]),
        track(ID.coinRig, "rotation", [[0, -2], [800, 2], [1600, -2]]),
        track(ID.coinRig, "rotation.y", [[0, -8], [800, 8], [1600, -8]]),
        track(ID.coinRig, "scale.x", [[0, 1], [800, 0.96], [1600, 1]]),
        track(ID.frontGlint, "translation.x", [[0, 0], [800, 18], [1600, 0]]),
        track(ID.groundShadow, "scale.x", [[0, 1], [800, 0.82], [1600, 1]]),
        track(ID.groundShadow, "opacity", [[0, 0.2], [800, 0.1], [1600, 0.2]]),
        track(ID.frontLayers, "opacity", [[0, 1, "linear"], [1600, 1, "linear"]]),
        track(ID.backLayers, "opacity", [[0, 0, "linear"], [1600, 0, "linear"]]),
      ],
    },
    {
      id: ID.flipHeadsTimeline,
      name: "flip_heads",
      duration_ms: 1220,
      loops: false,
      tracks: [
        track(ID.coinRig, "translation.y", [[0, 4, "ease_out"], [110, 12, "ease_in"], [410, -86, "ease_out"], [790, -48, "ease_in_out"], [1030, 10, "ease_out"], [1220, 0, "ease_out"]]),
        track(ID.coinRig, "rotation", [[0, -8], [410, 26, "linear"], [790, -16, "linear"], [1030, 5], [1220, 0]]),
        track(ID.coinRig, "rotation.y", [[0, 0, "linear"], [260, 170, "linear"], [520, 360, "linear"], [820, 570, "linear"], [1220, 720, "ease_out"]]),
        track(ID.coinRig, "scale.x", [[0, 1, "linear"], [245, 0.16, "linear"], [515, -0.94, "linear"], [805, 0.18, "linear"], [1220, 1, "ease_out"]]),
        track(ID.coinRig, "scale.y", [[0, 1], [410, 1.08], [1030, 0.94], [1220, 1]]),
        track(ID.frontLayers, "opacity", [[0, 1, "linear"], [300, 0, "linear"], [760, 0, "linear"], [900, 1, "linear"], [1220, 1, "linear"]]),
        track(ID.backLayers, "opacity", [[0, 0, "linear"], [300, 1, "linear"], [760, 1, "linear"], [900, 0, "linear"], [1220, 0, "linear"]]),
        track(ID.edgeStack, "opacity", [[0, 0.92], [245, 1], [515, 0.88], [805, 1], [1220, 0.92]]),
        track(ID.groundShadow, "scale.x", [[0, 1], [410, 0.48], [790, 0.72], [1030, 1.18], [1220, 1]]),
        track(ID.groundShadow, "opacity", [[0, 0.2], [410, 0.05], [790, 0.11], [1030, 0.28], [1220, 0.2]]),
        track(ID.contactSpark, "opacity", [[0, 0, "linear"], [990, 0, "linear"], [1050, 1, "ease_out"], [1220, 0, "ease_in"]]),
      ],
    },
    {
      id: ID.flipTailsTimeline,
      name: "flip_tails",
      duration_ms: 1220,
      loops: false,
      tracks: [
        track(ID.coinRig, "translation.y", [[0, 4, "ease_out"], [110, 12, "ease_in"], [410, -86, "ease_out"], [790, -48, "ease_in_out"], [1030, 10, "ease_out"], [1220, 0, "ease_out"]]),
        track(ID.coinRig, "rotation", [[0, 8], [410, -24, "linear"], [790, 18, "linear"], [1030, -5], [1220, 0]]),
        track(ID.coinRig, "rotation.y", [[0, 0, "linear"], [260, 180, "linear"], [520, 390, "linear"], [820, 690, "linear"], [1220, 900, "ease_out"]]),
        track(ID.coinRig, "scale.x", [[0, 1, "linear"], [245, 0.16, "linear"], [515, -0.94, "linear"], [805, 0.18, "linear"], [1220, 1, "ease_out"]]),
        track(ID.coinRig, "scale.y", [[0, 1], [410, 1.08], [1030, 0.94], [1220, 1]]),
        track(ID.frontLayers, "opacity", [[0, 1, "linear"], [300, 0, "linear"], [1220, 0, "linear"]]),
        track(ID.backLayers, "opacity", [[0, 0, "linear"], [300, 1, "linear"], [1220, 1, "linear"]]),
        track(ID.edgeStack, "opacity", [[0, 0.92], [245, 1], [515, 0.88], [805, 1], [1220, 0.92]]),
        track(ID.groundShadow, "scale.x", [[0, 1], [410, 0.48], [790, 0.72], [1030, 1.18], [1220, 1]]),
        track(ID.groundShadow, "opacity", [[0, 0.2], [410, 0.05], [790, 0.11], [1030, 0.28], [1220, 0.2]]),
        track(ID.contactSpark, "opacity", [[0, 0, "linear"], [990, 0, "linear"], [1050, 1, "ease_out"], [1220, 0, "ease_in"]]),
      ],
    },
    {
      id: ID.hoverTimeline,
      name: "hover",
      duration_ms: 520,
      loops: false,
      tracks: [
        track(ID.coinRig, "translation.y", [[0, 0], [260, -18, "ease_out"], [520, -12, "ease_in_out"]]),
        track(ID.coinRig, "scale", [[0, 1], [260, 1.07, "ease_out"], [520, 1.04, "ease_in_out"]]),
        track(ID.coinRig, "rotation.y", [[0, 0], [520, 14, "ease_out"]]),
        track(ID.groundShadow, "scale.x", [[0, 1], [260, 0.72], [520, 0.78]]),
        track(ID.groundShadow, "opacity", [[0, 0.2], [260, 0.09], [520, 0.11]]),
        track(ID.frontGlint, "opacity", [[0, 0.82], [260, 1], [520, 0.88]]),
      ],
    },
  ];

  return {
    id: ID.document,
    name: name.trim() || "Premium 2.5D Coin Flip",
    artboards: [{
      id: ID.artboard,
      name: "Premium 2.5D Coin Flip",
      width: 960,
      height: 540,
      nodes: [root],
    }],
    timelines,
    state_machines: [{
      id: ID.controller,
      name: "Controller",
      states: ["idle", "flip_heads", "flip_tails", "hover"],
      inputs: [{ name: "play", kind: "trigger" }],
      transitions: [],
    }],
    bindings: [],
    events: [],
  };
}
