import type { SandboxTab } from '../sandboxState';

interface Props {
  activeTab: SandboxTab;
  currentEnabled: boolean;
  onChange: (tab: SandboxTab) => void;
}

const tabs: Array<{
  id: SandboxTab;
  label: string;
}> = [
  { id: 'effective', label: 'Effective gate' },
  { id: 'previous', label: 'Previous completed' },
  { id: 'current', label: 'Current running' },
];

export function SandboxTabs({ activeTab, currentEnabled, onChange }: Props) {
  return (
    <div className='grid grid-cols-3 rounded-xl border bg-background p-1'>
      {tabs.map((tab) => {
        const disabled = tab.id === 'current' && !currentEnabled;

        return (
          <button
            key={tab.id}
            type='button'
            disabled={disabled}
            onClick={() => onChange(tab.id)}
            className={[
              'rounded-lg px-2 py-1.5 text-xs font-medium transition-colors',
              activeTab === tab.id
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-muted hover:text-foreground',
              disabled
                ? 'cursor-not-allowed opacity-45 hover:bg-transparent'
                : '',
            ].join(' ')}
          >
            {tab.label}
          </button>
        );
      })}
    </div>
  );
}
