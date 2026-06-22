export type CssStrutValue = string | number | boolean | null | undefined;
export type CssStrutStyle = Record<string, CssStrutValue>;

export type CssStrutTransform = {
  x?: number;
  y?: number;
  z?: number;
  rotate?: number;
  rotateX?: number;
  rotateY?: number;
  rotateZ?: number;
  scale?: number;
  scaleX?: number;
  scaleY?: number;
  scaleZ?: number;
};

export type CssStrutLayerKind = "group" | "plane" | "disc" | "ring" | "text" | "image" | "sprite" | "shadow" | "glow";

export type CssStrutLayer = {
  id: string;
  name: string;
  kind: CssStrutLayerKind;
  x?: number;
  y?: number;
  z?: number;
  width?: number;
  height?: number;
  text?: string;
  src?: string;
  frameWidth?: number;
  frameHeight?: number;
  columns?: number;
  rows?: number;
  transform?: CssStrutTransform;
  style?: CssStrutStyle;
  vars?: Record<string, CssStrutValue>;
  children?: CssStrutLayer[];
};

export type CssStrutKeyframe = {
  at: number;
  transform?: CssStrutTransform;
  style?: CssStrutStyle;
  vars?: Record<string, CssStrutValue>;
};

export type CssStrutTimeline = {
  id: string;
  name: string;
  state: string;
  durationMs: number;
  easing?: string;
  delayMs?: number;
  loops?: boolean;
  tracks: Array<{ target: string; keyframes: CssStrutKeyframe[] }>;
};

export type CssStrutDocument = {
  format: "strut-css";
  version: "1";
  name: string;
  artboard: { width: number; height: number; background?: string; perspective?: number };
  states: string[];
  initialState?: string;
  layers: CssStrutLayer[];
  timelines: CssStrutTimeline[];
};

export type CssStrutMountOptions = { initialState?: string; reducedMotion?: boolean };
export type MountedCssStrut = { document: CssStrutDocument; element: HTMLElement; setState(state: string): void; destroy(): void };

const cssIdent = (value: string) => value.replace(/[^a-zA-Z0-9_-]/g, "-");
const cssProp = (value: string) => value.replace(/[A-Z]/g, (match) => `-${match.toLowerCase()}`);

function cssValue(value: CssStrutValue): string | null {
  if (value === null || value === undefined || value === false) return null;
  if (value === true) return "1";
  return String(value);
}

function cssTransform(transform: CssStrutTransform | undefined): string {
  const t = transform ?? {};
  const scale = t.scale ?? 1;
  return [
    `translate3d(${t.x ?? 0}px, ${t.y ?? 0}px, ${t.z ?? 0}px)`,
    `rotateX(${t.rotateX ?? 0}deg)`,
    `rotateY(${t.rotateY ?? 0}deg)`,
    `rotateZ(${t.rotateZ ?? t.rotate ?? 0}deg)`,
    `scale3d(${t.scaleX ?? scale}, ${t.scaleY ?? scale}, ${t.scaleZ ?? 1})`,
  ].join(" ");
}

function applyStyle(element: HTMLElement, style: CssStrutStyle | undefined) {
  if (!style) return;
  for (const [key, raw] of Object.entries(style)) {
    const value = cssValue(raw);
    if (value !== null) element.style.setProperty(cssProp(key), value);
  }
}

function applyLayerKind(element: HTMLElement, layer: CssStrutLayer) {
  if (["disc", "ring", "shadow", "glow"].includes(layer.kind)) element.style.borderRadius = "999px";
  if (layer.kind === "ring") element.style.background = "transparent";
  if (layer.kind === "shadow") element.style.filter = "blur(var(--strut-shadow-blur, 0px))";
  if (layer.kind === "image" || layer.kind === "sprite") {
    element.style.backgroundImage = layer.src ? `url(${layer.src})` : "none";
    element.style.backgroundRepeat = "no-repeat";
    element.style.backgroundSize = layer.kind === "sprite" && layer.frameWidth && layer.frameHeight && layer.columns && layer.rows
      ? `${layer.frameWidth * layer.columns}px ${layer.frameHeight * layer.rows}px`
      : "cover";
  }
  if (layer.kind === "sprite") {
    element.style.backgroundPosition = "var(--strut-sprite-x, 0px) var(--strut-sprite-y, 0px)";
    element.style.overflow = "hidden";
  }
}

