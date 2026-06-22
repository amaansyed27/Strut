/**
 * Generation service — typed wrappers for generation and chat Tauri commands.
 */

import { tauriInvoke } from "../../lib/tauriClient";
import { ensureVisibleGeneratedDocument } from "../../lib/layoutBounds";
import { createCanonicalCoinDocument, shouldUseCanonicalCoin } from "../../lib/canonicalCoin";
import type {
  AssistantResult,
  GenerationContext,
  GenerationProvider,
  ReferenceAttachment,
  StudioStatus,
} from "../../types";
import type { MotionSpec } from "../../lib/motionArtifacts";

function localCoinResult(): AssistantResult {
  return {
    kind: "document_created",
    source: "local-coin",
    message: "Generated a premium 2.5D coin flip with front/back faces, rim depth, glint, reactive shadow, anticipation, flip, and settle states.",
    document: ensureVisibleGeneratedDocument(createCanonicalCoinDocument("Premium 2.5D Coin Flip")),
    activeState: "idle",
    operationCount: 4,
    planSummary: {
      subjectClassification: "object",
      subjectLabel: "premium 2.5D coin flip",
      partNames: ["Reactive Ground Shadow", "Coin Rig", "Rim Depth Back Plate", "Warm Side Thickness", "Front Face Group", "Back Face Group", "Moving Glint Highlight", "Settle Spark"],
      timelineNames: ["idle", "anticipation", "flip", "settle"],
    },
  };
}

function normalizeGeneratedMotion(prompt: string, result: AssistantResult): AssistantResult {
  if (result.kind !== "document_created" && result.kind !== "document_updated") return result;
  if (shouldUseCanonicalCoin(prompt)) return localCoinResult();
  result.document = ensureVisibleGeneratedDocument(result.document);
  return result;
}

export const generationService = {
  async assistantMessage(
    prompt: string,
    provider: GenerationProvider,
    references: ReferenceAttachment[],
    context: GenerationContext,
  ): Promise<AssistantResult> {
    if (shouldUseCanonicalCoin(prompt)) return localCoinResult();

    const imageReferences = references
      .filter((ref) => ref.kind !== "layer" && ref.dataUrl?.startsWith("data:image/"))
      .map((ref) => ({
        name: ref.name,
        mimeType: ref.mimeType,
        dataUrl: ref.dataUrl ?? "",
      }));

    const result = await tauriInvoke<AssistantResult>("assistant_message_v2", {
      prompt,
      provider,
      references: imageReferences,
      context,
    });

    return normalizeGeneratedMotion(prompt, result);
  },

  async motionSpecRoute(prompt: string): Promise<MotionSpec | null> {
    return tauriInvoke<MotionSpec | null>("motion_spec_route", { prompt });
  },

  async studioStatus(): Promise<StudioStatus> {
    return tauriInvoke<StudioStatus>("studio_status");
  },
};
