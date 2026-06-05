import { Fragment, useEffect, useMemo, useRef, useState, type CSSProperties, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ChevronRight,
  Cpu,
  FileText,
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
      </div>
    </div>
  );
}

function flattenNodes(nodes: StrutNode[]): StrutNode[] {
  return nodes.flatMap((node) => [node, ...flattenNodes(node.children ?? [])]);
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

function projectFiles(project: ProjectRecord): ProjectFile[] {
  return [
    { name: "strut.project.json", path: `${project.path}\\strut.project.json`, kind: "project" },
    { name: "starter.strut.json", path: `${project.path}\\scenes\\starter.strut.json`, kind: "scene" },
    { name: "assets", path: `${project.path}\\assets`, kind: "folder" },
    { name: "exports", path: `${project.path}\\exports`, kind: "folder" },
  ];
}

function CharacterPreview({ document, activeState }: { document: StrutDocument; activeState: string }) {
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
        {artboard.nodes.map((node) => <StrutNodePreview key={node.id} node={node} />)}
      </g>
      <text className="state-label" x={width / 2} y={height - 24} textAnchor="middle">{titleCase(activeState)}</text>
    </svg>
  );
}

function StrutNodePreview({ node }: { node: StrutNode }) {
  const common = {
    "data-node-id": node.id,
    "data-node-name": node.name,
    className: `strut-node node-${cssIdent(node.name)} kind-${cssIdent(node.kind)}`,
    transform: transformAttribute(node.transform),
    style: nodeStyle(node.style),
  };
  const children = node.children?.map((child) => <StrutNodePreview key={child.id} node={child} />);
  const shape = node.shape ?? { type: "none" };
  if (node.kind === "group" || shape.type === "none") {
    return <g {...common}>{children}</g>;
  }
  if (shape.type === "rect") {
    return <rect {...common} x={shape.x} y={shape.y} width={shape.width} height={shape.height} rx={shape.rx}>{children}</rect>;
  }
  if (shape.type === "ellipse") {
    return <ellipse {...common} cx={shape.cx} cy={shape.cy} rx={shape.rx} ry={shape.ry}>{children}</ellipse>;
  }
  if (shape.type === "path") {
    return <path {...common} d={shape.d}>{children}</path>;
  }
  if (shape.type === "text") {
    return <text {...common} x={shape.x} y={shape.y} fontSize={shape.size}>{shape.value}{children}</text>;
  }
  return <g {...common}>{children}</g>;
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
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [pendingReferences, setPendingReferences] = useState<ReferenceAttachment[]>([]);
  const [providerMode, setProviderMode] = useState<ProviderMode>("local");
  const [localAdapters, setLocalAdapters] = useState<LocalAdapter[]>(browserLocalAdapters);
  const [selectedLocalAdapterId, setSelectedLocalAdapterId] = useState("ollama");
  const [selectedByokProviderId, setSelectedByokProviderId] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [providerEndpoint, setProviderEndpoint] = useState(byokProviders[0].endpoint);
  const [providerModel, setProviderModel] = useState(byokProviders[0].model);
  const [activity, setActivity] = useState("Select a real local CLI, Ollama, or BYOK provider");
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
  const projectPreview = latestPreviewForProject(activeProject, activeChatId);
  const currentDocument = projectPreview?.document ?? null;
  const currentActiveState = projectPreview?.activeState ?? "wave";
  const files = activeProject ? projectFiles(activeProject) : [];
  const activeArtboard = currentDocument?.artboards[0] ?? emptyArtboard;
  const activeMachine = currentDocument?.state_machines[0] ?? emptyMachine;
  const layers = useMemo(() => flattenNodes(activeArtboard.nodes), [activeArtboard.nodes]);
  const selectedLayer = layers.find((layer) => layer.id === selectedNodeId) ?? null;
  const selectedTargetLabel = selectedLayer?.name ?? (currentDocument ? activeArtboard.name : "No selection");
  const activeLocalAdapter = localAdapters.find((adapter) => adapter.id === selectedLocalAdapterId) ?? localAdapters[0] ?? browserLocalAdapters[0];
  const activeByokProvider = byokProviders.find((provider) => provider.id === selectedByokProviderId) ?? byokProviders[0];
  const activeProviderLabel = providerMode === "local" ? activeLocalAdapter.name : activeByokProvider.name;
  const activeProviderType = providerMode === "local" ? "Local CLI" : "BYOK";
  const activeProviderDetail = providerMode === "local" ? activeLocalAdapter.detail : `${providerModel.trim() || activeByokProvider.model} / ${providerEndpoint.trim() || activeByokProvider.endpoint}`;
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

  useEffect(() => {
    if (selectedNodeId && !layers.some((layer) => layer.id === selectedNodeId)) {
      setSelectedNodeId(null);
    }
  }, [layers, selectedNodeId]);

  function providerPayload(): GenerationProvider {
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
  }

  function removeProject(projectId: string) {
    setProjects((current) => current.filter((project) => project.id !== projectId));
    if (activeProjectId === projectId) {
      setActiveProjectId(null);
      setActiveChatId(null);
    }
  }

  function setCurrentActiveState(state: string) {
    if (!activeProjectId) {
      return;
    }
    const ownerChatId = projectPreview?.chatId ?? activeChatId;
    if (!ownerChatId) {
      return;
    }
    updateChat(activeProjectId, ownerChatId, (chat) => ({ ...chat, activeState: state, updated: "now" }));
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
      currentDocumentSummary: documentSummary(currentDocument),
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
          ? await invoke<ProviderOperationResult>("test_local_adapter", { adapterId: selectedLocalAdapterId })
          : await invoke<ProviderOperationResult>("test_byok_provider", { config: providerPayload().byok });
      setActivity(result.status);
    } catch (error) {
      setActivity(String(error));
    }
  }

  async function openActiveProjectFolder() {
    if (!activeProject) {
      setActivity("Select a project first");
      return;
    }
    if (!desktopRuntime) {
      setActivity("Desktop app required to open project folder");
      return;
    }
    try {
      await invoke("open_project_folder", { path: activeProject.path });
      setActivity(`Opened ${activeProject.name}`);
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
    const generationPrompt = trimmed || "Use the attached reference image to create an editable Strut motion document.";
    appendUserMessage(trimmed || "Use the attached reference image.", references);
    updateChat(activeProjectId, activeChatId, (chat) => ({
      ...chat,
      title: chat.title === "New motion chat" || chat.title === "New character chat" || chat.title === "Project brief" ? promptTitle(trimmed || references[0]?.name || "Reference motion") : chat.title,
      updated: "now",
    }));
    setPendingReferences([]);
    setActivity("Generating");

    try {
      if (!desktopRuntime) {
        throw new Error("Desktop app required for real generation. Run the Tauri app and connect a local CLI, Ollama, or BYOK provider.");
      }
      const args = { prompt: generationPrompt, provider: providerPayload(), references, context: generationContext() };
      const result = await invoke<GeneratedCharacter>("generate_character", args);
      updateChat(activeProjectId, activeChatId, (chat) => ({
        ...chat,
        title: chat.title === "New motion chat" || chat.title === "New character chat" || chat.title === "Project brief" ? promptTitle(trimmed || references[0]?.name || "Reference motion") : chat.title,
        updated: "now",
        document: result.document,
        activeState: result.document.state_machines[0]?.states.includes("wave") ? "wave" : "idle",
      }));
      setActivity(`${result.source}: ${result.message}`);
      appendMessage(
        "assistant",
        `**${result.document.name} is ready.**\n\nProvider: ${activeProviderLabel}\n\nI ${currentDocument ? "updated" : "created"} editable layers, states, timelines, bindings, and events.`,
      );
    } catch (error) {
      setActivity(String(error));
      appendMessage("assistant", `**Generation stopped**\n\nProvider: ${activeProviderLabel}\n\n${String(error)}`);
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
            <span>{activeProviderLabel}</span>
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
          <div className="workspace-status" aria-label="Project status">
            <span>{viewMode === "editor" ? "AI editor" : titleCase(viewMode)}</span>
            <span>{currentDocument ? `${layers.length} layers` : "No scene"}</span>
            <span data-testid="selected-provider-chip">Provider: {activeProviderLabel}</span>
          </div>
          <button
            aria-label="Open in file explorer"
            className="open-folder-button"
            disabled={!activeProject}
            title="Open in file explorer"
            type="button"
            onClick={() => void openActiveProjectFolder()}
          >
            <FolderOpen size={16} />
            <span>Open</span>
          </button>
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
            <div className="provider-summary" data-testid="selected-provider-summary">
              <span>Selected provider</span>
              <strong>{activeProviderLabel}</strong>
              <em>{activeProviderType} / {activeProviderDetail}</em>
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
                  {mode === "local" ? "Local CLI" : "BYOK"}
                </button>
              ))}
            </div>
            {providerMode === "local" ? (
              <div className="provider-list">
                {localAdapters.map((adapter) => (
                  <button
                    aria-pressed={selectedLocalAdapterId === adapter.id}
                    className={selectedLocalAdapterId === adapter.id ? "active" : ""}
                    key={adapter.id}
                    type="button"
                    onClick={() => setSelectedLocalAdapterId(adapter.id)}
                  >
                    <span>
                      <strong>{adapter.name}</strong>
                      <em>{adapter.kind}</em>
                    </span>
                    <span>
                      {selectedLocalAdapterId === adapter.id ? <strong>Selected</strong> : null}
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
              <section className="settings-section">
                <div>
                  <h2>Editor</h2>
                  <p>Default panels and viewport behavior.</p>
                </div>
                <div className="settings-controls compact">
                  <label className="toggle-row">
                    <input checked={partsVisible} type="checkbox" onChange={(event) => setPartsVisible(event.currentTarget.checked)} />
                    <span>Show scene layers in editor</span>
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
                {activeChat.messages.length === 0 ? (
                  <div className="home-heading">
                    <h1>What motion should Strut build?</h1>
                    <p>Animate a logo, SVG, loader, product state, storyboard, mascot, or full scene.</p>
                  </div>
                ) : null}
                {activeChat.messages.map((message) => <ChatMessageView key={message.id} message={message} />)}
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
                <div className="prompt-examples" aria-label="Prompt examples">
                  {["Quiet loader", "Soft logo", "Button state", "Calm mascot", "State badge", "Tiny success"].map((example) => (
                    <button key={example} type="button" onClick={() => setPrompt((current) => current || `Make a ${example.toLowerCase()} as an editable Strut animation`)}>
                      {example}
                    </button>
                  ))}
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
                    <span className="composer-provider">Provider: {activeProviderLabel}</span>
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
          <section className="editor-layout" aria-label="AI editor shell">
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
            <div className="ai-editor-shell">
              <aside className="ai-edit-rail" aria-label="AI edit rail">
                <div className="rail-heading">
                  <span>AI edit mode</span>
                  <strong>{activeChat.title}</strong>
                  <p>Describe a change, attach reference material, or target the current selection. Operations are preview-only placeholders in Phase 1.</p>
                </div>

                <div className="selection-card" data-testid="selection-context">
                  <span>Selected target</span>
                  <strong>{selectedTargetLabel}</strong>
                  <p>{currentDocument ? "Future AI edits will use this target as context." : "Generate or open a scene before selecting a target."}</p>
                  <button disabled type="button">
                    <WandSparkles size={15} />
                    Ask AI to edit selection
                  </button>
                </div>

                <div className="operation-placeholder" aria-label="Operation preview placeholder">
                  <span>Pending operation</span>
                  <strong>No operation staged</strong>
                  <p>Phase 1 reserves this area for inspectable AI patches without applying any scene changes.</p>
                  <div>
                    <button disabled type="button">Apply operation</button>
                    <button disabled type="button">Reject</button>
                  </div>
                </div>

                <div className="rail-transcript">
                  {activeChat.messages.length ? (
                    activeChat.messages.map((message) => <ChatMessageView compact key={message.id} message={message} />)
                  ) : (
                    <div className="rail-empty">
                      <strong>No edit history yet</strong>
                      <span>Ask for a motion draft, then refine a layer or state from here.</span>
                    </div>
                  )}
                </div>

                <div className="composer compact-composer">
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
                  <textarea aria-label="Motion prompt" value={prompt} onChange={(event) => setPrompt(event.currentTarget.value)} placeholder={`Ask Strut to edit ${selectedTargetLabel.toLowerCase()}`} />
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
                      <span className="composer-provider">Provider: {activeProviderLabel}</span>
                    </div>
                    <button aria-label="Generate" type="button" onClick={() => void runGeneration()}>
                      <Send size={17} />
                    </button>
                  </div>
                </div>
              </aside>

              <div className="editor-main">
                <div className="preview-workspace">
                  <PreviewPane
                    activeMachine={activeMachine}
                    activeState={currentActiveState}
                    document={currentDocument}
                    selectedTargetLabel={selectedTargetLabel}
                    setActiveState={setCurrentActiveState}
                    showSelectionAffordances
                  />
                </div>
                <aside className="editor-inspector" aria-label="Project files and scene layers">
                  <div className="context-section">
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
                  </div>
                  <div className="context-section">
                    <div className="panel-title">
                      <strong>Scene layers</strong>
                      <em>{activeArtboard.name}</em>
                    </div>
                    {partsVisible ? (
                      <div className="layer-list">
                        {layers.length ? layers.map((layer) => (
                          <button
                            aria-pressed={selectedNodeId === layer.id}
                            className={selectedNodeId === layer.id ? "active" : ""}
                            key={layer.id}
                            type="button"
                            onClick={() => setSelectedNodeId((current) => (current === layer.id ? null : layer.id))}
                          >
                            <span>{layer.name}</span>
                            <em>{layer.kind}</em>
                          </button>
                        )) : <p className="panel-empty">No editable layers yet.</p>}
                      </div>
                    ) : <p className="panel-empty">Parts hidden</p>}
                  </div>
                </aside>
              </div>
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

function PreviewPane({
  activeMachine,
  activeState,
  document,
  selectedTargetLabel,
  setActiveState,
  showSelectionAffordances = false,
}: {
  activeMachine: StateMachine;
  activeState: string;
  document: StrutDocument | null;
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
          <CharacterPreview document={document} activeState={activeState} />
        ) : (
          <div className="preview-empty">
            <ImagePlus size={26} />
            <strong>No scene yet</strong>
            <span>Attach a reference or describe a logo, SVG, UI state, mascot, storyboard, or scene.</span>
          </div>
        )}
        {showSelectionAffordances && document ? (
          <div className="selection-outline">
            <span>{selectedTargetLabel ?? "No selection"}</span>
          </div>
        ) : null}
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
          <span>Selection outline and edit targeting are visual placeholders for Phase 1.</span>
        </div>
      ) : null}
    </aside>
  );
}

export default App;
