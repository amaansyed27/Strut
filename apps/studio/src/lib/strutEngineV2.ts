import type { StrutDocument, StrutNode, Timeline } from "../types";

type Easing = "linear" | "ease_in" | "ease_out" | "ease_in_out" | "steps";
type Bounds = { x: number; y: number; width: number; height: number; cx: number; cy: number };

type VisualStats = {
  visibleNonShadow: number;
  visibleDetail: number;
  visibleColors: number;
  activeTracks: number;
};

const STATE_KEYWORDS = ["idle", "hover", "press", "success", "loading", "anticipation", "flip", "settle", "jump", "wave"];

let counter = 800000;
const nextId = (label: string) => `00000000-0000-4000-9000-${(counter++).toString(16).padStart(12, "0")}-${label}`.slice(0, 36);

function cloneDocument(document: StrutDocument): StrutDocument {
  return JSON.parse(JSON.stringify(document)) as StrutDocument;
}

function lower(value: string | undefined | null) {
  return (value ?? "").toLowerCase();
}

function value(v: number) {
  return { type: "number", value: v } as const;
}

function key(time_ms: number, v: number, easing: Easing = "ease_in_out") {
  return { time_ms, value: value(v), easing };
}

function track(target: string, property: string, frames: Array<[number, number, Easing]>): NonNullable<Timeline["tracks"]>[number] {
  return { target, property, keyframes: frames.map(([t, v, e]) => key(t, v, e)) };
}

function timeline(name: string, duration: number, loops: boolean, tracks: NonNullable<Timeline["tracks"]>): Timeline {
  return { id: nextId(`tl-${name}`), name, duration_ms: duration, loops, tracks };
}

function nodeBounds(node: StrutNode, parentX = 0, parentY = 0): Bounds | null {
  const tx = parentX + (node.transform?.translate_x ?? 0);
  const ty = parentY + (node.transform?.translate_y ?? 0);
  const shape = node.shape ?? { type: "none" as const };
  let own: Bounds | null = null;
  if (shape.type === "ellipse") own = { x: tx + shape.cx - shape.rx, y: ty + shape.cy - shape.ry, width: shape.rx * 2, height: shape.ry * 2, cx: tx + shape.cx, cy: ty + shape.cy };
  if (shape.type === "rect") own = { x: tx + shape.x, y: ty + shape.y, width: shape.width, height: shape.height, cx: tx + shape.x + shape.width / 2, cy: ty + shape.y + shape.height / 2 };
  if (shape.type === "text") own = { x: tx + shape.x, y: ty + shape.y - shape.size, width: Math.max(shape.value.length * shape.size * 0.58, 24), height: shape.size, cx: tx + shape.x, cy: ty + shape.y - shape.size / 2 };
  const childBounds = (node.children ?? []).map((child) => nodeBounds(child, tx, ty)).filter((b): b is Bounds => Boolean(b));
  const all = own ? [own, ...childBounds] : childBounds;
  if (!all.length) return null;
  const minX = Math.min(...all.map((b) => b.x));
  const minY = Math.min(...all.map((b) => b.y));
  const maxX = Math.max(...all.map((b) => b.x + b.width));
  const maxY = Math.max(...all.map((b) => b.y + b.height));
  return { x: minX, y: minY, width: maxX - minX, height: maxY - minY, cx: (minX + maxX) / 2, cy: (minY + maxY) / 2 };
}

function visitNodes(nodes: StrutNode[], visitor: (node: StrutNode, inheritedOpacity: number) => void, opacity = 1) {
  for (const node of nodes) {
    const nextOpacity = opacity * (node.style?.opacity ?? 1);
    visitor(node, nextOpacity);
    visitNodes(node.children ?? [], visitor, nextOpacity);
  }
}

function isShadowNode(node: StrutNode) {
  const text = `${lower(node.name)} ${lower(node.role)}`;
  return text.includes("shadow") || text.includes("ground");
}

function isDetailNode(node: StrutNode) {
  const text = `${lower(node.name)} ${lower(node.role)}`;
  return ["rim", "edge", "depth", "side", "bezel", "glint", "highlight", "spark", "emblem", "mark", "detail", "accent", "part", "eye", "arm", "leg", "face"].some((word) => text.includes(word));
}

function visualStats(document: StrutDocument): VisualStats {
  let visibleNonShadow = 0;
  let visibleDetail = 0;
  const colors = new Set<string>();
  visitNodes(document.artboards[0]?.nodes ?? [], (node, opacity) => {
    if (!node.shape || node.shape.type === "none" || opacity <= 0.08) return;
    if (!isShadowNode(node)) visibleNonShadow += 1;
    if (isDetailNode(node)) visibleDetail += 1;
    const fill = node.style?.fill?.trim().toLowerCase();
    if (fill && fill !== "none") colors.add(fill);
  });
  const activeTracks = document.timelines.reduce((count, tl) => count + (tl.tracks ?? []).filter((t) => ["translation.x", "translation.y", "rotation", "rotation.x", "rotation.y", "scale", "scale.x", "scale.y", "opacity"].includes(t.property)).length, 0);
  return { visibleNonShadow, visibleDetail, visibleColors: colors.size, activeTracks };
}

