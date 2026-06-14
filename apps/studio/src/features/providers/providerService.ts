/**
 * Provider service — typed wrappers for provider-related Tauri commands.
 */

import { tauriInvoke } from "../../lib/tauriClient";
import type { LocalAdapter, ProviderOperationResult } from "../../types";

export type ByokProviderConfig = {
  providerId: string;
  apiKey?: string;
  endpoint: string;
  model: string;
};

export const providerService = {
  async listLocalAdapters(): Promise<LocalAdapter[]> {
    return tauriInvoke<LocalAdapter[]>("local_agent_adapters");
  },

  async testLocalAdapter(adapterId: string): Promise<ProviderOperationResult> {
    return tauriInvoke<ProviderOperationResult>("test_local_adapter", { adapterId });
  },

  async saveByokProvider(config: ByokProviderConfig): Promise<ProviderOperationResult> {
    return tauriInvoke<ProviderOperationResult>("save_byok_provider", { config });
  },

  async testByokProvider(config: ByokProviderConfig): Promise<ProviderOperationResult> {
    return tauriInvoke<ProviderOperationResult>("test_byok_provider", { config });
  },
};
