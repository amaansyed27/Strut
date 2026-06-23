/**
 * Generation service — typed wrappers for generation and chat Tauri commands.
 */

import { tauriInvoke } from "../../lib/tauriClient";
import { ensureVisibleGeneratedDocument } from "../../lib/layoutBounds";
import { engineIssuesV2, upgradeGeneratedDocumentV2 } from "../../lib/strutEngineV2";
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

function finalizeDocumentResult(prompt: string, result: AssistantResult): AssistantResult {
  if (result.kind !== "document_created" && result.kind !== "document_updated") return result;
  const document = ensureVisibleGeneratedDocument(upgradeGeneratedDocumentV2(prompt, result.document));
  const issues = engineIssuesV2(prompt, document);
  if (issues.length) {
    return {
      kind: "chat",
      source: "quality-gate",
      message: `Generation blocked: ${issues.join("; ")}. The provider returned a structurally valid scene, but it is visually too weak to save.`,
    };
  }
  result.document = document;
  return result;
}

export const generationService = {
  async assistantMessage(
    prompt: string,
    provider: GenerationProvider,
    references: ReferenceAttachment[],
    context: GenerationContext,
  ): Promise<AssistantResult> {
    const imageReferences = references
      .filter((ref) => ref.kind !== "layer" && ref.dataUrl?.startsWith("data:image/"))
      .map((ref) => ({
        name: ref.name,
        mimeType: ref.mimeType,
        dataUrl: ref.dataUrl ?? "",
      }));

    const key = requestKey(prompt, provider, imageReferences, context);
    if (inFlightAssistantMessage?.key === key) {
      return inFlightAssistantMessage.promise;
    }

    const promise = tauriInvoke<AssistantResult>("assistant_message_v2", {
      prompt,
      provider,
      references: imageReferences,
      context,
    }).then((result) => finalizeDocumentResult(prompt, result)).finally(() => {
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