function renderLayer(layer: CssStrutLayer): HTMLElement {
  const element = document.createElement("div");
  element.dataset.strutLayer = layer.id;
  element.dataset.strutLayerName = layer.name;
  element.className = `strut-css-layer strut-css-${cssIdent(layer.kind)} layer-${cssIdent(layer.name)}`;
  element.style.position = "absolute";
  element.style.left = `${layer.x ?? 0}px`;
  element.style.top = `${layer.y ?? 0}px`;
  element.style.width = `${layer.width ?? 0}px`;
  element.style.height = `${layer.height ?? 0}px`;
  element.style.transform = cssTransform({ x: 0, y: 0, z: layer.z ?? 0, ...(layer.transform ?? {}) });
  element.style.transformStyle = "preserve-3d";
  element.style.transformOrigin = "50% 50%";
  element.style.willChange = "transform, opacity, filter, background-position";
  applyLayerKind(element, layer);
  applyStyle(element, layer.style);
  if (layer.kind === "text") element.textContent = layer.text ?? layer.name;
  for (const child of layer.children ?? []) element.append(renderLayer(child));
  return element;
}

function frameCss(frame: CssStrutKeyframe): string {
  const rules: string[] = [];
  if (frame.transform) rules.push(`transform: ${cssTransform(frame.transform)};`);
  if (frame.style) for (const [key, raw] of Object.entries(frame.style)) {
    const value = cssValue(raw);
    if (value !== null) rules.push(`${cssProp(key)}: ${value};`);
  }
  if (frame.vars) for (const [key, raw] of Object.entries(frame.vars)) {
    const value = cssValue(raw);
    if (value !== null) rules.push(`${key.startsWith("--") ? key : `--${key}`}: ${value};`);
  }
  return rules.join(" ");
}

function animationName(documentName: string, timeline: CssStrutTimeline, target: string): string {
  return `strut-${cssIdent(documentName)}-${cssIdent(timeline.id)}-${cssIdent(target)}`;
}

export function cssStrutStyles(model: CssStrutDocument, reducedMotion = false): string {
  const base = `.strut-css-stage{position:relative;width:100%;height:100%;overflow:hidden;contain:layout paint style;perspective:${model.artboard.perspective ?? 1000}px;transform-style:preserve-3d;background:${model.artboard.background ?? "transparent"}}.strut-css-stage,.strut-css-stage *{box-sizing:border-box}.strut-css-layer{pointer-events:none;transform-style:preserve-3d;backface-visibility:hidden}`;
  if (reducedMotion) return base;
  const blocks = [base];
  for (const timeline of model.timelines) for (const track of timeline.tracks) {
    const name = animationName(model.name, timeline, track.target);
    const frames = track.keyframes.map((frame) => `${Math.max(0, Math.min(100, frame.at))}%{${frameCss(frame)}}`).join("\n");
    blocks.push(`@keyframes ${name}{${frames}}`);
    blocks.push(`.strut-css-stage.state-${cssIdent(timeline.state)} [data-strut-layer="${track.target}"]{animation:${name} ${timeline.durationMs}ms ${timeline.easing ?? "cubic-bezier(.2,.8,.2,1)"} ${timeline.delayMs ?? 0}ms ${timeline.loops ? "infinite" : "1 both"}}`);
  }
  return blocks.join("\n");
}

export function renderCssStrut(model: CssStrutDocument, options: CssStrutMountOptions = {}): HTMLElement {
  const stage = document.createElement("div");
  const state = options.initialState ?? model.initialState ?? model.states[0] ?? "idle";
  stage.className = `strut-css-stage state-${cssIdent(state)}`;
  stage.dataset.strut = "css-runtime";
  stage.dataset.state = state;
  stage.style.aspectRatio = `${model.artboard.width} / ${model.artboard.height}`;
  const style = document.createElement("style");
  style.textContent = cssStrutStyles(model, options.reducedMotion ?? false);
  stage.append(style);
  for (const layer of model.layers) stage.append(renderLayer(layer));
  return stage;
}

export function mountCssStrut(target: HTMLElement, model: CssStrutDocument, options: CssStrutMountOptions = {}): MountedCssStrut {
  const element = renderCssStrut(model, options);
  target.replaceChildren(element);
  return {
    document: model,
    element,
    setState(state: string) {
      if (!model.states.includes(state)) throw new Error(`unknown css strut state: ${state}`);
      element.classList.forEach((className) => { if (className.startsWith("state-")) element.classList.remove(className); });
      element.classList.add(`state-${cssIdent(state)}`);
      element.dataset.state = state;
      for (const animated of Array.from(element.querySelectorAll<HTMLElement>("[data-strut-layer]"))) {
        animated.style.animation = "none";
        void animated.offsetWidth;
        animated.style.animation = "";
      }
    },
    destroy() { target.replaceChildren(); },
  };
}
