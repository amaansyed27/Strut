import "./cssPreviewRuntime.css";

const INSTALLED_FLAG = "__strutCssPreviewRuntimeInstalled";
const BACKGROUND_KEY = "strut.preview.background";
const BACKGROUND_MODES = ["transparent", "white", "black"] as const;

type RuntimeWindow = Window & typeof globalThis & Record<string, unknown>;
type PreviewBackgroundMode = (typeof BACKGROUND_MODES)[number];

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
  const mode = currentBackgroundMode();
  document.documentElement.dataset.strutPreviewBackground = mode;
  const previews = document.querySelectorAll<SVGSVGElement>("svg.character-preview");
  previews.forEach((svg) => {
    svg.removeAttribute("data-css-preview-hidden");
    svg.setAttribute("data-css-preview-enhanced", "true");
    const parent = svg.parentElement;
    parent?.querySelectorAll(":scope > .css-character-preview").forEach((node) => node.remove());
    if (parent instanceof HTMLElement) {
      parent.dataset.previewBackground = mode;
      ensureBackgroundToggle(parent, mode);
    }
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

function ensureBackgroundToggle(stage: HTMLElement, mode: PreviewBackgroundMode) {
  let toggle = stage.querySelector<HTMLElement>(":scope > .preview-background-toggle");
  if (!toggle) {
    toggle = document.createElement("div");
    toggle.className = "preview-background-toggle";
    toggle.setAttribute("aria-label", "Preview background");
    toggle.setAttribute("role", "group");
    for (const option of BACKGROUND_MODES) {
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.previewBackgroundOption = option;
      button.textContent = option === "transparent" ? "Trans" : titleCase(option);
      button.title = `Preview background: ${option}`;
      button.addEventListener("click", () => setBackgroundMode(option));
      toggle.appendChild(button);
    }
    stage.appendChild(toggle);
  }
  Array.from(toggle.querySelectorAll<HTMLButtonElement>("button[data-preview-background-option]")).forEach((button) => {
    button.dataset.active = button.dataset.previewBackgroundOption === mode ? "true" : "false";
  });
}

function setBackgroundMode(mode: PreviewBackgroundMode) {
  window.localStorage.setItem(BACKGROUND_KEY, mode);
  document.documentElement.dataset.strutPreviewBackground = mode;
  document.querySelectorAll<HTMLElement>(".preview-stage").forEach((stage) => {
    stage.dataset.previewBackground = mode;
    ensureBackgroundToggle(stage, mode);
  });
}

function currentBackgroundMode(): PreviewBackgroundMode {
  const stored = window.localStorage.getItem(BACKGROUND_KEY);
  return isBackgroundMode(stored) ? stored : "transparent";
}

function isBackgroundMode(value: string | null): value is PreviewBackgroundMode {
  return BACKGROUND_MODES.some((mode) => mode === value);
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

function titleCase(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}
