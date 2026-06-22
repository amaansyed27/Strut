/**
 * Settings page component.
 *
 * Extracted from App.tsx for modularity.
 */

import { useEffect, useState } from "react";
import { Cpu, Monitor, Moon, RefreshCw, Sun } from "lucide-react";
import type { LocalAdapter, ProviderMode, ThemeMode } from "../../types";
import { byokProviders } from "../../types";
import { ProviderCard } from "../providers/ProviderCard";
import { providerService } from "../providers/providerService";

type SettingsPageProps = {
  themeMode: ThemeMode;
  setThemeMode: (mode: ThemeMode) => void;
  providerMode: ProviderMode;
  setProviderMode: (mode: ProviderMode) => void;
  localAdapters: LocalAdapter[];
  selectedLocalAdapterId: string;
  setSelectedLocalAdapterId: (id: string) => void;
  selectedByokProviderId: string;
  setSelectedByokProviderId: (id: string) => void;
  apiKey: string;
  setApiKey: (key: string) => void;
  providerEndpoint: string;
  setProviderEndpoint: (endpoint: string) => void;
  providerModel: string;
  setProviderModel: (model: string) => void;
  desktopRuntime: boolean;
  onRefreshProviders: () => void;
  onSaveProvider: () => void;
  onTestProvider: () => void | Promise<void>;
};

type CachedProviderSettings = {
  providerMode?: ProviderMode;
  selectedLocalAdapterId?: string;
  selectedByokProviderId?: string;
  apiKey?: string;
  providerEndpoint?: string;
  providerModel?: string;
};

const PROVIDER_SETTINGS_STORAGE_KEY = "strut-studio.provider-settings-v1";

const themeOptions: Array<{ id: ThemeMode; icon: typeof Sun; label: string }> = [
  { id: "system", icon: Monitor, label: "System" },
  { id: "light", icon: Sun, label: "Light" },
  { id: "dark", icon: Moon, label: "Dark" },
];

