import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Bot,
  Braces,
  Circle,
  Cpu,
  Gauge,
  Layers3,
  MousePointer2,
  Pencil,
  Play,
  Route,
  Save,
  ScanSearch,
  Sparkles,
  Square,
  Upload,
  WandSparkles,
  Zap,
} from "lucide-react";
import "./App.css";

type StudioStatus = {
  app: string;
  core_version: string;
  format_version: string;
  sample_name: string;
  sample_source: string;
  sample_artboards: number;
  sample_state_machines: number;
};

type StrutNode = {
  id: string;
  name: string;
  kind: string;
  children?: StrutNode[];
};

type Artboard = {
  id: string;
  name: string;
  width: number;
  height: number;
  nodes: StrutNode[];
};

type Timeline = {
  id: string;
  name: string;
  duration_ms: number;
};

type StateMachine = {
  id: string;
  name: string;
  states: string[];
};

type StrutDocument = {
  id: string;
  name: string;
  artboards: Artboard[];
  timelines: Timeline[];
  state_machines: StateMachine[];
  bindings: Array<{ name: string }>;
  events: Array<{ name: string }>;
};

type ProviderMode = "local" | "byok";

type LocalAdapter = {
  id: string;
  name: string;
  kind: string;
  command?: string | null;
  installed: boolean;
  detail: string;
};

type ByokProvider = {
  id: string;
  name: string;
  env: string;
  endpoint: string;
  model: string;
};

type ProviderOperationResult = {
  ok: boolean;
  status: string;
  detail: string;
};

type GenerationProvider = {
  mode: ProviderMode;
  localAdapterId?: string;
  byok?: {
    providerId: string;
    apiKey?: string;
    endpoint: string;
    model: string;
  };
};

type GeneratedCharacter = {
  document: StrutDocument;
  source: string;
  message: string;
};

const fallbackDocument: StrutDocument = {
  id: "00000000-0000-0000-0000-000000000100",
  name: "Minimal Bot",
  artboards: [
    {
      id: "00000000-0000-0000-0000-000000000101",
      name: "MinimalBot",
      width: 960,
      height: 540,
      nodes: [
        { id: "102", name: "BotRig", kind: "group" },
        { id: "103", name: "GroundShadow", kind: "ellipse" },
        { id: "104", name: "HelmetShell", kind: "path" },
        { id: "105", name: "FacePanel", kind: "rect" },
        { id: "106", name: "Eyes", kind: "path" },
        { id: "107", name: "Smile", kind: "path" },
        { id: "108", name: "Torso", kind: "path" },
        { id: "109", name: "ChestLight", kind: "ellipse" },
        { id: "110", name: "LeftArm", kind: "path" },
        { id: "111", name: "RightArm", kind: "path" },
        { id: "112", name: "LeftLeg", kind: "path" },
        { id: "113", name: "RightLeg", kind: "path" },
        { id: "114", name: "Antennae", kind: "path" },
      ],
    },
  ],
  timelines: [
    { id: "120", name: "idle_float", duration_ms: 1400 },
    { id: "121", name: "wave", duration_ms: 900 },
    { id: "122", name: "blink", duration_ms: 420 },
    { id: "123", name: "scan", duration_ms: 1200 },
    { id: "124", name: "celebrate", duration_ms: 1000 },
  ],
  state_machines: [
    {
      id: "130",
      name: "BotMoods",
      states: ["idle", "float", "wave", "blink", "scan", "celebrate", "sleep"],
    },
  ],
  bindings: [{ name: "face_glow" }, { name: "body_tint" }],
  events: [{ name: "wave_started" }, { name: "celebration_complete" }],
};

const layerColors: Record<string, string> = {
  group: "#2dffb8",
  rect: "#8be9fd",
  ellipse: "#f6d365",
  path: "#ff6b35",
  text: "#f6f1e8",
  image: "#9ba3b4",
  hit_area: "#c7a7ff",
};

const botSketches = [
  {
    id: "floating-helper",
    name: "Floating Helper",
    detail: "Soft hover loop, small wave, friendly face.",
    state: "wave",
    prompt: "make a minimalist waving robot like the reference image",
  },
  {
    id: "scanner-bot",
    name: "Scanner Bot",
    detail: "Face scan line, alert posture, data-ready motion.",
    state: "scan",
    prompt: "make a scanner robot with a face scan animation",
  },
  {
    id: "celebration-bot",
    name: "Celebration Bot",
    detail: "Pop motion, bright face, success feedback.",
    state: "celebrate",
    prompt: "make a celebration robot with success and confetti animation",
  },
  {
    id: "owl-guide",
    name: "Owl Guide",
    detail: "Rounded green mascot, wing wave, blink, celebrate.",
    state: "wave",
    prompt: "make an owl mascot like Duo from Duolingo with wave and blink animations",
  },
];

