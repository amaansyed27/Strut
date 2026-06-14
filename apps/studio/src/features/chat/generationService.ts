/**
 * Generation service — typed wrappers for generation and chat Tauri commands.
 */

import { tauriInvoke } from "../../lib/tauriClient";
import type {
  AssistantResult,
  GenerationContext,
  GenerationProvider,
  ReferenceAttachment,
  StudioStatus,
} from "../../types";

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

    return tauriInvoke<AssistantResult>("assistant_message", {
      prompt,
      provider,
      references: imageReferences,
      context,
    });
  },

  async studioStatus(): Promise<StudioStatus> {
    return tauriInvoke<StudioStatus>("studio_status");
  },
};
