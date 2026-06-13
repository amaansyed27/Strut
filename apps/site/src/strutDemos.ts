import type { StrutDocument, StrutNode } from "@strut/runtime-web";

type Demo = {
  title: string;
  kind: string;
  copy: string;
  state: string;
  intervalMs: number;
  document: StrutDocument;
};

let idCounter = 1;

function id(prefix: string) {
  idCounter += 1;
  return `${prefix}-${idCounter}`;
}

function node(name: string, kind: string, shape: StrutNode["shape"], style: StrutNode["style"], children: StrutNode[] = []): StrutNode {
  return {
    id: name,
    name,
    kind,
    transform: { translate_x: 0, translate_y: 0, rotate: 0, scale_x: 1, scale_y: 1 },
    style,
    shape,
    children,
  };
}

function document(name: string, nodes: StrutNode[], state: string, target: string, property: string, values: [number, number, number]): StrutDocument {
  return {
    id: id("doc"),
    name,
    artboards: [{ id: id("artboard"), name: "Preview", width: 720, height: 460, nodes }],
    timelines: [
      {
        id: id("timeline"),
        name: state,
        duration_ms: 1200,
        tracks: [
          {
            target,
            property,
            keyframes: [
              { time_ms: 0, value: { type: "number", value: values[0] }, easing: "ease_in_out" },
              { time_ms: 620, value: { type: "number", value: values[1] }, easing: "ease_out" },
              { time_ms: 1200, value: { type: "number", value: values[2] }, easing: "ease_in_out" },
            ],
          },
        ],
      },
    ],
    state_machines: [
      {
        id: id("machine"),
        name: "Demo",
        states: ["idle", state],
        transitions: [{ from: "idle", to: state, on: state, timeline: state }],
      },
    ],
    bindings: [],
    events: [{ name: `${state}_started`, description: `${name} started` }],
  };
}

const ink = "#10231f";
const mint = "#76d6b4";
const sky = "#7dd3fc";
const coral = "#ff7f66";

