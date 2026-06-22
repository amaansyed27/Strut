import "./cssPreviewRuntime.css";

const INSTALLED_FLAG = "__strutCssPreviewRuntimeInstalled";
const OVERLAY_CLASS = "css-character-preview";
const HIDDEN_ATTR = "data-css-preview-hidden";

type RuntimeWindow = Window & typeof globalThis & Record<string, unknown>;

type LayerSnapshot = {
  id: string;
  name: string;
  semantic: string;
  shape: string;
  left: number;
  top: number;
  width: number;
  height: number;
  fill: string;
  stroke: string;
  opacity: string;
  text?: string;
};

export function installCssPreviewRuntime() {
  if (typeof window === "undefined" || typeof document === "undefined") return;
  const runtimeWindow = window as RuntimeWindow;
  if (runtimeWindow[INSTALLED_FLAG]) return;
  runtimeWindow[INSTALLED_FLAG] = true;

  let pending = false;
  const scheduleSync = () => {
    if (pending) return;
    pending = true;
    window.requestAnimationFrame(() => {
      pending = false;
      syncPreviewStages();
    });
  };

  const observer = new MutationObserver((mutations) => {
    if (mutations.every((mutation) => mutation.target instanceof Element && mutation.target.closest(`.${OVERLAY_CLASS}`))) return;
    scheduleSync();
  });

  const start = () => {
    observer.observe(document.body, {
      attributes: true,
      childList: true,
      subtree: true,
      attributeFilter: ["class", "style", "transform", "data-state"],
    });
    scheduleSync();
  };

  if (document.body) start();
  else window.addEventListener("DOMContentLoaded", start, { once: true });
}

function syncPreviewStages() {
  document.querySelectorAll<SVGSVGElement>("svg.character-preview").forEach((svg) => {
    const parent = svg.parentElement;
    if (!parent) return;
    const snapshot = snapshotSvg(svg);
    const signature = JSON.stringify(snapshot);
    let overlay = parent.querySelector<HTMLElement>(`:scope > .${OVERLAY_CLASS}`);
    if (!overlay) {
      overlay = document.createElement("div");
      overlay.className = OVERLAY_CLASS;
      overlay.setAttribute("data-testid", "css-character-preview");
      parent.appendChild(overlay);
    }
    if (overlay.dataset.signature === signature) {
      svg.setAttribute(HIDDEN_ATTR, "true");
      return;
    }
    overlay.dataset.signature = signature;
    renderOverlay(overlay, snapshot, svg);
    svg.setAttribute(HIDDEN_ATTR, "true");
  });
}

function snapshotSvg(svg: SVGSVGElement) {
  const state = svg.dataset.state || "idle";
  const svgRect = svg.getBoundingClientRect();
  const layers = Array.from(svg.querySelectorAll<SVGGElement>(".strut-node"))
    .map((group, index) => snapshotLayer(group, svgRect, index))
    .filter((layer): layer is LayerSnapshot => Boolean(layer));

  return {
    label: svg.getAttribute("data-character") || "Strut scene",
    state,
    width: Math.max(svgRect.width, 1),
    height: Math.max(svgRect.height, 1),
    layers,
  };
}

function snapshotLayer(group: SVGGElement, svgRect: DOMRect, index: number): LayerSnapshot | null {
  const shape = directShape(group);
  if (!shape) return null;

  const rect = shape.getBoundingClientRect();
  if (rect.width < 2 || rect.height < 2) return null;

  const computed = window.getComputedStyle(shape);
  const groupComputed = window.getComputedStyle(group);
  const name = group.dataset.nodeName || `Layer ${index + 1}`;
  const semantic = semanticTokens(`${name} ${group.className.baseVal || ""}`);
  const tag = shape.tagName.toLowerCase();

  return {
    id: group.dataset.nodeId || `layer-${index}`,
    name,
    semantic,
    shape: tag,
    left: percentage(rect.left - svgRect.left, svgRect.width),
    top: percentage(rect.top - svgRect.top, svgRect.height),
    width: percentage(rect.width, svgRect.width),
    height: percentage(rect.height, svgRect.height),
    fill: normalizePaint(computed.fill || groupComputed.fill),
    stroke: normalizePaint(computed.stroke || groupComputed.stroke),
    opacity: computed.opacity || groupComputed.opacity || "1",
    text: tag === "text" ? shape.textContent?.trim() : undefined,
  };
}

