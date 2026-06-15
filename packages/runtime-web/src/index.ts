import JSZip from "jszip";

const SVG_NS = "http://www.w3.org/2000/svg";

export type StrutManifest = {
  format: "strut";
  schemaVersion: string;
  document: string;
  createdBy: string;
  minimumRuntime: string;
  features?: string[];
};

export type StrutTransform = {
  translate_x?: number;
  translate_y?: number;
  rotate?: number;
  rotate_x?: number;
  rotate_y?: number;
  scale_x?: number;
  scale_y?: number;
};

export type StrutStyle = {
  fill?: string | null;
  stroke?: string | null;
  stroke_width?: number;
  opacity?: number;
  linecap?: string | null;
  linejoin?: string | null;
};

export type StrutShape =
  | { type: "none" }
  | { type: "rect"; x: number; y: number; width: number; height: number; rx: number }
  | { type: "ellipse"; cx: number; cy: number; rx: number; ry: number }
  | { type: "path"; d: string }
  | { type: "text"; x: number; y: number; value: string; size: number }
  | { type: "sprite"; url: string; frame_width: number; frame_height: number; columns: number; rows: number };

export type StrutNode = {
  id: string;
  name: string;
  kind: string;
  transform?: StrutTransform;
  style?: StrutStyle;
  shape?: StrutShape;
  children?: StrutNode[];
};

export type StrutArtboard = {
  id: string;
  name: string;
  width: number;
  height: number;
  nodes: StrutNode[];
};

export type StrutEasing = "linear" | "ease_in" | "ease_out" | "ease_in_out" | "steps";

export type StrutPropertyValue =
  | { type: "number"; value: number }
  | { type: "text"; value: string }
  | { type: "color"; value: string }
  | { type: "point"; value: { x: number; y: number } };

export type StrutKeyframe = {
  time_ms: number;
  value: StrutPropertyValue;
  easing: StrutEasing;
};

export type StrutTrack = {
  target: string;
  property: string;
  keyframes: StrutKeyframe[];
};

export type StrutTimeline = {
  id: string;
  name: string;
  duration_ms: number;
  tracks?: StrutTrack[];
  loops?: boolean;
};

export type StrutInput = {
  name: string;
  kind: "boolean" | "number" | "trigger" | "enum";
};

export type StrutTransition = {
  from: string;
  to: string;
  on: string;
  timeline: string;
};

export type StrutStateMachine = {
  id: string;
  name: string;
  inputs?: StrutInput[];
  states: string[];
  transitions?: StrutTransition[];
};

export type StrutBinding = {
  name: string;
  target?: string;
  property?: string;
};

export type StrutEvent = {
  name: string;
  description?: string;
};

export type StrutDocument = {
  id: string;
  name: string;
  artboards: StrutArtboard[];
  timelines: StrutTimeline[];
  state_machines: StrutStateMachine[];
  bindings: StrutBinding[];
  events: StrutEvent[];
};

export type StrutPackage = {
  manifest: StrutManifest;
  document: StrutDocument;
};


export type MountOptions = {
  artboard?: string;
  stateMachine?: string;
  initialState?: string;
  reducedMotion?: boolean;
};

export type MountedStrut = {
  document: StrutDocument;
  svg: SVGSVGElement;
  setState(state: string): void;
  setInput(name: string, value: boolean | number | string): void;
  fireTrigger(name: string): void;
  setBinding(name: string, value: string): void;
  on(eventName: string, handler: (event: CustomEvent) => void): () => void;
  destroy(): void;
};

export async function loadStrutUrl(url: string): Promise<StrutPackage> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`failed to load Strut package: ${response.status} ${response.statusText}`);
  }

  return loadStrutPackage(await response.arrayBuffer());
}

export async function loadStrutPackage(input: ArrayBuffer | Blob): Promise<StrutPackage> {
  const zip = await JSZip.loadAsync(input);
  const manifestFile = zip.file("manifest.json");
  if (!manifestFile) {
    throw new Error("missing manifest.json");
  }

  const manifest = JSON.parse(await manifestFile.async("string")) as StrutManifest;
  validateManifest(manifest);

  const documentFile = zip.file(manifest.document);
  if (!documentFile) {
    throw new Error(`missing ${manifest.document}`);
  }

  const document = JSON.parse(await documentFile.async("string")) as StrutDocument;
  validateDocument(document);

  return { manifest, document };
}

