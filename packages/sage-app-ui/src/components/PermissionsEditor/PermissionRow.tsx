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
    <label className='flex items-start gap-3 rounded-xl border border-border px-3 py-3 text-sm'>
      <input
        type='checkbox'
        checked={entry.granted}
        disabled={!editable || entry.required}
        onChange={(event) => {
          onToggle(entry, event.target.checked);
        }}
        className='mt-1 h-4 w-4'
      />

      <div className='min-w-0 flex-1'>
        <div className='flex items-center gap-2'>
          <div
            className={
              entry.kind === 'network'
                ? 'min-w-0 flex-1 truncate font-mono text-sm'
                : 'min-w-0 flex-1 truncate font-medium'
            }
          >
            {entry.label}
          </div>

          {entry.kind === 'capability' ? (
            <CapabilityHelp description={entry.description} />
          ) : null}

          {entry.required ? (
            <span className='rounded-full border border-border px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted-foreground'>
              Required
            </span>
          ) : null}
        </div>
      </div>
    </label>
  );
}
