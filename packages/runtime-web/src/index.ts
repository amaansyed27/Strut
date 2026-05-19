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
  | { type: "text"; x: number; y: number; value: string; size: number };

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

export type StrutEasing = "linear" | "ease_in" | "ease_out" | "ease_in_out";

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

export type BotState = "idle" | "float" | "wave" | "blink" | "scan" | "celebrate" | "sleep";

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
  initialState: BotState = "idle",
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

export function createMinimalBotSvg(initialState: BotState = "idle"): SVGSVGElement {
  const document = minimalBotDocument();
  return renderStrutSvg(document, { initialState });
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
  if (transform.rotate) {
    transforms.push(`rotate(${transform.rotate})`);
  }
  if (transform.scale_x !== undefined || transform.scale_y !== undefined) {
    transforms.push(`scale(${transform.scale_x ?? 1} ${transform.scale_y ?? 1})`);
  }
  if (transforms.length) {
    element.setAttribute("transform", transforms.join(" "));
  }
}

function runtimeCss(document: StrutDocument, stateMachine: StrutStateMachine, reducedMotion: boolean) {
  const base = `
.strut-runtime-svg { width: 100%; height: 100%; display: block; overflow: visible; }
.strut-runtime-svg [data-node-id] { transform-box: fill-box; transform-origin: center; }
.strut-runtime-svg .strut-state-label { fill: #17142f; font: 800 22px system-ui, sans-serif; }
`;
  if (reducedMotion) {
    return base;
  }
  const timelines = document.timelines.filter((timeline) => timeline.tracks?.length);
  const animations = timelines
    .flatMap((timeline) => (timeline.tracks ?? []).map((track) => trackCss(timeline, track)))
    .filter(Boolean)
    .join("\n");
  const stateRules = stateMachine.states
    .flatMap((state) => timelinesForState(document, stateMachine, state).flatMap((timeline) => stateTimelineCss(state, timeline)))
    .join("\n");
  return `${base}\n${animations}\n${stateRules}`;
}

function trackCss(timeline: StrutTimeline, track: StrutTrack) {
  const numericKeyframes = track.keyframes
    .map((keyframe) => ({ ...keyframe, value: numericValue(keyframe.value) }))
    .filter((keyframe) => keyframe.value !== null) as Array<StrutKeyframe & { value: number }>;
  if (numericKeyframes.length < 2) {
    return "";
  }
  const frames = numericKeyframes
    .map((keyframe) => {
      const percent = Math.max(0, Math.min(100, (keyframe.time_ms / timeline.duration_ms) * 100));
      return `${percent}% { ${propertyCss(track.property, keyframe.value)} }`;
    })
    .join("\n");
  return `@keyframes ${animationName(timeline, track)} { ${frames} }`;
}

function stateTimelineCss(state: string, timeline: StrutTimeline) {
  return (timeline.tracks ?? [])
    .filter((track) => track.keyframes.length > 1 && track.keyframes.some((keyframe) => numericValue(keyframe.value) !== null))
    .map((track) => {
      const easing = cssEasing(track.keyframes[0]?.easing ?? "linear");
      return `.strut-runtime-svg.state-${cssIdent(state)} [data-node-id="${track.target}"] { animation: ${animationName(timeline, track)} ${timeline.duration_ms}ms ${easing} infinite; }`;
    });
}

function propertyCss(property: string, value: number) {
  if (property === "translation.y") return `transform: translateY(${value}px);`;
  if (property === "translation.x") return `transform: translateX(${value}px);`;
  if (property === "rotation") return `transform: rotate(${value}deg);`;
  if (property === "scale") return `transform: scale(${value});`;
  if (property === "scale.y") return `transform: scaleY(${value});`;
  if (property === "scale.x") return `transform: scaleX(${value});`;
  return `opacity: ${value};`;
}

function timelinesForState(document: StrutDocument, stateMachine: StrutStateMachine, state: string) {
  const transitionTimelines = new Set(
    (stateMachine.transitions ?? [])
      .filter((transition) => transition.to === state)
      .map((transition) => transition.timeline),
  );
  const timelineNames = new Set([state, ...transitionTimelines]);
  if (state === "float") timelineNames.add("idle_float");
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

function animationName(timeline: StrutTimeline, track: StrutTrack) {
  return `strut-${cssIdent(timeline.name)}-${cssIdent(track.target)}-${cssIdent(track.property)}`;
}

function cssEasing(easing: StrutEasing) {
  if (easing === "ease_in") return "ease-in";
  if (easing === "ease_out") return "ease-out";
  if (easing === "ease_in_out") return "ease-in-out";
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

function minimalBotDocument(): StrutDocument {
  return {
    id: "minimal-bot-runtime-fallback",
    name: "Minimal Bot",
    artboards: [
      {
        id: "minimal-bot-artboard",
        name: "MinimalBot",
        width: 960,
        height: 540,
        nodes: [],
      },
    ],
    timelines: [],
    state_machines: [
      {
        id: "minimal-bot-machine",
        name: "BotMoods",
        states: ["idle", "float", "wave", "blink", "scan", "celebrate", "sleep"],
      },
    ],
    bindings: [],
    events: [],
  };
}
