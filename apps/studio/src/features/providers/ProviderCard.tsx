import type { LocalAdapter } from '../../types';

type ProviderCardProps = {
  adapter: LocalAdapter;
  selected: boolean;
  onSelect: () => void;
};

export function ProviderCard({ adapter, selected, onSelect }: ProviderCardProps) {
  const unchecked =
    !adapter.installed &&
    (adapter.detail.toLowerCase().includes('desktop app') ||
      adapter.detail.toLowerCase().includes('desktop check') ||
      adapter.detail.toLowerCase().includes('browser preview'));
  const status = adapter.installed ? 'installed' : unchecked ? 'unchecked' : 'missing';

  return (
    <button
      className={`provider-list-item ${status} ${selected ? 'active' : ''}`}
      type="button"
      aria-pressed={selected}
      onClick={onSelect}
    >
      <div className="provider-row-main">
        <span className={`provider-status-dot ${status}`} />
        <strong>{adapter.name}</strong>
        <span className="provider-kind">{adapter.kind}</span>
      </div>
      <div className="provider-row-meta">
        <span className="provider-detail">{adapter.detail}</span>
        {selected ? <span className="badge badge-selected">Selected</span> : null}
        {adapter.installed ? (
          <span className="badge badge-installed">Installed</span>
        ) : unchecked ? (
          <span className="badge badge-pending">Not checked</span>
        ) : (
          <span className="badge badge-missing">Not found</span>
        )}
      </div>
    </button>
  );
}