export function mountStrut(
  target: HTMLElement,
  document: StrutDocument,
  options: MountOptions = {},
): MountedStrut {
  validateDocument(document);
  const artboard = selectArtboard(document, options.artboard);
  const stateMachine = selectStateMachine(document, options.stateMachine);
  const initialState = options.initialState ?? stateMachine.states[0] ?? "idle";
  const svg = renderStrutSvg(document, {
    artboard: artboard.name,
    stateMachine: stateMachine.name,
    state: initialState,
    reducedMotion: options.reducedMotion,
  });

  target.replaceChildren(svg);

  const mounted: MountedStrut = {
    document,
    svg,
    setState(state: string) {
      assertKnownState(stateMachine, state);
      svg.dataset.state = state;
      svg.classList.forEach((className) => {
        if (className.startsWith("state-")) {
          svg.classList.remove(className);
        }
      });
      svg.classList.add(`state-${cssIdent(state)}`);
      updateStateLabel(svg, state);
      emitStateEvents(svg, document, stateMachine, state);
    },
    setInput(name, value) {
      const input = stateMachine.inputs?.find((candidate) => candidate.name === name);
      if (!input) {
        throw new Error(`unknown input: ${name}`);
      }
      svg.dataset[`input${pascalCase(name)}`] = String(value);
    },
    fireTrigger(name) {
      const input = stateMachine.inputs?.find((candidate) => candidate.name === name);
      if (!input || input.kind !== "trigger") {
        throw new Error(`unknown trigger: ${name}`);
      }
      svg.dispatchEvent(new CustomEvent("strut:trigger", { detail: { name } }));
    },
    setBinding(name, value) {
      const binding = document.bindings.find((candidate) => candidate.name === name);
      if (!binding?.target || !binding.property) {
        throw new Error(`unknown binding: ${name}`);
      }
      const element = svg.querySelector<SVGElement>(`[data-node-id="${cssEscape(binding.target)}"]`);
      if (!element) {
        throw new Error(`binding target missing: ${binding.target}`);
      }
      applyBinding(element, binding.property, value);
    },
    on(eventName, handler) {
      const type = `strut:${eventName}`;
      svg.addEventListener(type, handler as EventListener);
      return () => svg.removeEventListener(type, handler as EventListener);
    },
    destroy() {
      target.replaceChildren();
    },
  };

  mounted.setState(initialState);
  return mounted;
}

export function mountMinimalBot(
  target: HTMLElement,
  document: StrutDocument,
  initialState: string = "idle",
): MountedStrut {
  return mountStrut(target, document, { initialState });
}

export function renderStrutSvg(
  document: StrutDocument,
  options: MountOptions & { state?: string } = {},
): SVGSVGElement {
  const artboard = selectArtboard(document, options.artboard);
  const stateMachine = selectStateMachine(document, options.stateMachine);
  const state = options.state ?? options.initialState ?? stateMachine.states[0] ?? "idle";
  const svg = createSvg("svg");
  svg.classList.add("strut-runtime-svg", `state-${cssIdent(state)}`);
  svg.dataset.strut = "runtime";
  svg.dataset.strutBot = "";
  svg.dataset.state = state;
  svg.dataset.artboard = artboard.name;
  svg.dataset.stateMachine = stateMachine.name;
  svg.setAttribute("viewBox", `0 0 ${artboard.width} ${artboard.height}`);
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", document.name);

  const style = createSvg("style");
  style.textContent = runtimeCss(document, stateMachine, options.reducedMotion ?? false);
  svg.append(style);

  for (const node of artboard.nodes) {
    svg.append(renderNode(node));
  }

  const label = createSvg("text");
  label.dataset.stateLabel = "";
  label.classList.add("strut-state-label");
  label.setAttribute("x", String(artboard.width / 2));
  label.setAttribute("y", String(artboard.height - 34));
  label.setAttribute("text-anchor", "middle");
  label.textContent = titleCase(state);
  svg.append(label);

  return svg;
}

function renderNode(node: StrutNode): SVGElement {
  const shape = node.shape ?? { type: "none" };
  const element = shapeElement(node, shape);
  element.dataset.nodeId = node.id;
  element.dataset.nodeName = node.name;
  element.classList.add("strut-node", `strut-${cssIdent(node.kind)}`, `node-${cssIdent(node.name)}`);
  applyTransform(element, node.transform);
  applyStyle(element, node.style);
  for (const child of node.children ?? []) {
    element.append(renderNode(child));
  }
  return element;
}

