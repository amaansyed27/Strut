import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ChevronRight,
  Cpu,
  FileText,
  Folder,
  FolderPlus,
  Layers3,
  MessageSquarePlus,
  PanelRight,
  Play,
  Plus,
  Route,
  Save,
  Search,
  Send,
  Settings2,
  Square,
  WandSparkles,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import "./App.css";

type StudioStatus = {
  format_version: string;
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
type ViewMode = "chat" | "preview" | "editor";
type MainPanel = "chat" | "providers" | "settings";

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

type ChatThread = {
  id: string;
  title: string;
  projectId: string;
  updated: string;
};

type ProjectRecord = {
  id: string;
  name: string;
  path: string;
  chats: ChatThread[];
};

type ViewModeOption = {
  id: ViewMode;
  Icon: LucideIcon;
  label: string;
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
  state_machines: [{ ...fallbackDocument.state_machines[0], id: "230", name: "OwlMoods" }],
  bindings: [{ name: "face_glow" }, { name: "feather_tint" }],
  events: [{ name: "wing_wave_started" }, { name: "celebration_complete" }],
};

const initialProjects: ProjectRecord[] = [
  {
    id: "strut",
    name: "Strut",
    path: "D:\\Strut Projects\\Strut",
    chats: [
      { id: "strut-plan", title: "Strut Plan", projectId: "strut", updated: "now" },
      { id: "bot-test", title: "Build bot character", projectId: "strut", updated: "1h" },
    ],
  },
  {
    id: "brand-motion",
    name: "Brand motion",
    path: "D:\\Strut Projects\\Brand motion",
    chats: [{ id: "owl-guide", title: "Owl guide animation", projectId: "brand-motion", updated: "2d" }],
  },
];

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
  return normalized.includes("owl") || normalized.includes("duo") || normalized.includes("duolingo")
    ? owlDocument
    : fallbackDocument;
}

function projectFiles(project: ProjectRecord): ProjectFile[] {
  return [
    { name: "strut.project.json", path: `${project.path}\\strut.project.json`, kind: "project" },
    { name: "starter.strut.json", path: `${project.path}\\scenes\\starter.strut.json`, kind: "scene" },
    { name: "assets", path: `${project.path}\\assets`, kind: "folder" },
    { name: "exports", path: `${project.path}\\exports`, kind: "folder" },
  ];
}