function renderOverlay(
  overlay: HTMLElement,
  snapshot: ReturnType<typeof snapshotSvg>,
  svg: SVGSVGElement,
) {
  overlay.replaceChildren();
  overlay.setAttribute("role", "img");
  overlay.setAttribute("aria-label", `${snapshot.label} ${snapshot.state}`);
  overlay.dataset.state = snapshot.state;

  const scene = document.createElement("div");
  scene.className = "css-preview-scene";
  overlay.appendChild(scene);

  snapshot.layers.forEach((layer, index) => {
    const element = document.createElement("button");
    element.type = "button";
    element.className = "css-preview-layer";
    element.dataset.layerId = layer.id;
    element.dataset.layerName = layer.name;
    element.dataset.semantic = layer.semantic;
    element.dataset.shape = layer.shape;
    element.style.left = `${layer.left}%`;
    element.style.top = `${layer.top}%`;
    element.style.width = `${layer.width}%`;
    element.style.height = `${layer.height}%`;
    element.style.opacity = layer.opacity;
    element.style.setProperty("--layer-fill", layer.fill);
    element.style.setProperty("--layer-stroke", layer.stroke);
    element.style.setProperty("--layer-depth", `${depthFor(layer.semantic, index)}px`);
    element.style.setProperty("--layer-tilt", `${tiltFor(layer.semantic, index)}deg`);
    element.title = layer.name;
    if (layer.text) element.textContent = layer.text;
    element.addEventListener("click", () => {
      const source = svg.querySelector<SVGGElement>(`.strut-node[data-node-id="${cssEscape(layer.id)}"]`);
      source?.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, view: window }));
    });
    scene.appendChild(element);
  });

  const label = document.createElement("div");
  label.className = "css-preview-state-label";
  label.textContent = titleCase(snapshot.state || "idle");
  overlay.appendChild(label);
}

function directShape(group: SVGGElement): SVGGraphicsElement | null {
  return Array.from(group.children).find((child): child is SVGGraphicsElement => {
    const tag = child.tagName.toLowerCase();
    return tag === "rect" || tag === "ellipse" || tag === "path" || tag === "text";
  }) ?? null;
}

function semanticTokens(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

function normalizePaint(value: string) {
  if (!value || value === "none" || value === "rgba(0, 0, 0, 0)") return "rgba(255,255,255,0.72)";
  return value;
}

function percentage(value: number, total: number) {
  if (!Number.isFinite(value) || !Number.isFinite(total) || total <= 0) return 0;
  return Math.max(-20, Math.min(140, (value / total) * 100));
}

function depthFor(semantic: string, index: number) {
  if (hasAny(semantic, ["shadow", "cast"])) return -40;
  if (hasAny(semantic, ["back", "rear"])) return -16;
  if (hasAny(semantic, ["rim", "edge", "depth", "side"])) return -6 + (index % 5);
  if (hasAny(semantic, ["glint", "highlight", "spark", "shine"])) return 34;
  if (hasAny(semantic, ["text", "mark", "symbol", "face"])) return 26;
  return 8 + (index % 7) * 2;
}

function tiltFor(semantic: string, index: number) {
  if (hasAny(semantic, ["shadow", "cast"])) return 0;
  if (hasAny(semantic, ["rim", "edge", "depth", "side"])) return -8 + (index % 4) * 4;
  if (hasAny(semantic, ["glint", "highlight", "spark", "shine"])) return 10;
  return -4 + (index % 5) * 2;
}

function hasAny(value: string, tokens: string[]) {
  return tokens.some((token) => value.includes(token));
}

function titleCase(value: string) {
  return value.replace(/(^|[_\s-]+)([a-z])/g, (_, spacer: string, letter: string) => `${spacer ? " " : ""}${letter.toUpperCase()}`);
}

function cssEscape(value: string) {
  if (typeof CSS !== "undefined" && CSS.escape) return CSS.escape(value);
  return value.replace(/["\\]/g, "\\$&");
}