function shapeElement(node: StrutNode, shape: StrutShape): SVGElement {
  if (node.kind === "group" || shape.type === "none") {
    return createSvg("g");
  }
  if (shape.type === "rect") {
    const rect = createSvg("rect");
    setAttributes(rect, {
      x: shape.x,
      y: shape.y,
      width: shape.width,
      height: shape.height,
      rx: shape.rx,
    });
    return rect;
  }
  if (shape.type === "ellipse") {
    const ellipse = createSvg("ellipse");
    setAttributes(ellipse, { cx: shape.cx, cy: shape.cy, rx: shape.rx, ry: shape.ry });
    return ellipse;
  }
  if (shape.type === "path") {
    const path = createSvg("path");
    path.setAttribute("d", shape.d);
    return path;
  }
  if (shape.type === "text") {
    const text = createSvg("text");
    setAttributes(text, { x: shape.x, y: shape.y });
    text.setAttribute("font-size", String(shape.size));
    text.textContent = shape.value;
    return text;
  }
  if (shape.type === "sprite") {
    const wrapper = createSvg("svg");
    setAttributes(wrapper, { width: shape.frame_width, height: shape.frame_height });
    wrapper.style.overflow = "hidden";
    const img = createSvg("image");
    img.setAttribute("href", shape.url);
    img.setAttribute("width", String(shape.frame_width * shape.columns));
    img.setAttribute("height", String(shape.frame_height * shape.rows));
    img.classList.add("strut-sprite-image");
    wrapper.append(img);
    return wrapper;
  }
  return createSvg("g");
}

function applyStyle(element: SVGElement, style: StrutStyle | undefined) {
  if (!style) {
    return;
  }
  if (style.fill) element.setAttribute("fill", style.fill);
  if (style.stroke) element.setAttribute("stroke", style.stroke);
  if (style.stroke_width !== undefined) element.setAttribute("stroke-width", String(style.stroke_width));
  if (style.opacity !== undefined) element.setAttribute("opacity", String(style.opacity));
  if (style.linecap) element.setAttribute("stroke-linecap", style.linecap);
  if (style.linejoin) element.setAttribute("stroke-linejoin", style.linejoin);
}

function applyTransform(element: SVGElement, transform: StrutTransform | undefined) {
  if (!transform) {
    return;
  }
  const transforms = [];
  if (transform.translate_x || transform.translate_y) {
    transforms.push(`translate(${transform.translate_x ?? 0} ${transform.translate_y ?? 0})`);
  }
  if (transform.rotate || transform.rotate_x || transform.rotate_y) {
    if (transform.rotate) transforms.push(`rotateZ(${transform.rotate}deg)`);
    if (transform.rotate_x) transforms.push(`rotateX(${transform.rotate_x}deg)`);
    if (transform.rotate_y) transforms.push(`rotateY(${transform.rotate_y}deg)`);
  }
  if (transform.scale_x !== undefined || transform.scale_y !== undefined) {
    transforms.push(`scale(${transform.scale_x ?? 1}, ${transform.scale_y ?? 1})`);
  }
  if (transforms.length) {
    element.style.transform = transforms.join(" ");
  }
}

function runtimeCss(document: StrutDocument, stateMachine: StrutStateMachine, reducedMotion: boolean) {
const base = `
.strut-runtime-svg { width: 100%; height: 100%; display: block; overflow: visible; perspective: 800px; transform-style: preserve-3d; }
.strut-runtime-svg [data-node-id] { transform-box: fill-box; transform-origin: center; transform-style: preserve-3d; }
.strut-runtime-svg .strut-state-label { fill: #17142f; font: 800 22px system-ui, sans-serif; }
.strut-runtime-svg .strut-sprite-image { transform: translate(var(--sprite-x, 0), var(--sprite-y, 0)); }
`;
  if (reducedMotion) {
    return base;
  }
  const transforms = nodeTransformMap(document);
  const shapes = nodeShapeMap(document);
  const timelines = document.timelines.filter((timeline) => timeline.tracks?.length);
  const animations = timelines
    .map((timeline) => timelineAnimationCss(timeline, transforms, shapes))
    .filter(Boolean)
    .join("\n");
  const stateRules = stateMachine.states
    .flatMap((state) => timelinesForState(document, stateMachine, state).flatMap((timeline) => stateTimelineCss(state, timeline, transforms, shapes)))
    .join("\n");
  return `${base}\n${animations}\n${stateRules}`;
}

