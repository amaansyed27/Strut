import type { LocalAdapter } from '../../types';

type ProviderCardProps = {
  adapter: LocalAdapter;
  selected: boolean;
  onSelect: () => void;
};

export function ProviderCard({ adapter, selected, onSelect }: ProviderCardProps) {
  return (
    <button
      className={`provider-list-item ${selected ? 'active' : ''}`}
      type="button"
      aria-pressed={selected}
      onClick={onSelect}
    >
      <div className="provider-row-main">
        <span className={`provider-status-dot ${adapter.installed ? 'installed' : 'missing'}`} />
        <strong>{adapter.name}</strong>
        <span className="provider-kind">{adapter.kind}</span>
      </div>
      <div className="provider-row-meta">
        <span className="provider-detail">{adapter.detail}</span>
        {selected ? <span className="badge badge-selected">Selected</span> : null}
        {adapter.installed ? (
          <span className="badge badge-installed">Installed</span>
        ) : (
          <span className="badge badge-missing">Not found</span>
        )}
      </div>
    </button>
  );
}
