/**
 * Generation service — typed wrappers for generation and chat Tauri commands.
 */

import { tauriInvoke } from "../../lib/tauriClient";
import { ensureVisibleGeneratedDocument } from "../../lib/layoutBounds";
import {
  canonicalCoinPlanSummary,
  createCanonicalCoinDocument,
  shouldUseCanonicalCoin,
} from "../../lib/canonicalCoin";
import type {
  AssistantResult,
  GenerationContext,
  GenerationProvider,
  ReferenceAttachment,
  StudioStatus,
} from "../../types";
import type { MotionSpec } from "../../lib/motionArtifacts";

function normalizeGeneratedMotion(prompt: string, result: AssistantResult): AssistantResult {
  if (result.kind !== "document_created" && result.kind !== "document_updated") return result;

  if (shouldUseCanonicalCoin(prompt, result.document)) {
    const document = ensureVisibleGeneratedDocument(
      createCanonicalCoinDocument(result.document.name || "Premium 2.5D Coin Flip"),
    );
    const summary = canonicalCoinPlanSummary(document);
    result.document = document;
    result.activeState = "idle";
    result.planSummary = summary;
    result.operationCount = summary.partNames.length + document.timelines.length;
    result.source = `${result.source}+coin`;
    result.message = "Generated a premium 2.5D coin animation with rim depth, front/back outcome layers, glints, reactive shadow, anticipation, flip, hover, and settle motion.";
    return result;
  }

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