function timelineAnimationCss(timeline: StrutTimeline, transforms: Map<string, StrutTransform>, shapes: Map<string, StrutShape>) {
  return Array.from(timelineTrackGroups(timeline).entries())
    .flatMap(([target, tracks]) => [
      transformTracksCss(timeline, target, tracks.filter((track) => isTransformProperty(track.property)), transforms.get(target)),
      ...tracks.filter((track) => isScalarProperty(track.property)).map((track) => scalarTrackCss(timeline, track, shapes.get(target))),
    ])
    .filter(Boolean)
    .join("\n");
}

function transformTracksCss(
  timeline: StrutTimeline,
  target: string,
  tracks: StrutTrack[],
  baseTransform: StrutTransform | undefined,
) {
  if (!tracks.length) {
    return "";
  }
  const times = sortedTimelineTimes(timeline, tracks);
  const frames = times
    .map((time) => {
      const percent = Math.max(0, Math.min(100, (time / timeline.duration_ms) * 100));
      const base = normalizeTransform(baseTransform);
      const tx = base.translate_x + trackValue(tracks, "translation.x", time, 0);
      const ty = base.translate_y + trackValue(tracks, "translation.y", time, 0);
      const rotate = base.rotate + trackValue(tracks, "rotation", time, 0);
      const rotate_x = base.rotate_x + trackValue(tracks, "rotation.x", time, 0);
      const rotate_y = base.rotate_y + trackValue(tracks, "rotation.y", time, 0);
      const scale = trackValue(tracks, "scale", time, 1);
      const sx = base.scale_x * scale * trackValue(tracks, "scale.x", time, 1);
      const sy = base.scale_y * scale * trackValue(tracks, "scale.y", time, 1);
      return `${percent}% { transform: translate(${round(tx)}px, ${round(ty)}px) rotateZ(${round(rotate)}deg) rotateX(${round(rotate_x)}deg) rotateY(${round(rotate_y)}deg) scale(${round(sx)}, ${round(sy)}); }`;
    })
    .join("\n");
  return `@keyframes ${transformAnimationName(timeline, target)} { ${frames} }`;
}

function scalarTrackCss(timeline: StrutTimeline, track: StrutTrack, shape: StrutShape | undefined) {
  const numericKeyframes = numericTrackKeyframes(track);
  if (numericKeyframes.length < 2) {
    return "";
  }
  const frames = numericKeyframes
    .map((keyframe) => {
      const percent = Math.max(0, Math.min(100, (keyframe.time_ms / timeline.duration_ms) * 100));
      if (track.property === "frame" && shape?.type === "sprite") {
        const frame = Math.round(keyframe.value);
        const x = (frame % shape.columns) * shape.frame_width;
        const y = Math.floor(frame / shape.columns) * shape.frame_height;
        return `${percent}% { --sprite-x: -${x}px; --sprite-y: -${y}px; }`;
      }
      return `${percent}% { ${track.property}: ${round(keyframe.value)}; }`;
    })
    .join("\n");
  return `@keyframes ${scalarAnimationName(timeline, track)} { ${frames} }`;
}

function stateTimelineCss(state: string, timeline: StrutTimeline, transforms: Map<string, StrutTransform>, shapes: Map<string, StrutShape>) {
  const iteration = timeline.loops ? "infinite" : "1 both";
  return Array.from(timelineTrackGroups(timeline).entries())
    .map(([target, tracks]) => {
      const animations = [
        tracks.some((track) => isTransformProperty(track.property))
          ? `${transformAnimationName(timeline, target)} ${timeline.duration_ms}ms ${groupEasing(tracks)} ${iteration}`
          : "",
        ...tracks
          .filter((track) => isScalarProperty(track.property))
          .map((track) => {
            const easing = cssEasing(track.keyframes[0]?.easing ?? "linear");
            // If the easing is a step easing, we format it as `steps(...)` instead of `linear`.
            // But actually CSS custom properties don't interpolate smoothly anyway.
            // If the user specifies 'steps', we just pass it to the generated CSS.
            return `${scalarAnimationName(timeline, track)} ${timeline.duration_ms}ms ${easing} ${iteration}`;
          }),
      ].filter(Boolean);
      if (!animations.length) {
        return "";
      }
      const base = transforms.get(target);
      const baseRule = tracks.some((track) => isTransformProperty(track.property))
        ? ` transform: ${transformCss(normalizeTransform(base))};`
        : "";
      return `.strut-runtime-svg.state-${cssIdent(state)} [data-node-id="${target}"] {${baseRule} animation: ${animations.join(", ")}; }`;
    })
    .filter(Boolean);
}

