/**
 * Project animation helpers.
 *
 * Project-level animation records are the durable library shown beside the
 * preview. Chats can still own documents, but generated scenes are promoted
 * into this list so they can be reopened, referenced, or deleted as files.
 */

import type {
  ChatThread,
  LayerUiState,
  OperationBatch,
  ProjectAnimationRecord,
  ProjectRecord,
  ReferenceAttachment,
  StrutDocument,
} from "../../types";
import { documentRevisionId, firstAvailableState, nowStamp } from "../../lib/documentUtils";

export function createLocalProjectAnimationRecord(
  chatId: string,
  document: StrutDocument,
  operationBatches: OperationBatch[],
  activeState: string,
  layerUi: Record<string, LayerUiState> = {},
): ProjectAnimationRecord {
  return {
    id: `local-${chatId}-${documentRevisionId(document)}-${Date.now()}`,
    name: document.name,
    chatId,
    scene: "",
    operationBatches,
    selection: {
      activeState: activeState || firstAvailableState(document),
      selectedNodeId: null,
      layerUi,
    },
    document,
    updatedAt: Date.now(),
  };
}

export function projectAnimationLibrary(project: ProjectRecord | null): ProjectAnimationRecord[] {
  if (!project) return [];
  const animations = [...(project.animations ?? [])];
  const existingKeys = new Set(animations.map(animationKey));
  for (const chat of project.chats) {
    if (!chat.document) continue;
    const fallback = animationFromChat(chat);
    const key = animationKey(fallback);
    if (!existingKeys.has(key)) {
      animations.push(fallback);
      existingKeys.add(key);
    }
  }
  return animations.sort((left, right) => right.updatedAt - left.updatedAt);
}

export function upsertProjectAnimation(
  project: ProjectRecord,
  animation: ProjectAnimationRecord,
): ProjectRecord {
  const animations = [
    animation,
    ...(project.animations ?? []).filter((item) => (
      item.id !== animation.id &&
      !(item.chatId === animation.chatId && item.name === animation.name)
    )),
  ];
  return { ...project, animations };
}

export function removeProjectAnimation(project: ProjectRecord, animationId: string): ProjectRecord {
  const removed = (project.animations ?? []).find((animation) => animation.id === animationId);
  const linkedChatId = removed?.chatId ?? (animationId.startsWith("chat-") ? animationId.slice(5) : null);
  return {
    ...project,
    animations: (project.animations ?? []).filter((animation) => animation.id !== animationId),
    chats: project.chats.map((chat) =>
      chat.document && linkedChatId && chat.id === linkedChatId
        ? { ...chat, document: null, activeState: "", updated: nowStamp() }
        : chat,
    ),
  };
}

export function findProjectAnimationForDocument(
  animations: ProjectAnimationRecord[],
  document: StrutDocument | null,
  chatId?: string | null,
): ProjectAnimationRecord | null {
  if (!document) return null;
  const revision = documentRevisionId(document);
  return (
    animations.find((animation) => animation.chatId && animation.chatId === chatId && documentRevisionId(animation.document) === revision) ??
    animations.find((animation) => documentRevisionId(animation.document) === revision) ??
    null
  );
}

export function animationToAttachment(animation: ProjectAnimationRecord): ReferenceAttachment {
  return {
    id: `animation-${animation.id}`,
    name: animation.name,
    kind: "animation",
    mimeType: "application/x-strut-animation",
    size: 0,
    animationId: animation.id,
    documentId: animation.document.id,
  };
}

function animationFromChat(chat: ChatThread): ProjectAnimationRecord {
  const document = chat.document as StrutDocument;
  return {
    id: `chat-${chat.id}`,
    name: document.name || chat.title,
    chatId: chat.id,
    scene: "",
    operationBatches: chat.operationBatches ?? chat.operationHistory ?? [],
    selection: {
      activeState: chat.activeState || firstAvailableState(document),
      selectedNodeId: chat.selectedNodeId ?? null,
      layerUi: chat.layerUi ?? {},
    },
    document,
    updatedAt: Date.parse(chat.updated) || Date.now(),
  };
}

function animationKey(animation: ProjectAnimationRecord): string {
  return `${animation.chatId ?? "project"}:${documentRevisionId(animation.document)}`;
}
