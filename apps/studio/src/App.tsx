import { Fragment, useEffect, useMemo, useRef, useState, type CSSProperties, type MouseEvent, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ChevronRight,
  Cpu,
  Folder,
  FolderOpen,
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
  Sun,
  Trash2,
  X,
  MoreHorizontal,
  Pin,
  Pencil,
  RefreshCw,
  RotateCcw,
  RotateCw,
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
  role?: string;
  transform?: {
    translate_x?: number;
    translate_y?: number;
    rotate?: number;
    scale_x?: number;
    scale_y?: number;
  };
  style?: {
    fill?: string | null;
    stroke?: string | null;
    stroke_width?: number;
    opacity?: number;
    linecap?: string | null;
    linejoin?: string | null;
  };
  shape?:
    | { type: "none" }
    | { type: "rect"; x: number; y: number; width: number; height: number; rx: number }
    | { type: "ellipse"; cx: number; cy: number; rx: number; ry: number }
    | { type: "path"; d: string }
    | { type: "text"; x: number; y: number; value: string; size: number };
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
  tracks?: Array<{
    target: string;
    property: string;
    keyframes: Array<{
      time_ms: number;
      value: { type: "number"; value: number } | { type: string; value: unknown };
      easing: "linear" | "ease_in" | "ease_out" | "ease_in_out";
    }>;
  }>;
};

type StateMachine = {
  id: string;
  name: string;
  inputs?: Array<{ name: string; kind: string }>;
  states: string[];
  transitions?: Array<{ from: string; to: string; on: string; timeline: string }>;
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

type ProviderMode = "local" | "byok";
type ViewMode = "chat" | "preview";
type MainPanel = "chat" | "providers" | "settings";
type ThemeMode = "system" | "light" | "dark";
type RunState = "idle" | "thinking" | "generating";

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

type GenerationContext = {
  projectName?: string;
  projectPath?: string;
  activeChatTitle?: string;
  currentDocumentSummary?: string;
  chatHistory: Array<{
    role: ChatMessage["role"];
    text: string;
    attachments?: string[];
  }>;
  currentDocument?: StrutDocument;
};

type GeneratedCharacter = {
  document: StrutDocument;
  source: string;
  message: string;
  planSummary?: {
    subjectClassification: string;
    subjectLabel: string;
    partNames: string[];
    timelineNames: string[];
  } | null;
  operationCount?: number | null;
};

type ChatAnswer = {
  source: string;
  message: string;
};

type ReferenceAttachment = {
  id: string;
  name: string;
  kind?: "image" | "layer";
  mimeType: string;
  dataUrl?: string;
  size: number;
  nodeId?: string;
  nodeKind?: string;
  nodeRole?: string;
};

type ChatMessage = {
  id: number;
  role: "assistant" | "user" | "system";
  text: string;
  attachments?: ReferenceAttachment[];
  operationBatchId?: string;
};

type ChatThread = {
  id: string;
  title: string;
  projectId: string;
  updated: string;
  pinned?: boolean;
  messages: ChatMessage[];
  references: ReferenceAttachment[];
  document: StrutDocument | null;
  activeState: string;
  selectedNodeId?: string | null;
  layerUi?: Record<string, LayerUiState>;
  pendingOperation?: OperationBatch | null;
  operationHistory?: OperationBatch[];
  operationBatches?: OperationBatch[];
  undoStack?: string[];
  redoStack?: string[];
};

type LayerUiState = {
  visible: boolean;
  locked: boolean;
};

type OperationPreview = {
  id: string;
  targetId: string;
  targetName: string;
  intent: string;
  operationType: "style.patch" | "transform.patch" | "timeline.patch";
  affectedProperties: string[];
  createdAt: string;
};

type OperationSourceType = "ai" | "sprite-python" | "manual" | "cli";
type OperationBatchStatus = "pending" | "applied" | "rejected" | "undone";

type OperationValidationResult = {
  ok: boolean;
  message: string;
  validator: string;
  validatedAt: string;
};

type SetPropertyOperation = {
  id: string;
  type: "set_property";
  targetId: string;
  targetName: string;
  property: string;
  previousValue: unknown;
  value: unknown;
};

type ReplaceDocumentOperation = {
  id: string;
  type: "replace_document";
  previousDocument: StrutDocument | null;
  nextDocument: StrutDocument;
};

type OperationRecord = SetPropertyOperation | ReplaceDocumentOperation;

type OperationBatch = OperationPreview & {
  sourceType: OperationSourceType;
  status: OperationBatchStatus;
  validationResult: OperationValidationResult;
  documentRevisionId: string;
  previousDocumentRevisionId?: string | null;
  prompt?: string;
  sourceMetadata?: Record<string, unknown>;
  operations: OperationRecord[];
  updatedAt: string;
  appliedAt?: string | null;
  rejectedAt?: string | null;
};

type ProjectSnapshot = {
  project: ProjectInfo;
  document: StrutDocument;
  operationBatches: OperationBatch[];
  selection?: {
    activeState: string;
    selectedNodeId?: string | null;
    layerUi: Record<string, LayerUiState>;
  } | null;
  mainScene: string;
};

type ProjectRecord = {
  id: string;
  name: string;
  path: string;
  pinned?: boolean;
  chats: ChatThread[];
};

type SidebarMenuState =
  | { kind: "project"; projectId: string }
  | { kind: "chat"; projectId: string; chatId: string }
  | null;

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

const STORAGE_KEY = "strut-studio-workspace-v4";
const BROWSER_SNAPSHOT_KEY = "strut-studio-saved-project-v1";

function createChat(projectId: string, title: string, messages: ChatMessage[] = []): ChatThread {
  return {
    id: `chat-${Date.now()}-${Math.round(Math.random() * 10000)}`,
    title,
    projectId,
    updated: nowStamp(),
    messages,
    references: [],
    document: null,
    activeState: "wave",
    selectedNodeId: null,
    layerUi: {},
    pendingOperation: null,
    operationHistory: [],
    operationBatches: [],
    undoStack: [],
    redoStack: [],
  };
}

const initialProjects: ProjectRecord[] = [];

const browserLocalAdapters: LocalAdapter[] = [
  { id: "ollama", name: "Ollama", kind: "local-model", command: "ollama", installed: false, detail: "desktop check required" },
  { id: "codex", name: "Codex", kind: "local-agent", command: "codex", installed: false, detail: "desktop check required" },
  { id: "gemini-cli", name: "Gemini CLI", kind: "local-agent", command: "gemini", installed: false, detail: "desktop check required" },
  { id: "claude-code", name: "Claude Code", kind: "local-agent", command: "claude / openclaude", installed: false, detail: "desktop check required" },
  { id: "opencode", name: "OpenCode", kind: "local-agent", command: "opencode-cli", installed: false, detail: "desktop check required" },
  { id: "cursor-agent", name: "Cursor Agent", kind: "local-agent", command: "cursor-agent", installed: false, detail: "desktop check required" },
  { id: "qwen", name: "Qwen Code", kind: "local-agent", command: "qwen", installed: false, detail: "desktop check required" },
  { id: "qoder", name: "Qoder CLI", kind: "local-agent", command: "qodercli", installed: false, detail: "desktop check required" },
  { id: "copilot-cli", name: "Copilot CLI", kind: "local-agent", command: "copilot", installed: false, detail: "desktop check required" },
  { id: "kiro", name: "Kiro", kind: "local-agent", command: "kiro-cli", installed: false, detail: "desktop check required" },
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
    .map((attachment): ReferenceAttachment => {
      const candidate = attachment as Partial<ReferenceAttachment>;
      return {
        id: typeof candidate.id === "string" && candidate.id ? candidate.id : `ref-${Date.now()}-${Math.random()}`,
        name: typeof candidate.name === "string" ? candidate.name : "reference image",
        kind: candidate.kind === "layer" ? "layer" : "image",
        mimeType: typeof candidate.mimeType === "string" ? candidate.mimeType : "image/png",
        dataUrl: typeof candidate.dataUrl === "string" ? candidate.dataUrl : "",
        size: typeof candidate.size === "number" ? candidate.size : 0,
        nodeId: typeof candidate.nodeId === "string" ? candidate.nodeId : undefined,
        nodeKind: typeof candidate.nodeKind === "string" ? candidate.nodeKind : undefined,
        nodeRole: typeof candidate.nodeRole === "string" ? candidate.nodeRole : undefined,
      };
    })
    .filter((attachment) => attachment.kind === "layer" || attachment.dataUrl?.startsWith("data:image/"));
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
        operationBatchId: typeof candidate.operationBatchId === "string" ? candidate.operationBatchId : undefined,
      };
    })
    .filter((message) => message.text.trim().length > 0 || (message.attachments?.length ?? 0) > 0);

  return messages;
}

function normalizeLayerUi(value: unknown): Record<string, LayerUiState> {
  if (!value || typeof value !== "object") {
    return {};
  }

  return Object.fromEntries(
    Object.entries(value as Record<string, Partial<LayerUiState>>)
      .filter(([nodeId]) => nodeId.trim().length > 0)
      .map(([nodeId, state]) => [
        nodeId,
        {
          visible: typeof state.visible === "boolean" ? state.visible : true,
          locked: typeof state.locked === "boolean" ? state.locked : false,
        },
      ]),
  );
}

function isOperationSourceType(value: unknown): value is OperationSourceType {
  return value === "ai" || value === "sprite-python" || value === "manual" || value === "cli";
}

function isOperationBatchStatus(value: unknown): value is OperationBatchStatus {
  return value === "pending" || value === "applied" || value === "rejected" || value === "undone";
}

function normalizeValidationResult(value: unknown): OperationValidationResult {
  if (!value || typeof value !== "object") {
    return {
      ok: false,
      message: "Migrated preview has not been validated for persistence",
      validator: "strut-studio-migration",
      validatedAt: "migration",
    };
  }
  const candidate = value as Partial<OperationValidationResult>;
  return {
    ok: Boolean(candidate.ok),
    message: typeof candidate.message === "string" ? candidate.message : "Validation state unavailable",
    validator: typeof candidate.validator === "string" ? candidate.validator : "strut-studio",
    validatedAt: typeof candidate.validatedAt === "string" ? candidate.validatedAt : "migration",
  };
}

function normalizeOperationRecord(value: unknown): OperationRecord | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const candidate = value as Partial<OperationRecord>;
  if (candidate.type === "set_property") {
    const operation = candidate as Partial<SetPropertyOperation>;
    if (
      typeof operation.id === "string"
      && typeof operation.targetId === "string"
      && typeof operation.targetName === "string"
      && typeof operation.property === "string"
    ) {
      return {
        id: operation.id,
        type: "set_property",
        targetId: operation.targetId,
        targetName: operation.targetName,
        property: operation.property,
        previousValue: operation.previousValue,
        value: operation.value,
      };
    }
  }
  if (candidate.type === "replace_document") {
    const operation = candidate as Partial<ReplaceDocumentOperation>;
    if (typeof operation.id === "string" && isStrutDocument(operation.nextDocument)) {
      return {
        id: operation.id,
        type: "replace_document",
        previousDocument: isStrutDocument(operation.previousDocument) ? operation.previousDocument : null,
        nextDocument: operation.nextDocument,
      };
    }
  }
  return null;
}