function findPrimaryNode(nodes: StrutNode[]): { node: StrutNode; bounds: Bounds } | null {
  let best: { node: StrutNode; bounds: Bounds; area: number } | null = null;
  const walk = (node: StrutNode) => {
    const bounds = nodeBounds(node);
    const shape = node.shape ?? { type: "none" as const };
    if (bounds && shape.type !== "none" && !isShadowNode(node)) {
      const area = bounds.width * bounds.height;
      if (!best || area > best.area) best = { node, bounds, area };
    }
    for (const child of node.children ?? []) walk(child);
  };
  for (const node of nodes) walk(node);
  return best ? { node: best.node, bounds: best.bounds } : null;
}

function requestedStates(prompt: string, document: StrutDocument) {
  const lowerPrompt = prompt.toLowerCase();
  const states = new Set(document.state_machines[0]?.states ?? ["idle"]);
  states.add("idle");
  for (const state of STATE_KEYWORDS) if (lowerPrompt.includes(state)) states.add(state);
  return Array.from(states);
}

function ensureStateMachine(document: StrutDocument, states: string[]) {
  if (!document.state_machines.length) document.state_machines.push({ id: nextId("machine"), name: "Controller", states: [], inputs: [], transitions: [] });
  const machine = document.state_machines[0];
  const existing = new Set(machine.states ?? []);
  for (const state of states) existing.add(state);
  machine.states = Array.from(existing);
}

function hasTimeline(document: StrutDocument, state: string) {
  return document.timelines.some((tl) => tl.name === state);
}

function makeStateTimeline(state: string, target: string): Timeline | null {
  const baseTracks: NonNullable<Timeline["tracks"]> = [];
  if (state === "idle") {
    baseTracks.push(track(target, "translation.y", [[0, 0, "ease_in_out"], [800, -8, "ease_in_out"], [1600, 0, "ease_in_out"]]));
    return timeline("idle", 1600, true, baseTracks);
  }
  if (state === "hover") baseTracks.push(track(target, "translation.y", [[0, 0, "ease_out"], [420, -16, "ease_out"], [840, -10, "ease_in_out"]]));
  if (state === "press") baseTracks.push(track(target, "scale", [[0, 1, "ease_out"], [120, 0.94, "ease_in"], [260, 1.02, "ease_out"], [420, 1, "ease_out"]]));
  if (state === "success") baseTracks.push(track(target, "scale", [[0, 0.96, "ease_out"], [220, 1.1, "ease_out"], [560, 1, "ease_in_out"]]));
  if (state === "anticipation") baseTracks.push(track(target, "scale.x", [[0, 1, "ease_out"], [220, 1.12, "ease_in"], [520, 0.92, "ease_out"]]));
  if (state === "settle") baseTracks.push(track(target, "translation.y", [[0, -18, "ease_out"], [260, 8, "ease_in_out"], [680, 0, "ease_out"]]));
  if (state === "jump") baseTracks.push(track(target, "translation.y", [[0, 0, "ease_out"], [280, -70, "ease_out"], [700, 0, "ease_in"], [920, -8, "ease_out"], [1100, 0, "ease_out"]]));
  if (state === "wave") baseTracks.push(track(target, "rotation", [[0, -8, "ease_in_out"], [180, 10, "ease_in_out"], [360, -10, "ease_in_out"], [540, 8, "ease_in_out"], [720, 0, "ease_out"]]));
  if (state === "flip") {
    baseTracks.push(track(target, "translation.y", [[0, 0, "ease_out"], [420, -76, "ease_out"], [960, 0, "ease_out"]]));
    baseTracks.push(track(target, "scale.x", [[0, 1, "ease_in_out"], [240, 0.16, "ease_in_out"], [480, -1, "ease_in_out"], [720, -0.16, "ease_in_out"], [960, 1, "ease_out"]]));
    baseTracks.push(track(target, "rotation", [[0, 0, "linear"], [960, 360, "linear"]]));
  }
  if (!baseTracks.length) return null;
  return timeline(state, state === "flip" ? 960 : state === "jump" ? 1100 : 720, false, baseTracks);
}

export function engineIssuesV2(prompt: string, document: StrutDocument): string[] {
  const issues: string[] = [];
  const stats = visualStats(document);
  const lowerPrompt = prompt.toLowerCase();
  const needsPremium = ["premium", "2.5d", "3d", "mascot", "component", "flip", "jump", "wave", "hover", "press"].some((word) => lowerPrompt.includes(word));
  if (needsPremium && stats.visibleNonShadow < 6) issues.push(`too few visible non-shadow layers (${stats.visibleNonShadow}/6)`);
  if (needsPremium && stats.visibleDetail < 2) issues.push(`too few visible detail/rig layers (${stats.visibleDetail}/2)`);
  if (needsPremium && stats.visibleColors < 3) issues.push(`too few visible material colors (${stats.visibleColors}/3)`);
  if (needsPremium && stats.activeTracks < 8) issues.push(`too few active motion tracks (${stats.activeTracks}/8)`);
  for (const state of requestedStates(prompt, document)) {
    if (!document.state_machines[0]?.states?.includes(state)) issues.push(`missing state ${state}`);
  }
  return issues;
}

export function upgradeGeneratedDocumentV2(prompt: string, document: StrutDocument): StrutDocument {
  const next = cloneDocument(document);
  const artboard = next.artboards[0];
  if (!artboard) return next;
  const primary = findPrimaryNode(artboard.nodes);
  if (!primary) return next;
  const states = requestedStates(prompt, next);
  ensureStateMachine(next, states);
  for (const state of states) {
    if (hasTimeline(next, state)) continue;
    const generated = makeStateTimeline(state, primary.node.id);
    if (generated) next.timelines.push(generated);
  }
  return next;
}