function timelineTrackGroups(timeline: StrutTimeline) {
  const groups = new Map<string, StrutTrack[]>();
  for (const track of timeline.tracks ?? []) {
    if (!hasNumericMotion(track) || (!isTransformProperty(track.property) && !isScalarProperty(track.property))) {
      continue;
    }
    groups.set(track.target, [...(groups.get(track.target) ?? []), track]);
  }
  return groups;
}

function sortedTimelineTimes(timeline: StrutTimeline, tracks: StrutTrack[]) {
  return Array.from(
    new Set([
      0,
      timeline.duration_ms,
      ...tracks.flatMap((track) => numericTrackKeyframes(track).map((keyframe) => keyframe.time_ms)),
    ]),
  ).sort((a, b) => a - b);
}

function trackValue(tracks: StrutTrack[], property: string, time: number, fallback: number) {
  const track = tracks.find((candidate) => candidate.property === property);
  return track ? interpolatedTrackValue(track, time, fallback) : fallback;
}

function interpolatedTrackValue(track: StrutTrack, time: number, fallback: number) {
  const keyframes = numericTrackKeyframes(track).sort((a, b) => a.time_ms - b.time_ms);
  if (!keyframes.length) {
    return fallback;
  }
  if (time <= keyframes[0].time_ms) {
    return keyframes[0].value;
  }
  const last = keyframes[keyframes.length - 1];
  if (time >= last.time_ms) {
    return last.value;
  }
  for (let index = 0; index < keyframes.length - 1; index += 1) {
    const left = keyframes[index];
    const right = keyframes[index + 1];
    if (time >= left.time_ms && time <= right.time_ms) {
      const span = Math.max(1, right.time_ms - left.time_ms);
      const progress = (time - left.time_ms) / span;
      return left.value + (right.value - left.value) * progress;
    }
  }
  return fallback;
}

function numericTrackKeyframes(track: StrutTrack) {
  return track.keyframes
    .map((keyframe) => ({ ...keyframe, value: numericValue(keyframe.value) }))
    .filter((keyframe) => keyframe.value !== null) as Array<StrutKeyframe & { value: number }>;
}

function hasNumericMotion(track: StrutTrack) {
  return numericTrackKeyframes(track).length > 1;
}

function isTransformProperty(property: string) {
  return ["translation.x", "translation.y", "rotation", "scale", "scale.x", "scale.y"].includes(property);
}

function isScalarProperty(property: string) {
  return property === "opacity" || property === "frame";
}

function groupEasing(tracks: StrutTrack[]) {
  return cssEasing(tracks[0]?.keyframes[0]?.easing ?? "linear");
}

function nodeTransformMap(document: StrutDocument) {
  const transforms = new Map<string, StrutTransform>();
  const visit = (node: StrutNode) => {
    transforms.set(node.id, node.transform ?? {});
    for (const child of node.children ?? []) {
      visit(child);
    }
  };
  for (const artboard of document.artboards) {
    for (const node of artboard.nodes) {
      visit(node);
    }
  }
  return transforms;
}

function nodeShapeMap(document: StrutDocument) {
  const shapes = new Map<string, StrutShape>();
  const visit = (node: StrutNode) => {
    shapes.set(node.id, node.shape ?? { type: "none" });
    for (const child of node.children ?? []) {
      visit(child);
    }
  };
  for (const artboard of document.artboards) {
    for (const node of artboard.nodes) {
      visit(node);
    }
  }
  return shapes;
}

function normalizeTransform(transform: StrutTransform | undefined): Required<StrutTransform> {
  return {
    translate_x: transform?.translate_x ?? 0,
    translate_y: transform?.translate_y ?? 0,
    rotate: transform?.rotate ?? 0,
    rotate_x: transform?.rotate_x ?? 0,
    rotate_y: transform?.rotate_y ?? 0,
    scale_x: transform?.scale_x ?? 1,
    scale_y: transform?.scale_y ?? 1,
  };
}

function transformCss(transform: Required<StrutTransform>) {
  return `translate(${round(transform.translate_x)}px, ${round(transform.translate_y)}px) rotate(${round(transform.rotate)}deg) scale(${round(transform.scale_x)}, ${round(transform.scale_y)})`;
}

function round(value: number) {
  return Number(value.toFixed(4));
}

