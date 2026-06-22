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

    if (result.kind === "document_created" || result.kind === "document_updated") {
      const upgraded = upgradeGeneratedDocumentV2(prompt, result.document);
      const issues = engineIssuesV2(prompt, upgraded);
      if (issues.length) {
        return {
          kind: "chat",
          source: "engine-quality",
          message: `Strut blocked this generation because it is visually underbuilt: ${issues.join(", ")}. Regenerate with a more detailed model response; the result was not saved to the preview.`,
        };
      }
      result.document = ensureVisibleGeneratedDocument(upgraded);
    }
    return result;
  },

  async motionSpecRoute(prompt: string): Promise<MotionSpec | null> {
    return tauriInvoke<MotionSpec | null>("motion_spec_route", { prompt });
  },

  async studioStatus(): Promise<StudioStatus> {
    return tauriInvoke<StudioStatus>("studio_status");
  },
};
