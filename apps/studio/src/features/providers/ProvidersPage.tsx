import type { ChangeEvent } from 'react';
import { Cpu, RefreshCw } from 'lucide-react';
import type { LocalAdapter } from '../../types';
import { byokProviders } from '../../types';
import { ProviderCard } from './ProviderCard';

type ProvidersPageProps = {
  providerMode: 'local' | 'byok';
  setProviderMode: (mode: 'local' | 'byok') => void;
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
  onSaveProvider: () => void;
  onTestProvider: () => void;
  activity: string;
  desktopRuntime: boolean;
};

export function ProvidersPage({
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
  onSaveProvider,
  onTestProvider,
  activity: _activity,
  desktopRuntime: _desktopRuntime,
}: ProvidersPageProps) {
  const activeLocalAdapter =
    localAdapters.find((a) => a.id === selectedLocalAdapterId) ?? localAdapters[0];
  const activeByokProvider =
    byokProviders.find((p) => p.id === selectedByokProviderId) ?? byokProviders[0];

  const activeProviderLabel =
    providerMode === 'local' ? activeLocalAdapter?.name ?? 'None' : activeByokProvider.name;
  const activeProviderType = providerMode === 'local' ? 'Local CLI' : 'BYOK';
  const activeProviderDetail =
    providerMode === 'local'
      ? (activeLocalAdapter?.detail ?? '')
      : `${providerModel.trim() || activeByokProvider.model} / ${providerEndpoint.trim() || activeByokProvider.endpoint}`;

  const handleByokProviderChange = (event: ChangeEvent<HTMLSelectElement>) => {
    const provider =
      byokProviders.find((item) => item.id === event.currentTarget.value) ?? byokProviders[0];
    setSelectedByokProviderId(provider.id);
    setProviderEndpoint(provider.endpoint);
    setProviderModel(provider.model);
  };

  return (
    <section className="provider-page page-shell">
      <div className="page-heading">
        <h1>Providers</h1>
        <p>Select the model or coding agent Strut should use for chat and generation.</p>
      </div>

      <div className="provider-header-card">
        <div className="provider-summary" data-testid="selected-provider-summary">
          <span>Selected</span>
          <strong>{activeProviderLabel}</strong>
          <em>{activeProviderType}</em>
          <p>{activeProviderDetail}</p>
        </div>
        <div className="provider-tabs" role="group" aria-label="Provider source">
          {(['local', 'byok'] as const).map((mode) => (
            <button
              aria-pressed={providerMode === mode}
              className={providerMode === mode ? 'active' : ''}
              key={mode}
              type="button"
              onClick={() => setProviderMode(mode)}
            >
              {mode === 'local' ? 'Local' : 'BYOK'}
            </button>
          ))}
        </div>
      </div>

      {providerMode === 'local' ? (
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

      {providerMode === 'byok' ? (
        <div className="byok-form">
          <label>
            <span>Provider</span>
            <select
              aria-label="BYOK provider"
              value={selectedByokProviderId}
              onChange={handleByokProviderChange}
            >
              {byokProviders.map((provider) => (
                <option key={provider.id} value={provider.id}>
                  {provider.name}
                </option>
              ))}
            </select>
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
          <button type="button" onClick={onSaveProvider}>
            <Cpu size={16} />
            Save provider
          </button>
        </div>
      ) : null}

      <div className="provider-actions">
        <button className="secondary-button" type="button" onClick={onTestProvider}>
          <RefreshCw size={14} />
          Test selected provider
        </button>
      </div>
    </section>
  );
}
