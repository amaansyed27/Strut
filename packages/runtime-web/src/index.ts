import JSZip from "jszip";

export type StrutManifest = {
  format: "strut";
  schemaVersion: string;
  document: string;
  createdBy: string;
  minimumRuntime: string;
};

export type StrutNode = {
  id: string;
  name: string;
  kind: string;
};

export type StrutArtboard = {
  id: string;
  name: string;
  width: number;
  height: number;
  nodes: StrutNode[];
};

export type StrutTimeline = {
  id: string;
  name: string;
  duration_ms: number;
};

export type StrutStateMachine = {
  id: string;
  name: string;
  states: string[];
};

export type StrutDocument = {
  id: string;
  name: string;
  artboards: StrutArtboard[];
  timelines: StrutTimeline[];
  state_machines: StrutStateMachine[];
  bindings: Array<{ name: string }>;
  events: Array<{ name: string }>;
};

export type StrutPackage = {
  manifest: StrutManifest;
  document: StrutDocument;
};

export type BotState = "idle" | "float" | "wave" | "blink" | "scan" | "celebrate" | "sleep";

export type MountedStrut = {
  document: StrutDocument;
  setState(state: string): void;
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

export function mountMinimalBot(
  target: HTMLElement,
  document: StrutDocument,
  initialState: BotState = "idle",
): MountedStrut {
  injectMinimalBotStyles();
  target.replaceChildren(createMinimalBotSvg(initialState));

  return {
    document,
    setState(state: string) {
      const nextState = coerceBotState(document, state);
      const svg = target.querySelector<SVGSVGElement>("[data-strut-bot]");
      if (!svg) {
        return;
      }

      for (const currentClass of Array.from(svg.classList)) {
        if (currentClass.startsWith("state-")) {
          svg.classList.remove(currentClass);
        }
      }
      svg.classList.add(`state-${nextState}`);
      svg.dataset.state = nextState;
      const label = svg.querySelector("[data-state-label]");
      if (label) {
        label.textContent = titleCase(nextState);
      }
    },
    destroy() {
      target.replaceChildren();
    },
  };
}

export function createMinimalBotSvg(initialState: BotState = "idle"): SVGSVGElement {
  const template = document.createElement("template");
  template.innerHTML = minimalBotSvg(initialState).trim();
  const svg = template.content.firstElementChild;
  if (!(svg instanceof SVGSVGElement)) {
    throw new Error("failed to create minimal bot SVG");
  }
  return svg;
}

function validateManifest(manifest: StrutManifest) {
  if (manifest.format !== "strut") {
    throw new Error("manifest format must be strut");
  }

  if (!manifest.schemaVersion.startsWith("0.1")) {
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

function coerceBotState(document: StrutDocument, state: string): BotState {
  const states = document.state_machines[0]?.states ?? [];
  if (!states.includes(state)) {
    throw new Error(`unknown state: ${state}`);
  }
  return state as BotState;
}

function injectMinimalBotStyles() {
  if (document.getElementById("strut-minimal-bot-runtime-styles")) {
    return;
  }

  const style = document.createElement("style");
  style.id = "strut-minimal-bot-runtime-styles";
  style.textContent = minimalBotCss;
  document.head.append(style);
}

function titleCase(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function minimalBotSvg(state: BotState) {
  return `
<svg class="strut-minimal-bot state-${state}" data-strut-bot data-state="${state}" viewBox="0 0 960 540" role="img" aria-label="Minimal bot">
  <rect width="960" height="540" class="bot-sky" />
  <ellipse class="bot-shadow" cx="480" cy="442" rx="116" ry="18" />
  <g class="bot-rig">
    <g class="right-arm">
      <path d="M624 286 C686 296 716 252 690 218 C666 186 622 214 596 260" />
      <circle cx="696" cy="216" r="12" />
      <circle cx="716" cy="228" r="10" />
    </g>
    <g class="legs">
      <path d="M420 376 C390 410 392 448 426 454 C458 460 476 424 482 388" />
      <path d="M532 382 C552 424 584 448 612 424 C636 402 610 366 570 344" />
    </g>
    <g class="body">
      <path d="M372 274 C386 226 430 204 492 206 C558 208 602 236 612 288 C624 356 576 402 486 400 C398 398 350 346 372 274Z" />
      <circle class="chest-light" cx="512" cy="308" r="17" />
    </g>
    <g class="helmet">
      <path class="helmet-shell" d="M330 154 C348 70 434 38 544 58 C628 74 662 142 642 224 C620 312 540 350 432 328 C354 312 312 236 330 154Z" />
      <rect class="face-panel" x="386" y="118" width="214" height="136" rx="42" />
      <rect class="scan-line" x="404" y="136" width="178" height="12" rx="6" />
      <path class="eye" d="M430 176 C438 154 462 154 470 176" />
      <path class="eye" d="M518 174 C526 152 550 152 558 174" />
      <path class="smile" d="M464 210 C480 230 512 230 528 208" />
    </g>
  </g>
  <text data-state-label class="state-label" x="480" y="506" text-anchor="middle">${titleCase(state)}</text>
</svg>`;
}

const minimalBotCss = `
.strut-minimal-bot { width: 100%; height: 100%; display: block; }
.strut-minimal-bot .bot-sky { fill: #51bfd0; }
.strut-minimal-bot .bot-rig { transform-box: fill-box; transform-origin: center; }
.strut-minimal-bot .bot-shadow { fill: #17142f; opacity: 0.9; transform-origin: center; }
.strut-minimal-bot .helmet-shell,
.strut-minimal-bot .body path,
.strut-minimal-bot .legs path,
.strut-minimal-bot .right-arm path,
.strut-minimal-bot .right-arm circle {
  fill: #f6f1e8;
  stroke: #17142f;
  stroke-width: 8;
  stroke-linejoin: round;
}
.strut-minimal-bot .face-panel { fill: #17142f; stroke: #ffffff; stroke-width: 6; }
.strut-minimal-bot .eye,
.strut-minimal-bot .smile { fill: none; stroke: #51bfd0; stroke-width: 9; stroke-linecap: round; stroke-linejoin: round; }
.strut-minimal-bot .chest-light { fill: #51bfd0; stroke: #17142f; stroke-width: 5; }
.strut-minimal-bot .scan-line { fill: #2dffb8; opacity: 0; }
.strut-minimal-bot .state-label { fill: #17142f; font: 800 22px system-ui, sans-serif; }
.strut-minimal-bot.state-idle .bot-rig,
.strut-minimal-bot.state-float .bot-rig { animation: strutBotFloat 1.5s ease-in-out infinite; }
.strut-minimal-bot.state-wave .right-arm { animation: strutBotWave 0.9s ease-in-out infinite; transform-box: fill-box; transform-origin: 18% 18%; }
.strut-minimal-bot.state-blink .eye { animation: strutBotBlink 0.9s ease-in-out infinite; transform-box: fill-box; transform-origin: center; }
.strut-minimal-bot.state-scan .scan-line { animation: strutFaceScan 1.2s linear infinite; opacity: 0.75; }
.strut-minimal-bot.state-celebrate .bot-rig { animation: strutCelebrate 1s ease-in-out infinite; }
.strut-minimal-bot.state-sleep .eye { transform: scaleY(0.16); transform-box: fill-box; transform-origin: center; }
@keyframes strutBotFloat { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-18px); } }
@keyframes strutBotWave { 0%, 100% { transform: rotate(0deg); } 45% { transform: rotate(-24deg); } 70% { transform: rotate(12deg); } }
@keyframes strutBotBlink { 0%, 45%, 100% { transform: scaleY(1); } 55%, 62% { transform: scaleY(0.08); } }
@keyframes strutFaceScan { 0% { transform: translateY(0); } 100% { transform: translateY(92px); } }
@keyframes strutCelebrate { 0%, 100% { transform: scale(1) translateY(-8px); } 35% { transform: scale(1.08) translateY(-24px); } }
`;
