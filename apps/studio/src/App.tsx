import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Cpu,
  FileText,
  FolderOpen,
  Layers3,
  Play,
  Route,
  Save,
  Send,
  Settings2,
  Sparkles,
} from "lucide-react";
import "./App.css";

type StudioStatus = {
  format_version: string;
  sample_source: string;
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

type ProjectFile = {
  name: string;
  path: string;
  kind: string;
};

type ProjectInfo = {
  name: string;
  path: string;
  files: ProjectFile[];
};

type ProviderMode = "built-in" | "local" | "byok";

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

type ChatMessage = {
  id: number;
  role: "assistant" | "user" | "system";
  text: string;
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
        { id: "103", name: "OwlBody", kind: "path" },
        { id: "104", name: "FaceMask", kind: "path" },
        { id: "105", name: "Beak", kind: "path" },
        { id: "106", name: "LeftWing", kind: "path" },
        { id: "107", name: "RightWing", kind: "path" },
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

const fallbackLocalAdapters: LocalAdapter[] = [
  { id: "ollama", name: "Ollama", kind: "local-model", command: "ollama", installed: false, detail: "desktop check required" },
  { id: "codex", name: "Codex", kind: "local-agent", command: "codex", installed: false, detail: "desktop check required" },
  { id: "gemini-cli", name: "Gemini CLI", kind: "local-agent", command: "gemini", installed: false, detail: "desktop check required" },
  { id: "claude-code", name: "Claude Code", kind: "local-agent", command: "claude", installed: false, detail: "desktop check required" },
  { id: "copilot-cli", name: "Copilot CLI", kind: "local-agent", command: "gh", installed: false, detail: "desktop check required" },
];

const byokProviders: ByokProvider[] = [
  { id: "openai", name: "OpenAI", env: "OPENAI_API_KEY", endpoint: "https://api.openai.com/v1", model: "gpt-5.2" },
  { id: "anthropic", name: "Anthropic", env: "ANTHROPIC_API_KEY", endpoint: "https://api.anthropic.com", model: "claude-opus-4-5" },
  { id: "gemini", name: "Gemini", env: "GEMINI_API_KEY", endpoint: "https://generativelanguage.googleapis.com", model: "gemini-3-pro" },
  { id: "openrouter", name: "OpenRouter", env: "OPENROUTER_API_KEY", endpoint: "https://openrouter.ai/api/v1", model: "openai/gpt-5.2" },
  { id: "openai-compatible", name: "OpenAI Compatible", env: "API_KEY", endpoint: "http://localhost:1234/v1", model: "local-model" },
];

const defaultPrompt = "make a minimalist waving robot character like the reference image";

function titleCase(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function flattenNodes(nodes: StrutNode[]): StrutNode[] {
  return nodes.flatMap((node) => [node, ...flattenNodes(node.children ?? [])]);
}

function fallbackGenerateCharacter(prompt: string): StrutDocument {
  const normalized = prompt.toLowerCase();
  if (normalized.includes("owl") || normalized.includes("duo") || normalized.includes("duolingo")) {
    return owlDocument;
  }
  return fallbackDocument;
}

function CharacterPreview({ document, activeState }: { document: StrutDocument; activeState: string }) {
  const isOwl = document.name.toLowerCase().includes("owl");

  return (
    <svg className="character-preview" data-testid="character-preview" data-character={isOwl ? "owl" : "bot"} data-state={activeState} viewBox="0 0 640 420" role="img">
      <rect className="preview-bg" width="640" height="420" rx="0" />
      <ellipse className="preview-shadow" cx="320" cy="340" rx={isOwl ? 88 : 78} ry="12" />
      {isOwl ? (
        <g className={`owl-figure state-${activeState}`}>
          <path className="owl-wing left" d="M238 202 C190 226 188 284 230 302 C252 276 258 236 238 202Z" />
          <path className="owl-wing right" d="M402 202 C450 226 452 284 410 302 C388 276 382 236 402 202Z" />
          <path className="owl-body" d="M228 156 C236 88 292 70 320 96 C348 70 404 88 412 156 C426 258 382 326 320 330 C258 326 214 258 228 156Z" />
          <path className="owl-face" d="M262 166 C272 128 304 126 320 150 C336 126 368 128 378 166 C388 208 350 236 320 220 C290 236 252 208 262 166Z" />
          <path className="owl-eye" d="M288 172 C294 158 306 158 312 172 M328 172 C334 158 346 158 352 172" />
          <path className="owl-beak" d="M312 194 L328 194 L320 208Z" />
        </g>
      ) : (
        <g className={`bot-figure state-${activeState}`}>
          <path className="bot-body" d="M242 214 C256 174 292 158 336 160 C380 162 410 184 416 226 C424 282 384 318 322 316 C260 314 226 268 242 214Z" />
          <path className="bot-head" d="M224 118 C240 62 292 44 354 56 C406 66 430 104 418 160 C404 218 352 240 288 226 C240 216 210 174 224 118Z" />
          <rect className="bot-face" x="266" y="104" width="124" height="82" rx="26" />
          <path className="bot-eye" d="M292 140 C298 126 312 126 318 140 M344 140 C350 126 364 126 370 140" />
          <path className="bot-smile" d="M308 164 C320 178 344 178 356 164" />
          <path className="bot-arm left" d="M240 232 C198 248 190 286 218 302" />
          <path className="bot-arm right" d="M416 220 C462 222 470 184 448 164" />
        </g>
      )}
      <text className="state-label" x="320" y="386" textAnchor="middle">{titleCase(activeState)}</text>
    </svg>
  );
}

function App() {
  const [status, setStatus] = useState<StudioStatus | null>(null);
  const [desktopRuntime, setDesktopRuntime] = useState(true);
  const [project, setProject] = useState<ProjectInfo | null>(null);
  const [projectName, setProjectName] = useState("Untitled Strut Project");
  const [projectLocation, setProjectLocation] = useState("");
  const [document, setDocument] = useState<StrutDocument>(fallbackDocument);
  const [activeState, setActiveState] = useState("wave");
  const [activeView, setActiveView] = useState<"chat" | "files" | "editor" | "ai">("chat");
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [messages, setMessages] = useState<ChatMessage[]>([
    { id: 1, role: "assistant", text: "Create a project, then describe the character or interaction you want. Strut will build an editable scene, not a flat image." },
  ]);
  const [providerMode, setProviderMode] = useState<ProviderMode>("built-in");
  const [localAdapters, setLocalAdapters] = useState<LocalAdapter[]>(fallbackLocalAdapters);
  const [selectedLocalAdapterId, setSelectedLocalAdapterId] = useState("ollama");
  const [selectedByokProviderId, setSelectedByokProviderId] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [providerEndpoint, setProviderEndpoint] = useState(byokProviders[0].endpoint);
  const [providerModel, setProviderModel] = useState(byokProviders[0].model);
  const [activity, setActivity] = useState("Ready");

  useEffect(() => {
    invoke<StudioStatus>("studio_status")
      .then((loadedStatus) => {
        setDesktopRuntime(true);
        setStatus(loadedStatus);
      })
      .catch(() => {
        setDesktopRuntime(false);
        setStatus(null);
      });

    invoke<string>("default_project_location")
      .then(setProjectLocation)
      .catch(() => {
        setDesktopRuntime(false);
        setProjectLocation("D:\\Strut Projects");
      });

    invoke<StrutDocument>("sample_document").then(setDocument).catch(() => setDocument(fallbackDocument));
    invoke<LocalAdapter[]>("local_agent_adapters").then(setLocalAdapters).catch(() => setDesktopRuntime(false));
  }, []);

  const activeArtboard = document.artboards[0] ?? fallbackDocument.artboards[0];
  const activeMachine = document.state_machines[0] ?? fallbackDocument.state_machines[0];
  const layers = useMemo(() => flattenNodes(activeArtboard.nodes), [activeArtboard.nodes]);
  const activeByokProvider = byokProviders.find((provider) => provider.id === selectedByokProviderId) ?? byokProviders[0];

  function providerPayload(): GenerationProvider | undefined {
    if (providerMode === "built-in") {
      return undefined;
    }
    if (providerMode === "local") {
      return { mode: "local", localAdapterId: selectedLocalAdapterId };
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

  function appendMessage(role: ChatMessage["role"], text: string) {
    setMessages((current) => [...current, { id: Date.now() + Math.random(), role, text }]);
  }

  async function createProject() {
    if (!desktopRuntime) {
      const previewProject = {
        name: projectName.trim() || "Untitled Strut Project",
        path: projectLocation,
        files: [
          { name: "strut.project.json", path: `${projectLocation}\\strut.project.json`, kind: "project" },
          { name: "starter.strut.json", path: `${projectLocation}\\scenes\\starter.strut.json`, kind: "scene" },
        ],
      };
      setProject(previewProject);
      setActivity("Browser preview project. Disk was not written.");
      appendMessage("system", "Browser preview opened an in-memory project. Run the desktop app to create files on disk.");
      return;
    }

    try {
      const created = await invoke<ProjectInfo>("create_project", { name: projectName, location: projectLocation });
      setProject(created);
      setActivity(`Project created at ${created.path}`);
      appendMessage("system", `Project created: ${created.path}`);
    } catch (error) {
      setActivity(String(error));
    }
  }

  async function saveProvider() {
    if (providerMode !== "byok") {
      setActivity("Select BYOK first");
      return;
    }
    if (!desktopRuntime) {
      setActivity("Desktop app required for provider config");
      return;
    }
    try {
      const result = await invoke<ProviderOperationResult>("save_byok_provider", {
        config: providerPayload()?.byok,
      });
      setActivity(result.status);
    } catch (error) {
      setActivity(String(error));
    }
  }

  async function testProvider() {
    if (!desktopRuntime) {
      setActivity("Desktop app required for real provider checks");
      return;
    }
    try {
      const result =
        providerMode === "local"
          ? await invoke<ProviderOperationResult>("test_local_adapter", { adapterId: selectedLocalAdapterId })
          : providerMode === "byok"
            ? await invoke<ProviderOperationResult>("test_byok_provider", { config: providerPayload()?.byok })
            : { status: "Built-in planner ready", detail: "", ok: true };
      setActivity(result.status);
    } catch (error) {
      setActivity(String(error));
    }
  }

  async function runGeneration(input = prompt) {
    const trimmed = input.trim();
    if (!trimmed) {
      return;
    }

    appendMessage("user", trimmed);
    setActivity("Generating");

    try {
      const args = providerPayload() ? { prompt: trimmed, provider: providerPayload() } : { prompt: trimmed };
      const result = await invoke<GeneratedCharacter>("generate_character", args);
      setDocument(result.document);
      setActiveState(result.document.state_machines[0]?.states.includes("wave") ? "wave" : "idle");
      setActivity(`${result.source}: ${result.message}`);
      appendMessage("assistant", `${result.document.name} is ready. I created editable layers, states, timelines, bindings, and events.`);
    } catch (error) {
      if (desktopRuntime) {
        setActivity(String(error));
        appendMessage("assistant", `Generation stopped: ${String(error)}`);
        return;
      }
      const generated = fallbackGenerateCharacter(trimmed);
      setDocument(generated);
      setActiveState("wave");
      setActivity("Browser preview used built-in generator");
      appendMessage("assistant", `${generated.name} preview is ready. Open the desktop app for real provider-routed generation.`);
    }
  }

  if (!project) {
    return (
      <main className="home-shell">
        <header className="home-top">
          <div className="brand">
            <img src="/strut-mark.svg" alt="" />
            <span>Strut</span>
          </div>
          <span>{status?.format_version ?? "desktop app required for disk projects"}</span>
        </header>

        <section className="home-grid">
          <div className="home-intro">
            <p>AI-first motion design for editable characters and product animation.</p>
            <textarea aria-label="Initial prompt" value={prompt} onChange={(event) => setPrompt(event.currentTarget.value)} />
          </div>

          <div className="new-project">
            <h1>New project</h1>
            {!desktopRuntime ? <p className="runtime-note">Browser preview only. Project creation here will not write files.</p> : null}
            <label>
              <span>Name</span>
              <input aria-label="Project name" value={projectName} onChange={(event) => setProjectName(event.currentTarget.value)} />
            </label>
            <label>
              <span>Location</span>
              <input aria-label="Project location" value={projectLocation} onChange={(event) => setProjectLocation(event.currentTarget.value)} />
            </label>
            <button type="button" onClick={createProject}>
              <FolderOpen size={17} />
              Create project
            </button>
          </div>
        </section>
      </main>
    );
  }

  return (
    <main className="app-shell">
      <header className="app-top">
        <div className="brand">
          <img src="/strut-mark.svg" alt="" />
          <span>{project.name}</span>
        </div>
        <nav aria-label="Workspace">
          {[
            ["chat", Sparkles, "Chat"],
            ["files", FileText, "Files"],
            ["editor", Layers3, "Editor"],
            ["ai", Cpu, "AI"],
          ].map(([id, Icon, label]) => (
            <button className={activeView === id ? "active" : ""} key={String(id)} type="button" onClick={() => setActiveView(id as typeof activeView)}>
              <Icon size={15} />
              {String(label)}
            </button>
          ))}
        </nav>
        <span className="activity" data-testid="activity-pill">{activity}</span>
      </header>

      <section className="workspace-shell">
        <aside className="file-rail">
          <strong>Files</strong>
          {project.files.map((file) => (
            <button key={file.path} type="button" onClick={() => setActiveView("files")}>
              <FileText size={14} />
              <span>{file.name}</span>
              <em>{file.kind}</em>
            </button>
          ))}
        </aside>

        <section className="main-work">
          {activeView === "chat" ? (
            <div className="chat-view">
              <div className="messages">
                {messages.map((message) => (
                  <p className={`message ${message.role}`} key={message.id}>
                    <span>{message.role}</span>
                    {message.text}
                  </p>
                ))}
              </div>
              <div className="composer">
                <textarea aria-label="Character prompt" value={prompt} onChange={(event) => setPrompt(event.currentTarget.value)} />
                <button type="button" onClick={() => void runGeneration()}>
                  <Send size={17} />
                  Generate
                </button>
              </div>
            </div>
          ) : null}

          {activeView === "files" ? (
            <div className="files-view">
              <h2>Project files</h2>
              {project.files.map((file) => (
                <div className="file-row" key={file.path}>
                  <span>{file.name}</span>
                  <em>{file.path}</em>
                </div>
              ))}
            </div>
          ) : null}

          {activeView === "editor" ? (
            <div className="editor-view">
              <div>
                <h2>{activeArtboard.name}</h2>
                <p>{activeMachine.name}: {layers.length} layers, {document.timelines.length} timelines</p>
              </div>
              <div className="layer-list">
                {layers.map((layer) => (
                  <button key={layer.id} type="button">
                    <span>{layer.name}</span>
                    <em>{layer.kind}</em>
                  </button>
                ))}
              </div>
            </div>
          ) : null}

          {activeView === "ai" ? (
            <div className="ai-view">
              <h2>AI provider</h2>
              <div className="segmented">
                {["built-in", "local", "byok"].map((mode) => (
                  <button className={providerMode === mode ? "active" : ""} key={mode} type="button" onClick={() => setProviderMode(mode as ProviderMode)}>
                    {mode === "built-in" ? "Built-in" : mode === "local" ? "Local CLI" : "BYOK"}
                  </button>
                ))}
              </div>

              {providerMode === "local" ? (
                <div className="provider-list">
                  {localAdapters.map((adapter) => (
                    <button className={selectedLocalAdapterId === adapter.id ? "active" : ""} key={adapter.id} type="button" onClick={() => setSelectedLocalAdapterId(adapter.id)}>
                      <span>{adapter.name}</span>
                      <em>{adapter.detail}</em>
                    </button>
                  ))}
                </div>
              ) : null}

              {providerMode === "byok" ? (
                <div className="byok-form">
                  <select aria-label="BYOK provider" value={selectedByokProviderId} onChange={(event) => {
                    const provider = byokProviders.find((item) => item.id === event.currentTarget.value) ?? byokProviders[0];
                    setSelectedByokProviderId(provider.id);
                    setProviderEndpoint(provider.endpoint);
                    setProviderModel(provider.model);
                  }}>
                    {byokProviders.map((provider) => <option key={provider.id} value={provider.id}>{provider.name}</option>)}
                  </select>
                  <input aria-label={`${activeByokProvider.name} API key`} placeholder={activeByokProvider.env} type="password" value={apiKey} onChange={(event) => setApiKey(event.currentTarget.value)} />
                  <input aria-label={`${activeByokProvider.name} base URL`} value={providerEndpoint} onChange={(event) => setProviderEndpoint(event.currentTarget.value)} />
                  <input aria-label={`${activeByokProvider.name} model`} value={providerModel} onChange={(event) => setProviderModel(event.currentTarget.value)} />
                  <button type="button" onClick={() => void saveProvider()}>
                    <Save size={16} />
                    Save provider
                  </button>
                </div>
              ) : null}

              <button className="test-provider" type="button" onClick={() => void testProvider()}>
                <Settings2 size={16} />
                Test selected provider
              </button>
            </div>
          ) : null}
        </section>

        <aside className="preview-rail">
          <div className="preview-title">
            <span>{document.name}</span>
            <button type="button" onClick={() => setActiveState("wave")}>
              <Play size={15} />
              Preview
            </button>
          </div>
          <CharacterPreview document={document} activeState={activeState} />
          <div className="state-row">
            {activeMachine.states.map((state) => (
              <button className={state === activeState ? "active" : ""} key={state} type="button" onClick={() => setActiveState(state)}>
                <Route size={13} />
                {titleCase(state)}
              </button>
            ))}
          </div>
        </aside>
      </section>
    </main>
  );
}

export default App;
