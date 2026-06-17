/**
 * Strut Studio — shared domain types.
 *
 * Extracted from the original monolithic App.tsx so that services,
 * components, and utilities can import only the shapes they need.
 */

/* ------------------------------------------------------------------ */
/*  Studio runtime                                                     */
/* ------------------------------------------------------------------ */

export type StudioStatus = {
  format_version: string;
};

/* ------------------------------------------------------------------ */
/*  Document model                                                     */
/* ------------------------------------------------------------------ */

export type StrutNode = {
  id: string;
  name: string;
  kind: string;
  role?: string;
  transform?: {
    translate_x?: number;
    translate_y?: number;
    rotate?: number;
    rotate_x?: number;
    rotate_y?: number;
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
    | { type: "text"; x: number; y: number; value: string; size: number }
    | { type: "sprite"; url: string; frame_width: number; frame_height: number; columns: number; rows: number };
  children?: StrutNode[];
};

export type Artboard = {
  id: string;
  name: string;
  width: number;
  height: number;
  nodes: StrutNode[];
};

export type Timeline = {
  id: string;
  name: string;
  duration_ms: number;
  loops?: boolean;
  tracks?: Array<{
    target: string;
    property: string;
    keyframes: Array<{
      time_ms: number;
      value: { type: "number"; value: number } | { type: string; value: unknown };
      easing: "linear" | "ease_in" | "ease_out" | "ease_in_out" | "steps";
    }>;
  }>;
};

export type StateMachine = {
  id: string;
  name: string;
  inputs?: Array<{ name: string; kind: string }>;
  states: string[];
  transitions?: Array<{ from: string; to: string; on: string; timeline: string }>;
};

export type StrutDocument = {
  id: string;
  name: string;
  artboards: Artboard[];
  timelines: Timeline[];
  state_machines: StateMachine[];
  bindings: Array<{ name: string }>;
  events: Array<{ name: string }>;
};

/* ------------------------------------------------------------------ */
/*  Project model                                                      */
/* ------------------------------------------------------------------ */

export type ProjectFile = {
  name: string;
  path: string;
  kind: string;
};

export type ProjectInfo = {
  name: string;
  path: string;
  files: ProjectFile[];
};

/* ------------------------------------------------------------------ */
/*  Navigation & view state                                            */
/* ------------------------------------------------------------------ */

export type ProviderMode = "local" | "byok";
export type ViewMode = "chat" | "preview";
export type MainPanel = "chat" | "settings";
export type ThemeMode = "system" | "light" | "dark";
export type RunState = "idle" | "thinking" | "generating";

/* ------------------------------------------------------------------ */
/*  Provider models                                                    */
/* ------------------------------------------------------------------ */

export type LocalAdapter = {
  id: string;
  name: string;
  kind: string;
  command?: string | null;
  installed: boolean;
  detail: string;
};

export type ByokProvider = {
  id: string;
  name: string;
  env: string;
  endpoint: string;
  model: string;
};

export type ProviderOperationResult = {
  ok: boolean;
  status: string;
  detail: string;
};

export type GenerationProvider = {
  mode: ProviderMode;
  localAdapterId?: string;
  byok?: {
    providerId: string;
    apiKey?: string;
    endpoint: string;
    model: string;
  };
};

/* ------------------------------------------------------------------ */
/*  Generation / chat                                                  */
/* ------------------------------------------------------------------ */

export type GenerationContext = {
  projectName?: string;
  projectPath?: string;
  activeChatTitle?: string;
  responseMode?: "chat" | "preview";
  currentDocumentSummary?: string;
  chatHistory: Array<{
    role: ChatMessage["role"];
    text: string;
    attachments?: string[];
  }>;
  currentDocument?: StrutDocument;
};

export type AssistantResult =
  | {
      kind: "chat";
      message: string;
      source: string;
    }
  | {
      kind: "document_created";
      message: string;
      source: string;
      document: StrutDocument;
      activeState?: string;
      planSummary?: {
        subjectClassification: string;
        subjectLabel: string;
        partNames: string[];
        timelineNames: string[];
      } | null;
      operationCount?: number | null;
    }
  | {
      kind: "document_updated";
      message: string;
      source: string;
      document: StrutDocument;
      activeState?: string;
      changedAnimation?: string;
      planSummary?: {
        subjectClassification: string;
        subjectLabel: string;
        partNames: string[];
        timelineNames: string[];
      } | null;
      operationCount?: number | null;
    };

/* ------------------------------------------------------------------ */
/*  References & attachments                                           */
/* ------------------------------------------------------------------ */

export type ReferenceAttachment = {
  id: string;
  name: string;
  kind?: "image" | "layer" | "animation";
  mimeType: string;
  dataUrl?: string;
  size: number;
  nodeId?: string;
  nodeKind?: string;
  nodeRole?: string;
  animationId?: string;
  documentId?: string;
};

/* ------------------------------------------------------------------ */
/*  Chat thread                                                        */
/* ------------------------------------------------------------------ */

export type ChatMessage = {
  id: number;
  role: "assistant" | "user" | "system";
  text: string;
  attachments?: ReferenceAttachment[];
  operationBatchId?: string;
};

export type ChatThread = {
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

export type LayerUiState = {
  visible: boolean;
  locked: boolean;
};

/* ------------------------------------------------------------------ */
/*  Operations                                                         */
/* ------------------------------------------------------------------ */

export type OperationPreview = {
  id: string;
  targetId: string;
  targetName: string;
  intent: string;
  operationType: "style.patch" | "transform.patch" | "timeline.patch";
  affectedProperties: string[];
  createdAt: string;
};

export type OperationSourceType = "ai" | "sprite-python" | "manual" | "cli";
export type OperationBatchStatus = "pending" | "applied" | "rejected" | "undone";

export type OperationValidationResult = {
  ok: boolean;
  message: string;
  validator: string;
  validatedAt: string;
};

export type SetPropertyOperation = {
  id: string;
  type: "set_property";
  targetId: string;
  targetName: string;
  property: string;
  previousValue: unknown;
  value: unknown;
};

export type ReplaceDocumentOperation = {
  id: string;
  type: "replace_document";
  previousDocument: StrutDocument | null;
  nextDocument: StrutDocument;
};

export type OperationRecord = SetPropertyOperation | ReplaceDocumentOperation;

export type OperationBatch = OperationPreview & {
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

/* ------------------------------------------------------------------ */
/*  Persistence                                                        */
/* ------------------------------------------------------------------ */

export type ProjectSnapshot = {
  project: ProjectInfo;
  document: StrutDocument;
  operationBatches: OperationBatch[];
  selection?: {
    activeState: string;
    selectedNodeId?: string | null;
    layerUi: Record<string, LayerUiState>;
  } | null;
  mainScene: string;
  animations?: ProjectAnimationRecord[];
};

export type ProjectAnimationRecord = {
  id: string;
  name: string;
  chatId?: string | null;
  scene: string;
  operationBatches: OperationBatch[];
  selection?: {
    activeState: string;
    selectedNodeId?: string | null;
    layerUi: Record<string, LayerUiState>;
  } | null;
  document: StrutDocument;
  updatedAt: number;
};

export type ProjectRecord = {
  id: string;
  name: string;
  path: string;
  pinned?: boolean;
  chats: ChatThread[];
  animations?: ProjectAnimationRecord[];
};

export type SidebarMenuState =
  | { kind: "project"; projectId: string }
  | { kind: "chat"; projectId: string; chatId: string }
  | null;

export type WorkspaceState = {
  projects: ProjectRecord[];
  activeProjectId: string | null;
  activeChatId: string | null;
  themeMode: ThemeMode;
};

/* ------------------------------------------------------------------ */
/*  Constants                                                          */
/* ------------------------------------------------------------------ */

export const emptyArtboard: Artboard = {
  id: "empty-artboard",
  name: "No scene yet",
  width: 960,
  height: 540,
  nodes: [],
};

export const emptyMachine: StateMachine = {
  id: "empty-machine",
  name: "No state machine",
  states: [],
};

export const STORAGE_KEY = "strut-studio-workspace-v4";
export const BROWSER_SNAPSHOT_KEY = "strut-studio-saved-project-v1";

export const browserLocalAdapters: LocalAdapter[] = [
  { id: "ollama", name: "Ollama", kind: "local-model", command: "ollama", installed: false, detail: "Not checked in browser preview" },
  { id: "codex", name: "Codex", kind: "local-agent", command: "codex", installed: false, detail: "Not checked in browser preview" },
  { id: "gemini-cli", name: "Gemini CLI", kind: "local-agent", command: "gemini", installed: false, detail: "Not checked in browser preview" },
  { id: "claude-code", name: "Claude Code", kind: "local-agent", command: "claude / openclaude", installed: false, detail: "Not checked in browser preview" },
  { id: "opencode", name: "OpenCode", kind: "local-agent", command: "opencode-cli", installed: false, detail: "Not checked in browser preview" },
  { id: "cursor-agent", name: "Cursor Agent", kind: "local-agent", command: "cursor-agent", installed: false, detail: "Not checked in browser preview" },
  { id: "qwen", name: "Qwen Code", kind: "local-agent", command: "qwen", installed: false, detail: "Not checked in browser preview" },
  { id: "qoder", name: "Qoder CLI", kind: "local-agent", command: "qodercli", installed: false, detail: "Not checked in browser preview" },
  { id: "copilot-cli", name: "Copilot CLI", kind: "local-agent", command: "copilot", installed: false, detail: "Not checked in browser preview" },
  { id: "kiro", name: "Kiro", kind: "local-agent", command: "kiro-cli", installed: false, detail: "Not checked in browser preview" },
];

export const byokProviders: ByokProvider[] = [
  { id: "openai", name: "OpenAI", env: "OPENAI_API_KEY", endpoint: "https://api.openai.com/v1", model: "gpt-5.2" },
  { id: "anthropic", name: "Anthropic", env: "ANTHROPIC_API_KEY", endpoint: "https://api.anthropic.com", model: "claude-opus-4-5" },
  { id: "gemini", name: "Gemini", env: "GEMINI_API_KEY", endpoint: "https://generativelanguage.googleapis.com", model: "gemini-3-pro" },
  { id: "openrouter", name: "OpenRouter", env: "OPENROUTER_API_KEY", endpoint: "https://openrouter.ai/api/v1", model: "openai/gpt-5.2" },
  { id: "openai-compatible", name: "OpenAI Compatible", env: "API_KEY", endpoint: "http://localhost:1234/v1", model: "local-model" },
];