function CharacterPreview({ document, activeState }: { document: StrutDocument; activeState: string }) {
  const isOwl = document.name.toLowerCase().includes("owl");

  return (
    <svg
      className="character-preview"
      data-character={isOwl ? "owl" : "bot"}
      data-state={activeState}
      data-testid="character-preview"
      viewBox="0 0 640 420"
      role="img"
    >
      <rect className="preview-bg" width="640" height="420" rx="18" />
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
  const [projects, setProjects] = useState<ProjectRecord[]>(initialProjects);
  const [activeProjectId, setActiveProjectId] = useState("strut");
  const [activeChatId, setActiveChatId] = useState("strut-plan");
  const [mainPanel, setMainPanel] = useState<MainPanel>("chat");
  const [viewMode, setViewMode] = useState<ViewMode>("chat");
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [newProjectOpen, setNewProjectOpen] = useState(false);
  const [projectName, setProjectName] = useState("Untitled Strut Project");
  const [projectLocation, setProjectLocation] = useState("");
  const [document, setDocument] = useState<StrutDocument>(fallbackDocument);
  const [activeState, setActiveState] = useState("wave");
  const [partsVisible, setPartsVisible] = useState(true);
  const [activeTool, setActiveTool] = useState("select");
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [messages, setMessages] = useState<ChatMessage[]>([
    { id: 1, role: "assistant", text: "What should we build in Strut? Describe a character, attach a mockup later, or ask for a plan first." },
  ]);
  const [providerMode, setProviderMode] = useState<ProviderMode>("built-in");
  const [localAdapters, setLocalAdapters] = useState<LocalAdapter[]>(fallbackLocalAdapters);
  const [selectedLocalAdapterId, setSelectedLocalAdapterId] = useState("ollama");
  const [selectedByokProviderId, setSelectedByokProviderId] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [providerEndpoint, setProviderEndpoint] = useState(byokProviders[0].endpoint);
  const [providerModel, setProviderModel] = useState(byokProviders[0].model);
  const [activity, setActivity] = useState("Built-in planner ready");

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

  const activeProject = projects.find((project) => project.id === activeProjectId) ?? projects[0];
  const activeChat = activeProject.chats.find((chat) => chat.id === activeChatId) ?? activeProject.chats[0];
  const files = projectFiles(activeProject);
  const activeArtboard = document.artboards[0] ?? fallbackDocument.artboards[0];
  const activeMachine = document.state_machines[0] ?? fallbackDocument.state_machines[0];
  const layers = useMemo(() => flattenNodes(activeArtboard.nodes), [activeArtboard.nodes]);
  const activeByokProvider = byokProviders.find((provider) => provider.id === selectedByokProviderId) ?? byokProviders[0];
  const viewModes: ViewModeOption[] = [
    { id: "chat", Icon: MessageSquarePlus, label: "Chat only" },
    { id: "preview", Icon: PanelRight, label: "Chat + preview" },
    { id: "editor", Icon: Layers3, label: "Editor" },
  ];

  const filteredProjects = projects
    .map((project) => ({
      ...project,
      chats: project.chats.filter((chat) =>
        `${project.name} ${chat.title}`.toLowerCase().includes(searchQuery.toLowerCase()),
      ),
    }))
    .filter((project) => project.chats.length > 0 || project.name.toLowerCase().includes(searchQuery.toLowerCase()));

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

  function openChat(projectId: string, chatId: string) {
    setActiveProjectId(projectId);
    setActiveChatId(chatId);
    setMainPanel("chat");
  }

  function newChat(projectId = activeProjectId) {
    const project = projects.find((item) => item.id === projectId) ?? activeProject;
    const chat: ChatThread = {
      id: `chat-${Date.now()}`,
      title: "New character chat",
      projectId: project.id,
      updated: "now",
    };
    setProjects((current) =>
      current.map((item) => (item.id === project.id ? { ...item, chats: [chat, ...item.chats] } : item)),
    );
    setActiveProjectId(project.id);
    setActiveChatId(chat.id);
    setMainPanel("chat");
    setMessages([{ id: Date.now(), role: "assistant", text: "New chat ready. Tell Strut what to design or ask for a plan." }]);
  }

  async function createProject() {
    if (!desktopRuntime) {
      const id = `project-${Date.now()}`;
      const project: ProjectRecord = {
        id,
        name: projectName.trim() || "Untitled Strut Project",
        path: projectLocation,
        chats: [{ id: `${id}-chat`, title: "Project brief", projectId: id, updated: "now" }],
      };
      setProjects((current) => [project, ...current]);
      setActiveProjectId(id);
      setActiveChatId(project.chats[0].id);
      setNewProjectOpen(false);
      setActivity("Browser preview project. Disk was not written.");
      appendMessage("system", "Browser preview opened an in-memory project. Run the desktop app to create files on disk.");
      return;
    }

    try {
      const created = await invoke<ProjectInfo>("create_project", { name: projectName, location: projectLocation });
      const id = `project-${Date.now()}`;
      const project: ProjectRecord = {
        id,
        name: created.name,
        path: created.path,
        chats: [{ id: `${id}-chat`, title: "Project brief", projectId: id, updated: "now" }],
      };
      setProjects((current) => [project, ...current]);
      setActiveProjectId(id);
      setActiveChatId(project.chats[0].id);
      setNewProjectOpen(false);
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

  async function runGeneration() {
    const trimmed = prompt.trim();
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

  return (
    <main className="strut-shell">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <img src="/strut-mark.svg" alt="" />
          <span>Strut</span>
        </div>

        <div className="sidebar-actions">
          <button type="button" onClick={() => newChat()}>
            <MessageSquarePlus size={16} />
            New chat
          </button>
          <button type="button" onClick={() => setNewProjectOpen(true)}>
            <FolderPlus size={16} />
            New project
          </button>
          <button type="button" onClick={() => setSearchOpen((isOpen) => !isOpen)}>
            <Search size={16} />
            Search
          </button>
          <button type="button" onClick={() => setMainPanel("providers")}>
            <Cpu size={16} />
            Providers
          </button>
        </div>

        {searchOpen ? (
          <label className="sidebar-search">
            <Search size={14} />
            <input aria-label="Search projects and chats" value={searchQuery} onChange={(event) => setSearchQuery(event.currentTarget.value)} placeholder="Search projects" />
          </label>
        ) : null}

        <div className="project-list">
          <span className="section-label">Projects</span>
          {filteredProjects.map((project) => (
            <div className="project-group" key={project.id}>
              <div className="project-button">
                <button className="project-open" type="button" onClick={() => openChat(project.id, project.chats[0]?.id ?? activeChatId)}>
                  <Folder size={15} />
                  <span>{project.name}</span>
                </button>
                <button
                  aria-label={`New chat in ${project.name}`}
                  className="inline-add"
                  type="button"
                  onClick={(event) => {
                    event.stopPropagation();
                    newChat(project.id);
                  }}
                >
                  <Plus size={13} />
                </button>
              </div>
              {project.chats.map((chat) => (
                <button
                  className={chat.id === activeChatId ? "chat-link active" : "chat-link"}
                  key={chat.id}
                  type="button"
                  onClick={() => openChat(project.id, chat.id)}
                >
                  <span>{chat.title}</span>
                  <em>{chat.updated}</em>
                </button>
              ))}
            </div>
          ))}
        </div>

        <div className="sidebar-footer">
          <button className="provider-status" data-testid="activity-pill" type="button" onClick={() => setMainPanel("providers")}>
            <span>{providerMode === "built-in" ? "Built-in planner" : providerMode === "local" ? "Local CLI" : activeByokProvider.name}</span>
            <em>{activity}</em>
          </button>
          <button type="button" onClick={() => setMainPanel("settings")}>
            <Settings2 size={16} />
            Settings
          </button>
        </div>
      </aside>

      <section className="workspace">
        <header className="workspace-top">
          <nav className="view-switcher" aria-label="View mode">
            {viewModes.map(({ id, Icon, label }) => (
              <button
                aria-pressed={viewMode === id}
                className={viewMode === id ? "active" : ""}
                key={id}
                type="button"
                onClick={() => {
                  setViewMode(id);
                  setMainPanel("chat");
                }}
              >
                <Icon size={15} />
                {label}
              </button>
            ))}
          </nav>
          <div className="workspace-context">
            <strong>{activeChat?.title ?? "New chat"}</strong>
            <span>{activeProject.name} / {status?.format_version ?? "browser preview"}</span>
          </div>
        </header>

        {newProjectOpen ? (
          <section className="project-sheet" aria-label="New project panel">
            <div>
              <h2>New project</h2>
              <p>Choose where Strut should create the editable scene files.</p>
            </div>
            <label>
              <span>Name</span>
              <input aria-label="Project name" value={projectName} onChange={(event) => setProjectName(event.currentTarget.value)} />
            </label>
            <label>
              <span>Location</span>
              <input aria-label="Project location" value={projectLocation} onChange={(event) => setProjectLocation(event.currentTarget.value)} />
            </label>
            <div className="sheet-actions">
              <button type="button" onClick={() => setNewProjectOpen(false)}>Cancel</button>
              <button type="button" onClick={() => void createProject()}>Create project</button>
            </div>
          </section>
        ) : null}

        {mainPanel === "providers" ? (
          <section className="provider-page">
            <div className="page-heading">
              <h1>Providers</h1>
              <p>Connect local CLIs, local models, or BYOK APIs. Browser preview cannot run real provider checks.</p>
            </div>
            <div className="provider-tabs">
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
            <button className="secondary-button" type="button" onClick={() => void testProvider()}>
              Test selected provider
              <ChevronRight size={16} />
            </button>
          </section>
        ) : null}

        {mainPanel === "settings" ? (
          <section className="settings-page">
            <div className="page-heading">
              <h1>Settings</h1>
              <p>Workspace defaults, editor behavior, and provider status.</p>
            </div>
            <div className="settings-list">
              <section className="settings-section">
                <div>
                  <h2>Workspace</h2>
                  <p>Project creation and local file defaults.</p>
                </div>
                <label>
                  <span>Default project location</span>
                  <input aria-label="Default project location" value={projectLocation} onChange={(event) => setProjectLocation(event.currentTarget.value)} />
                </label>
              </section>
              <section className="settings-section">
                <div>
                  <h2>Generation</h2>
                  <p>Choose the provider Strut should use for new character work.</p>
                </div>
                <label>
                  <span>Generation mode</span>
                  <select aria-label="Generation mode" value={providerMode} onChange={(event) => setProviderMode(event.currentTarget.value as ProviderMode)}>
                    <option value="built-in">Built-in planner</option>
                    <option value="local">Local CLI</option>
                    <option value="byok">BYOK provider</option>
                  </select>
                </label>
                <div className="status-line">
                  <span>Current provider</span>
                  <strong>{providerMode === "built-in" ? "Built-in planner" : providerMode === "local" ? selectedLocalAdapterId : activeByokProvider.name}</strong>
                  <em>{activity}</em>
                </div>
              </section>
              <section className="settings-section">
                <div>
                  <h2>Editor</h2>
                  <p>Default panels and viewport behavior.</p>
                </div>
                <label className="toggle-row">
                  <input checked={partsVisible} type="checkbox" onChange={(event) => setPartsVisible(event.currentTarget.checked)} />
                  <span>Show character parts in editor</span>
                </label>
                <label className="toggle-row">
                  <input defaultChecked type="checkbox" />
                  <span>Preview motion when switching states</span>
                </label>
              </section>
            </div>
          </section>
        ) : null}

        {mainPanel === "chat" && viewMode !== "editor" ? (
          <section className={viewMode === "preview" ? "chat-layout with-preview" : "chat-layout"}>
            <div className="chat-panel">
              <div className="message-stack">
                <div className="home-heading">
                  <h1>What should we build in Strut?</h1>
                  <p>{activeProject.name} is ready for a prompt, mockup, or plan-first sketch.</p>
                </div>
                {messages.map((message) => (
                  <p className={`message ${message.role}`} key={message.id}>
                    <span>{message.role}</span>
                    {message.text}
                  </p>
                ))}
              </div>
              <div className="composer">
                <textarea aria-label="Character prompt" value={prompt} onChange={(event) => setPrompt(event.currentTarget.value)} placeholder="Ask Strut to make a character, storyboard, or editable animation" />
                <div className="composer-controls">
                  <span>{providerMode === "built-in" ? "Built-in" : providerMode === "local" ? "Local CLI" : activeByokProvider.name}</span>
                  <button aria-label="Generate" type="button" onClick={() => void runGeneration()}>
                    <Send size={17} />
                  </button>
                </div>
              </div>
            </div>
            {viewMode === "preview" ? (
              <PreviewPane activeMachine={activeMachine} activeState={activeState} document={document} setActiveState={setActiveState} />
            ) : null}
          </section>
        ) : null}

        {mainPanel === "chat" && viewMode === "editor" ? (
          <section className="editor-layout">
            <div className="editor-toolbar">
              {["select", "shape", "path", "bind", "animate"].map((tool) => (
                <button className={activeTool === tool ? "active" : ""} key={tool} type="button" onClick={() => setActiveTool(tool)}>
                  {tool === "select" ? <Square size={15} /> : <WandSparkles size={15} />}
                  {titleCase(tool)}
                </button>
              ))}
              <label>
                <input checked={partsVisible} type="checkbox" onChange={(event) => setPartsVisible(event.currentTarget.checked)} />
                Parts
              </label>
            </div>
            <div className="editor-main">
              <div className="parts-panel">
                <strong>Project files</strong>
                <div className="file-list">
                  {files.map((file) => (
                    <button key={file.path} type="button">
                      <FileText size={14} />
                      <span>{file.name}</span>
                      <em>{file.kind}</em>
                    </button>
                  ))}
                </div>
                <strong>{activeArtboard.name}</strong>
                {partsVisible ? (
                  <div className="layer-list">
                    {layers.map((layer) => (
                      <button key={layer.id} type="button">
                        <span>{layer.name}</span>
                        <em>{layer.kind}</em>
                      </button>
                    ))}
                  </div>
                ) : <p>Parts hidden</p>}
              </div>
              <PreviewPane activeMachine={activeMachine} activeState={activeState} document={document} setActiveState={setActiveState} />
            </div>
          </section>
        ) : null}
      </section>
    </main>
  );
}

function PreviewPane({
  activeMachine,
  activeState,
  document,
  setActiveState,
}: {
  activeMachine: StateMachine;
  activeState: string;
  document: StrutDocument;
  setActiveState: (state: string) => void;
}) {
  return (
    <aside className="preview-pane">
      <div className="preview-title">
        <div>
          <span>{document.name}</span>
          <em>{activeMachine.name}</em>
        </div>
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
  );
}

export default App;
