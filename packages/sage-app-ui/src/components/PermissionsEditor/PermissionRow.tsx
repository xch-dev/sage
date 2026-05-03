import type { PermissionEntry } from './types';
import { CapabilityHelp } from './CapabilityHelp';

export function PermissionRow({
  entry,
  editable,
  onToggle,
}: {
  entry: PermissionEntry;
  editable: boolean;
  onToggle: (entry: PermissionEntry, nextGranted: boolean) => void;
}) {
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
        <span
          className={
            entry.kind === 'network'
              ? 'truncate font-mono text-sm'
              : 'truncate font-medium'
          }
        >
          {entry.label}
        </span>

        {entry.kind === 'capability' ? (
          <CapabilityHelp description={entry.description} />
        ) : null}
      </div>
    </label>
  );
}
