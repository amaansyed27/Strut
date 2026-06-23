/**
 * Generation service — typed wrappers for generation and chat Tauri commands.
 */

import { tauriInvoke } from "../../lib/tauriClient";
import { ensureVisibleGeneratedDocument } from "../../lib/layoutBounds";
import { upgradeGeneratedDocumentV2 } from "../../lib/strutEngineV2";
import type {
  AssistantResult,
  GenerationContext,
  GenerationProvider,
  ReferenceAttachment,
  StudioStatus,
} from "../../types";
import type { MotionSpec } from "../../lib/motionArtifacts";

type InFlightGeneration = {
  key: string;
  promise: Promise<AssistantResult>;
};

let inFlightAssistantMessage: InFlightGeneration | null = null;

const GENERATION_HINTS = [
  "generate", "create", "make", "build", "animate", "animation", "motion", "loader", "logo", "mascot",
  "icon", "badge", "dice", "coin", "flip", "scene", "export", "draw", "design", "button", "component",
];

function requestKey(
  prompt: string,
  provider: GenerationProvider,
  references: { name: string; mimeType?: string; dataUrl: string }[],
  context: GenerationContext,
) {
  return JSON.stringify({
    prompt,
    provider,
    references: references.map((reference) => ({
      name: reference.name,
      mimeType: reference.mimeType,
      bytes: reference.dataUrl.length,
      signature: reference.dataUrl.slice(0, 64),
    })),
    chat: context.activeChatTitle,
    project: context.projectName,
    document: context.currentDocumentSummary,
  });
}

function localGreeting(prompt: string): AssistantResult | null {
  const value = prompt.trim().toLowerCase().replace(/[!.\s]+$/g, "");
  if (!["hi", "hello", "hey", "yo", "sup"].includes(value)) return null;
  return {
    kind: "chat",
    source: "local",
    message: "Hi. Describe the animation you want, or say `generate it` after we plan it.",
  } as AssistantResult;
}

function looksAffirmativeGenerate(prompt: string) {
  const value = prompt.trim().toLowerCase();
  return [
    "yes", "yes please", "go ahead", "please go ahead", "do it", "make it", "make it now",
    "generate", "generate it", "create it", "build it", "animate it", "yes make it", "yes generate it",
  ].some((phrase) => value === phrase || value.includes(phrase));
}

function containsGenerationHint(value: string) {
  const lower = value.toLowerCase();
  return GENERATION_HINTS.some((hint) => lower.includes(hint));
}

function resolvePrompt(prompt: string, context: GenerationContext) {
  if (!looksAffirmativeGenerate(prompt)) return prompt;
  const history = [...(context.chatHistory ?? [])].reverse();
  const priorUserRequest = history.find((message) => message.role === "user" && containsGenerationHint(message.text));
  if (!priorUserRequest) return prompt;
  return `${priorUserRequest.text.trim()}\n\nProceed now and generate the animation. Do not ask for confirmation or describe a plan.`;
}

export const generationService = {
  async assistantMessage(
    prompt: string,
    provider: GenerationProvider,
    references: ReferenceAttachment[],
    context: GenerationContext,
  ): Promise<AssistantResult> {
    const local = references.length === 0 ? localGreeting(prompt) : null;
    if (local) return local;

    const resolvedPrompt = resolvePrompt(prompt, context);
    const imageReferences = references
      .filter((ref) => ref.kind !== "layer" && ref.dataUrl?.startsWith("data:image/"))
      .map((ref) => ({
        name: ref.name,
        mimeType: ref.mimeType,
        dataUrl: ref.dataUrl ?? "",
      }));

    const key = requestKey(resolvedPrompt, provider, imageReferences, context);
    if (inFlightAssistantMessage?.key === key) {
      return inFlightAssistantMessage.promise;
    }

    const promise = tauriInvoke<AssistantResult>("assistant_message_v2", {
      prompt: resolvedPrompt,
      provider,
      references: imageReferences,
      context,
    }).then((result) => {
      if (result.kind === "document_created" || result.kind === "document_updated") {
        result.document = ensureVisibleGeneratedDocument(upgradeGeneratedDocumentV2(resolvedPrompt, result.document));
      }
      return result;
    }).finally(() => {
      if (inFlightAssistantMessage?.key === key) {
        inFlightAssistantMessage = null;
      }
    });

    inFlightAssistantMessage = { key, promise };
    return promise;
  },

  async motionSpecRoute(prompt: string): Promise<MotionSpec | null> {
    return tauriInvoke<MotionSpec | null>("motion_spec_route", { prompt });
  },

  async studioStatus(): Promise<StudioStatus> {
    return tauriInvoke<StudioStatus>("studio_status");
  },
};
