/**
 * Workspace persistence — load and save workspace state to localStorage.
 *
 * Extracted from App.tsx to isolate persistence concerns.
 */

import type { WorkspaceState, ProjectRecord, ChatThread, ThemeMode, ChatMessage, ProjectAnimationRecord } from "../types";
import { STORAGE_KEY, BROWSER_SNAPSHOT_KEY } from "../types";

/** Check if a value is a valid ThemeMode. */
export function isThemeMode(value: unknown): value is ThemeMode {
  return value === "system" || value === "light" || value === "dark";
}

/** Load workspace state from localStorage. */
export function loadWorkspaceState(): WorkspaceState {
  const defaults: WorkspaceState = {
    projects: [],
    activeProjectId: null,
    activeChatId: null,
    themeMode: "system",
  };

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaults;
    const parsed = JSON.parse(raw) as Partial<WorkspaceState>;
    return {
      projects: normalizeProjects(parsed.projects),
      activeProjectId: typeof parsed.activeProjectId === "string" ? parsed.activeProjectId : null,
      activeChatId: typeof parsed.activeChatId === "string" ? parsed.activeChatId : null,
      themeMode: isThemeMode(parsed.themeMode) ? parsed.themeMode : "system",
    };
  } catch {
    return defaults;
  }
}

/** Save workspace state to localStorage. */
export function saveWorkspaceState(state: WorkspaceState): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Storage quota exceeded — silently fail
  }
}

/** Save a browser snapshot for non-desktop mode. */
export function saveBrowserSnapshot(state: WorkspaceState): void {
  try {
    window.localStorage.setItem(
      BROWSER_SNAPSHOT_KEY,
      JSON.stringify({
        ...state,
        savedAt: new Date().toISOString(),
      }),
    );
  } catch {
    // Silently fail
  }
}

/** Load a browser snapshot. */
export function loadBrowserSnapshot(): WorkspaceState | null {
  try {
    const raw = window.localStorage.getItem(BROWSER_SNAPSHOT_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<WorkspaceState>;
    return {
      projects: normalizeProjects(parsed.projects),
      activeProjectId: typeof parsed.activeProjectId === "string" ? parsed.activeProjectId : null,
      activeChatId: typeof parsed.activeChatId === "string" ? parsed.activeChatId : null,
      themeMode: isThemeMode(parsed.themeMode) ? parsed.themeMode : "system",
    };
  } catch {
    return null;
  }
}

/** Normalize projects from localStorage (fix malformed data). */
export function normalizeProjects(value: unknown): ProjectRecord[] {
  if (!Array.isArray(value)) return [];
  return value.filter(isProjectRecord).map((project) => ({
    ...project,
    chats: Array.isArray(project.chats) ? project.chats.filter(isChatThread).map(normalizeChat) : [],
    animations: normalizeAnimations(project.animations),
  }));
}

function normalizeAnimations(value: unknown): ProjectAnimationRecord[] {
  if (!Array.isArray(value)) return [];
  return value.filter(isProjectAnimationRecord).map((animation) => ({
    ...animation,
    operationBatches: Array.isArray(animation.operationBatches) ? animation.operationBatches : [],
    selection: animation.selection ?? null,
    updatedAt: typeof animation.updatedAt === "number" ? animation.updatedAt : Date.now(),
  }));
}

function normalizeChat(chat: ChatThread): ChatThread {
  return {
    ...chat,
    messages: normalizeMessages(chat.messages),
    references: Array.isArray(chat.references) ? chat.references : [],
    document: chat.document ?? null,
    activeState: typeof chat.activeState === "string" ? chat.activeState : "",
    selectedNodeId: chat.selectedNodeId ?? null,
    layerUi: chat.layerUi ?? {},
    pendingOperation: chat.pendingOperation ?? null,
    operationBatches: Array.isArray(chat.operationBatches) ? chat.operationBatches : [],
    operationHistory: Array.isArray(chat.operationHistory) ? chat.operationHistory : [],
    undoStack: Array.isArray(chat.undoStack) ? chat.undoStack : [],
    redoStack: Array.isArray(chat.redoStack) ? chat.redoStack : [],
  };
}

/** Normalize messages from localStorage. */
export function normalizeMessages(value: unknown): ChatMessage[] {
  if (!Array.isArray(value)) return [];
  return value.filter((message) =>
    Boolean(
      message &&
      typeof message === "object" &&
      typeof (message as ChatMessage).id === "number" &&
      typeof (message as ChatMessage).role === "string" &&
      typeof (message as ChatMessage).text === "string",
    ),
  ) as ChatMessage[];
}

function isProjectRecord(value: unknown): value is ProjectRecord {
  return Boolean(
    value &&
    typeof value === "object" &&
    typeof (value as ProjectRecord).id === "string" &&
    typeof (value as ProjectRecord).name === "string",
  );
}

function isChatThread(value: unknown): value is ChatThread {
  return Boolean(
    value &&
    typeof value === "object" &&
    typeof (value as ChatThread).id === "string" &&
    typeof (value as ChatThread).title === "string",
  );
}

function isProjectAnimationRecord(value: unknown): value is ProjectAnimationRecord {
  return Boolean(
    value &&
    typeof value === "object" &&
    typeof (value as ProjectAnimationRecord).id === "string" &&
    typeof (value as ProjectAnimationRecord).name === "string" &&
    typeof (value as ProjectAnimationRecord).document === "object",
  );
}
