/**
 * Provider service — typed wrappers for provider-related Tauri commands.
 * Uses the resilient provider test/save commands on desktop.
 */

import { tauriInvoke } from "../../lib/tauriClient";
import type { LocalAdapter, ProviderOperationResult } from "../../types";

export type ByokProviderConfig = {
  providerId: string;
  endpoint: string;
  model: string;
  [key: string]: string | undefined;
};

export const providerService = {
  async listLocalAdapters(): Promise<LocalAdapter[]> {
    return tauriInvoke<LocalAdapter[]>("local_agent_adapters");
  },

  async testLocalAdapter(adapterId: string): Promise<ProviderOperationResult> {
    return tauriInvoke<ProviderOperationResult>("test_local_adapter", { adapterId });
  },

  async saveByokProvider(config: ByokProviderConfig): Promise<ProviderOperationResult> {
    return tauriInvoke<ProviderOperationResult>("save_byok_provider_v2", { config });
  },

  async testByokProvider(config: ByokProviderConfig): Promise<ProviderOperationResult> {
    return tauriInvoke<ProviderOperationResult>("test_byok_provider_v2", { config });
  },
};