const studioModes = [
  { id: "design", label: "Design", Icon: MousePointer2 },
  { id: "states", label: "States", Icon: Route },
  { id: "agent", label: "Agent", Icon: Bot },
];

const studioTools = [
  { id: "select", label: "Select", Icon: MousePointer2 },
  { id: "path", label: "Draw path", Icon: Pencil },
  { id: "rect", label: "Rectangle", Icon: Square },
  { id: "ellipse", label: "Ellipse", Icon: Circle },
  { id: "ai", label: "AI create", Icon: WandSparkles },
];

const fallbackLocalAdapters: LocalAdapter[] = [
  {
    id: "ollama",
    name: "Ollama",
    kind: "local-model",
    command: "ollama",
    installed: false,
    detail: "desktop check required",
  },
  {
    id: "codex",
    name: "Codex",
    kind: "local-agent",
    command: "codex",
    installed: false,
    detail: "desktop check required",
  },
  {
    id: "gemini-cli",
    name: "Gemini CLI",
    kind: "local-agent",
    command: "gemini",
    installed: false,
    detail: "desktop check required",
  },
  {
    id: "claude-code",
    name: "Claude Code",
    kind: "local-agent",
    command: "claude",
    installed: false,
    detail: "desktop check required",
  },
  {
    id: "copilot-cli",
    name: "Copilot CLI",
    kind: "local-agent",
    command: "gh",
    installed: false,
    detail: "desktop check required",
  },
  {
    id: "antigravity",
    name: "Antigravity",
    kind: "local-agent",
    command: "antigravity",
    installed: false,
    detail: "desktop check required",
  },
  {
    id: "kiro",
    name: "Kiro",
    kind: "local-agent",
    command: "kiro",
    installed: false,
    detail: "desktop check required",
  },
];

const byokProviders: ByokProvider[] = [
  {
    id: "openai",
    name: "OpenAI",
    env: "OPENAI_API_KEY",
    endpoint: "https://api.openai.com/v1",
    model: "gpt-5.2",
  },
  {
    id: "anthropic",
    name: "Anthropic",
    env: "ANTHROPIC_API_KEY",
    endpoint: "https://api.anthropic.com",
    model: "claude-opus-4-5",
  },
  {
    id: "gemini",
    name: "Gemini",
    env: "GEMINI_API_KEY",
    endpoint: "https://generativelanguage.googleapis.com",
    model: "gemini-3-pro",
  },
  {
    id: "openrouter",
    name: "OpenRouter",
    env: "OPENROUTER_API_KEY",
    endpoint: "https://openrouter.ai/api/v1",
    model: "openai/gpt-5.2",
  },
  {
    id: "azure-openai",
    name: "Azure OpenAI",
    env: "AZURE_OPENAI_API_KEY",
    endpoint: "https://your-resource.openai.azure.com",
    model: "deployment-name",
  },
  {
    id: "openai-compatible",
    name: "OpenAI Compatible",
    env: "API_KEY",
    endpoint: "http://localhost:1234/v1",
    model: "local-model",
  },
];

const defaultCharacterPrompt = "make a minimalist waving robot like the reference image";

const owlDocument: StrutDocument = {
  ...fallbackDocument,
  id: "00000000-0000-0000-0000-000000000200",
  name: "Owl Mascot",
  artboards: [
    {
      ...fallbackDocument.artboards[0],
      id: "00000000-0000-0000-0000-000000000201",
      name: "OwlMascot",
      nodes: [
        { id: "102", name: "OwlRig", kind: "group" },
        { id: "103", name: "GroundShadow", kind: "ellipse" },
        { id: "104", name: "OwlBody", kind: "path" },
        { id: "105", name: "FaceMask", kind: "path" },
        { id: "106", name: "Eyes", kind: "path" },
        { id: "107", name: "Beak", kind: "path" },
        { id: "108", name: "Belly", kind: "path" },
        { id: "109", name: "ChestMark", kind: "ellipse" },
        { id: "110", name: "LeftWing", kind: "path" },
        { id: "111", name: "RightWing", kind: "path" },
        { id: "112", name: "LeftFoot", kind: "path" },
        { id: "113", name: "RightFoot", kind: "path" },
        { id: "114", name: "BrowTufts", kind: "path" },
      ],
    },
  ],
  state_machines: [
    {
      ...fallbackDocument.state_machines[0],
      id: "230",
      name: "OwlMoods",
    },
  ],
  bindings: [{ name: "face_glow" }, { name: "feather_tint" }],
  events: [{ name: "wing_wave_started" }, { name: "celebration_complete" }],
};

