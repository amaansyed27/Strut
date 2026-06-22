import "./cssPreviewRuntime.css";

const INSTALLED_FLAG = "__strutCssPreviewRuntimeInstalled";

type RuntimeWindow = Window & typeof globalThis & Record<string, unknown>;

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
      enhancePreviewStages();
    });
  };
  const observer = new MutationObserver(scheduleSync);
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

function enhancePreviewStages() {
  const previews = document.querySelectorAll<SVGSVGElement>("svg.character-preview");
  previews.forEach((svg) => {
    svg.removeAttribute("data-css-preview-hidden");
    svg.setAttribute("data-css-preview-enhanced", "true");
    const parent = svg.parentElement;
    parent?.querySelectorAll(":scope > .css-character-preview").forEach((node) => node.remove());
    const scene = svg.querySelector<SVGGElement>(".document-scene");
    scene?.setAttribute("data-css-scene", "true");
    Array.from(svg.querySelectorAll<SVGGElement>(".strut-node")).forEach((group, index) => {
      const semantic = semanticTokens(`${group.dataset.nodeName || ""} ${group.className.baseVal || ""}`);
      group.setAttribute("data-semantic", semantic);
      group.style.setProperty("--layer-depth", `${depthFor(semantic, index)}px`);
      group.style.setProperty("--layer-tilt", `${tiltFor(semantic, index)}deg`);
      group.style.setProperty("--layer-delay", `${Math.min(index * 34, 360)}ms`);
    });
  });
}

function semanticTokens(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
}

function depthFor(semantic: string, index: number) {
  if (hasAny(semantic, ["shadow", "cast"])) return -48;
  if (hasAny(semantic, ["back", "rear"])) return -14;
  if (hasAny(semantic, ["rim", "edge", "depth", "side"])) return -4 + (index % 4);
  if (hasAny(semantic, ["glint", "highlight", "spark", "shine"])) return 34;
  if (hasAny(semantic, ["text", "mark", "symbol", "face"])) return 24;
  return 6 + (index % 6) * 2;
}

function tiltFor(semantic: string, index: number) {
  if (hasAny(semantic, ["shadow", "cast"])) return 0;
  if (hasAny(semantic, ["rim", "edge", "depth", "side"])) return -10 + (index % 4) * 4;
  if (hasAny(semantic, ["glint", "highlight", "spark", "shine"])) return 12;
  return -4 + (index % 5) * 2;
}

function hasAny(value: string, tokens: string[]) {
  return tokens.some((token) => value.includes(token));
}
