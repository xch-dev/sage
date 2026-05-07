import {
  formatCapabilityLabel,
  listSandboxCapabilities,
  type SandboxGateState,
} from '../sandboxState';

function statusClass(status: string) {
  switch (status) {
    case 'passed':
      return 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700';
    case 'failed':
      return 'border-destructive/40 bg-destructive/10 text-destructive';
    case 'running':
      return 'border-yellow-500/30 bg-yellow-500/10 text-yellow-700';
    case 'pending':
      return 'border-border bg-muted text-muted-foreground';
    default:
      return 'border-border bg-muted text-muted-foreground';
  }
}

function statusLabel(status: string) {
  return status.slice(0, 1).toUpperCase() + status.slice(1);
}

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

function normalizeCapabilityResult(value: unknown): {
  status: string;
  details: string | null;
} {
  if (!isObject(value)) {
    return {
      status: 'unknown',
      details: null,
    };
  }

  const status = typeof value.status === 'string' ? value.status : 'unknown';

  const details =
    typeof value.details === 'string' && value.details.length > 0
      ? value.details
      : null;

  return { status, details };
}

export function SandboxResultList({
  state,
  emptyText,
}: {
  state: SandboxGateState | null;
  emptyText: string;
}) {
  const entries = listSandboxCapabilities(state);

  if (!state || entries.length === 0) {
    return (
      <div className='rounded-xl border border-border bg-background/70 p-4 text-sm text-muted-foreground'>
        {emptyText}
      </div>
    );
  }

  return (
    <div className='space-y-3'>
      <div className='flex items-center justify-between gap-3 rounded-xl border bg-background/70 px-3 py-2'>
        <div>
          <div className='text-xs font-medium uppercase tracking-wide text-muted-foreground'>
            Overall status
          </div>
          <div className='mt-0.5 text-sm font-semibold'>
            {statusLabel(state.overallCriticalStatus)}
          </div>
        </div>

        <span
          className={[
            'rounded-full border px-2 py-0.5 text-xs font-medium',
            statusClass(state.overallCriticalStatus),
          ].join(' ')}
        >
          {state.overallCriticalStatus}
        </span>
      </div>

      <div className='overflow-hidden rounded-xl border bg-background/70'>
        {entries.map(([capability, rawResult], index) => {
          const result = normalizeCapabilityResult(rawResult);

          return (
            <div
              key={capability}
              className={[
                'p-3',
                index > 0 ? 'border-t border-border' : '',
              ].join(' ')}
            >
              <div className='flex items-start justify-between gap-3'>
                <div className='min-w-0'>
                  <div className='truncate text-sm font-medium'>
                    {formatCapabilityLabel(capability)}
                  </div>

                  {result.details ? (
                    <div className='mt-1 text-xs text-muted-foreground'>
                      {result.details}
                    </div>
                  ) : null}
                </div>

                <span
                  className={[
                    'shrink-0 rounded-full border px-2 py-0.5 text-xs font-medium',
                    statusClass(result.status),
                  ].join(' ')}
                >
                  {result.status}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