export function SettingsPage({
  themeMode,
  setThemeMode,
  providerMode,
  setProviderMode,
  localAdapters,
  selectedLocalAdapterId,
  setSelectedLocalAdapterId,
  selectedByokProviderId,
  setSelectedByokProviderId,
  apiKey,
  setApiKey,
  providerEndpoint,
  setProviderEndpoint,
  providerModel,
  setProviderModel,
  desktopRuntime,
  onRefreshProviders,
}: SettingsPageProps) {
  const [providerUiMessage, setProviderUiMessage] = useState("");
  const [providerBusy, setProviderBusy] = useState(false);

  useEffect(() => {
    const raw = window.localStorage.getItem(PROVIDER_SETTINGS_STORAGE_KEY);
    if (!raw) return;
    try {
      const cached = JSON.parse(raw) as CachedProviderSettings;
      if (cached.providerMode === "local" || cached.providerMode === "byok") setProviderMode(cached.providerMode);
      if (cached.selectedLocalAdapterId) setSelectedLocalAdapterId(cached.selectedLocalAdapterId);
      if (cached.selectedByokProviderId && byokProviders.some((provider) => provider.id === cached.selectedByokProviderId)) {
        setSelectedByokProviderId(cached.selectedByokProviderId);
      }
      if (typeof cached.apiKey === "string") setApiKey(cached.apiKey);
      if (typeof cached.providerEndpoint === "string") setProviderEndpoint(cached.providerEndpoint);
      if (typeof cached.providerModel === "string") setProviderModel(cached.providerModel);
    } catch {
      window.localStorage.removeItem(PROVIDER_SETTINGS_STORAGE_KEY);
    }
  }, [setApiKey, setProviderEndpoint, setProviderMode, setProviderModel, setSelectedByokProviderId, setSelectedLocalAdapterId]);

  useEffect(() => {
    const payload: CachedProviderSettings = {
      providerMode,
      selectedLocalAdapterId,
      selectedByokProviderId,
      apiKey,
      providerEndpoint,
      providerModel,
    };
    window.localStorage.setItem(PROVIDER_SETTINGS_STORAGE_KEY, JSON.stringify(payload));
  }, [apiKey, providerEndpoint, providerMode, providerModel, selectedByokProviderId, selectedLocalAdapterId]);

  const activeLocalAdapter = localAdapters.find((adapter) => adapter.id === selectedLocalAdapterId) ?? localAdapters[0];
  const activeByokProvider = byokProviders.find((provider) => provider.id === selectedByokProviderId) ?? byokProviders[0];
  const activeProviderLabel = providerMode === "local" ? activeLocalAdapter?.name ?? "None" : activeByokProvider.name;
  const activeProviderType = providerMode === "local" ? "Local CLI" : "BYOK";
  const activeProviderDetail =
    providerMode === "local"
      ? activeLocalAdapter?.detail ?? ""
      : `${providerModel.trim() || activeByokProvider.model} / ${
          providerEndpoint.trim() || activeByokProvider.endpoint
        }`;
  const providerStatusText = desktopRuntime
    ? "Detected/selected only means Strut can see or store the provider settings. It does not mean the provider is online or valid. Run Test selected provider for a real smoke test."
    : "Browser preview cannot inspect providers. Open the desktop app to run real installed-provider and BYOK smoke tests.";

  const byokConfig = () => ({
    providerId: selectedByokProviderId,
    apiKey: apiKey.trim() || undefined,
    endpoint: providerEndpoint.trim(),
    model: providerModel.trim(),
  });

  const showProviderResult = (title: string, ok: boolean, status: string, detail?: string) => {
    const message = `${ok ? "PASS" : "FAIL"}: ${status}${detail ? ` — ${detail}` : ""}`;
    setProviderUiMessage(message);
    window.alert(`${title}\n\n${message}`);
  };

  const handleByokProviderChange = (providerId: string) => {
    const provider = byokProviders.find((item) => item.id === providerId) ?? byokProviders[0];
    setSelectedByokProviderId(provider.id);
    setProviderEndpoint(provider.endpoint);
    setProviderModel(provider.model);
  };

  const handleSaveProvider = async () => {
    if (providerMode !== "byok") {
      setProviderUiMessage("Select BYOK before saving provider settings.");
      return;
    }
    if (!desktopRuntime) {
      setProviderUiMessage("Desktop app required to save provider settings.");
      return;
    }
    setProviderBusy(true);
    setProviderUiMessage("Saving provider settings...");
    try {
      const result = await providerService.saveByokProvider(byokConfig());
      showProviderResult("Provider saved", result.ok, result.status, result.detail);
    } catch (error) {
      showProviderResult("Provider save failed", false, String(error));
    } finally {
      setProviderBusy(false);
    }
  };

  const handleTestProvider = async () => {
    if (!desktopRuntime) {
      setProviderUiMessage("Desktop app required to test providers.");
      return;
    }
    setProviderBusy(true);
    setProviderUiMessage("Running provider smoke test...");
    try {
      const result = providerMode === "local"
        ? await providerService.testLocalAdapter(activeLocalAdapter?.id ?? selectedLocalAdapterId)
        : await providerService.testByokProvider(byokConfig());
      showProviderResult("Provider smoke test", result.ok, result.status, result.detail);
    } catch (error) {
      showProviderResult("Provider smoke test failed", false, String(error));
    } finally {
      setProviderBusy(false);
    }
  };

  return (
    <section className="settings-page page-shell">
      <div className="page-heading">
        <h1>Settings</h1>
        <p>Configure Strut Studio preferences, model providers, and workspace behavior.</p>
      </div>

      <div className="settings-section">
        <h2>Appearance</h2>
        <div className="theme-options">
          {themeOptions.map(({ id, icon: Icon, label }) => (
            <button
              aria-pressed={themeMode === id}
              className={themeMode === id ? "active" : ""}
              key={id}
              type="button"
              onClick={() => setThemeMode(id)}
            >
              <Icon size={15} />
              {label}
            </button>
          ))}
        </div>
      </div>

      <div className="settings-section provider-settings-section">
        <div>
          <h2>Providers</h2>
          <p>Select the local CLI, Ollama adapter, or BYOK model Strut should use for chat and generation.</p>
        </div>
        <div className="provider-settings">
          <div className="provider-header-card">
            <div className="provider-summary" data-testid="selected-provider-summary">
              <span>Selected</span>
              <strong>{activeProviderLabel}</strong>
              <em>{activeProviderType}</em>
              <p>{activeProviderDetail}</p>
            </div>
            <div className="provider-tabs" role="group" aria-label="Provider source">
              {(["local", "byok"] as const).map((mode) => (
                <button
                  aria-pressed={providerMode === mode}
                  className={providerMode === mode ? "active" : ""}
                  key={mode}
                  type="button"
                  onClick={() => setProviderMode(mode)}
                >
                  {mode === "local" ? "Local" : "BYOK"}
                </button>
              ))}
            </div>
          </div>

          <div className={desktopRuntime ? "provider-check-status ready" : "provider-check-status pending"}>
            <span>{providerStatusText}</span>
          </div>

          {providerMode === "local" ? (
            <div className="provider-list" aria-label="Local providers">
              {localAdapters.map((adapter) => (
                <ProviderCard
                  key={adapter.id}
                  adapter={adapter}
                  selected={selectedLocalAdapterId === adapter.id}
                  onSelect={() => setSelectedLocalAdapterId(adapter.id)}
                />
              ))}
            </div>
          ) : null}

          {providerMode === "byok" ? (
            <div className="byok-form">
              <label>
                <span>Provider</span>
                <div className="byok-provider-grid" role="radiogroup" aria-label="BYOK provider">
                  {byokProviders.map((provider) => (
                    <button
                      aria-checked={selectedByokProviderId === provider.id}
                      className={selectedByokProviderId === provider.id ? "active" : ""}
                      key={provider.id}
                      role="radio"
                      type="button"
                      onClick={() => handleByokProviderChange(provider.id)}
                    >
                      {provider.name}
                    </button>
                  ))}
                </div>
              </label>
              <label>
                <span>API key</span>
                <input
                  aria-label={`${activeByokProvider.name} API key`}
                  placeholder={activeByokProvider.env}
                  type="password"
                  value={apiKey}
                  onChange={(event) => setApiKey(event.currentTarget.value)}
                />
              </label>
              <label>
                <span>Base URL</span>
                <input
                  aria-label={`${activeByokProvider.name} base URL`}
                  value={providerEndpoint}
                  onChange={(event) => setProviderEndpoint(event.currentTarget.value)}
                />
              </label>
              <label>
                <span>Model</span>
                <input
                  aria-label={`${activeByokProvider.name} model`}
                  value={providerModel}
                  onChange={(event) => setProviderModel(event.currentTarget.value)}
                />
              </label>
              <button type="button" disabled={providerBusy} onClick={handleSaveProvider}>
                <Cpu size={16} />
                {providerBusy ? "Working..." : "Save provider"}
              </button>
            </div>
          ) : null}

          <div className="provider-actions">
            {providerMode === "local" ? (
              <button
                className="secondary-button"
                type="button"
                disabled={!desktopRuntime || providerBusy}
                title={desktopRuntime ? "Refresh local provider detection" : "Available in the desktop app"}
                onClick={onRefreshProviders}
              >
                <RefreshCw size={14} />
                Check installed providers
              </button>
            ) : null}
            <button
              className="secondary-button"
              type="button"
              disabled={!desktopRuntime || providerBusy}
              title={desktopRuntime ? "Run a real provider smoke test" : "Available in the desktop app"}
              onClick={handleTestProvider}
            >
              <RefreshCw size={14} />
              {providerBusy ? "Testing..." : "Test selected provider"}
            </button>
          </div>

          {providerUiMessage ? (
            <div className="provider-check-status pending" role="status">
              <span>{providerUiMessage}</span>
            </div>
          ) : null}
        </div>
      </div>

      <div className="settings-section">
        <h2>About</h2>
        <div className="settings-about-block">
          <p className="settings-about">
            Strut Studio — AI-native motion design studio for interactive product graphics.
          </p>
          <p className="settings-version">Version 1.0.0</p>
          <div className="developer-links" aria-label="Developers">
            <span>Developers</span>
            <a href="https://github.com/s41r4j" target="_blank" rel="noreferrer">s41r4j</a>
            <a href="https://github.com/amaansyed27" target="_blank" rel="noreferrer">amaansyed27</a>
          </div>
        </div>
      </div>
    </section>
  );
}