export const demos: Demo[] = [
  {
    title: "Companion mascot",
    kind: "sprite-python",
    copy: "Editable body, face, wing, badge, halo, and shadow parts with quiet blink and wave motion.",
    state: "wave",
    intervalMs: 1500,
    document: document(
      "Companion Mascot",
      [
        node("Shadow", "ellipse", { type: "ellipse", cx: 360, cy: 360, rx: 125, ry: 18 }, { fill: "#08251f", opacity: 0.16 }),
        node("Body", "ellipse", { type: "ellipse", cx: 360, cy: 242, rx: 82, ry: 104 }, { fill: "#b7f5d8", stroke: ink, stroke_width: 5, opacity: 1 }),
        node("Head", "ellipse", { type: "ellipse", cx: 360, cy: 146, rx: 76, ry: 62 }, { fill: "#f3fff8", stroke: ink, stroke_width: 5, opacity: 1 }),
        node("LeftEye", "ellipse", { type: "ellipse", cx: 336, cy: 144, rx: 8, ry: 12 }, { fill: ink, opacity: 1 }),
        node("RightEye", "ellipse", { type: "ellipse", cx: 384, cy: 144, rx: 8, ry: 12 }, { fill: ink, opacity: 1 }),
        node("RightWing", "path", { type: "path", d: "M430 244 C492 262 510 314 456 334 C430 314 422 278 430 244Z" }, { fill: mint, stroke: "#08734f", stroke_width: 4, opacity: 1 }),
      ],
      "wave",
      "RightWing",
      "rotation",
      [-4, 9, 0],
    ),
  },
  {
    title: "Twitter-style bird",
    kind: "procedural SVG",
    copy: "Prompt-specific parts, no mascot anatomy, and a clean flight silhouette coding agents can patch.",
    state: "flight",
    intervalMs: 1350,
    document: document(
      "Twitter Bird Taking Flight",
      [
        node("Trail", "path", { type: "path", d: "M160 316 C250 354 456 346 560 288" }, { fill: null, stroke: "#b8f7df", stroke_width: 8, opacity: 0.55 }),
        node("Tail", "path", { type: "path", d: "M266 254 L196 214 L226 294 Z" }, { fill: "#bae6fd", stroke: "#075985", stroke_width: 4, opacity: 1 }),
        node("Body", "path", { type: "path", d: "M250 254 C326 160 472 178 532 262 C458 246 392 288 342 356 C302 326 268 292 250 254Z" }, { fill: sky, stroke: ink, stroke_width: 5, opacity: 1 }),
        node("Wing", "path", { type: "path", d: "M330 252 C400 172 520 178 586 244 C488 244 424 296 372 356 Z" }, { fill: "#38bdf8", stroke: "#075985", stroke_width: 4, opacity: 1 }),
        node("Beak", "path", { type: "path", d: "M532 262 L592 236 L544 296 Z" }, { fill: "#fbbf24", stroke: "#92400e", stroke_width: 3, opacity: 1 }),
      ],
      "flight",
      "Wing",
      "translation.y",
      [0, -18, 0],
    ),
  },
  {
    title: "Logo reveal",
    kind: "semantic SVG",
    copy: "Primary mark, wordmark, accent stroke, and reveal mask stay named and editable.",
    state: "reveal",
    intervalMs: 1600,
    document: document(
      "Abstract Logo Reveal",
      [
        node("Glow", "ellipse", { type: "ellipse", cx: 360, cy: 226, rx: 142, ry: 86 }, { fill: "#dcfce7", opacity: 0.32 }),
        node("PrimaryMark", "path", { type: "path", d: "M240 190 C302 110 430 126 496 228 C420 212 354 252 298 342 C258 294 222 238 240 190 Z" }, { fill: mint, stroke: ink, stroke_width: 5, opacity: 1 }),
        node("AccentStroke", "path", { type: "path", d: "M252 342 C344 382 458 366 542 312" }, { fill: null, stroke: "#2563eb", stroke_width: 9, opacity: 1 }),
      ],
      "reveal",
      "PrimaryMark",
      "scale",
      [0.82, 1.06, 1],
    ),
  },
  {
    title: "Progress loader",
    kind: "runtime loop",
    copy: "Track, active segment, pulse dot, and label form a calm product loading state.",
    state: "loading",
    intervalMs: 1200,
    document: document(
      "Progress Loader",
      [
        node("Track", "ellipse", { type: "ellipse", cx: 360, cy: 226, rx: 118, ry: 118 }, { fill: null, stroke: "#cbd5e1", stroke_width: 14, opacity: 1 }),
        node("ActiveSegment", "path", { type: "path", d: "M360 108 A118 118 0 0 1 478 226" }, { fill: null, stroke: "#14b8a6", stroke_width: 18, opacity: 1 }),
        node("PulseDot", "ellipse", { type: "ellipse", cx: 478, cy: 226, rx: 15, ry: 15 }, { fill: "#0f766e", stroke: "#064e3b", stroke_width: 3, opacity: 1 }),
      ],
      "loading",
      "ActiveSegment",
      "rotation",
      [0, 180, 360],
    ),
  },
  {
    title: "UI microinteraction",
    kind: "app state",
    copy: "Small hover, focus, and success motion without distracting users from the actual product.",
    state: "hover",
    intervalMs: 1300,
    document: document(
      "Button Microinteraction",
      [
        node("HoverGlow", "ellipse", { type: "ellipse", cx: 360, cy: 230, rx: 165, ry: 60 }, { fill: "#bae6fd", opacity: 0.28 }),
        node("ButtonSurface", "rect", { type: "rect", x: 230, y: 188, width: 260, height: 86, rx: 18 }, { fill: "#e0f2fe", stroke: ink, stroke_width: 4, opacity: 1 }),
        node("ButtonLabel", "text", { type: "text", x: 300, y: 242, value: "Continue", size: 28 }, { fill: ink, opacity: 1 }),
      ],
      "hover",
      "ButtonSurface",
      "translation.y",
      [0, -8, 0],
    ),
  },
  {
    title: "Icon badge",
    kind: "export proof",
    copy: "A small badge animation that can be exported as readable scene JSON and React playback.",
    state: "success",
    intervalMs: 1450,
    document: document(
      "Icon Badge",
      [
        node("BadgePlate", "ellipse", { type: "ellipse", cx: 360, cy: 226, rx: 112, ry: 112 }, { fill: "#eef2ff", stroke: "#1e1b4b", stroke_width: 5, opacity: 1 }),
        node("InnerShield", "path", { type: "path", d: "M360 126 L438 166 L424 264 C408 310 384 338 360 350 C336 338 312 310 296 264 L282 166 Z" }, { fill: "#c7d2fe", stroke: "#312e81", stroke_width: 4, opacity: 1 }),
        node("StatusDot", "ellipse", { type: "ellipse", cx: 454, cy: 150, rx: 20, ry: 20 }, { fill: "#22c55e", stroke: "#14532d", stroke_width: 3, opacity: 1 }),
        node("SparkGlyph", "path", { type: "path", d: "M360 172 L376 214 L420 226 L378 240 L360 284 L342 240 L300 226 L344 214 Z" }, { fill: "#fde68a", stroke: "#92400e", stroke_width: 4, opacity: 1 }),
      ],
      "success",
      "StatusDot",
      "scale",
      [0.88, 1.12, 0.88],
    ),
  },
];

export const releaseChecks = [
  {
    status: "Built",
    title: "Desktop Studio",
    copy: "Rust/Tauri app with AI edit mode, validated operations, persistence, undo/redo, and local-first projects.",
  },
  {
    status: "Built",
    title: "Strut Sprite",
    copy: "Python sprite/vector authoring emits plans and operations that Rust validates before persistence.",
  },
  {
    status: "Built",
    title: "Agent workflow",
    copy: "CLI can inspect, plan, patch, verify, render proof, and export React integration files.",
  },
  {
    status: "Needs release pass",
    title: "Installers and signed builds",
    copy: "Windows, macOS, and Linux release artifacts still need packaging, signing, and a final smoke on clean machines.",
  },
];
