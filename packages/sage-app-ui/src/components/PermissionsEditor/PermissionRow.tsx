import type { NetworkPermissionScheme, PermissionEntry } from './types';
import { CapabilityHelp } from './CapabilityHelp';

function NetworkSchemeButton({
  scheme,
  checked,
  disabled,
  onClick,
}: {
  scheme: NetworkPermissionScheme;
  checked: boolean;
  disabled: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type='button'
      disabled={disabled}
      onClick={(event) => {
        event.preventDefault();
        onClick();
      }}
      className={[
        'h-7 min-w-12 px-2 text-xs font-medium transition-colors',
        checked
          ? 'bg-primary text-primary-foreground'
          : 'bg-muted text-muted-foreground hover:bg-muted/80 hover:text-foreground',
        disabled ? 'cursor-not-allowed opacity-70' : '',
      ].join(' ')}
    >
      {scheme}
    </button>
  );
}

export function PermissionRow({
  entry,
  editable,
  onToggle,
}: {
  entry: PermissionEntry;
  editable: boolean;
  onToggle: (
    entry: PermissionEntry,
    nextGranted: boolean,
    scheme?: NetworkPermissionScheme,
  ) => void;
}) {
  if (entry.kind === 'network') {
    const visibleSchemes = (['http', 'https', 'wss'] as const).filter(
      (scheme) => entry.schemes[scheme].visible,
    );

    const primaryScheme = visibleSchemes.includes('wss')
      ? 'wss'
      : visibleSchemes.includes('https')
        ? 'https'
        : 'http';
    const primaryState = entry.schemes[primaryScheme];
    const checkboxDisabled = !editable || primaryState.disabled;

    return (
      <div className='flex min-h-9 items-center gap-2.5 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-muted/50'>
        <input
          type='checkbox'
          checked={primaryState.granted}
          disabled={checkboxDisabled}
          onChange={(event) => {
            onToggle(entry, event.target.checked, primaryScheme);
          }}
          className='h-4 w-4 shrink-0'
        />

        <div className='min-w-0 flex items-center gap-2'>
          <span className='truncate font-mono text-sm'>{entry.host}</span>

          <div className='inline-flex overflow-hidden rounded-md border border-border'>
            {visibleSchemes.map((scheme, index) => {
              const state = entry.schemes[scheme];

              return (
                <button
                  key={scheme}
                  type='button'
                  disabled={!editable || state.disabled}
                  onClick={(event) => {
                    event.preventDefault();
                    onToggle(entry, !state.granted, scheme);
                  }}
                  className={[
                    'h-6 px-2 text-[11px] font-medium transition-colors',
                    index > 0 ? 'border-l border-border' : '',
                    state.granted
                      ? 'bg-primary text-primary-foreground'
                      : 'text-muted-foreground hover:bg-muted hover:text-foreground',
                    state.disabled ? 'opacity-60 cursor-not-allowed' : '',
                  ].join(' ')}
                >
                  {scheme.toUpperCase()}
                </button>
              );
            })}
          </div>
        </div>
      </div>
    );
  }

  return (
    <label className='flex min-h-9 items-center gap-2.5 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-muted/50'>
      <input
        type='checkbox'
        checked={entry.granted}
        disabled={!editable || entry.required}
        onChange={(event) => {
          onToggle(entry, event.target.checked);
        }}
        className='h-4 w-4 shrink-0'
      />

      <div className='min-w-0 flex items-center gap-1.5'>
        <span className='truncate font-medium'>{entry.label}</span>
        <CapabilityHelp description={entry.description} />
      </div>
    </label>
  );
}