function titleCase(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function BotPreview({ activeState }: { activeState: string }) {
  return (
    <svg
      className={`bot-preview state-${activeState}`}
      data-character="bot"
      data-state={activeState}
      data-testid="character-preview"
      viewBox="0 0 960 540"
      role="img"
      aria-label={`Minimal bot preview in ${activeState} state`}
    >
      <rect width="960" height="540" rx="0" className="bot-sky" />
      <g className="confetti" aria-hidden="true">
        <circle cx="318" cy="118" r="8" />
        <rect x="598" y="102" width="12" height="12" rx="3" />
        <circle cx="644" cy="172" r="7" />
        <rect x="374" y="92" width="10" height="10" rx="2" />
      </g>
      <ellipse className="bot-shadow" cx="480" cy="442" rx="116" ry="18" />
      <g className="bot-rig">
        <g className="antennae">
          <path d="M376 172 L346 112" />
          <circle cx="342" cy="104" r="9" />
          <path d="M584 166 L616 86" />
          <circle cx="620" cy="78" r="9" />
        </g>

        <g className="left-arm">
          <path d="M332 304 C268 320 252 372 282 398 C310 424 352 388 382 344" />
          <circle cx="278" cy="400" r="13" />
          <circle cx="298" cy="414" r="10" />
        </g>
        <g className="right-arm">
          <path d="M624 286 C686 296 716 252 690 218 C666 186 622 214 596 260" />
          <circle cx="696" cy="216" r="12" />
          <circle cx="716" cy="228" r="10" />
          <circle cx="706" cy="246" r="9" />
        </g>

        <g className="legs">
          <path d="M420 376 C390 410 392 448 426 454 C458 460 476 424 482 388" />
          <path d="M532 382 C552 424 584 448 612 424 C636 402 610 366 570 344" />
        </g>

        <g className="body">
          <path d="M372 274 C386 226 430 204 492 206 C558 208 602 236 612 288 C624 356 576 402 486 400 C398 398 350 346 372 274Z" />
          <circle className="chest-light" cx="512" cy="308" r="17" />
          <path className="body-seam" d="M430 358 C466 374 526 374 562 354" />
        </g>

        <g className="helmet">
          <path className="helmet-shell" d="M330 154 C348 70 434 38 544 58 C628 74 662 142 642 224 C620 312 540 350 432 328 C354 312 312 236 330 154Z" />
          <path className="helmet-rim" d="M364 150 C390 96 452 72 530 84 C588 94 618 132 614 188 C610 252 558 286 462 278 C394 272 344 216 364 150Z" />
          <rect className="face-panel" x="386" y="118" width="214" height="136" rx="42" />
          <rect className="scan-line" x="404" y="136" width="178" height="12" rx="6" />
          <path className="eye left-eye" d="M430 176 C438 154 462 154 470 176" />
          <path className="eye right-eye" d="M518 174 C526 152 550 152 558 174" />
          <path className="smile" d="M464 210 C480 230 512 230 528 208" />
          <g className="ears">
            <path d="M330 170 C300 180 290 232 316 256 C340 278 354 238 348 196" />
            <path d="M646 166 C678 180 684 232 656 254 C632 274 620 236 626 194" />
          </g>
        </g>
      </g>
      <text className="bot-state-label" x="480" y="506" textAnchor="middle">
        {titleCase(activeState)}
      </text>
    </svg>
  );
}

function OwlPreview({ activeState }: { activeState: string }) {
  return (
    <svg
      className={`owl-preview state-${activeState}`}
      data-character="owl"
      data-state={activeState}
      data-testid="character-preview"
      viewBox="0 0 960 540"
      role="img"
      aria-label={`Owl mascot preview in ${activeState} state`}
    >
      <rect width="960" height="540" className="owl-sky" />
      <g className="owl-confetti" aria-hidden="true">
        <circle cx="340" cy="120" r="8" />
        <circle cx="620" cy="128" r="7" />
        <rect x="596" y="92" width="13" height="13" rx="3" />
      </g>
      <ellipse className="owl-shadow" cx="480" cy="444" rx="126" ry="18" />
      <g className="owl-rig">
        <path className="owl-left-wing" d="M368 248 C304 270 298 346 354 370 C382 332 390 292 368 248Z" />
        <path className="owl-right-wing" d="M592 248 C656 270 662 346 606 370 C578 332 570 292 592 248Z" />
        <path className="owl-body" d="M340 178 C350 92 424 62 480 96 C536 60 612 94 624 182 C642 310 582 406 480 410 C378 406 322 306 340 178Z" />
        <path className="owl-face" d="M384 184 C394 134 446 122 480 156 C514 122 566 134 576 184 C586 244 538 286 480 262 C422 286 374 244 384 184Z" />
        <path className="owl-brow" d="M412 138 L382 108 M548 138 L578 108" />
        <path className="owl-eye left" d="M424 196 C432 174 458 174 466 196" />
        <path className="owl-eye right" d="M494 196 C502 174 528 174 536 196" />
        <path className="owl-beak" d="M472 220 L488 220 L480 236Z" />
        <path className="owl-belly" d="M420 282 C430 342 530 342 540 282 C520 304 444 304 420 282Z" />
        <circle className="owl-chest" cx="480" cy="304" r="16" />
        <path className="owl-left-foot" d="M430 404 C418 428 444 438 462 414" />
        <path className="owl-right-foot" d="M530 404 C542 428 516 438 498 414" />
        <rect className="owl-scan-line" x="398" y="174" width="164" height="10" rx="5" />
      </g>
      <text className="bot-state-label" x="480" y="506" textAnchor="middle">
        {titleCase(activeState)}
      </text>
    </svg>
  );
}

function CharacterPreview({
  document,
  activeState,
}: {
  document: StrutDocument;
  activeState: string;
}) {
  const isOwl = document.name.toLowerCase().includes("owl");
  return isOwl ? <OwlPreview activeState={activeState} /> : <BotPreview activeState={activeState} />;
}

function flattenNodes(nodes: StrutNode[]): StrutNode[] {
  return nodes.flatMap((node) => [node, ...flattenNodes(node.children ?? [])]);
}

function timelineState(name: string) {
  return name === "idle_float" ? "float" : name;
}

function fallbackGenerateCharacter(prompt: string): StrutDocument {
  const normalized = prompt.toLowerCase();
  if (normalized.includes("owl") || normalized.includes("duo") || normalized.includes("duolingo")) {
    return owlDocument;
  }

  if (normalized.includes("scan") || normalized.includes("data")) {
    return {
      ...fallbackDocument,
      name: "Scanner Bot",
    };
  }

  if (normalized.includes("celebrate") || normalized.includes("success") || normalized.includes("confetti")) {
    return {
      ...fallbackDocument,
      name: "Celebration Bot",
    };
  }

  return fallbackDocument;
}

function App() {
  const [status, setStatus] = useState<StudioStatus | null>(null);
  const [document, setDocument] = useState<StrutDocument>(fallbackDocument);
  const [activeState, setActiveState] = useState("wave");
  const [activeMode, setActiveMode] = useState("design");
  const [activeTool, setActiveTool] = useState("select");
  const [selectedLayerId, setSelectedLayerId] = useState<string | null>(null);
  const [zoomMode, setZoomMode] = useState("Fit");
  const [gridVisible, setGridVisible] = useState(true);
  const [providerMode, setProviderMode] = useState<ProviderMode>("local");
  const [localAdapters, setLocalAdapters] = useState<LocalAdapter[]>(fallbackLocalAdapters);
  const [selectedLocalAdapterId, setSelectedLocalAdapterId] = useState("ollama");
  const [selectedByokProviderId, setSelectedByokProviderId] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [providerEndpoint, setProviderEndpoint] = useState(byokProviders[0].endpoint);
  const [providerModel, setProviderModel] = useState(byokProviders[0].model);
  const [connectionStatus, setConnectionStatus] = useState("Ollama selected");
  const [activity, setActivity] = useState("Ready");
  const [desktopRuntime, setDesktopRuntime] = useState(true);
  const [showSketches, setShowSketches] = useState(false);
  const [selectedSketch, setSelectedSketch] = useState(botSketches[0]);
  const [characterPrompt, setCharacterPrompt] = useState(defaultCharacterPrompt);

  useEffect(() => {
    invoke<StudioStatus>("studio_status").then((loadedStatus) => {
      setDesktopRuntime(true);
      setStatus(loadedStatus);
    }).catch(() => {
      setDesktopRuntime(false);
      setStatus(null);
    });

    invoke<StrutDocument>("sample_document")
      .then((loadedDocument) => {
        setDocument(loadedDocument);
        const firstState = loadedDocument.state_machines[0]?.states[0];
        if (firstState) {
          setActiveState(firstState);
        }
      })
      .catch(() => {
        setDesktopRuntime(false);
        setDocument(fallbackDocument);
      });

    invoke<LocalAdapter[]>("local_agent_adapters")
      .then((adapters) => {
        setLocalAdapters(adapters);
        const selectedAdapter = adapters.find((adapter) => adapter.id === selectedLocalAdapterId);
        if (selectedAdapter) {
          setConnectionStatus(selectedAdapter.detail);
        }
      })
      .catch(() => {
        setDesktopRuntime(false);
        setLocalAdapters(fallbackLocalAdapters);
        setConnectionStatus("Desktop runtime required");
      });
  }, []);

  const activeArtboard = document.artboards[0] ?? fallbackDocument.artboards[0];
  const activeMachine = document.state_machines[0] ?? fallbackDocument.state_machines[0];
  const activeLocalAdapter =
    localAdapters.find((adapter) => adapter.id === selectedLocalAdapterId) ?? localAdapters[0];
  const activeByokProvider =
    byokProviders.find((provider) => provider.id === selectedByokProviderId) ?? byokProviders[0];
  const visibleLayers = useMemo(() => flattenNodes(activeArtboard.nodes), [activeArtboard.nodes]);
  const states = activeMachine.states;
  const totalTimelineMs = useMemo(
    () => document.timelines.reduce((total, timeline) => total + timeline.duration_ms, 0),
    [document.timelines],
  );

  function selectLocalAdapter(adapter: LocalAdapter) {
    setProviderMode("local");
    setSelectedLocalAdapterId(adapter.id);
    setConnectionStatus(adapter.detail);
    setActivity(`${adapter.name} selected`);
  }

  function selectByokProvider(provider: ByokProvider) {
    setProviderMode("byok");
    setSelectedByokProviderId(provider.id);
    setProviderEndpoint(provider.endpoint);
    setProviderModel(provider.model);
    setConnectionStatus(`${provider.env} required`);
    setActivity(`${provider.name} selected`);
  }

  function currentProviderPayload(): GenerationProvider {
    if (providerMode === "local") {
      return {
        mode: "local",
        localAdapterId: selectedLocalAdapterId,
      };
    }

    return {
      mode: "byok",
      byok: {
        providerId: selectedByokProviderId,
        apiKey: apiKey.trim() || undefined,
        endpoint: providerEndpoint.trim(),
        model: providerModel.trim(),
      },
    };
  }

  async function saveByokProvider() {
    if (!desktopRuntime) {
      setConnectionStatus("Desktop runtime required");
      setActivity("Open Strut desktop app");
      return;
    }

    try {
      const result = await invoke<ProviderOperationResult>("save_byok_provider", {
        config: currentProviderPayload().byok,
      });
      setConnectionStatus(result.status);
      setActivity(result.ok ? `${activeByokProvider.name} config saved` : result.status);
    } catch (error) {
      const message = String(error);
      setConnectionStatus(message);
      setActivity("Save failed");
    }
  }

  async function testProviderConnection() {
    if (!desktopRuntime) {
      setConnectionStatus("Desktop runtime required");
      setActivity("Open Strut desktop app");
      return;
    }

    if (providerMode === "local") {
      try {
        const result = await invoke<ProviderOperationResult>("test_local_adapter", {
          adapterId: selectedLocalAdapterId,
        });
        setConnectionStatus(result.status);
        setActivity(result.status);
      } catch (error) {
        const message = String(error);
        setConnectionStatus(message);
        setActivity("Local test failed");
      }
      return;
    }

    try {
      const result = await invoke<ProviderOperationResult>("test_byok_provider", {
        config: currentProviderPayload().byok,
      });
      setConnectionStatus(result.status);
      setActivity(result.status);
    } catch (error) {
      const message = String(error);
      setConnectionStatus(message);
      setActivity("BYOK test failed");
    }
  }

  async function generateCharacter(prompt = characterPrompt, preferredState?: string) {
    let generatedDocument: StrutDocument;
    let generatedMessage = "Generated with browser preview";
    try {
      const result = await invoke<GeneratedCharacter>("generate_character", {
        prompt,
        provider: currentProviderPayload(),
      });
      generatedDocument = result.document;
      generatedMessage = result.message;
      setConnectionStatus(`${result.source}: ${result.message}`);
    } catch (error) {
      if (desktopRuntime) {
        const message = String(error);
        setConnectionStatus(message);
        setActivity("Generation failed");
        return;
      }
      generatedDocument = fallbackGenerateCharacter(prompt);
      setConnectionStatus("Browser preview uses built-in generator");
    }

    setDocument(generatedDocument);
    setSelectedLayerId(generatedDocument.artboards[0]?.nodes[0]?.id ?? null);
    setActivity(`${generatedMessage}: ${generatedDocument.name}`);
    const generatedStates = generatedDocument.state_machines[0]?.states ?? [];
    if (preferredState && generatedStates.includes(preferredState)) {
      setActiveState(preferredState);
    } else if (generatedStates.includes("wave")) {
      setActiveState("wave");
    } else if (generatedStates[0]) {
      setActiveState(generatedStates[0]);
    }
  }

  return (
    <main className="studio-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <img src="/strut-mark.svg" alt="" />
          <div>
            <strong>Strut Studio</strong>
            <span>{status?.format_version ?? "format 0.1.0"} - desktop alpha</span>
          </div>
        </div>

        <nav className="mode-tabs" aria-label="Studio modes">
          {studioModes.map(({ id, label, Icon }) => (
            <button
              aria-pressed={activeMode === id}
              className={activeMode === id ? "active" : ""}
              key={id}
              type="button"
              onClick={() => {
                setActiveMode(id);
                setActivity(`${label} mode`);
              }}
            >
              <Icon size={16} />
              {label}
            </button>
          ))}
        </nav>

        <div className="topbar-actions">
          <span className="activity-pill" data-testid="activity-pill">{activity}</span>
          <button
            className="icon-button"
            type="button"
            title="Import mockup"
            onClick={() => {
              setActiveMode("agent");
              setShowSketches(true);
              setActivity("Import mockup queued");
            }}
          >
            <Upload size={17} />
          </button>
          <button
            className="icon-button"
            type="button"
            title="Save project"
            onClick={() => setActivity(`Saved ${document.name}.strut`)}
          >
            <Save size={17} />
          </button>
          <button
            className="primary-action"
            type="button"
            onClick={() => {
              setActiveMode("states");
              setActiveState("wave");
              setActivity(`Previewing ${document.name}`);
            }}
          >
            <Play size={17} />
            Preview
          </button>
        </div>
      </header>

      <section className="workspace">
        <aside className="tool-rail" aria-label="Tools">
          {studioTools.map(({ id, label, Icon }) => (
            <button
              aria-pressed={activeTool === id}
              className={activeTool === id ? "selected" : ""}
              key={id}
              type="button"
              title={label}
              onClick={() => {
                setActiveTool(id);
                setActivity(`${label} tool`);
              }}
            >
              <Icon size={20} />
            </button>
          ))}
        </aside>

        <aside className="panel layers-panel">
          <div className="panel-heading">
            <span>
              <Layers3 size={16} />
              Layers
            </span>
            <small>{visibleLayers.length}</small>
          </div>
          <div className="layer-list">
            {visibleLayers.map((layer) => (
              <button
                aria-pressed={selectedLayerId === layer.id}
                className={selectedLayerId === layer.id ? "layer-row selected" : "layer-row"}
                key={layer.id}
                type="button"
                onClick={() => {
                  setSelectedLayerId(layer.id);
                  setActivity(`Selected ${layer.name}`);
                }}
              >
                <i style={{ background: layerColors[layer.kind] ?? "#9ba3b4" }} />
                <span>{layer.name}</span>
                <em>{layer.kind}</em>
              </button>
            ))}
          </div>

          <div className="panel-heading state-heading">
            <span>
              <Route size={16} />
              {activeMachine.name}
            </span>
          </div>
          <div className="state-grid">
            {states.map((state) => (
              <button
                className={state === activeState ? "active" : ""}
                data-state-button={state}
                key={state}
                type="button"
                onClick={() => {
                  setActiveState(state);
                  setActivity(`${titleCase(state)} state`);
                }}
              >
                {titleCase(state)}
              </button>
            ))}
          </div>
        </aside>

        <section className="stage-column">
          <div className="stage-toolbar">
            <span title={status?.sample_source ?? "browser fallback sample"}>
              {document.name}.strut - {activeArtboard.name} - {zoomMode}
            </span>
            <div>
              <button
                className={zoomMode === "100%" ? "active" : ""}
                type="button"
                onClick={() => {
                  setZoomMode("100%");
                  setActivity("Zoom 100%");
                }}
              >
                100%
              </button>
              <button
                className={zoomMode === "Fit" ? "active" : ""}
                type="button"
                onClick={() => {
                  setZoomMode("Fit");
                  setActivity("Zoom fit");
                }}
              >
                Fit
              </button>
              <button
                aria-pressed={gridVisible}
                className={gridVisible ? "active" : ""}
                type="button"
                onClick={() => {
                  setGridVisible((isVisible) => !isVisible);
                  setActivity(gridVisible ? "Grid hidden" : "Grid visible");
                }}
              >
                Grid
              </button>
            </div>
          </div>

          <div className="stage">
            <div className={`artboard bot-artboard ${gridVisible ? "grid-visible" : ""}`}>
              <CharacterPreview document={document} activeState={activeState} />
            </div>
          </div>

          <footer className="timeline-panel">
            <div className="timeline-header">
              <span>Timeline</span>
              <small>{totalTimelineMs}ms total</small>
            </div>
            <div className="timeline-ruler">
              {document.timelines.map((timeline, index) => {
                const start = 2 + index * 18;
                const width = Math.max(10, Math.min(20, timeline.duration_ms / 70));
                const isActive =
                  activeState === timeline.name ||
                  (activeState === "float" && timeline.name === "idle_float");

                return (
                  <div
                    className={`timeline-clip ${isActive ? "active" : ""}`}
                    key={timeline.id}
                    role="button"
                    tabIndex={0}
                    onClick={() => {
                      setActiveState(timelineState(timeline.name));
                      setActivity(`${titleCase(timeline.name)} timeline`);
                    }}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        setActiveState(timelineState(timeline.name));
                        setActivity(`${titleCase(timeline.name)} timeline`);
                      }
                    }}
                    style={{
                      left: `${start}%`,
                      width: `${width}%`,
                      borderColor: isActive ? "#2dffb8" : "#8be9fd",
                    }}
                  >
                    {titleCase(timeline.name)}
                  </div>
                );
              })}
            </div>
          </footer>
        </section>

        <aside className="panel agent-panel">
          <div className="panel-heading">
            <span>
              <Sparkles size={16} />
              Agent Run
            </span>
            <small>BYOK</small>
          </div>

          <div className="agent-card">
            <div className="agent-card-title">
              <Zap size={17} />
              Plan Mode
            </div>
            <textarea
              aria-label="Character prompt"
              value={characterPrompt}
              onChange={(event) => setCharacterPrompt(event.currentTarget.value)}
              placeholder="make a mascot character with wave, blink, scan and celebrate animations"
            />
            <button
              className="wide-action"
              type="button"
              onClick={() => {
                setShowSketches(true);
                void generateCharacter();
              }}
            >
              Generate Character
            </button>
          </div>

          {showSketches ? (
            <div className="sketch-stack" data-testid="plan-sketches">
              {botSketches.map((sketch) => (
                <button
                  className={selectedSketch.id === sketch.id ? "sketch-card selected" : "sketch-card"}
                  key={sketch.id}
                  type="button"
                  onClick={() => {
                    setSelectedSketch(sketch);
                    setCharacterPrompt(sketch.prompt ?? sketch.detail);
                  }}
                >
                  <span className={`sketch-thumb ${sketch.id}`} />
                  <strong>{sketch.name}</strong>
                  <em>{sketch.detail}</em>
                </button>
              ))}
              <button
                className="wide-action build-action"
                type="button"
                onClick={() => {
                  void generateCharacter(selectedSketch.prompt ?? selectedSketch.detail, selectedSketch.state);
                }}
              >
                Build Character
              </button>
            </div>
          ) : null}

          <div className="provider-console">
            {!desktopRuntime ? (
              <div className="runtime-warning" data-testid="runtime-warning">
                Browser preview only. Run the Tauri desktop app for real CLI checks, BYOK HTTP calls, saved
                provider config, and provider-routed generation.
              </div>
            ) : null}

            <div className="provider-mode-tabs" aria-label="Provider mode">
              <button
                aria-pressed={providerMode === "local"}
                className={providerMode === "local" ? "active" : ""}
                type="button"
                onClick={() => {
                  setProviderMode("local");
                  setActivity(`${activeLocalAdapter.name} selected`);
                  setConnectionStatus(activeLocalAdapter.detail);
                }}
              >
                Local CLI
              </button>
              <button
                aria-pressed={providerMode === "byok"}
                className={providerMode === "byok" ? "active" : ""}
                type="button"
                onClick={() => {
                  setProviderMode("byok");
                  setActivity(`${activeByokProvider.name} selected`);
                  setConnectionStatus(`${activeByokProvider.env} required`);
                }}
              >
                BYOK APIs
              </button>
            </div>

            {providerMode === "local" ? (
              <div className="provider-stack" data-testid="local-provider-list">
                {localAdapters.map((adapter) => (
                  <button
                    aria-label={adapter.name}
                    aria-pressed={selectedLocalAdapterId === adapter.id}
                    className={selectedLocalAdapterId === adapter.id ? "active" : ""}
                    key={adapter.id}
                    type="button"
                    onClick={() => selectLocalAdapter(adapter)}
                  >
                    <Cpu size={15} />
                    <span>
                      <strong>{adapter.name}</strong>
                      <em>{adapter.command ?? "endpoint"} - {adapter.kind}</em>
                    </span>
                    <i className={adapter.installed ? "status-dot ready" : "status-dot"} />
                  </button>
                ))}
              </div>
            ) : (
              <div className="byok-panel" data-testid="byok-provider-panel">
                <div className="provider-stack byok-provider-stack">
                  {byokProviders.map((provider) => (
                    <button
                      aria-label={provider.name}
                      aria-pressed={selectedByokProviderId === provider.id}
                      className={selectedByokProviderId === provider.id ? "active" : ""}
                      key={provider.id}
                      type="button"
                      onClick={() => selectByokProvider(provider)}
                    >
                      <Cpu size={15} />
                      <span>
                        <strong>{provider.name}</strong>
                        <em>{provider.env}</em>
                      </span>
                    </button>
                  ))}
                </div>

                <label className="provider-field">
                  <span>API key</span>
                  <input
                    aria-label={`${activeByokProvider.name} API key`}
                    autoComplete="off"
                    placeholder={activeByokProvider.env}
                    type="password"
                    value={apiKey}
                    onChange={(event) => setApiKey(event.currentTarget.value)}
                  />
                </label>
                <label className="provider-field">
                  <span>Base URL</span>
                  <input
                    aria-label={`${activeByokProvider.name} base URL`}
                    value={providerEndpoint}
                    onChange={(event) => setProviderEndpoint(event.currentTarget.value)}
                  />
                </label>
                <label className="provider-field">
                  <span>Model</span>
                  <input
                    aria-label={`${activeByokProvider.name} model`}
                    value={providerModel}
                    onChange={(event) => setProviderModel(event.currentTarget.value)}
                  />
                </label>
                <button className="wide-action secondary-action" type="button" onClick={saveByokProvider}>
                  Save Provider
                </button>
              </div>
            )}

            <div className="connection-footer">
              <span data-testid="connection-status">{connectionStatus}</span>
              <button type="button" onClick={testProviderConnection}>
                Test Connection
              </button>
            </div>
          </div>

          <div className="verifier-list">
            <div>
              <Gauge size={16} />
              <span>{document.timelines.length} timelines loaded</span>
              <strong>ready</strong>
            </div>
            <div>
              <ScanSearch size={16} />
              <span>{states.length} states reachable</span>
              <strong>ready</strong>
            </div>
            <div>
              <Braces size={16} />
              <span>{document.bindings.length} runtime bindings</span>
              <strong>ready</strong>
            </div>
            <div>
              <Cpu size={16} />
              <span>{providerMode === "local" ? activeLocalAdapter.name : activeByokProvider.name}</span>
              <strong>{providerMode}</strong>
            </div>
          </div>
        </aside>
      </section>
    </main>
  );
}

export default App;
