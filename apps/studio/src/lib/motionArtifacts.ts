import type { StrutDocument } from "../types";

export type MotionRenderer = "svg-css" | "dom-css" | "dom-css3d" | "sprite-css" | "canvas2d" | "webgl";

export type MotionSpec = {
  id: string;
  name: string;
  renderer: MotionRenderer;
  recipe: string;
  states: string[];
  inputs: Record<string, unknown>;
};

export type RuntimeAsset = {
  name: string;
  kind: "image" | "sprite" | "audio" | "data";
  url?: string;
  dataUrl?: string;
};

export type RuntimeComponent = {
  id: string;
  name: string;
  html: string;
  css: string;
  js: string;
  states: string[];
  inputs: Array<{ name: string; kind: "trigger" | "number" | "boolean" | "enum" }>;
  assets: RuntimeAsset[];
  recipeId?: string;
  previewWidth: number;
  previewHeight: number;
};

export type MotionArtifact =
  | { kind: "strut_document"; renderer: "svg-css"; document: StrutDocument; activeState: string }
  | { kind: "runtime_component"; renderer: Exclude<MotionRenderer, "svg-css">; spec: MotionSpec; component: RuntimeComponent; activeState: string };

export function classifyMotionRenderer(prompt: string): MotionRenderer {
  const text = prompt.toLowerCase();
  if (/(dice|\bdie\b|coin|card flip|cube|product spin|3d button)/.test(text)) return "dom-css3d";
  if (/(mascot|pet|character|duolingo|sprite|walk cycle|jump|wave)/.test(text)) return "sprite-css";
  if (/(particle|confetti|liquid|smoke|fire|physics)/.test(text)) return "canvas2d";
  if (/(button|toggle|hover|press|success|microinteraction)/.test(text)) return "dom-css";
  return "svg-css";
}

export function buildComponentPreviewHtml(component: RuntimeComponent): string {
  return `<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><style>html,body,#root{margin:0;width:100%;height:100%;overflow:hidden}body{display:grid;place-items:center;background:#eef4ef}${component.css}</style></head><body><div id="root">${component.html}</div><script>${component.js}<\/script></body></html>`;
}

export function verifyRuntimeComponent(component: RuntimeComponent, renderer: MotionRenderer): string[] {
  const issues: string[] = [];
  if (!component.html.trim()) issues.push("runtime component is missing html");
  if (!component.css.trim()) issues.push("runtime component is missing css");
  if (!component.states.length) issues.push("runtime component needs at least one state");
  if (renderer === "dom-css3d") {
    const css = component.css.toLowerCase();
    const html = component.html.toLowerCase();
    if (!css.includes("perspective")) issues.push("dom-css3d component is missing perspective");
    if (!css.includes("transform-style: preserve-3d") && !css.includes("transform-style:preserve-3d")) {
      issues.push("dom-css3d component is missing transform-style: preserve-3d");
    }
    if (!css.includes("translatez")) issues.push("dom-css3d component is missing translateZ depth");
    if ((html.match(/class="face/g) ?? []).length < 2) issues.push("dom-css3d component has no visible face elements");
  }
  return issues;
}