function timelinesForState(document: StrutDocument, stateMachine: StrutStateMachine, state: string) {
  const transitionTimelines = new Set(
    (stateMachine.transitions ?? [])
      .filter((transition) => transition.to === state)
      .map((transition) => transition.timeline),
  );
  const timelineNames = new Set([state, ...transitionTimelines]);
  return document.timelines.filter((timeline) => timelineNames.has(timeline.name));
}

function emitStateEvents(svg: SVGSVGElement, document: StrutDocument, stateMachine: StrutStateMachine, state: string) {
  const transition = (stateMachine.transitions ?? []).find((candidate) => candidate.to === state);
  const event = document.events.find((candidate) => candidate.name === `${state}_started`) ?? document.events[0];
  if (event && transition) {
    svg.dispatchEvent(new CustomEvent(`strut:${event.name}`, { detail: { state, transition } }));
  }
}

function updateStateLabel(svg: SVGSVGElement, state: string) {
  const label = svg.querySelector("[data-state-label]");
  if (label) {
    label.textContent = titleCase(state);
  }
}

function applyBinding(element: SVGElement, property: string, value: string) {
  if (property === "text") {
    element.textContent = value;
  } else if (property === "fill" || property === "stroke" || property === "opacity") {
    element.setAttribute(property, value);
  } else {
    throw new Error(`unsupported binding property: ${property}`);
  }
}

function selectArtboard(document: StrutDocument, name: string | undefined) {
  const artboard = name
    ? document.artboards.find((candidate) => candidate.name === name || candidate.id === name)
    : document.artboards[0];
  if (!artboard) {
    throw new Error(name ? `unknown artboard: ${name}` : "document must contain at least one artboard");
  }
  return artboard;
}

function selectStateMachine(document: StrutDocument, name: string | undefined) {
  const stateMachine = name
    ? document.state_machines.find((candidate) => candidate.name === name || candidate.id === name)
    : document.state_machines[0];
  if (!stateMachine) {
    throw new Error(name ? `unknown state machine: ${name}` : "document must contain at least one state machine");
  }
  return stateMachine;
}

function assertKnownState(stateMachine: StrutStateMachine, state: string) {
  if (!stateMachine.states.includes(state)) {
    throw new Error(`unknown state: ${state}`);
  }
}

function validateManifest(manifest: StrutManifest) {
  if (manifest.format !== "strut") {
    throw new Error("manifest format must be strut");
  }
  if (!manifest.schemaVersion.startsWith("0.1") && !manifest.schemaVersion.startsWith("0.2")) {
    throw new Error(`unsupported schema version: ${manifest.schemaVersion}`);
  }
  if (manifest.document.includes("..") || manifest.document.startsWith("/")) {
    throw new Error("manifest document path is unsafe");
  }
}

function validateDocument(document: StrutDocument) {
  if (!document.artboards.length) {
    throw new Error("document must contain at least one artboard");
  }
  if (!document.state_machines.length) {
    throw new Error("document must contain at least one state machine");
  }
}

function createSvg<K extends keyof SVGElementTagNameMap>(tag: K): SVGElementTagNameMap[K] {
  return document.createElementNS(SVG_NS, tag);
}

function setAttributes(element: SVGElement, attributes: Record<string, string | number>) {
  for (const [key, value] of Object.entries(attributes)) {
    element.setAttribute(key, String(value));
  }
}

function numericValue(value: StrutPropertyValue): number | null {
  return value.type === "number" ? value.value : null;
}

function transformAnimationName(timeline: StrutTimeline, target: string) {
  return `strut-${cssIdent(timeline.name)}-${cssIdent(target)}-transform`;
}

function scalarAnimationName(timeline: StrutTimeline, track: StrutTrack) {
  return `strut-${cssIdent(timeline.name)}-${cssIdent(track.target)}-${cssIdent(track.property)}`;
}

function cssEasing(easing: StrutEasing) {
  if (easing === "ease_in") return "ease-in";
  if (easing === "ease_out") return "ease-out";
  if (easing === "ease_in_out") return "ease-in-out";
  if (easing === "steps") return "step-end";
  return "linear";
}

function cssIdent(value: string) {
  return value.replace(/[^a-zA-Z0-9_-]/g, "-");
}

function cssEscape(value: string) {
  return value.replace(/"/g, '\\"');
}

function pascalCase(value: string) {
  return value
    .split(/[_\s-]+/)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

function titleCase(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}


