/**
 * Document manipulation utilities.
 *
 * Pure functions for working with StrutDocument, nodes, operations, etc.
 * Extracted from App.tsx for reuse across features.
 */

import type {
  StrutNode,
  StrutDocument,
  OperationBatch,
  OperationValidationResult,
  AssistantResult,
  OperationSourceType,
  ReferenceAttachment,
  ChatThread,
  ChatMessage,
} from "../types";

/** Flatten a node tree into a flat array. */
export function flattenNodes(nodes: StrutNode[]): StrutNode[] {
  return nodes.flatMap((node) => [node, ...flattenNodes(node.children ?? [])]);
}

/** Find a node by ID in a nested tree. */
export function findNodeById(nodes: StrutNode[], nodeId: string): StrutNode | null {
  for (const node of nodes) {
    if (node.id === nodeId) return node;
    const child = findNodeById(node.children ?? [], nodeId);
    if (child) return child;
  }
  return null;
}

/** Set a property on a node (style.* or transform.*). */
export function setNodeProperty(node: StrutNode, property: string, value: unknown): StrutNode {
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

/** Recursively update a node by ID. */
export function updateNodeById(nodes: StrutNode[], nodeId: string, updater: (node: StrutNode) => StrutNode): StrutNode[] {
  return nodes.map((node) => {
    if (node.id === nodeId) return updater(node);
    if (node.children?.length) return { ...node, children: updateNodeById(node.children, nodeId, updater) };
    return node;
  });
}

/** Validate an operation batch against the current document. */
export function validateOperationBatch(document: StrutDocument | null, batch: OperationBatch): OperationValidationResult {
  const validatedAt = nowStamp();
  if (!document && !batch.operations.some((op) => op.type === "replace_document")) {
    return { ok: false, message: "No validated document is open for this operation batch", validator: "strut-studio-browser", validatedAt };
  }
  if (!batch.operations.length) {
    return { ok: false, message: "Operation batch has no operations to apply", validator: "strut-studio-browser", validatedAt };
  }
  const nodes = document?.artboards[0]?.nodes ?? [];
  for (const op of batch.operations) {
    if (op.type === "replace_document") {
      if (!isStrutDocument(op.nextDocument)) {
        return { ok: false, message: "Replacement operation does not contain a valid Strut document", validator: "strut-studio-browser", validatedAt };
      }
      continue;
    }
    const node = findNodeById(nodes, op.targetId);
    if (!node) {
      return { ok: false, message: `Operation targets missing node ${op.targetId}`, validator: "strut-studio-browser", validatedAt };
    }
    if (!/^style\.(fill|stroke|opacity|stroke_width)$|^transform\.(translate_x|translate_y|rotate|scale_x|scale_y)$/.test(op.property)) {
      return { ok: false, message: `Unsupported operation property ${op.property}`, validator: "strut-studio-browser", validatedAt };
    }
  }
  return { ok: true, message: "Operation batch validated against the current Strut document", validator: "strut-studio-browser", validatedAt };
}

/** Apply or undo an operation batch. */
export function applyOperationBatch(document: StrutDocument | null, batch: OperationBatch, direction: "apply" | "undo"): StrutDocument | null {
  if (!batch.validationResult.ok) return document;
  if (!document && !batch.operations.some((op) => op.type === "replace_document")) return document;
  let nextDocument = document;
  for (const op of batch.operations) {
    if (op.type === "replace_document") {
      nextDocument = direction === "apply" ? op.nextDocument : op.previousDocument;
      continue;
    }
    if (!nextDocument) return null;
    const value = direction === "apply" ? op.value : op.previousValue;
    nextDocument = {
      ...nextDocument,
      artboards: nextDocument.artboards.map((artboard, i) =>
        i === 0
          ? { ...artboard, nodes: updateNodeById(artboard.nodes, op.targetId, (node) => setNodeProperty(node, op.property, value)) }
          : artboard,
      ),
    };
  }
  return nextDocument;
}

/** Create an OperationBatch from a generation result. */
export function createGenerationBatch(
  result: AssistantResult & { kind: "document_created" | "document_updated" },
  previousDocument: StrutDocument | null,
  prompt: string,
  sourceType: OperationSourceType,
): OperationBatch {
  const timestamp = nowStamp();
  return {
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
}

/** Get the latest preview from a project's chats. */
export function latestPreviewForProject(
  project: { chats: ChatThread[] } | null,
  activeChatId: string | null,
) {
  if (!project) return null;
  const activeChat = project.chats.find((c) => c.id === activeChatId) ?? null;
  if (activeChat?.document) {
    return { activeState: activeChat.activeState, chatId: activeChat.id, document: activeChat.document, inherited: false };
  }
  const previewChat = project.chats.find((c) => c.document);
  if (!previewChat?.document) return null;
  return { activeState: previewChat.activeState, chatId: previewChat.id, document: previewChat.document, inherited: true };
}

/** Generate a document revision ID for tracking. */
export function documentRevisionId(document: StrutDocument | null): string {
  if (!document) return "rev-empty";
  const artboard = document.artboards[0];
  const layerCount = artboard ? flattenNodes(artboard.nodes).length : 0;
  return `rev-${slugToken(document.name)}-${layerCount}-${document.timelines.length}-${document.state_machines.length}`;
}

/** Create a document summary string. */
export function documentSummary(document: StrutDocument | null): string {
  if (!document) return "No current document";
  const artboard = document.artboards[0];
  const layerCount = artboard ? flattenNodes(artboard.nodes).length : 0;
  const states = document.state_machines[0]?.states.join(", ") || "no states";
  return `${document.name}; ${layerCount} editable layers; states: ${states}`;
}

/** Check if a value looks like a StrutDocument. */
export function isStrutDocument(value: unknown): value is StrutDocument {
  return Boolean(
    value && typeof value === "object"
    && Array.isArray((value as StrutDocument).artboards)
    && Array.isArray((value as StrutDocument).state_machines),
  );
}

/** Convert a string to a URL-safe slug token. */
export function slugToken(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "untitled";
}

/** Get current ISO timestamp. */
export function nowStamp(): string {
  return new Date().toISOString();
}

/** Convert snake_case to Title Case. */
export function titleCase(value: string): string {
  return value.split("_").map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(" ");
}

/** Create a compact title from a prompt. */
export function promptTitle(prompt: string): string {
  const compact = prompt.replace(/\s+/g, " ").trim();
  if (!compact) return "New motion chat";
  return compact.length > 34 ? `${compact.slice(0, 31)}...` : compact;
}

/** Classify whether a prompt is chat or generation intent. */
export function promptIntent(value: string, hasReferences = false): "chat" | "generate" {
  const prompt = value.trim().toLowerCase();
  if (hasReferences && prompt.length === 0) return "generate";

  const generationWords = [
    "generate", "create", "make", "build", "animate", "motion", "loader", "logo",
    "mascot", "icon", "badge", "dice", "svg", "scene", "draw", "design",
    // Smart chat routing: action words that imply "do it"
    "add that", "do that", "try that", "apply that", "use that",
    "the first", "the second", "the third", "that one",
    "add it", "do it", "try it",
  ];
  if (generationWords.some((word) => prompt.includes(word))) return "generate";

  const chatWords = [
    "who are you", "what are you", "explain", "brainstorm", "ideate",
    "should i", "how would", "what do you think", "help me think", "plan",
    "what animation", "what style", "what kind", "suggest", "ideas",
    "can you explain", "how can i", "how do i",
  ];
  if (prompt.endsWith("?") || chatWords.some((word) => prompt.includes(word))) return "chat";

  return "chat";
}

/** Chat fallback for when no provider is available. */
export function localChatFallback(prompt: string): string {
  const value = prompt.trim().toLowerCase();
  if (value.includes("who are you") || value.includes("what are you")) {
    return "I'm Strut's animation design assistant. I can chat through ideas, help plan edits, and when you ask for motion, turn the direction into validated editable Strut scenes.";
  }
  if (value.includes("brainstorm") || value.includes("ideate")) {
    return "Let's brainstorm before generating. A good Strut animation direction usually needs three choices: the subject, the emotional pace, and the editable parts you want to control. Tell me the object or UI moment, and I can suggest a few motion routes.";
  }
  return "I can talk through the idea first. Ask me for direction, critique, or options; when you're ready to create motion, use words like generate, animate, create, or make.";
}

function browserId(prefix: string): string {
  const random = typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `${prefix}-${random}`;
}

function numericKeyframe(timeMs: number, value: number, easing: "linear" | "ease_in" | "ease_out" | "ease_in_out" = "ease_in_out") {
  return { time_ms: timeMs, value: { type: "number" as const, value }, easing };
}

export function createBrowserPreviewGeneration(prompt: string): Extract<AssistantResult, { kind: "document_created" }> {
  const value = prompt.toLowerCase();
  const isDice = value.includes("dice") || value.includes("die");
  const isLoader = value.includes("loader") || value.includes("spinner");
  const subject = isDice ? "Browser Rolling Dice" : isLoader ? "Browser Loader" : "Browser Bouncing Ball";
  const docId = browserId("doc");
  const artboardId = browserId("artboard");
  const bodyId = browserId("body");
  const accentId = browserId("accent");
  const shadowId = browserId("shadow");
  const idleTimelineId = browserId("timeline-idle");
  const motionTimelineId = browserId("timeline-motion");
  const machineId = browserId("machine");

  const bodyShape = isLoader
    ? { type: "ellipse" as const, cx: 256, cy: 232, rx: 54, ry: 54 }
    : { type: "rect" as const, x: 196, y: 162, width: 120, height: 120, rx: isDice ? 22 : 60 };
  const accentShape = isDice
    ? { type: "path" as const, d: "M226 194a8 8 0 1 0 0.1 0M286 194a8 8 0 1 0 0.1 0M226 250a8 8 0 1 0 0.1 0M286 250a8 8 0 1 0 0.1 0" }
    : isLoader
      ? { type: "path" as const, d: "M256 178 A54 54 0 0 1 310 232" }
      : { type: "ellipse" as const, cx: 256, cy: 222, rx: 18, ry: 18 };

  const document: StrutDocument = {
    id: docId,
    name: subject,
    artboards: [{
      id: artboardId,
      name: "Main",
      width: 512,
      height: 384,
      nodes: [
        {
          id: shadowId,
          name: "Ground Shadow",
          kind: "ellipse",
          role: "shadow",
          shape: { type: "ellipse", cx: 256, cy: 314, rx: isLoader ? 72 : 82, ry: 14 },
          style: { fill: "#111827", stroke: null, stroke_width: 0, opacity: 0.18 },
          children: [],
        },
        {
          id: bodyId,
          name: isDice ? "Die Body" : isLoader ? "Loader Ring" : "Ball Body",
          kind: isLoader ? "ellipse" : "rect",
          role: "body",
          shape: bodyShape,
          style: {
            fill: isDice ? "#f8fafc" : isLoader ? "#dff4ea" : "#f97316",
            stroke: "#111827",
            stroke_width: isLoader ? 4 : 3,
            opacity: 1,
          },
          children: [],
        },
        {
          id: accentId,
          name: isDice ? "Face Pips" : isLoader ? "Progress Sweep" : "Highlight",
          kind: isDice || isLoader ? "path" : "ellipse",
          role: "detail",
          shape: accentShape,
          style: {
            fill: isDice ? "#111827" : isLoader ? "none" : "#fed7aa",
            stroke: isLoader ? "#0f766e" : "none",
            stroke_width: isLoader ? 10 : 0,
            opacity: 1,
            linecap: "round",
            linejoin: "round",
          },
          children: [],
        },
      ],
    }],
    timelines: [
      {
        id: idleTimelineId,
        name: "idle",
        duration_ms: 1400,
        loops: true,
        tracks: [
          { target: bodyId, property: "translation.y", keyframes: [numericKeyframe(0, 0), numericKeyframe(700, -8), numericKeyframe(1400, 0)] },
          { target: accentId, property: "translation.y", keyframes: [numericKeyframe(0, 0), numericKeyframe(700, -8), numericKeyframe(1400, 0)] },
          { target: shadowId, property: "scale.x", keyframes: [numericKeyframe(0, 1), numericKeyframe(700, 0.82), numericKeyframe(1400, 1)] },
        ],
      },
      {
        id: motionTimelineId,
        name: isDice ? "rolling" : isLoader ? "loading" : "bounce",
        duration_ms: isDice ? 1200 : 1000,
        loops: true,
        tracks: [
          { target: bodyId, property: "rotation", keyframes: [numericKeyframe(0, 0, "linear"), numericKeyframe(isDice ? 1200 : 1000, isLoader ? 360 : 18, "linear")] },
          { target: accentId, property: "rotation", keyframes: [numericKeyframe(0, 0, "linear"), numericKeyframe(isDice ? 1200 : 1000, isLoader ? 360 : 18, "linear")] },
          { target: bodyId, property: "translation.y", keyframes: [numericKeyframe(0, 0), numericKeyframe(500, -54, "ease_out"), numericKeyframe(1000, 0, "ease_in")] },
          { target: shadowId, property: "opacity", keyframes: [numericKeyframe(0, 0.18), numericKeyframe(500, 0.07), numericKeyframe(1000, 0.18)] },
        ],
      },
    ],
    state_machines: [{
      id: machineId,
      name: "Controller",
      inputs: [{ name: "play", kind: "trigger" }],
      states: ["idle", isDice ? "rolling" : isLoader ? "loading" : "bounce"],
      transitions: [{ from: "idle", to: isDice ? "rolling" : isLoader ? "loading" : "bounce", on: "play", timeline: isDice ? "rolling" : isLoader ? "loading" : "bounce" }],
    }],
    bindings: [],
    events: [],
  };

  return {
    kind: "document_created",
    message: "Generated a browser preview document. Use the desktop app with a configured provider for production output.",
    source: "browser-preview",
    document,
    activeState: "idle",
    planSummary: {
      subjectClassification: isDice ? "dice" : isLoader ? "loader" : "ball",
      subjectLabel: subject,
      partNames: document.artboards[0].nodes.map((node) => node.name),
      timelineNames: document.timelines.map((timeline) => timeline.name),
    },
    operationCount: document.artboards[0].nodes.length + document.timelines.length,
  };
}

/** Unique attachments by kind + ID. */
export function uniqueAttachments(attachments: ReferenceAttachment[]): ReferenceAttachment[] {
  const seen = new Set<string>();
  return attachments.filter((a) => {
    const key = a.kind === "layer" ? `layer:${a.nodeId ?? a.name}` : `image:${a.id}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

/** Build a layer reference prompt suffix. */
export function layerReferencePrompt(attachments: ReferenceAttachment[]): string {
  const layers = attachments.filter((a) => a.kind === "layer");
  const animations = attachments.filter((a) => a.kind === "animation");
  const parts: string[] = [];
  if (animations.length) {
    parts.push(`Target animations selected from the project: ${animations
      .map((a) => `${a.name} (${a.animationId ?? a.documentId ?? "unknown animation"})`)
      .join("; ")}. Use these whole animations as edit/reference context.`);
  }
  if (layers.length) {
    parts.push(`Target layers selected from the scene: ${layers
    .map((a) => `${a.name} (${a.nodeId ?? "unknown id"}, ${a.nodeKind ?? "layer"}${a.nodeRole ? `, role: ${a.nodeRole}` : ""})`)
    .join("; ")}. Use these as edit context.`);
  }
  return parts.length ? `\n\n${parts.join("\n")}` : "";
}

/** Relative time label (e.g. "2m", "3h", "1d"). */
export function relativeTimeLabel(value: string, nowMs = Date.now()): string {
  const parsed = Date.parse(value);
  if (!Number.isFinite(parsed)) return value === "now" ? "now" : value;
  const seconds = Math.max(0, Math.floor((nowMs - parsed) / 1000));
  if (seconds < 60) return "now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d`;
  const weeks = Math.floor(days / 7);
  if (weeks < 5) return `${weeks}w`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${Math.max(1, months)}mo`;
  return `${Math.max(1, Math.floor(days / 365))}y`;
}

/** Create a new chat thread. */
export function createChat(projectId: string, title: string, messages: ChatMessage[] = []): ChatThread {
  return {
    id: `chat-${Date.now()}-${Math.round(Math.random() * 10000)}`,
    title,
    projectId,
    updated: nowStamp(),
    messages,
    references: [],
    document: null,
    activeState: "",
    selectedNodeId: null,
    layerUi: {},
    pendingOperation: null,
    operationHistory: [],
    operationBatches: [],
    undoStack: [],
    redoStack: [],
  };
}

/** Convert a File to a ReferenceAttachment. */
export function fileToAttachment(file: File): Promise<ReferenceAttachment> {
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

/** Convert a StrutNode to a layer reference attachment. */
export function layerToAttachment(layer: StrutNode): ReferenceAttachment {
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

/** Get LayerUiState for a node, defaulting to visible and unlocked. */
export function layerUiFor(layerUi: Record<string, { visible: boolean; locked: boolean }>, nodeId: string) {
  return layerUi[nodeId] ?? { visible: true, locked: false };
}

/** Get the first available animation state from a document. */
export function firstAvailableState(document: StrutDocument | null): string {
  return document?.state_machines[0]?.states[0] ?? "";
}