function normalizeOperationPreview(value: unknown): OperationBatch | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const candidate = value as Partial<OperationBatch>;
  if (
    typeof candidate.id !== "string"
    || typeof candidate.targetId !== "string"
    || typeof candidate.targetName !== "string"
    || typeof candidate.intent !== "string"
  ) {
    return null;
  }

  const operationType =
    candidate.operationType === "style.patch" || candidate.operationType === "transform.patch" || candidate.operationType === "timeline.patch"
      ? candidate.operationType
      : "style.patch";

  return {
    id: candidate.id,
    targetId: candidate.targetId,
    targetName: candidate.targetName,
    intent: candidate.intent,
    operationType,
    affectedProperties: Array.isArray(candidate.affectedProperties)
      ? candidate.affectedProperties.filter((item): item is string => typeof item === "string")
      : [],
    createdAt: typeof candidate.createdAt === "string" ? candidate.createdAt : "now",
    sourceType: isOperationSourceType(candidate.sourceType) ? candidate.sourceType : "manual",
    status: isOperationBatchStatus(candidate.status) ? candidate.status : "pending",
    validationResult: normalizeValidationResult(candidate.validationResult),
    documentRevisionId: typeof candidate.documentRevisionId === "string" ? candidate.documentRevisionId : "rev-migrated-local-state",
    previousDocumentRevisionId: typeof candidate.previousDocumentRevisionId === "string" ? candidate.previousDocumentRevisionId : null,
    prompt: typeof candidate.prompt === "string" ? candidate.prompt : candidate.intent,
    sourceMetadata: candidate.sourceMetadata && typeof candidate.sourceMetadata === "object" ? candidate.sourceMetadata as Record<string, unknown> : { migratedFrom: "operationPreview" },
    operations: Array.isArray(candidate.operations)
      ? candidate.operations.map(normalizeOperationRecord).filter((item): item is OperationRecord => item !== null)
      : [],
    updatedAt: typeof candidate.updatedAt === "string" ? candidate.updatedAt : "migration",
    appliedAt: typeof candidate.appliedAt === "string" ? candidate.appliedAt : null,
    rejectedAt: typeof candidate.rejectedAt === "string" ? candidate.rejectedAt : null,
  };
}

function normalizeOperationHistory(value: unknown): OperationBatch[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.map(normalizeOperationPreview).filter((item): item is OperationBatch => item !== null);
}

function normalizeOperationBatches(value: unknown, fallback: unknown): OperationBatch[] {
  const batches = normalizeOperationHistory(value);
  if (batches.length) {
    return batches;
  }
  return normalizeOperationHistory(fallback);
}

function normalizeStringList(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
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
              const operationBatches = normalizeOperationBatches(chatCandidate.operationBatches, chatCandidate.operationHistory);
              return {
                id: typeof chatCandidate.id === "string" && chatCandidate.id ? chatCandidate.id : `chat-${Date.now()}-${Math.random()}`,
                title: typeof chatCandidate.title === "string" && chatCandidate.title ? chatCandidate.title : "Untitled chat",
                projectId: id,
                updated: typeof chatCandidate.updated === "string" ? chatCandidate.updated : "now",
                pinned: typeof chatCandidate.pinned === "boolean" ? chatCandidate.pinned : false,
                messages: normalizeMessages(chatCandidate.messages),
                references: normalizeAttachments(chatCandidate.references),
                document: isStrutDocument(chatCandidate.document) ? chatCandidate.document : null,
                activeState: typeof chatCandidate.activeState === "string" ? chatCandidate.activeState : "wave",
                selectedNodeId: typeof chatCandidate.selectedNodeId === "string" ? chatCandidate.selectedNodeId : null,
                layerUi: normalizeLayerUi(chatCandidate.layerUi),
                pendingOperation: normalizeOperationPreview(chatCandidate.pendingOperation),
                operationHistory: operationBatches,
                operationBatches,
                undoStack: normalizeStringList(chatCandidate.undoStack),
                redoStack: normalizeStringList(chatCandidate.redoStack),
              };
            })
        : [];

      return {
        id,
        name: typeof candidate.name === "string" && candidate.name ? candidate.name : "Untitled project",
        path: typeof candidate.path === "string" ? candidate.path : "D:\\Strut Projects",
        pinned: typeof candidate.pinned === "boolean" ? candidate.pinned : false,
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
    return "New motion chat";
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
        kind: "image",
        mimeType: file.type || "image/png",
        dataUrl,
        size: file.size,
      });
    };
    reader.onerror = () => reject(new Error(`Could not read ${file.name}`));
    reader.readAsDataURL(file);
  });
}

function layerToAttachment(layer: StrutNode): ReferenceAttachment {
  return {
    id: `layer-${layer.id}`,
    name: layer.name,
    kind: "layer",
    mimeType: "application/x-strut-layer",
    size: 0,
    nodeId: layer.id,
    nodeKind: layer.kind,
    nodeRole: layer.role,
  };
}

function layerReferencePrompt(attachments: ReferenceAttachment[]) {
  const layerAttachments = attachments.filter((attachment) => attachment.kind === "layer");
  if (!layerAttachments.length) {
    return "";
  }
  return `\n\nTarget layers selected from the scene: ${layerAttachments
    .map((attachment) =>
      `${attachment.name} (${attachment.nodeId ?? "unknown id"}, ${attachment.nodeKind ?? "layer"}${
        attachment.nodeRole ? `, role: ${attachment.nodeRole}` : ""
      })`,
    )
    .join("; ")}. Use these as edit context.`;
}

