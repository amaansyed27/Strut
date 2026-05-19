import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ChevronRight,
  Cpu,
  FileText,
  Folder,
  FolderPlus,
  Home,
  ImagePlus,
  Layers3,
  MessageSquarePlus,
  Monitor,
  Moon,
  PanelRight,
  Play,
  Plus,
  Route,
  Save,
  Search,
  Send,
  Settings2,
  Square,
  Sun,
  Trash2,
  WandSparkles,
  X,
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
type ThemeMode = "system" | "light" | "dark";

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

type ReferenceAttachment = {
  id: string;
  name: string;
  mimeType: string;
  dataUrl: string;
  size: number;
};

type ChatMessage = {
  id: number;
  role: "assistant" | "user" | "system";
  text: string;
  attachments?: ReferenceAttachment[];
};

type ChatThread = {
  id: string;
  title: string;
  projectId: string;
  updated: string;
  messages: ChatMessage[];
  references: ReferenceAttachment[];
  document: StrutDocument | null;
  activeState: string;
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

type WorkspaceState = {
  projects: ProjectRecord[];
  activeProjectId: string | null;
  activeChatId: string | null;
  themeMode: ThemeMode;
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

const emptyArtboard: Artboard = {
  id: "empty-artboard",
  name: "No scene yet",
  width: 960,
  height: 540,
  nodes: [],
};

const emptyMachine: StateMachine = {
  id: "empty-machine",
  name: "No state machine",
  states: ["idle"],
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

const STORAGE_KEY = "strut-studio-workspace-v4";

function createChat(projectId: string, title: string, messages: ChatMessage[] = []): ChatThread {
  return {
    id: `chat-${Date.now()}-${Math.round(Math.random() * 10000)}`,
    title,
    projectId,
    updated: "now",
    messages,
    references: [],
    document: null,
    activeState: "wave",
  };
}

const initialProjects: ProjectRecord[] = [];

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

const defaultPrompt = "";

function isThemeMode(value: unknown): value is ThemeMode {
  return value === "system" || value === "light" || value === "dark";
}

function isStrutDocument(value: unknown): value is StrutDocument {
  return Boolean(
    value
      && typeof value === "object"
      && Array.isArray((value as StrutDocument).artboards)
      && Array.isArray((value as StrutDocument).state_machines),
  );
}

function normalizeAttachments(value: unknown): ReferenceAttachment[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value
    .filter((attachment) => attachment && typeof attachment === "object")
    .map((attachment) => {
      const candidate = attachment as Partial<ReferenceAttachment>;
      return {
        id: typeof candidate.id === "string" && candidate.id ? candidate.id : `ref-${Date.now()}-${Math.random()}`,
        name: typeof candidate.name === "string" ? candidate.name : "reference image",
        mimeType: typeof candidate.mimeType === "string" ? candidate.mimeType : "image/png",
        dataUrl: typeof candidate.dataUrl === "string" ? candidate.dataUrl : "",
        size: typeof candidate.size === "number" ? candidate.size : 0,
      };
    })
    .filter((attachment) => attachment.dataUrl.startsWith("data:image/"));
}

function normalizeMessages(value: unknown): ChatMessage[] {
  if (!Array.isArray(value)) {
    return [];
  }

  const messages = value
    .filter((message) => message && typeof message === "object")
    .map((message) => {
      const candidate = message as Partial<ChatMessage>;
      const role: ChatMessage["role"] =
        candidate.role === "user" || candidate.role === "system" || candidate.role === "assistant"
          ? candidate.role
          : "assistant";
      return {
        id: typeof candidate.id === "number" ? candidate.id : Date.now() + Math.random(),
        role,
        text: typeof candidate.text === "string" ? candidate.text : "",
        attachments: normalizeAttachments(candidate.attachments),
      };
    })
    .filter((message) => message.text.trim().length > 0 || (message.attachments?.length ?? 0) > 0);

  return messages;
}

function normalizeProjects(value: unknown): ProjectRecord[] {
  if (!Array.isArray(value)) {
    return initialProjects;
  }

  const projects = value
    .filter((project) => project && typeof project === "object")
    .map((project) => {
      const candidate = project as Partial<ProjectRecord>;
      const id = typeof candidate.id === "string" && candidate.id ? candidate.id : `project-${Date.now()}-${Math.random()}`;
      const chats = Array.isArray(candidate.chats)
        ? candidate.chats
            .filter((chat) => chat && typeof chat === "object")
            .map((chat) => {
              const chatCandidate = chat as Partial<ChatThread>;
              return {
                id: typeof chatCandidate.id === "string" && chatCandidate.id ? chatCandidate.id : `chat-${Date.now()}-${Math.random()}`,
                title: typeof chatCandidate.title === "string" && chatCandidate.title ? chatCandidate.title : "Untitled chat",
                projectId: id,
                updated: typeof chatCandidate.updated === "string" ? chatCandidate.updated : "now",
                messages: normalizeMessages(chatCandidate.messages),
                references: normalizeAttachments(chatCandidate.references),
        document: isStrutDocument(chatCandidate.document) ? chatCandidate.document : null,
        activeState: typeof chatCandidate.activeState === "string" ? chatCandidate.activeState : "wave",
      };
            })
        : [];

      return {
        id,
        name: typeof candidate.name === "string" && candidate.name ? candidate.name : "Untitled project",
        path: typeof candidate.path === "string" ? candidate.path : "D:\\Strut Projects",
        chats,
      };
    });

  return projects;
}

function loadWorkspaceState(): WorkspaceState {
  if (typeof window === "undefined") {
    return { projects: initialProjects, activeProjectId: null, activeChatId: null, themeMode: "system" };
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return { projects: initialProjects, activeProjectId: null, activeChatId: null, themeMode: "system" };
    }

    const parsed = JSON.parse(raw) as Partial<WorkspaceState>;
    const projects = normalizeProjects(parsed.projects);
    const activeProjectId = projects.some((project) => project.id === parsed.activeProjectId) ? parsed.activeProjectId ?? null : null;
    const activeProject = projects.find((project) => project.id === activeProjectId) ?? null;
    const activeChatId = activeProject?.chats.some((chat) => chat.id === parsed.activeChatId) ? parsed.activeChatId ?? null : null;

    return {
      projects,
      activeProjectId,
      activeChatId,
      themeMode: isThemeMode(parsed.themeMode) ? parsed.themeMode : "system",
    };
  } catch {
    return { projects: initialProjects, activeProjectId: null, activeChatId: null, themeMode: "system" };
  }
}

function promptTitle(prompt: string) {
  const compact = prompt.replace(/\s+/g, " ").trim();
  if (!compact) {
    return "New character chat";
  }
  return compact.length > 34 ? `${compact.slice(0, 31)}...` : compact;
}

function fileToAttachment(file: File): Promise<ReferenceAttachment> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = typeof reader.result === "string" ? reader.result : "";
      if (!dataUrl.startsWith("data:image/")) {
        reject(new Error(`${file.name} is not a supported image`));
        return;
      }
      resolve({
        id: `ref-${Date.now()}-${Math.round(Math.random() * 10000)}`,
        name: file.name,
        mimeType: file.type || "image/png",
        dataUrl,
        size: file.size,
      });
    };
    reader.onerror = () => reject(new Error(`Could not read ${file.name}`));
    reader.readAsDataURL(file);
  });
}

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
  const [initialWorkspace] = useState<WorkspaceState>(() => loadWorkspaceState());
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [status, setStatus] = useState<StudioStatus | null>(null);
  const [desktopRuntime, setDesktopRuntime] = useState(true);
  const [projects, setProjects] = useState<ProjectRecord[]>(initialWorkspace.projects);
  const [activeProjectId, setActiveProjectId] = useState<string | null>(initialWorkspace.activeProjectId);
  const [activeChatId, setActiveChatId] = useState<string | null>(initialWorkspace.activeChatId);
  const [mainPanel, setMainPanel] = useState<MainPanel>("chat");
  const [viewMode, setViewMode] = useState<ViewMode>("chat");
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [newProjectOpen, setNewProjectOpen] = useState(false);
  const [projectName, setProjectName] = useState("Untitled Strut Project");
  const [projectLocation, setProjectLocation] = useState("");
  const [partsVisible, setPartsVisible] = useState(true);
  const [activeTool, setActiveTool] = useState("select");
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [pendingReferences, setPendingReferences] = useState<ReferenceAttachment[]>([]);
  const [providerMode, setProviderMode] = useState<ProviderMode>("built-in");
  const [localAdapters, setLocalAdapters] = useState<LocalAdapter[]>(fallbackLocalAdapters);
  const [selectedLocalAdapterId, setSelectedLocalAdapterId] = useState("ollama");
  const [selectedByokProviderId, setSelectedByokProviderId] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [providerEndpoint, setProviderEndpoint] = useState(byokProviders[0].endpoint);
  const [providerModel, setProviderModel] = useState(byokProviders[0].model);
  const [activity, setActivity] = useState("Built-in planner ready");
  const [themeMode, setThemeMode] = useState<ThemeMode>(initialWorkspace.themeMode);

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
    invoke<LocalAdapter[]>("local_agent_adapters").then(setLocalAdapters).catch(() => setDesktopRuntime(false));
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }
    window.localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ projects, activeProjectId, activeChatId, themeMode }),
    );
  }, [projects, activeProjectId, activeChatId, themeMode]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }
    window.document.documentElement.dataset.theme = themeMode;
  }, [themeMode]);

  const activeProject = projects.find((project) => project.id === activeProjectId) ?? null;
  const activeChat = activeProject?.chats.find((chat) => chat.id === activeChatId) ?? null;
  const currentDocument = activeChat?.document ?? null;
  const currentActiveState = activeChat?.activeState ?? "wave";
  const files = activeProject ? projectFiles(activeProject) : [];
  const activeArtboard = currentDocument?.artboards[0] ?? emptyArtboard;
  const activeMachine = currentDocument?.state_machines[0] ?? emptyMachine;
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

  function updateChat(projectId: string, chatId: string, updater: (chat: ChatThread) => ChatThread) {
    setProjects((current) =>
      current.map((project) =>
        project.id === projectId
          ? {
              ...project,
              chats: project.chats.map((chat) => (chat.id === chatId ? updater(chat) : chat)),
            }
          : project,
      ),
    );
  }

  function updateCurrentChat(updater: (chat: ChatThread) => ChatThread) {
    if (!activeProjectId || !activeChatId) {
      return;
    }
    updateChat(activeProjectId, activeChatId, updater);
  }

  function appendMessage(role: ChatMessage["role"], text: string) {
    updateCurrentChat((chat) => ({
      ...chat,
      updated: "now",
      messages: [...chat.messages, { id: Date.now() + Math.random(), role, text }],
    }));
  }

  function appendUserMessage(text: string, attachments: ReferenceAttachment[]) {
    updateCurrentChat((chat) => ({
      ...chat,
      updated: "now",
      references: [...chat.references, ...attachments],
      messages: [...chat.messages, { id: Date.now() + Math.random(), role: "user", text, attachments }],
    }));
  }

  function openChat(projectId: string, chatId: string) {
    setActiveProjectId(projectId);
    setActiveChatId(chatId);
    setMainPanel("chat");
  }

  function openProject(projectId: string) {
    const project = projects.find((item) => item.id === projectId);
    setActiveProjectId(projectId);
    setActiveChatId(project?.chats[0]?.id ?? null);
    setMainPanel("chat");
  }

  function newChat(projectId = activeProjectId ?? projects[0]?.id ?? null) {
    const project = projects.find((item) => item.id === projectId);
    if (!project) {
      setNewProjectOpen(true);
      return;
    }
    const chat = createChat(project.id, "New character chat");
    setProjects((current) =>
      current.map((item) => (item.id === project.id ? { ...item, chats: [chat, ...item.chats] } : item)),
    );
    setActiveProjectId(project.id);
    setActiveChatId(chat.id);
    setMainPanel("chat");
  }

  function deleteChat(projectId: string, chatId: string) {
    setProjects((current) =>
      current.map((project) =>
        project.id === projectId
          ? { ...project, chats: project.chats.filter((chat) => chat.id !== chatId) }
          : project,
      ),
    );
    if (activeProjectId === projectId && activeChatId === chatId) {
      setActiveChatId(null);
    }
  }

  function removeProject(projectId: string) {
    setProjects((current) => current.filter((project) => project.id !== projectId));
    if (activeProjectId === projectId) {
      setActiveProjectId(null);
      setActiveChatId(null);
    }
  }

  function setCurrentActiveState(state: string) {
    updateCurrentChat((chat) => ({ ...chat, activeState: state, updated: "now" }));
  }

  async function attachReferenceImages(files: FileList | null) {
    if (!files || files.length === 0) {
      return;
    }
    const imageFiles = Array.from(files).filter((file) => file.type.startsWith("image/") || file.name.toLowerCase().endsWith(".svg"));
    if (imageFiles.length === 0) {
      setActivity("Choose PNG, JPG, WebP, GIF, or SVG references");
      return;
    }

    try {
      const attachments = await Promise.all(imageFiles.slice(0, 6).map(fileToAttachment));
      setPendingReferences((current) => [...current, ...attachments]);
      setActivity(`${attachments.length} reference image${attachments.length === 1 ? "" : "s"} attached`);
    } catch (error) {
      setActivity(String(error));
    } finally {
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  }

  function removePendingReference(id: string) {
    setPendingReferences((current) => current.filter((reference) => reference.id !== id));
  }

  async function createProject() {
    if (!desktopRuntime) {
      const id = `project-${Date.now()}`;
      const chat = createChat(id, "Project brief", [
        { id: Date.now(), role: "system", text: "Browser preview opened an in-memory project. Run the desktop app to create files on disk." },
      ]);
      const project: ProjectRecord = {
        id,
        name: projectName.trim() || "Untitled Strut Project",
        path: projectLocation,
        chats: [chat],
      };
      setProjects((current) => [project, ...current]);
      setActiveProjectId(id);
      setActiveChatId(chat.id);
      setNewProjectOpen(false);
      setActivity("Browser preview project. Disk was not written.");
      return;
    }

    try {
      const created = await invoke<ProjectInfo>("create_project", { name: projectName, location: projectLocation });
      const id = `project-${Date.now()}`;
      const chat = createChat(id, "Project brief", [
        { id: Date.now(), role: "system", text: `Project created: ${created.path}` },
      ]);
      const project: ProjectRecord = {
        id,
        name: created.name,
        path: created.path,
        chats: [chat],
      };
      setProjects((current) => [project, ...current]);
      setActiveProjectId(id);
      setActiveChatId(chat.id);
      setNewProjectOpen(false);
      setActivity(`Project created at ${created.path}`);
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
    if (!trimmed && pendingReferences.length === 0) {
      return;
    }
    if (!activeProjectId || !activeChatId) {
      newChat();
      setActivity("Start a chat first");
      return;
    }
    const references = pendingReferences;
    const generationPrompt = trimmed || "Use the attached reference image to create an editable animated character.";
    appendUserMessage(trimmed || "Use the attached reference image.", references);
    setPendingReferences([]);
    setActivity("Generating");

    try {
      const args = providerPayload()
        ? { prompt: generationPrompt, provider: providerPayload(), references }
        : { prompt: generationPrompt, references };
      const result = await invoke<GeneratedCharacter>("generate_character", args);
      updateChat(activeProjectId, activeChatId, (chat) => ({
        ...chat,
        title: chat.title === "New character chat" || chat.title === "Project brief" ? promptTitle(trimmed || references[0]?.name || "Reference character") : chat.title,
        updated: "now",
        document: result.document,
        activeState: result.document.state_machines[0]?.states.includes("wave") ? "wave" : "idle",
      }));
      setActivity(`${result.source}: ${result.message}`);
      appendMessage("assistant", `${result.document.name} is ready. I created editable layers, states, timelines, bindings, and events.`);
    } catch (error) {
      if (desktopRuntime) {
        setActivity(String(error));
        appendMessage("assistant", `Generation stopped: ${String(error)}`);
        return;
      }
      const generated = fallbackGenerateCharacter(`${trimmed} ${references.map((reference) => reference.name).join(" ")}`);
      updateChat(activeProjectId, activeChatId, (chat) => ({
        ...chat,
        title: chat.title === "New character chat" || chat.title === "Project brief" ? promptTitle(trimmed || references[0]?.name || "Reference character") : chat.title,
        updated: "now",
        document: generated,
        activeState: "wave",
      }));
      setActivity("Browser preview used built-in generator");
      appendMessage("assistant", `${generated.name} preview is ready${references.length ? ` from ${references.length} reference image${references.length === 1 ? "" : "s"}` : ""}. Open the desktop app for real provider-routed generation.`);
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
          <button type="button" onClick={() => {
            setActiveProjectId(null);
            setActiveChatId(null);
            setMainPanel("chat");
          }}>
            <Home size={16} />
            Home
          </button>
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
                <button className="project-open" type="button" onClick={() => openProject(project.id)}>
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
                <button
                  aria-label={`Remove project ${project.name}`}
                  className="inline-delete"
                  type="button"
                  onClick={() => removeProject(project.id)}
                >
                  <Trash2 size={13} />
                </button>
              </div>
              {project.chats.map((chat) => (
                <div className={chat.id === activeChatId ? "chat-row active" : "chat-row"} key={chat.id}>
                  <button
                    className="chat-link"
                    type="button"
                    onClick={() => openChat(project.id, chat.id)}
                  >
                    <span>{chat.title}</span>
                    <em>{chat.updated}</em>
                  </button>
                  <button
                    aria-label={`Delete chat ${chat.title}`}
                    className="chat-delete"
                    type="button"
                    onClick={() => deleteChat(project.id, chat.id)}
                  >
                    <Trash2 size={12} />
                  </button>
                </div>
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
            <strong>{activeChat?.title ?? "Home"}</strong>
            <span>{activeProject?.name ?? "No project selected"} / {status?.format_version ?? "browser preview"}</span>
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
                  <h2>Appearance</h2>
                  <p>Choose the Studio theme or follow your system setting.</p>
                </div>
                <div className="theme-options" role="radiogroup" aria-label="Theme">
                  {[
                    { id: "system", label: "Auto", Icon: Monitor },
                    { id: "light", label: "Light", Icon: Sun },
                    { id: "dark", label: "Dark", Icon: Moon },
                  ].map(({ id, label, Icon }) => (
                    <button
                      aria-checked={themeMode === id}
                      className={themeMode === id ? "active" : ""}
                      key={id}
                      role="radio"
                      type="button"
                      onClick={() => setThemeMode(id as ThemeMode)}
                    >
                      <Icon size={15} />
                      {label}
                    </button>
                  ))}
                </div>
              </section>
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
                <div className="settings-controls">
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
                </div>
              </section>
              <section className="settings-section">
                <div>
                  <h2>Editor</h2>
                  <p>Default panels and viewport behavior.</p>
                </div>
                <div className="settings-controls compact">
                  <label className="toggle-row">
                    <input checked={partsVisible} type="checkbox" onChange={(event) => setPartsVisible(event.currentTarget.checked)} />
                    <span>Show character parts in editor</span>
                  </label>
                  <label className="toggle-row">
                    <input defaultChecked type="checkbox" />
                    <span>Preview motion when switching states</span>
                  </label>
                </div>
              </section>
            </div>
          </section>
        ) : null}

        {mainPanel === "chat" && !activeChat ? (
          <HomePanel
            projects={projects}
            onNewProject={() => setNewProjectOpen(true)}
            onOpenProviders={() => setMainPanel("providers")}
            onStartChat={() => newChat(projects[0]?.id ?? null)}
          />
        ) : null}

        {mainPanel === "chat" && activeChat && viewMode !== "editor" ? (
          <section className={viewMode === "preview" ? "chat-layout with-preview" : "chat-layout"}>
            <div className="chat-panel">
              <div className="message-stack">
                <div className="home-heading">
                  <h1>What should we build in Strut?</h1>
                  <p>{activeProject?.name ?? "This project"} is ready for a prompt, mockup, or plan-first sketch.</p>
                </div>
                {activeChat.messages.map((message) => (
                  <p className={`message ${message.role}`} key={message.id}>
                    <span>{message.role}</span>
                    <span className="message-body">
                      {message.text}
                      {message.attachments?.length ? (
                        <span className="message-attachments">
                          {message.attachments.map((attachment) => (
                            <span className="message-attachment" key={attachment.id}>
                              <img src={attachment.dataUrl} alt="" />
                              <em>{attachment.name}</em>
                            </span>
                          ))}
                        </span>
                      ) : null}
                    </span>
                  </p>
                ))}
              </div>
              <div className="composer">
                {pendingReferences.length ? (
                  <div className="reference-tray">
                    {pendingReferences.map((reference) => (
                      <div className="reference-chip" key={reference.id}>
                        <img src={reference.dataUrl} alt="" />
                        <span>{reference.name}</span>
                        <button aria-label={`Remove reference ${reference.name}`} type="button" onClick={() => removePendingReference(reference.id)}>
                          <X size={13} />
                        </button>
                      </div>
                    ))}
                  </div>
                ) : null}
                <textarea aria-label="Character prompt" value={prompt} onChange={(event) => setPrompt(event.currentTarget.value)} placeholder="Ask Strut to make a character, storyboard, or editable animation" />
                <div className="composer-controls">
                  <div className="composer-left">
                    <input
                      ref={fileInputRef}
                      aria-label="Attach reference images"
                      className="reference-input"
                      type="file"
                      accept="image/png,image/jpeg,image/webp,image/gif,image/svg+xml"
                      multiple
                      onChange={(event) => void attachReferenceImages(event.currentTarget.files)}
                    />
                    <button aria-label="Attach reference images" type="button" onClick={() => fileInputRef.current?.click()}>
                      <ImagePlus size={16} />
                      Reference
                    </button>
                    <span>{providerMode === "built-in" ? "Built-in" : providerMode === "local" ? "Local CLI" : activeByokProvider.name}</span>
                  </div>
                  <button aria-label="Generate" type="button" onClick={() => void runGeneration()}>
                    <Send size={17} />
                  </button>
                </div>
              </div>
            </div>
            {viewMode === "preview" ? (
              <PreviewPane activeMachine={activeMachine} activeState={currentActiveState} document={currentDocument} setActiveState={setCurrentActiveState} />
            ) : null}
          </section>
        ) : null}

        {mainPanel === "chat" && activeChat && viewMode === "editor" ? (
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
                <div className="panel-title">
                  <strong>Layers</strong>
                  <em>{activeArtboard.name}</em>
                </div>
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
              <PreviewPane activeMachine={activeMachine} activeState={currentActiveState} document={currentDocument} setActiveState={setCurrentActiveState} />
            </div>
          </section>
        ) : null}
      </section>
    </main>
  );
}

function HomePanel({
  onNewProject,
  onOpenProviders,
  onStartChat,
  projects,
}: {
  onNewProject: () => void;
  onOpenProviders: () => void;
  onStartChat: () => void;
  projects: ProjectRecord[];
}) {
  return (
    <section className="empty-home">
      <div className="empty-hero">
        <div className="empty-mark">
          <img src="/strut-mark.svg" alt="" />
        </div>
        <h1>Start a motion project</h1>
        <p>Select a folder, open a project chat, or ask Strut to sketch a character direction before building the full animation.</p>
        <div className="empty-actions">
          <button type="button" onClick={onNewProject}>
            <FolderPlus size={16} />
            Select folder
          </button>
          <button type="button" onClick={onStartChat}>
            <MessageSquarePlus size={16} />
            Start chat
          </button>
        </div>
      </div>

      <div className="home-card-grid">
        <button type="button" onClick={onNewProject}>
          <span>New project</span>
          <em>Create a folder with scene, assets, and export directories.</em>
        </button>
        <button type="button" onClick={onStartChat} disabled={projects.length === 0}>
          <span>Plan first</span>
          <em>Start from a prompt and keep the conversation in the sidebar.</em>
        </button>
        <button type="button" onClick={onOpenProviders}>
          <span>Connect providers</span>
          <em>Choose built-in, local CLI, Ollama, or BYOK models.</em>
        </button>
      </div>
    </section>
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
  document: StrutDocument | null;
  setActiveState: (state: string) => void;
}) {
  return (
    <aside className="preview-pane">
      <div className="preview-title">
        <div>
          <span>Preview</span>
          <em>{document ? `${document.name} / ${activeMachine.name}` : "No generated scene"}</em>
        </div>
        <button disabled={!document} type="button" onClick={() => setActiveState("wave")}>
          <Play size={15} />
          Preview
        </button>
      </div>
      {document ? (
        <>
          <CharacterPreview document={document} activeState={activeState} />
          <div className="state-row">
            {activeMachine.states.map((state) => (
              <button className={state === activeState ? "active" : ""} key={state} type="button" onClick={() => setActiveState(state)}>
                <Route size={13} />
                {titleCase(state)}
              </button>
            ))}
          </div>
        </>
      ) : (
        <div className="preview-empty">
          <ImagePlus size={26} />
          <strong>No scene yet</strong>
          <span>Attach a reference image or describe the character in chat.</span>
        </div>
      )}
    </aside>
  );
}

export default App;