function uniqueAttachments(attachments: ReferenceAttachment[]) {
  const seen = new Set<string>();
  return attachments.filter((attachment) => {
    const key = attachment.kind === "layer" ? `layer:${attachment.nodeId ?? attachment.name}` : `image:${attachment.id}`;
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

function titleCase(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function renderInlineMarkdown(value: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  const pattern = /(`[^`]+`|\*\*[^*]+\*\*|\*[^*]+\*)/g;
  let cursor = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(value)) !== null) {
    if (match.index > cursor) {
      nodes.push(value.slice(cursor, match.index));
    }

    const token = match[0];
    if (token.startsWith("`")) {
      nodes.push(<code key={`${match.index}-code`}>{token.slice(1, -1)}</code>);
    } else if (token.startsWith("**")) {
      nodes.push(<strong key={`${match.index}-strong`}>{token.slice(2, -2)}</strong>);
    } else {
      nodes.push(<em key={`${match.index}-em`}>{token.slice(1, -1)}</em>);
    }
    cursor = pattern.lastIndex;
  }

  if (cursor < value.length) {
    nodes.push(value.slice(cursor));
  }

  return nodes;
}

function MarkdownResponse({ text }: { text: string }) {
  const blocks = text
    .split(/\n{2,}/)
    .map((block) => block.trim())
    .filter(Boolean);

  if (blocks.length === 0) {
    return null;
  }

  return (
    <div className="markdown-response">
      {blocks.map((block, blockIndex) => {
        const lines = block.split(/\n/).map((line) => line.trim()).filter(Boolean);
        if (lines.length > 0 && lines.every((line) => line.startsWith("- "))) {
          return (
            <ul key={`list-${blockIndex}`}>
              {lines.map((line, lineIndex) => (
                <li key={`${blockIndex}-${lineIndex}`}>{renderInlineMarkdown(line.slice(2))}</li>
              ))}
            </ul>
          );
        }

        return (
          <p key={`paragraph-${blockIndex}`}>
            {lines.map((line, lineIndex) => (
              <Fragment key={`${blockIndex}-${lineIndex}`}>
                {lineIndex > 0 ? <br /> : null}
                {renderInlineMarkdown(line)}
              </Fragment>
            ))}
          </p>
        );
      })}
    </div>
  );
}

function ChatMessageView({ compact = false, message }: { compact?: boolean; message: ChatMessage }) {
  const roleLabel = message.role === "assistant" ? "Strut" : titleCase(message.role);

  return (
    <div className={`message ${compact ? "compact-message" : ""} ${message.role}`}>
      <span className="message-role">{roleLabel}</span>
      <div className="message-body">
        {message.role === "user" ? <span className="message-text">{message.text}</span> : <MarkdownResponse text={message.text} />}
        {message.operationBatchId ? <span className="message-batch-link">Batch {message.operationBatchId}</span> : null}
        {message.attachments?.length ? (
          <span className="message-attachments">
            {message.attachments.map((attachment) => (
              <span className={`message-attachment ${attachment.kind === "layer" ? "layer-attachment" : ""}`} key={attachment.id}>
                {attachment.kind === "layer" ? <Layers3 size={13} /> : <img src={attachment.dataUrl} alt="" />}
                <em>{attachment.kind === "layer" ? `Layer: ${attachment.name}` : attachment.name}</em>
              </span>
            ))}
          </span>
        ) : null}
      </div>
    </div>
  );
}

function flattenNodes(nodes: StrutNode[]): StrutNode[] {
  return nodes.flatMap((node) => [node, ...flattenNodes(node.children ?? [])]);
}

function layerUiFor(layerUi: Record<string, LayerUiState>, nodeId: string): LayerUiState {
  return layerUi[nodeId] ?? { visible: true, locked: false };
}

function nowStamp() {
  return new Date().toISOString();
}

function relativeTimeLabel(value: string, nowMs = Date.now()) {
  const parsed = Date.parse(value);
  if (!Number.isFinite(parsed)) {
    return value === "now" ? "now" : value;
  }
  const seconds = Math.max(0, Math.floor((nowMs - parsed) / 1000));
  if (seconds < 60) {
    return "now";
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h`;
  }
  const days = Math.floor(hours / 24);
  if (days < 7) {
    return `${days}d`;
  }
  const weeks = Math.floor(days / 7);
  if (weeks < 5) {
    return `${weeks}w`;
  }
  const months = Math.floor(days / 30);
  if (months < 12) {
    return `${Math.max(1, months)}mo`;
  }
  return `${Math.max(1, Math.floor(days / 365))}y`;
}

function promptIntent(value: string, hasReferences = false): "chat" | "generate" {
  const prompt = value.trim().toLowerCase();
  if (hasReferences && prompt.length === 0) {
    return "generate";
  }
  const generationWords = [
    "generate",
    "create",
    "make",
    "build",
    "animate",
    "motion",
    "loader",
    "logo",
    "mascot",
    "icon",
    "badge",
    "dice",
    "svg",
    "scene",
    "draw",
    "design",
  ];
  if (generationWords.some((word) => prompt.includes(word))) {
    return "generate";
  }
  const chatWords = [
    "who are you",
    "what are you",
    "explain",
    "brainstorm",
    "ideate",
    "should i",
    "how would",
    "what do you think",
    "help me think",
    "plan",
  ];
  if (prompt.endsWith("?") || chatWords.some((word) => prompt.includes(word))) {
    return "chat";
  }
  return "chat";
}

function localChatFallback(prompt: string) {
  const value = prompt.trim().toLowerCase();
  if (value.includes("who are you") || value.includes("what are you")) {
    return "I'm Strut's animation design assistant. I can chat through ideas, help plan edits, and when you ask for motion, turn the direction into validated editable Strut scenes.";
  }
  if (value.includes("brainstorm") || value.includes("ideate")) {
    return "Let's brainstorm before generating. A good Strut animation direction usually needs three choices: the subject, the emotional pace, and the editable parts you want to control. Tell me the object or UI moment, and I can suggest a few motion routes.";
  }
  return "I can talk through the idea first. Ask me for direction, critique, or options; when you're ready to create motion, use words like generate, animate, create, or make.";
}

function documentRevisionId(document: StrutDocument | null) {
  if (!document) {
    return "rev-empty";
  }
  const artboard = document.artboards[0];
  const layerCount = artboard ? flattenNodes(artboard.nodes).length : 0;
  return `rev-${slugToken(document.name)}-${layerCount}-${document.timelines.length}-${document.state_machines.length}`;
}

function slugToken(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "untitled";
}

function findNodeById(nodes: StrutNode[], nodeId: string): StrutNode | null {
  for (const node of nodes) {
    if (node.id === nodeId) {
      return node;
    }
    const child = findNodeById(node.children ?? [], nodeId);
    if (child) {
      return child;
    }
  }
  return null;
}

function setNodeProperty(node: StrutNode, property: string, value: unknown): StrutNode {
  if (property.startsWith("style.")) {
    const key = property.slice(6) as keyof NonNullable<StrutNode["style"]>;
    return { ...node, style: { ...(node.style ?? {}), [key]: value } };
  }
  if (property.startsWith("transform.")) {
    const key = property.slice(10) as keyof NonNullable<StrutNode["transform"]>;
    return { ...node, transform: { ...(node.transform ?? {}), [key]: value } };
  }
  return node;
}

function updateNodeById(nodes: StrutNode[], nodeId: string, updater: (node: StrutNode) => StrutNode): StrutNode[] {
  return nodes.map((node) => {
    if (node.id === nodeId) {
      return updater(node);
    }
    if (node.children?.length) {
      return { ...node, children: updateNodeById(node.children, nodeId, updater) };
    }
    return node;
  });
}

function validateOperationBatch(document: StrutDocument | null, batch: OperationBatch): OperationValidationResult {
  const validatedAt = nowStamp();
  if (!document && !batch.operations.some((operation) => operation.type === "replace_document")) {
    return {
      ok: false,
      message: "No validated document is open for this operation batch",
      validator: "strut-studio-browser",
      validatedAt,
    };
  }
  if (!batch.operations.length) {
    return {
      ok: false,
      message: "Operation batch has no operations to apply",
      validator: "strut-studio-browser",
      validatedAt,
    };
  }
  const nodes = document?.artboards[0]?.nodes ?? [];
  for (const operation of batch.operations) {
    if (operation.type === "replace_document") {
      if (!isStrutDocument(operation.nextDocument)) {
        return {
          ok: false,
          message: "Replacement operation does not contain a valid Strut document",
          validator: "strut-studio-browser",
          validatedAt,
        };
      }
      continue;
    }
    const node = findNodeById(nodes, operation.targetId);
    if (!node) {
      return {
        ok: false,
        message: `Operation targets missing node ${operation.targetId}`,
        validator: "strut-studio-browser",
        validatedAt,
      };
    }
    if (!/^style\.(fill|stroke|opacity|stroke_width)$|^transform\.(translate_x|translate_y|rotate|scale_x|scale_y)$/.test(operation.property)) {
      return {
        ok: false,
        message: `Unsupported operation property ${operation.property}`,
        validator: "strut-studio-browser",
        validatedAt,
      };
    }
  }
  return {
    ok: true,
    message: "Operation batch validated against the current Strut document",
    validator: "strut-studio-browser",
    validatedAt,
  };
}

function applyOperationBatch(document: StrutDocument | null, batch: OperationBatch, direction: "apply" | "undo"): StrutDocument | null {
  if (!batch.validationResult.ok) {
    return document;
  }
  if (!document && !batch.operations.some((operation) => operation.type === "replace_document")) {
    return document;
  }
  let nextDocument = document;
  for (const operation of batch.operations) {
    if (operation.type === "replace_document") {
      nextDocument = direction === "apply" ? operation.nextDocument : operation.previousDocument;
      continue;
    }
    if (!nextDocument) {
      return null;
    }
    const value = direction === "apply" ? operation.value : operation.previousValue;
    nextDocument = {
      ...nextDocument,
      artboards: nextDocument.artboards.map((artboard, index) =>
        index === 0
          ? { ...artboard, nodes: updateNodeById(artboard.nodes, operation.targetId, (node) => setNodeProperty(node, operation.property, value)) }
          : artboard,
      ),
    };
  }
  return nextDocument;
}

function createGenerationBatch(
  result: GeneratedCharacter,
  previousDocument: StrutDocument | null,
  prompt: string,
  sourceType: OperationSourceType,
): OperationBatch {
  const timestamp = nowStamp();
  const batch: OperationBatch = {
    id: `batch-${sourceType}-${documentRevisionId(result.document)}-${Date.now()}`,
    targetId: result.document.id,
    targetName: result.document.name,
    intent: prompt,
    operationType: "timeline.patch",
    affectedProperties: ["document"],
    createdAt: timestamp,
    sourceType,
    status: "applied",
    validationResult: {
      ok: true,
      message: "Generated document was validated by Rust before Studio persistence",
      validator: "strut-studio-rust",
      validatedAt: timestamp,
    },
    previousDocumentRevisionId: documentRevisionId(previousDocument),
    documentRevisionId: documentRevisionId(result.document),
    prompt,
    sourceMetadata: {
      provider: result.source,
      subjectClassification: result.planSummary?.subjectClassification,
      subjectLabel: result.planSummary?.subjectLabel,
      operationCount: result.operationCount,
    },
    operations: [{
      id: `op-replace-${Date.now()}`,
      type: "replace_document",
      previousDocument,
      nextDocument: result.document,
    }],
    updatedAt: timestamp,
    appliedAt: timestamp,
    rejectedAt: null,
  };
  return batch;
}

function latestPreviewForProject(project: ProjectRecord | null, activeChatId: string | null) {
  if (!project) {
    return null;
  }
  const activeChat = project.chats.find((chat) => chat.id === activeChatId) ?? null;
  if (activeChat?.document) {
    return {
      activeState: activeChat.activeState,
      chatId: activeChat.id,
      document: activeChat.document,
      inherited: false,
    };
  }
  const previewChat = project.chats.find((chat) => chat.document);
  if (!previewChat?.document) {
    return null;
  }
  return {
    activeState: previewChat.activeState,
    chatId: previewChat.id,
    document: previewChat.document,
    inherited: true,
  };
}

function documentSummary(document: StrutDocument | null) {
  if (!document) {
    return "No current document";
  }
  const artboard = document.artboards[0];
  const layerCount = artboard ? flattenNodes(artboard.nodes).length : 0;
  const states = document.state_machines[0]?.states.join(", ") || "no states";
  return `${document.name}; ${layerCount} editable layers; states: ${states}`;
}

function CharacterPreview({
  activeState,
  document,
  layerUi,
  onSelectNode,
  selectedNodeId,
}: {
  activeState: string;
  document: StrutDocument;
  layerUi?: Record<string, LayerUiState>;
  onSelectNode?: (nodeId: string) => void;
  selectedNodeId?: string | null;
}) {
  const artboard = document.artboards[0] ?? emptyArtboard;
  const width = artboard.width || 960;
  const height = artboard.height || 540;

  return (
    <svg
      className="character-preview"
      data-character={artboard.name}
      data-state={activeState}
      data-testid="character-preview"
      viewBox={`0 0 ${width} ${height}`}
      role="img"
    >
      <style>{documentAnimationCss(document, activeState)}</style>
      <rect className="preview-bg" width={width} height={height} rx="18" />
      <g className={`document-scene state-${activeState}`}>
        {artboard.nodes.map((node) => (
          <StrutNodePreview
            key={node.id}
            layerUi={layerUi ?? {}}
            node={node}
            onSelectNode={onSelectNode}
            selectedNodeId={selectedNodeId}
          />
        ))}
      </g>
      <text className="state-label" x={width / 2} y={height - 24} textAnchor="middle">{titleCase(activeState)}</text>
    </svg>
  );
}

function StrutNodePreview({
  layerUi,
  node,
  onSelectNode,
  selectedNodeId,
}: {
  layerUi: Record<string, LayerUiState>;
  node: StrutNode;
  onSelectNode?: (nodeId: string) => void;
  selectedNodeId?: string | null;
}) {
  const ui = layerUiFor(layerUi, node.id);
  const selected = selectedNodeId === node.id;
  if (!ui.visible) {
    return null;
  }

  const common = {
    "data-node-id": node.id,
    "data-node-name": node.name,
    "data-selected": selected ? "true" : undefined,
    "data-locked": ui.locked ? "true" : undefined,
    className: `strut-node selectable-node node-${cssIdent(node.name)} kind-${cssIdent(node.kind)} ${selected ? "selected" : ""} ${ui.locked ? "locked" : ""}`,
    transform: transformAttribute(node.transform),
    style: nodeStyle(node.style),
    onClick: (event: MouseEvent<SVGGElement>) => {
      event.stopPropagation();
      if (!ui.locked) {
        onSelectNode?.(node.id);
      }
    },
  };
  const children = node.children?.map((child) => (
    <StrutNodePreview
      key={child.id}
      layerUi={layerUi}
      node={child}
      onSelectNode={onSelectNode}
      selectedNodeId={selectedNodeId}
    />
  ));
  return (
    <g {...common}>
      <StrutShape node={node} />
      {selected ? <SelectionHalo node={node} /> : null}
      {children}
    </g>
  );
}

function StrutShape({ node }: { node: StrutNode }) {
  const shape = node.shape ?? { type: "none" };
  if (node.kind === "group" || shape.type === "none") {
    return null;
  }
  if (shape.type === "rect") {
    return <rect x={shape.x} y={shape.y} width={shape.width} height={shape.height} rx={shape.rx} />;
  }
  if (shape.type === "ellipse") {
    return <ellipse cx={shape.cx} cy={shape.cy} rx={shape.rx} ry={shape.ry} />;
  }
  if (shape.type === "path") {
    return <path d={shape.d} />;
  }
  if (shape.type === "text") {
    return <text x={shape.x} y={shape.y} fontSize={shape.size}>{shape.value}</text>;
  }
  return null;
}

function SelectionHalo({ node }: { node: StrutNode }) {
  const shape = node.shape ?? { type: "none" };
  if (shape.type === "rect") {
    return <rect className="node-selection-halo" x={shape.x - 6} y={shape.y - 6} width={shape.width + 12} height={shape.height + 12} rx={Math.max(shape.rx, 8)} />;
  }
  if (shape.type === "ellipse") {
    return <ellipse className="node-selection-halo" cx={shape.cx} cy={shape.cy} rx={shape.rx + 7} ry={shape.ry + 7} />;
  }
  if (shape.type === "path") {
    return <path className="node-selection-halo" d={shape.d} />;
  }
  if (shape.type === "text") {
    return <rect className="node-selection-halo" x={shape.x - 8} y={shape.y - shape.size - 7} width={Math.max(shape.value.length * shape.size * 0.58, 24) + 16} height={shape.size + 14} rx={7} />;
  }
  const bounds = nodeBounds(node);
  if (!bounds) {
    return null;
  }
  return <rect className="node-selection-halo group-halo" x={bounds.x - 10} y={bounds.y - 10} width={bounds.width + 20} height={bounds.height + 20} rx={12} />;
}

function nodeBounds(node: StrutNode): { x: number; y: number; width: number; height: number } | null {
  const shape = node.shape ?? { type: "none" };
  if (shape.type === "rect") {
    return { x: shape.x, y: shape.y, width: shape.width, height: shape.height };
  }
  if (shape.type === "ellipse") {
    return { x: shape.cx - shape.rx, y: shape.cy - shape.ry, width: shape.rx * 2, height: shape.ry * 2 };
  }
  if (shape.type === "text") {
    return { x: shape.x, y: shape.y - shape.size, width: Math.max(shape.value.length * shape.size * 0.58, 24), height: shape.size };
  }
  const childBounds = (node.children ?? []).map(nodeBounds).filter((bounds): bounds is NonNullable<ReturnType<typeof nodeBounds>> => Boolean(bounds));
  if (!childBounds.length) {
    return null;
  }
  const minX = Math.min(...childBounds.map((bounds) => bounds.x));
  const minY = Math.min(...childBounds.map((bounds) => bounds.y));
  const maxX = Math.max(...childBounds.map((bounds) => bounds.x + bounds.width));
  const maxY = Math.max(...childBounds.map((bounds) => bounds.y + bounds.height));
  return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
}

function nodeStyle(style: StrutNode["style"]): CSSProperties {
  return {
    fill: style?.fill ?? undefined,
    stroke: style?.stroke ?? undefined,
    strokeWidth: style?.stroke_width,
    opacity: style?.opacity,
    strokeLinecap: style?.linecap as CSSProperties["strokeLinecap"],
    strokeLinejoin: style?.linejoin as CSSProperties["strokeLinejoin"],
  };
}

function transformAttribute(transform: StrutNode["transform"]) {
  if (!transform) {
    return undefined;
  }
  const parts = [];
  if (transform.translate_x || transform.translate_y) parts.push(`translate(${transform.translate_x ?? 0} ${transform.translate_y ?? 0})`);
  if (transform.rotate) parts.push(`rotate(${transform.rotate})`);
  if (transform.scale_x !== undefined || transform.scale_y !== undefined) parts.push(`scale(${transform.scale_x ?? 1} ${transform.scale_y ?? 1})`);
  return parts.length ? parts.join(" ") : undefined;
}

function documentAnimationCss(document: StrutDocument, activeState: string) {
  const timelines = timelinesForState(document, activeState);
  const transforms = nodeTransformMap(document);
  return timelines
    .flatMap((timeline) => [
      timelineAnimationCss(timeline, transforms),
      ...stateTimelineCss(timeline, transforms),
    ])
    .filter(Boolean)
    .join("\n");
}

function timelinesForState(document: StrutDocument, activeState: string) {
  const machine = document.state_machines[0];
  const timelineNames = new Set([activeState]);
  machine?.transitions?.filter((transition) => transition.to === activeState).forEach((transition) => timelineNames.add(transition.timeline));
  if (activeState === "float") timelineNames.add("idle_float");
  return document.timelines.filter((timeline) => timelineNames.has(timeline.name));
}

function activeStateFromTimeline(timelineName: string) {
  return timelineName === "idle_float" ? "float" : timelineName;
}

type TimelineTrack = NonNullable<Timeline["tracks"]>[number];
type NumericTimelineKeyframe = TimelineTrack["keyframes"][number] & { value: { type: "number"; value: number } };
type ResolvedTransform = Required<NonNullable<StrutNode["transform"]>>;

function timelineAnimationCss(timeline: Timeline, transforms: Map<string, StrutNode["transform"]>) {
  return Array.from(timelineTrackGroups(timeline).entries())
    .flatMap(([target, tracks]) => [
      transformTracksCss(timeline, target, tracks.filter((track) => isTransformProperty(track.property)), transforms.get(target)),
      ...tracks.filter((track) => isScalarProperty(track.property)).map((track) => scalarTrackCss(timeline, track)),
    ])
    .filter(Boolean)
    .join("\n");
}

function transformTracksCss(
  timeline: Timeline,
  target: string,
  tracks: TimelineTrack[],
  baseTransform: StrutNode["transform"],
) {
  if (!tracks.length) {
    return "";
  }
  const times = sortedTimelineTimes(timeline, tracks);
  const frames = times
    .map((time) => {
      const percent = Math.max(0, Math.min(100, (time / timeline.duration_ms) * 100));
      const base = normalizeTransform(baseTransform);
      const tx = base.translate_x + trackValue(tracks, "translation.x", time, 0);
      const ty = base.translate_y + trackValue(tracks, "translation.y", time, 0);
      const rotate = base.rotate + trackValue(tracks, "rotation", time, 0);
      const scale = trackValue(tracks, "scale", time, 1);
      const sx = base.scale_x * scale * trackValue(tracks, "scale.x", time, 1);
      const sy = base.scale_y * scale * trackValue(tracks, "scale.y", time, 1);
      return `${percent}% { transform: translate(${round(tx)}px, ${round(ty)}px) rotate(${round(rotate)}deg) scale(${round(sx)}, ${round(sy)}); }`;
    })
    .join("\n");
  return `@keyframes ${transformAnimationName(timeline, target)} { ${frames} }`;
}

function scalarTrackCss(timeline: Timeline, track: TimelineTrack) {
  const numericKeyframes = numericTrackKeyframes(track);
  if (numericKeyframes.length < 2) {
    return "";
  }
  const frames = numericKeyframes
    .map((keyframe) => {
      const percent = Math.max(0, Math.min(100, (keyframe.time_ms / timeline.duration_ms) * 100));
      return `${percent}% { ${track.property}: ${round(Number(keyframe.value.value))}; }`;
    })
    .join("\n");
  return `@keyframes ${scalarAnimationName(timeline, track)} { ${frames} }`;
}

function stateTimelineCss(timeline: Timeline, transforms: Map<string, StrutNode["transform"]>) {
  return Array.from(timelineTrackGroups(timeline).entries())
    .map(([target, tracks]) => {
      const animations = [
        tracks.some((track) => isTransformProperty(track.property))
          ? `${transformAnimationName(timeline, target)} ${timeline.duration_ms}ms ${groupEasing(tracks)} infinite`
          : "",
        ...tracks
          .filter((track) => isScalarProperty(track.property))
          .map((track) => `${scalarAnimationName(timeline, track)} ${timeline.duration_ms}ms ${cssEasing(track.keyframes[0]?.easing ?? "linear")} infinite`),
      ].filter(Boolean);
      if (!animations.length) {
        return "";
      }
      const stateName = activeStateFromTimeline(timeline.name);
      const base = transforms.get(target);
      const baseRule = tracks.some((track) => isTransformProperty(track.property))
        ? ` transform: ${transformCss(normalizeTransform(base))};`
        : "";
      return `
.document-scene.state-${cssIdent(timeline.name)} [data-node-id="${target}"],
.document-scene.state-${cssIdent(stateName)} [data-node-id="${target}"] {
  transform-box: fill-box;
  transform-origin: center;
  ${baseRule}
  animation: ${animations.join(", ")};
}`;
    })
    .filter(Boolean);
}

function timelineTrackGroups(timeline: Timeline) {
  const groups = new Map<string, TimelineTrack[]>();
  for (const track of timeline.tracks ?? []) {
    if (!hasNumericMotion(track) || (!isTransformProperty(track.property) && !isScalarProperty(track.property))) {
      continue;
    }
    groups.set(track.target, [...(groups.get(track.target) ?? []), track]);
  }
  return groups;
}

function sortedTimelineTimes(timeline: Timeline, tracks: TimelineTrack[]) {
  return Array.from(
    new Set([
      0,
      timeline.duration_ms,
      ...tracks.flatMap((track) => numericTrackKeyframes(track).map((keyframe) => keyframe.time_ms)),
    ]),
  ).sort((a, b) => a - b);
}

function trackValue(tracks: TimelineTrack[], property: string, time: number, fallback: number) {
  const track = tracks.find((candidate) => candidate.property === property);
  return track ? interpolatedTrackValue(track, time, fallback) : fallback;
}

function interpolatedTrackValue(track: TimelineTrack, time: number, fallback: number) {
  const keyframes = numericTrackKeyframes(track).sort((a, b) => a.time_ms - b.time_ms);
  if (!keyframes.length) {
    return fallback;
  }
  if (time <= keyframes[0].time_ms) {
    return Number(keyframes[0].value.value);
  }
  const last = keyframes[keyframes.length - 1];
  if (time >= last.time_ms) {
    return Number(last.value.value);
  }
  for (let index = 0; index < keyframes.length - 1; index += 1) {
    const left = keyframes[index];
    const right = keyframes[index + 1];
    if (time >= left.time_ms && time <= right.time_ms) {
      const span = Math.max(1, right.time_ms - left.time_ms);
      const progress = (time - left.time_ms) / span;
      return Number(left.value.value) + (Number(right.value.value) - Number(left.value.value)) * progress;
    }
  }
  return fallback;
}

function numericTrackKeyframes(track: TimelineTrack): NumericTimelineKeyframe[] {
  return track.keyframes.filter((keyframe): keyframe is NumericTimelineKeyframe => keyframe.value.type === "number");
}

function hasNumericMotion(track: TimelineTrack) {
  return numericTrackKeyframes(track).length > 1;
}

function isTransformProperty(property: string) {
  return ["translation.x", "translation.y", "rotation", "scale", "scale.x", "scale.y"].includes(property);
}

function isScalarProperty(property: string) {
  return property === "opacity";
}

function groupEasing(tracks: TimelineTrack[]) {
  return cssEasing(tracks[0]?.keyframes[0]?.easing ?? "linear");
}

function cssEasing(easing: TimelineTrack["keyframes"][number]["easing"]) {
  if (easing === "ease_in") return "ease-in";
  if (easing === "ease_out") return "ease-out";
  if (easing === "ease_in_out") return "ease-in-out";
  return "linear";
}

function nodeTransformMap(document: StrutDocument) {
  const transforms = new Map<string, StrutNode["transform"]>();
  const visit = (node: StrutNode) => {
    transforms.set(node.id, node.transform ?? {});
    for (const child of node.children ?? []) {
      visit(child);
    }
  };
  for (const artboard of document.artboards) {
    for (const node of artboard.nodes) {
      visit(node);
    }
  }
  return transforms;
}

function normalizeTransform(transform: StrutNode["transform"]): ResolvedTransform {
  return {
    translate_x: transform?.translate_x ?? 0,
    translate_y: transform?.translate_y ?? 0,
    rotate: transform?.rotate ?? 0,
    scale_x: transform?.scale_x ?? 1,
    scale_y: transform?.scale_y ?? 1,
  };
}

function transformCss(transform: ResolvedTransform) {
  return `translate(${round(transform.translate_x)}px, ${round(transform.translate_y)}px) rotate(${round(transform.rotate)}deg) scale(${round(transform.scale_x)}, ${round(transform.scale_y)})`;
}

function transformAnimationName(timeline: Timeline, target: string) {
  return `studio-${cssIdent(timeline.name)}-${cssIdent(target)}-transform`;
}

function scalarAnimationName(timeline: Timeline, track: TimelineTrack) {
  return `studio-${cssIdent(timeline.name)}-${cssIdent(track.target)}-${cssIdent(track.property)}`;
}

function round(value: number) {
  return Number(value.toFixed(4));
}

function cssIdent(value: string) {
  return value.replace(/[^a-zA-Z0-9_-]/g, "-");
}

function App() {
  const [initialWorkspace] = useState<WorkspaceState>(() => loadWorkspaceState());
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [, setStatus] = useState<StudioStatus | null>(null);
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
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [pendingReferences, setPendingReferences] = useState<ReferenceAttachment[]>([]);
  const [providerMode, setProviderMode] = useState<ProviderMode>("local");
  const [localAdapters, setLocalAdapters] = useState<LocalAdapter[]>(browserLocalAdapters);
  const [selectedLocalAdapterId, setSelectedLocalAdapterId] = useState("codex");
  const [selectedByokProviderId, setSelectedByokProviderId] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [providerEndpoint, setProviderEndpoint] = useState(byokProviders[0].endpoint);
  const [providerModel, setProviderModel] = useState(byokProviders[0].model);
  const [activity, setActivity] = useState("Select a real local CLI, Ollama, or BYOK provider");
  const [runState, setRunState] = useState<RunState>("idle");
  const [themeMode, setThemeMode] = useState<ThemeMode>(initialWorkspace.themeMode);
  const [collapsedProjectIds, setCollapsedProjectIds] = useState<Set<string>>(() => new Set());
  const [clockTick, setClockTick] = useState(Date.now());
  const [sidebarMenu, setSidebarMenu] = useState<SidebarMenuState>(null);
  const [topbarMenu, setTopbarMenu] = useState<SidebarMenuState>(null);
  const [composerToolsOpen, setComposerToolsOpen] = useState(true);
  const [layersRailCollapsed, setLayersRailCollapsed] = useState(false);

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
    const timer = window.setInterval(() => setClockTick(Date.now()), 30_000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (localAdapters.some((adapter) => adapter.id === selectedLocalAdapterId)) {
      return;
    }
    const preferred = localAdapters.find((adapter) => adapter.id === "codex") ?? localAdapters[0];
    if (preferred) {
      setSelectedLocalAdapterId(preferred.id);
    }
  }, [localAdapters, selectedLocalAdapterId]);

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

  useEffect(() => {
    function closeTransientPanels(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setSearchOpen(false);
        setNewProjectOpen(false);
      }
    }

    window.addEventListener("keydown", closeTransientPanels);
    return () => window.removeEventListener("keydown", closeTransientPanels);
  }, []);

  const activeProject = projects.find((project) => project.id === activeProjectId) ?? null;
  const activeChat = activeProject?.chats.find((chat) => chat.id === activeChatId) ?? null;
  const projectPreview = latestPreviewForProject(activeProject, activeChatId);
  const currentDocument = projectPreview?.document ?? null;
  const currentActiveState = projectPreview?.activeState ?? "wave";
  const activeArtboard = currentDocument?.artboards[0] ?? emptyArtboard;
  const activeMachine = currentDocument?.state_machines[0] ?? emptyMachine;
  const layers = useMemo(() => flattenNodes(activeArtboard.nodes), [activeArtboard.nodes]);
  const selectedNodeId = activeChat?.selectedNodeId ?? null;
  const layerUi = activeChat?.layerUi ?? {};
  const operationBatches = activeChat?.operationBatches ?? activeChat?.operationHistory ?? [];
  const persistedLayerReferences = activeChat?.references.filter((reference) => reference.kind === "layer") ?? [];
  const composerReferences = uniqueAttachments([...persistedLayerReferences, ...pendingReferences]);
  const undoStack = activeChat?.undoStack ?? [];
  const redoStack = activeChat?.redoStack ?? [];
  const selectedLayer = layers.find((layer) => layer.id === selectedNodeId) ?? null;
  const activeLocalAdapter = localAdapters.find((adapter) => adapter.id === selectedLocalAdapterId) ?? localAdapters[0] ?? browserLocalAdapters[0];
  const activeByokProvider = byokProviders.find((provider) => provider.id === selectedByokProviderId) ?? byokProviders[0];
  const activeProviderLabel = providerMode === "local" ? activeLocalAdapter.name : activeByokProvider.name;
  const activeProviderType = providerMode === "local" ? "Local CLI" : "BYOK";
  const activeProviderDetail = providerMode === "local" ? activeLocalAdapter.detail : `${providerModel.trim() || activeByokProvider.model} / ${providerEndpoint.trim() || activeByokProvider.endpoint}`;
  const viewModes: ViewModeOption[] = [
    { id: "chat", Icon: MessageSquarePlus, label: "Chat only" },
    { id: "preview", Icon: PanelRight, label: "Chat + preview" },
  ];

  const filteredProjects = projects
    .map((project) => ({
      ...project,
      chats: project.chats.filter((chat) =>
        `${project.name} ${chat.title}`.toLowerCase().includes(searchQuery.toLowerCase()),
      ),
    }))
    .filter((project) => project.chats.length > 0 || project.name.toLowerCase().includes(searchQuery.toLowerCase()));
  const pinnedProjects = projects.filter((project) => project.pinned);
  const pinnedChats = projects.flatMap((project) =>
    project.chats.filter((chat) => chat.pinned).map((chat) => ({ project, chat })),
  );

  function toggleProjectCollapsed(projectId: string) {
    setCollapsedProjectIds((current) => {
      const next = new Set(current);
      if (next.has(projectId)) {
        next.delete(projectId);
      } else {
        next.add(projectId);
      }
      return next;
    });
  }

  function toggleProjectPinned(projectId: string) {
    setProjects((current) =>
      current.map((project) => (project.id === projectId ? { ...project, pinned: !project.pinned } : project)),
    );
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function toggleChatPinned(projectId: string, chatId: string) {
    updateChat(projectId, chatId, (chat) => ({ ...chat, pinned: !chat.pinned, updated: nowStamp() }));
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function renameProject(projectId: string) {
    const project = projects.find((item) => item.id === projectId);
    if (!project) {
      return;
    }
    const nextName = window.prompt("Rename project", project.name)?.trim();
    if (!nextName || nextName === project.name) {
      setSidebarMenu(null);
      setTopbarMenu(null);
      return;
    }
    setProjects((current) => current.map((item) => (item.id === projectId ? { ...item, name: nextName } : item)));
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function renameChat(projectId: string, chatId: string) {
    const chat = projects.find((project) => project.id === projectId)?.chats.find((item) => item.id === chatId);
    if (!chat) {
      return;
    }
    const nextTitle = window.prompt("Rename chat", chat.title)?.trim();
    if (!nextTitle || nextTitle === chat.title) {
      setSidebarMenu(null);
      setTopbarMenu(null);
      return;
    }
    updateChat(projectId, chatId, (item) => ({ ...item, title: nextTitle, updated: nowStamp() }));
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  useEffect(() => {
    if (selectedNodeId && !layers.some((layer) => layer.id === selectedNodeId)) {
      setSelectedNode(null);
    }
  }, [layers, selectedNodeId]);

  function setSelectedNode(nodeId: string | null) {
    updateCurrentChat((chat) => ({ ...chat, selectedNodeId: nodeId, updated: nowStamp() }));
  }

  function undoLastBatch() {
    const batchId = undoStack[0];
    const batch = operationBatches.find((item) => item.id === batchId);
    if (!batch || batch.status !== "applied") {
      setActivity("No applied operation batch is available to undo");
      return;
    }
    const nextDocument = applyOperationBatch(currentDocument, batch, "undo");
    if (!nextDocument) {
      setActivity("Undo could not run because no document is open");
      return;
    }
    const timestamp = nowStamp();
    const undone = { ...batch, status: "undone" as const, updatedAt: timestamp };
    updateCurrentChat((chat) => ({
      ...chat,
      document: nextDocument,
      operationBatches: [undone, ...(chat.operationBatches ?? []).filter((item) => item.id !== batch.id)],
      operationHistory: [undone, ...(chat.operationBatches ?? []).filter((item) => item.id !== batch.id)].slice(0, 12),
      undoStack: (chat.undoStack ?? []).filter((id) => id !== batch.id),
      redoStack: [batch.id, ...(chat.redoStack ?? [])],
      updated: nowStamp(),
    }));
    setActivity(`Undid batch ${batch.id}`);
  }

  function redoLastBatch() {
    const batchId = redoStack[0];
    const batch = operationBatches.find((item) => item.id === batchId);
    if (!batch) {
      setActivity("No operation batch is available to redo");
      return;
    }
    const reapplied = { ...batch, status: "applied" as const, validationResult: validateOperationBatch(currentDocument, batch) };
    if (!reapplied.validationResult.ok) {
      setActivity(reapplied.validationResult.message);
      return;
    }
    const nextDocument = applyOperationBatch(currentDocument, reapplied, "apply");
    if (!nextDocument) {
      setActivity("Redo could not run because no document is open");
      return;
    }
    const timestamp = nowStamp();
    const updated = { ...reapplied, documentRevisionId: documentRevisionId(nextDocument), updatedAt: timestamp, appliedAt: timestamp };
    updateCurrentChat((chat) => ({
      ...chat,
      document: nextDocument,
      operationBatches: [updated, ...(chat.operationBatches ?? []).filter((item) => item.id !== batch.id)],
      operationHistory: [updated, ...(chat.operationBatches ?? []).filter((item) => item.id !== batch.id)].slice(0, 12),
      undoStack: [batch.id, ...(chat.undoStack ?? [])],
      redoStack: (chat.redoStack ?? []).filter((id) => id !== batch.id),
      updated: nowStamp(),
    }));
    setActivity(`Redid batch ${batch.id}`);
  }

  function providerPayload(): GenerationProvider {
    if (providerMode === "local") {
      return { mode: "local", localAdapterId: activeLocalAdapter.id };
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

  function appendMessage(role: ChatMessage["role"], text: string, operationBatchId?: string) {
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      messages: [...chat.messages, { id: Date.now() + Math.random(), role, text, operationBatchId }],
    }));
  }

  function appendUserMessage(text: string, attachments: ReferenceAttachment[]) {
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      references: uniqueAttachments([...chat.references, ...attachments]),
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
    const chat = createChat(project.id, "New motion chat");
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
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function removeProject(projectId: string) {
    setProjects((current) => current.filter((project) => project.id !== projectId));
    if (activeProjectId === projectId) {
      setActiveProjectId(null);
      setActiveChatId(null);
    }
    setSidebarMenu(null);
    setTopbarMenu(null);
  }

  function setCurrentActiveState(state: string) {
    if (!activeProjectId) {
      return;
    }
    const ownerChatId = projectPreview?.chatId ?? activeChatId;
    if (!ownerChatId) {
      return;
    }
    updateChat(activeProjectId, ownerChatId, (chat) => ({ ...chat, activeState: state, updated: nowStamp() }));
  }

  function generationContext(): GenerationContext {
    const activeMessages = activeChat?.messages ?? [];
    const projectMessages = activeProject?.chats
      .filter((chat) => chat.id !== activeChatId)
      .flatMap((chat) => chat.messages.slice(-3).map((message) => ({ chatTitle: chat.title, message }))) ?? [];
    const chatHistory = [
      ...activeMessages.slice(-10).map((message) => ({
        role: message.role,
        text: message.text,
        attachments: message.attachments?.map((attachment) => attachment.name),
      })),
      ...projectMessages.slice(0, 6).map(({ chatTitle, message }) => ({
        role: message.role,
        text: `[${chatTitle}] ${message.text}`,
        attachments: message.attachments?.map((attachment) => attachment.name),
      })),
    ].filter((message) => message.text.trim() || (message.attachments?.length ?? 0) > 0);

    return {
      projectName: activeProject?.name,
      projectPath: activeProject?.path,
      activeChatTitle: activeChat?.title,
      currentDocumentSummary: selectedLayer
        ? `${documentSummary(currentDocument)}; selected part: ${selectedLayer.name} (${selectedLayer.id}, ${selectedLayer.kind})`
        : documentSummary(currentDocument),
      chatHistory,
      currentDocument: currentDocument ?? undefined,
    };
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
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      references: chat.references.filter((reference) => reference.id !== id),
    }));
  }

  function attachLayerReference(layer: StrutNode) {
    const attachment = layerToAttachment(layer);
    const isAlreadyAttached = composerReferences.some((reference) => reference.kind === "layer" && reference.nodeId === layer.id);
    if (isAlreadyAttached) {
      removePendingReference(attachment.id);
      setSelectedNode(selectedNodeId === layer.id ? null : selectedNodeId);
      setActivity(`Removed layer ${layer.name} from the next prompt`);
      return;
    }
    setSelectedNode(layer.id);
    setPendingReferences((current) =>
      current.some((reference) => reference.kind === "layer" && reference.nodeId === layer.id)
        ? current
        : [...current, attachment],
    );
    updateCurrentChat((chat) => ({
      ...chat,
      updated: nowStamp(),
      references: uniqueAttachments([...chat.references, attachment]),
    }));
    setActivity(`Attached layer ${layer.name} to the next prompt`);
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
        config: providerPayload().byok,
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
          ? await invoke<ProviderOperationResult>("test_local_adapter", { adapterId: activeLocalAdapter.id })
          : await invoke<ProviderOperationResult>("test_byok_provider", { config: providerPayload().byok });
      setActivity(result.status);
    } catch (error) {
      setActivity(String(error));
    }
  }

  async function openProjectFolder(project = activeProject) {
    if (!project) {
      setActivity("Select a project first");
      return;
    }
    if (!desktopRuntime) {
      setActivity("Desktop app required to open project folder");
      setSidebarMenu(null);
      setTopbarMenu(null);
      return;
    }
    try {
      await invoke("open_project_folder", { path: project.path });
      setActivity(`Opened ${project.name}`);
    } catch (error) {
      setActivity(String(error));
    } finally {
      setSidebarMenu(null);
      setTopbarMenu(null);
    }
  }

  async function saveActiveProject() {
    if (!activeProject || !currentDocument) {
      setActivity("Open a validated scene before saving");
      return;
    }

    if (!desktopRuntime) {
      window.localStorage.setItem(
        BROWSER_SNAPSHOT_KEY,
        JSON.stringify({
          projects,
          activeProjectId,
          activeChatId,
          themeMode,
          savedAt: nowStamp(),
        }),
      );
      setActivity(`Saved browser snapshot for ${activeProject.name}`);
      return;
    }

    try {
      const snapshot = await invoke<ProjectSnapshot>("save_project_snapshot", {
        projectPath: activeProject.path,
        projectName: activeProject.name,
        document: currentDocument,
        operationBatches,
        selection: {
          activeState: currentActiveState,
          selectedNodeId,
          layerUi,
        },
      });
      setProjects((current) =>
        current.map((project) =>
          project.id === activeProject.id
            ? { ...project, name: snapshot.project.name, path: snapshot.project.path }
            : project,
        ),
      );
      setActivity(`Saved ${snapshot.mainScene} with ${snapshot.operationBatches.length} operation batches`);
    } catch (error) {
      setActivity(`Save rejected: ${String(error)}`);
    }
  }

  async function loadActiveProject() {
    if (!activeProject) {
      setActivity("Select a project before loading");
      return;
    }

    if (!desktopRuntime) {
      const raw = window.localStorage.getItem(BROWSER_SNAPSHOT_KEY);
      if (!raw) {
        setActivity("No browser snapshot has been saved yet");
        return;
      }
      try {
        const parsed = JSON.parse(raw) as Partial<WorkspaceState>;
        const loaded = {
          projects: normalizeProjects(parsed.projects),
          activeProjectId: typeof parsed.activeProjectId === "string" ? parsed.activeProjectId : activeProjectId,
          activeChatId: typeof parsed.activeChatId === "string" ? parsed.activeChatId : activeChatId,
          themeMode: isThemeMode(parsed.themeMode) ? parsed.themeMode : themeMode,
        };
        setProjects(loaded.projects);
        setActiveProjectId(loaded.activeProjectId ?? null);
        setActiveChatId(loaded.activeChatId ?? null);
        setThemeMode(loaded.themeMode);
        setActivity("Reopened browser snapshot");
      } catch (error) {
        setActivity(`Browser snapshot rejected: ${String(error)}`);
      }
      return;
    }

    try {
      const snapshot = await invoke<ProjectSnapshot>("load_project_snapshot", { projectPath: activeProject.path });
      const activeId = activeChatId ?? `chat-${Date.now()}`;
      const loadedChat: ChatThread = {
        ...(activeChat ?? createChat(activeProject.id, "Loaded scene")),
        id: activeId,
        title: activeChat?.title ?? "Loaded scene",
        projectId: activeProject.id,
        updated: nowStamp(),
        document: snapshot.document,
        activeState: snapshot.selection?.activeState ?? snapshot.document.state_machines[0]?.states[0] ?? "idle",
        selectedNodeId: snapshot.selection?.selectedNodeId ?? null,
        layerUi: snapshot.selection?.layerUi ?? {},
        pendingOperation: null,
        operationBatches: snapshot.operationBatches,
        operationHistory: snapshot.operationBatches,
        undoStack: snapshot.operationBatches.filter((batch) => batch.status === "applied").map((batch) => batch.id),
        redoStack: [],
      };
      setProjects((current) =>
        current.map((project) =>
          project.id === activeProject.id
            ? {
                ...project,
                name: snapshot.project.name,
                path: snapshot.project.path,
                chats: project.chats.some((chat) => chat.id === activeId)
                  ? project.chats.map((chat) => (chat.id === activeId ? loadedChat : chat))
                  : [loadedChat, ...project.chats],
              }
            : project,
        ),
      );
      setActiveChatId(activeId);
      setActivity(`Loaded ${snapshot.mainScene} with ${snapshot.operationBatches.length} operation batches`);
    } catch (error) {
      setActivity(`Load rejected: ${String(error)}`);
    }
  }

  async function runGeneration() {
    const trimmed = prompt.trim();
    if (!trimmed && composerReferences.length === 0) {
      return;
    }
    if (runState !== "idle") {
      return;
    }
    if (!activeProjectId || !activeChatId) {
      newChat();
      setActivity("Start a chat first");
      return;
    }
    const references = composerReferences;
    const imageReferences = references.filter((reference) => reference.kind !== "layer" && reference.dataUrl?.startsWith("data:image/"));

    if (promptIntent(trimmed, references.length > 0) === "chat") {
      const chatPrompt = `${trimmed}${layerReferencePrompt(references)}`;
      appendUserMessage(trimmed, references);
      updateChat(activeProjectId, activeChatId, (chat) => ({
        ...chat,
        title: chat.title === "New motion chat" || chat.title === "New character chat" || chat.title === "Project brief" ? promptTitle(trimmed || "Chat") : chat.title,
        updated: nowStamp(),
      }));
      setPrompt("");
      setPendingReferences([]);
      setActivity("Thinking");
      setRunState("thinking");
      if (!desktopRuntime) {
        appendMessage("assistant", localChatFallback(trimmed));
        setActivity("Answered in chat mode");
        setRunState("idle");
        return;
      }
      try {
        const answer = await invoke<ChatAnswer>("chat_with_provider", {
          prompt: chatPrompt,
          provider: providerPayload(),
          context: generationContext(),
        });
        appendMessage("assistant", answer.message || localChatFallback(trimmed));
        setActivity(`Answered through ${answer.source}`);
      } catch (error) {
        appendMessage("assistant", `${localChatFallback(trimmed)}\n\n_Provider chat was unavailable: ${String(error)}_`);
        setActivity("Answered locally; provider chat unavailable");
      } finally {
        setRunState("idle");
      }
      return;
    }

    const generationPrompt = `${trimmed || "Use the attached reference image to create an editable Strut motion document."}${layerReferencePrompt(references)}`;
    appendUserMessage(trimmed || "Use the attached reference image.", references);
    updateChat(activeProjectId, activeChatId, (chat) => ({
      ...chat,
      title: chat.title === "New motion chat" || chat.title === "New character chat" || chat.title === "Project brief" ? promptTitle(trimmed || references[0]?.name || "Reference motion") : chat.title,
      updated: nowStamp(),
    }));
    setPendingReferences([]);
    setActivity("Generating");
    setRunState("generating");

    try {
      if (!desktopRuntime) {
        throw new Error("Desktop app required for real generation. Run the Tauri app and connect a local CLI, Ollama, or BYOK provider.");
      }
      const args = { prompt: generationPrompt, provider: providerPayload(), references: imageReferences, context: generationContext() };
      const result = await invoke<GeneratedCharacter>("generate_character", args);
      const generationBatch = createGenerationBatch(result, currentDocument, generationPrompt, "ai");
      updateChat(activeProjectId, activeChatId, (chat) => ({
        ...chat,
        title: chat.title === "New motion chat" || chat.title === "New character chat" || chat.title === "Project brief" ? promptTitle(trimmed || references[0]?.name || "Reference motion") : chat.title,
        updated: nowStamp(),
        document: result.document,
        activeState: result.document.state_machines[0]?.states.includes("wave") ? "wave" : "idle",
        operationBatches: [generationBatch, ...(chat.operationBatches ?? [])],
        operationHistory: [generationBatch, ...(chat.operationBatches ?? [])].slice(0, 12),
        undoStack: [generationBatch.id, ...(chat.undoStack ?? [])],
        redoStack: [],
      }));
      const generatedPartSummary = result.planSummary?.partNames.length
        ? result.planSummary.partNames.slice(0, 6).join(", ")
        : "validated document layers";
      const generatedTimelineSummary = result.planSummary?.timelineNames.length
        ? result.planSummary.timelineNames.join(", ")
        : result.document.timelines.map((timeline) => timeline.name).join(", ");
      setActivity(`${result.source}: ${result.message}`);
      appendMessage(
        "assistant",
        `**${result.document.name} is ready.**\n\nProvider: ${activeProviderLabel}\n\nSubject: ${result.planSummary?.subjectLabel ?? "validated Strut document"} (${result.planSummary?.subjectClassification ?? "fallback"})\n\nOperations: ${result.operationCount ?? 0} validated before conversion\n\nParts: ${generatedPartSummary}\n\nTimelines: ${generatedTimelineSummary}\n\nI ${currentDocument ? "updated" : "created"} editable layers, states, timelines, bindings, and events.`,
        generationBatch.id,
      );
    } catch (error) {
      setActivity(String(error));
      appendMessage("assistant", `**Generation stopped**\n\nProvider: ${activeProviderLabel}\n\n${String(error)}`);
    } finally {
      setRunState("idle");
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
          <button type="button" onClick={() => setSearchOpen(true)}>
            <Search size={16} />
            Search
          </button>
          <button type="button" onClick={() => setMainPanel("providers")}>
            <Cpu size={16} />
            Providers
          </button>
        </div>

        <div className="project-list">
          {pinnedProjects.length || pinnedChats.length ? (
            <div className="pinned-list">
              <span className="section-label">Pinned</span>
              {pinnedProjects.map((project) => (
                <button
                  aria-label={`Pinned project ${project.name}`}
                  className="pinned-row"
                  key={`project-${project.id}`}
                  type="button"
                  onClick={() => openProject(project.id)}
                >
                  <Folder size={14} />
                  <span>{project.name}</span>
                </button>
              ))}
              {pinnedChats.map(({ project, chat }) => (
                <button
                  aria-label={`Pinned chat ${chat.title}`}
                  className="pinned-row"
                  key={`chat-${chat.id}`}
                  type="button"
                  onClick={() => openChat(project.id, chat.id)}
                >
                  <MessageSquarePlus size={14} />
                  <span>{chat.title}</span>
                </button>
              ))}
            </div>
          ) : null}
          <span className="section-label">Projects</span>
          {projects.map((project) => {
            const isCollapsed = collapsedProjectIds.has(project.id);
            const projectMenuOpen = sidebarMenu?.kind === "project" && sidebarMenu.projectId === project.id;
            return (
              <div className="project-group" key={project.id}>
                <div
                  className="project-button"
                  onContextMenu={(event) => {
                    event.preventDefault();
                    setSidebarMenu({ kind: "project", projectId: project.id });
                  }}
                >
                  <button
                    aria-expanded={!isCollapsed}
                    className="project-open"
                    type="button"
                    onClick={() => {
                      openProject(project.id);
                      toggleProjectCollapsed(project.id);
                    }}
                  >
                    <ChevronRight className={isCollapsed ? "" : "expanded"} size={14} />
                    <Folder size={15} />
                    <span>{project.name}</span>
                  </button>
                  <div className="project-actions">
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
                      aria-label={`Project options for ${project.name}`}
                      className="inline-menu"
                      type="button"
                      onClick={(event) => {
                        event.stopPropagation();
                        setSidebarMenu(projectMenuOpen ? null : { kind: "project", projectId: project.id });
                      }}
                    >
                      <MoreHorizontal size={14} />
                    </button>
                  </div>
                  {projectMenuOpen ? (
                    <div className="sidebar-menu" role="menu">
                      <button role="menuitem" type="button" onClick={() => toggleProjectPinned(project.id)}>
                        <Pin size={14} />
                        {project.pinned ? "Unpin project" : "Pin project"}
                      </button>
                      <button role="menuitem" type="button" onClick={() => void openProjectFolder(project)}>
                        <FolderOpen size={14} />
                        Open in Explorer
                      </button>
                      <button role="menuitem" type="button" onClick={() => renameProject(project.id)}>
                        <Pencil size={14} />
                        Rename project
                      </button>
                      <button role="menuitem" type="button" onClick={() => removeProject(project.id)}>
                        <Trash2 size={14} />
                        Delete project
                      </button>
                    </div>
                  ) : null}
                </div>
                {!isCollapsed ? project.chats.map((chat) => {
                  const chatMenuOpen = sidebarMenu?.kind === "chat" && sidebarMenu.projectId === project.id && sidebarMenu.chatId === chat.id;
                  return (
                  <div
                    className={chat.id === activeChatId ? "chat-row active" : "chat-row"}
                    key={chat.id}
                    onContextMenu={(event) => {
                      event.preventDefault();
                      setSidebarMenu({ kind: "chat", projectId: project.id, chatId: chat.id });
                    }}
                  >
                    <button
                      className="chat-link"
                      type="button"
                      onClick={() => openChat(project.id, chat.id)}
                    >
                      <span>{chat.title}</span>
                      <em>{relativeTimeLabel(chat.updated, clockTick)}</em>
                    </button>
                    <button
                      aria-label={`Chat options for ${chat.title}`}
                      className="chat-menu-button"
                      type="button"
                      onClick={(event) => {
                        event.stopPropagation();
                        setSidebarMenu(chatMenuOpen ? null : { kind: "chat", projectId: project.id, chatId: chat.id });
                      }}
                    >
                      <MoreHorizontal size={13} />
                    </button>
                    {chatMenuOpen ? (
                      <div className="sidebar-menu chat-menu" role="menu">
                        <button role="menuitem" type="button" onClick={() => toggleChatPinned(project.id, chat.id)}>
                          <Pin size={14} />
                          {chat.pinned ? "Unpin chat" : "Pin chat"}
                        </button>
                        <button role="menuitem" type="button" onClick={() => renameChat(project.id, chat.id)}>
                          <Pencil size={14} />
                          Rename chat
                        </button>
                        <button role="menuitem" type="button" onClick={() => deleteChat(project.id, chat.id)}>
                          <Trash2 size={14} />
                          Delete chat
                        </button>
                      </div>
                    ) : null}
                  </div>
                );}) : null}
              </div>
            );
          })}
        </div>

        <div className="sidebar-footer">
          <button type="button" onClick={() => setMainPanel("settings")}>
            <Settings2 size={16} />
            Settings
          </button>
        </div>
      </aside>

      <section className="workspace">
        <header className="workspace-top">
          <div className="workspace-context">
            <strong data-testid="workspace-title">{activeChat?.title ?? activeProject?.name ?? "Home"}</strong>
            {activeChat && activeProject ? (
              <button
                aria-label={`Title options for ${activeChat.title}`}
                className="title-menu-button"
                type="button"
                onClick={() => setTopbarMenu(topbarMenu?.kind === "chat" && topbarMenu.chatId === activeChat.id ? null : { kind: "chat", projectId: activeProject.id, chatId: activeChat.id })}
              >
                <MoreHorizontal size={15} />
              </button>
            ) : activeProject ? (
              <button
                aria-label={`Title options for ${activeProject.name}`}
                className="title-menu-button"
                type="button"
                onClick={() => setTopbarMenu(topbarMenu?.kind === "project" && topbarMenu.projectId === activeProject.id ? null : { kind: "project", projectId: activeProject.id })}
              >
                <MoreHorizontal size={15} />
              </button>
            ) : null}
            {activeChat && activeProject && topbarMenu?.kind === "chat" && topbarMenu.chatId === activeChat.id ? (
              <div className="topbar-menu" role="menu">
                <button role="menuitem" type="button" onClick={() => toggleChatPinned(activeProject.id, activeChat.id)}>
                  <Pin size={14} />
                  {activeChat.pinned ? "Unpin chat" : "Pin chat"}
                </button>
                <button role="menuitem" type="button" onClick={() => renameChat(activeProject.id, activeChat.id)}>
                  <Pencil size={14} />
                  Rename chat
                </button>
                <button role="menuitem" type="button" onClick={() => deleteChat(activeProject.id, activeChat.id)}>
                  <Trash2 size={14} />
                  Delete chat
                </button>
              </div>
            ) : activeProject && topbarMenu?.kind === "project" && topbarMenu.projectId === activeProject.id ? (
              <div className="topbar-menu" role="menu">
                <button role="menuitem" type="button" onClick={() => toggleProjectPinned(activeProject.id)}>
                  <Pin size={14} />
                  {activeProject.pinned ? "Unpin project" : "Pin project"}
                </button>
                <button role="menuitem" type="button" onClick={() => renameProject(activeProject.id)}>
                  <Pencil size={14} />
                  Rename project
                </button>
                <button role="menuitem" type="button" onClick={() => removeProject(activeProject.id)}>
                  <Trash2 size={14} />
                  Delete project
                </button>
              </div>
            ) : null}
            <span className="sr-status" data-testid="activity-pill">{activity}</span>
          </div>
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
        </header>

        {searchOpen ? (
          <div className="modal-backdrop" role="presentation" onMouseDown={() => setSearchOpen(false)}>
            <section className="modal-panel search-modal" role="dialog" aria-modal="true" aria-label="Search" onMouseDown={(event) => event.stopPropagation()}>
              <div className="modal-heading">
                <div>
                  <h2>Search</h2>
                  <p>Find a project or chat without changing the sidebar.</p>
                </div>
                <button aria-label="Close search" type="button" onClick={() => setSearchOpen(false)}>
                  <X size={16} />
                </button>
              </div>
              <label className="modal-search-field">
                <Search size={15} />
                <input aria-label="Search projects and chats" autoFocus value={searchQuery} onChange={(event) => setSearchQuery(event.currentTarget.value)} placeholder="Search projects or chats" />
              </label>
              <div className="search-results" aria-label="Search results">
                {filteredProjects.length ? (
                  filteredProjects.map((project) => (
                    <div className="search-project" key={project.id}>
                      <button type="button" onClick={() => {
                        openProject(project.id);
                        setSearchOpen(false);
                      }}>
                        <Folder size={14} />
                        <span>{project.name}</span>
                      </button>
                      {project.chats.map((chat) => (
                        <button key={chat.id} type="button" onClick={() => {
                          openChat(project.id, chat.id);
                          setSearchOpen(false);
                        }}>
                          <MessageSquarePlus size={14} />
                          <span>{chat.title}</span>
                          <em>{relativeTimeLabel(chat.updated, clockTick)}</em>
                        </button>
                      ))}
                    </div>
                  ))
                ) : (
                  <p className="panel-empty">No projects or chats match this search.</p>
                )}
              </div>
            </section>
          </div>
        ) : null}

        {newProjectOpen ? (
          <div className="modal-backdrop" role="presentation" onMouseDown={() => setNewProjectOpen(false)}>
            <section className="modal-panel project-sheet" role="dialog" aria-modal="true" aria-label="New project panel" onMouseDown={(event) => event.stopPropagation()}>
              <div className="modal-heading">
                <div>
                  <h2>New project</h2>
                  <p>Choose where Strut should create the editable scene files.</p>
                </div>
                <button aria-label="Close new project" type="button" onClick={() => setNewProjectOpen(false)}>
                  <X size={16} />
                </button>
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
          </div>
        ) : null}

        {mainPanel === "providers" ? (
          <section className="provider-page">
            <div className="page-heading">
              <h1>Providers</h1>
              <p>Select the model or coding agent Strut should use for chat and generation.</p>
            </div>
            <div className="provider-header-card">
              <div className="provider-summary" data-testid="selected-provider-summary">
                <span>Selected</span>
                <strong>{activeProviderLabel}</strong>
                <em>{activeProviderType}</em>
                <p>{activeProviderDetail}</p>
              </div>
              <div className="provider-tabs" role="group" aria-label="Provider source">
                {["local", "byok"].map((mode) => (
                  <button
                    aria-pressed={providerMode === mode}
                    className={providerMode === mode ? "active" : ""}
                    key={mode}
                    type="button"
                    onClick={() => setProviderMode(mode as ProviderMode)}
                  >
                    {mode === "local" ? "Local" : "BYOK"}
                  </button>
                ))}
              </div>
            </div>
            {providerMode === "local" ? (
              <div className="provider-list" aria-label="Local providers">
                {localAdapters.map((adapter) => (
                  <button
                    aria-pressed={selectedLocalAdapterId === adapter.id}
                    className={`${selectedLocalAdapterId === adapter.id ? "active" : ""} ${adapter.installed ? "installed" : "missing"}`}
                    key={adapter.id}
                    type="button"
                    onClick={() => setSelectedLocalAdapterId(adapter.id)}
                  >
                    <span className="provider-row-main">
                      <strong>{adapter.name}</strong>
                      <em>{adapter.kind}</em>
                    </span>
                    <span className="provider-row-meta">
                      <strong>{selectedLocalAdapterId === adapter.id ? "Selected" : adapter.installed ? "Ready" : "Not found"}</strong>
                      <em>{adapter.detail}</em>
                    </span>
                  </button>
                ))}
              </div>
            ) : null}
            {providerMode === "byok" ? (
              <div className="byok-form">
                <label>
                  <span>Provider</span>
                  <select aria-label="BYOK provider" value={selectedByokProviderId} onChange={(event) => {
                    const provider = byokProviders.find((item) => item.id === event.currentTarget.value) ?? byokProviders[0];
                    setSelectedByokProviderId(provider.id);
                    setProviderEndpoint(provider.endpoint);
                    setProviderModel(provider.model);
                  }}>
                    {byokProviders.map((provider) => <option key={provider.id} value={provider.id}>{provider.name}</option>)}
                  </select>
                </label>
                <label>
                  <span>API key</span>
                  <input aria-label={`${activeByokProvider.name} API key`} placeholder={activeByokProvider.env} type="password" value={apiKey} onChange={(event) => setApiKey(event.currentTarget.value)} />
                </label>
                <label>
                  <span>Base URL</span>
                  <input aria-label={`${activeByokProvider.name} base URL`} value={providerEndpoint} onChange={(event) => setProviderEndpoint(event.currentTarget.value)} />
                </label>
                <label>
                  <span>Model</span>
                  <input aria-label={`${activeByokProvider.name} model`} value={providerModel} onChange={(event) => setProviderModel(event.currentTarget.value)} />
                </label>
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
                  <p>Choose the provider Strut should use for motion documents.</p>
                </div>
                <div className="settings-controls">
                  <label>
                    <span>Generation mode</span>
                    <select aria-label="Generation mode" value={providerMode} onChange={(event) => setProviderMode(event.currentTarget.value as ProviderMode)}>
                      <option value="local">Local CLI</option>
                      <option value="byok">BYOK provider</option>
                    </select>
                  </label>
                  <div className="status-line">
                    <span>Current provider</span>
                    <strong>{activeProviderLabel}</strong>
                    <span>{activeProviderType}</span>
                    <em>{activity}</em>
                  </div>
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

        {mainPanel === "chat" && activeChat ? (
          <section className={viewMode === "preview" ? `chat-layout with-preview ${layersRailCollapsed ? "layers-collapsed" : ""}` : "chat-layout"}>
            <div className="chat-panel">
              <div className="message-stack">
                {activeChat.messages.length === 0 ? (
                  <div className="home-heading">
                    <h1>What motion should Strut build?</h1>
                    <p>Animate a logo, SVG, loader, product state, storyboard, mascot, or full scene.</p>
                  </div>
                ) : null}
                {activeChat.messages.map((message) => <ChatMessageView key={message.id} message={message} />)}
              </div>
              <div className="composer">
                {composerReferences.length ? (
                  <div className="reference-tray">
                    {composerReferences.map((reference) => (
                      <div className={`reference-chip ${reference.kind === "layer" ? "layer-reference-chip" : ""}`} key={reference.id}>
                        {reference.kind === "layer" ? <Layers3 size={14} /> : <img src={reference.dataUrl} alt="" />}
                        <span>{reference.kind === "layer" ? `Layer: ${reference.name}` : reference.name}</span>
                        <button aria-label={`Remove reference ${reference.name}`} type="button" onClick={() => removePendingReference(reference.id)}>
                          <X size={13} />
                        </button>
                      </div>
                    ))}
                  </div>
                ) : null}
                <div className="prompt-examples" aria-label="Prompt examples">
                  {["Quiet loader", "Soft logo", "Button state", "Calm mascot", "State badge", "Tiny success"].map((example) => (
                    <button key={example} type="button" onClick={() => setPrompt((current) => current || `Make a ${example.toLowerCase()} as an editable Strut animation`)}>
                      {example}
                    </button>
                  ))}
                </div>
                <div className="composer-toolbar" aria-label="Composer tools">
                  <button aria-expanded={composerToolsOpen} type="button" onClick={() => setComposerToolsOpen((isOpen) => !isOpen)}>
                    <MoreHorizontal size={15} />
                    Tools
                  </button>
                  {composerToolsOpen ? (
                    <div className="composer-tool-actions">
                      <button aria-label="Reload" disabled={!activeProject} title="Reload project" type="button" onClick={() => void loadActiveProject()}>
                        <RefreshCw size={15} />
                      </button>
                      <button aria-label="Save project" disabled={!activeProject || !currentDocument} title="Save project" type="button" onClick={() => void saveActiveProject()}>
                        <Save size={15} />
                      </button>
                      <button aria-label="Undo" disabled={!undoStack.length} title="Undo" type="button" onClick={undoLastBatch}>
                        <RotateCcw size={15} />
                      </button>
                      <button aria-label="Redo" disabled={!redoStack.length} title="Redo" type="button" onClick={redoLastBatch}>
                        <RotateCw size={15} />
                      </button>
                      <button aria-label={`Provider ${activeProviderLabel}`} className="provider-composer-button" type="button" onClick={() => setMainPanel("providers")}>
                        <Cpu size={15} />
                        {activeProviderLabel}
                      </button>
                    </div>
                  ) : null}
                  {runState !== "idle" ? (
                    <div className="generation-loader" role="status" aria-live="polite">
                      <span aria-hidden="true" />
                      <strong>{activeProviderLabel}</strong>
                      <em>{runState === "thinking" ? "thinking" : "generating"}</em>
                    </div>
                  ) : null}
                </div>
                <textarea aria-label="Motion prompt" value={prompt} onChange={(event) => setPrompt(event.currentTarget.value)} placeholder="Ask Strut for calm, low-energy motion for a logo, SVG, UI state, icon, mascot, storyboard, or scene" />
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
                  </div>
                  <button aria-label="Generate" disabled={runState !== "idle"} type="button" onClick={() => void runGeneration()}>
                    <Send size={17} />
                  </button>
                </div>
              </div>
            </div>
            {viewMode === "preview" ? (
              <div className="preview-area">
                <PreviewPane activeMachine={activeMachine} activeState={currentActiveState} document={currentDocument} setActiveState={setCurrentActiveState} />
                <LayerRail
                  collapsed={layersRailCollapsed}
                  layers={layers}
                  onAttachLayer={attachLayerReference}
                  onToggleCollapsed={() => setLayersRailCollapsed((isCollapsed) => !isCollapsed)}
                  pendingReferences={composerReferences}
                  selectedNodeId={selectedNodeId}
                />
              </div>
            ) : null}
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
        <p>Select a folder, open a project chat, or ask Strut to sketch a logo, SVG, UI state, mascot, storyboard, or full animation.</p>
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
          <em>Start from a prompt, reference, or rough direction before generating motion.</em>
        </button>
        <button type="button" onClick={onOpenProviders}>
          <span>Connect providers</span>
          <em>Choose a real local CLI, Ollama, or BYOK model.</em>
        </button>
      </div>
    </section>
  );
}

function LayerRail({
  collapsed,
  layers,
  onAttachLayer,
  onToggleCollapsed,
  pendingReferences,
  selectedNodeId,
}: {
  collapsed: boolean;
  layers: StrutNode[];
  onAttachLayer: (layer: StrutNode) => void;
  onToggleCollapsed: () => void;
  pendingReferences: ReferenceAttachment[];
  selectedNodeId: string | null;
}) {
  const attachedLayerIds = new Set(
    pendingReferences
      .filter((reference) => reference.kind === "layer")
      .map((reference) => reference.nodeId)
      .filter(Boolean),
  );

  return (
    <aside className={`layers-rail ${collapsed ? "collapsed" : ""}`} aria-label="Scene layers rail">
      <button
        aria-label={collapsed ? "Expand layers" : "Collapse layers"}
        className="layers-rail-toggle"
        type="button"
        onClick={onToggleCollapsed}
      >
        <Layers3 size={16} />
        {collapsed ? null : <span>Layers</span>}
      </button>
      {collapsed ? null : (
        <>
          <div className="layers-rail-heading">
            <strong>Scene layers</strong>
            <em>{layers.length ? `${layers.length} AI-named` : "No scene"}</em>
          </div>
          {layers.length ? (
            <div className="layer-attach-list">
              {layers.map((layer) => {
                const isAttached = attachedLayerIds.has(layer.id);
                return (
                  <button
                    aria-label={`${isAttached ? "Remove" : "Attach"} layer ${layer.name} ${layer.kind}`}
                    aria-pressed={isAttached}
                    className={`${selectedNodeId === layer.id ? "active" : ""} ${isAttached ? "attached" : ""}`}
                    key={layer.id}
                    type="button"
                    onClick={() => onAttachLayer(layer)}
                  >
                    <span>{layer.name}</span>
                    <em>{layer.role ?? layer.kind}</em>
                    {isAttached ? <strong>Attached</strong> : <strong>Add</strong>}
                  </button>
                );
              })}
            </div>
          ) : (
            <p className="panel-empty">No editable layers yet.</p>
          )}
        </>
      )}
    </aside>
  );
}

function PreviewPane({
  activeMachine,
  activeState,
  document,
  layerUi,
  onSelectNode,
  selectedNodeId,
  selectedTargetLabel,
  setActiveState,
  showSelectionAffordances = false,
}: {
  activeMachine: StateMachine;
  activeState: string;
  document: StrutDocument | null;
  layerUi?: Record<string, LayerUiState>;
  onSelectNode?: (nodeId: string | null) => void;
  selectedNodeId?: string | null;
  selectedTargetLabel?: string;
  setActiveState: (state: string) => void;
  showSelectionAffordances?: boolean;
}) {
  return (
    <aside className={showSelectionAffordances ? "preview-pane selection-aware" : "preview-pane"}>
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
      <div className="preview-stage">
        {document ? (
          <CharacterPreview
            activeState={activeState}
            document={document}
            layerUi={layerUi}
            onSelectNode={onSelectNode ? (nodeId) => onSelectNode(nodeId) : undefined}
            selectedNodeId={selectedNodeId}
          />
        ) : (
          <div className="preview-empty">
            <ImagePlus size={26} />
            <strong>No scene yet</strong>
            <span>Attach a reference or describe a logo, SVG, UI state, mascot, storyboard, or scene.</span>
          </div>
        )}
      </div>
      {document ? (
        <div className="state-row">
          {activeMachine.states.map((state) => (
            <button className={state === activeState ? "active" : ""} key={state} type="button" onClick={() => setActiveState(state)}>
              <Route size={13} />
              {titleCase(state)}
            </button>
          ))}
        </div>
      ) : null}
      {showSelectionAffordances ? (
        <div className="preview-edit-hint">
          <strong>{selectedTargetLabel ?? "No selection"}</strong>
          <span>{selectedNodeId ? "Preview selection is bound to the semantic scene node." : "Select a visible part or layer to target AI edits."}</span>
        </div>
      ) : null}
    </aside>
  );
}

export default App;
