import { CircleHelp } from 'lucide-react';
import { resolveBackgroundTintWithAlpha } from '../../presentation';

export function CapabilityHelp({
  description,
}: {
  description: string | null;
}) {
  if (!description) return null;

  return (
    <span className='relative inline-flex shrink-0 group'>
      <button
        type='button'
        className='rounded-sm p-0.5 text-muted-foreground transition-colors hover:text-foreground'
        onClick={(event) => event.preventDefault()}
        aria-label='Permission details'
      >
        <CircleHelp className='h-3.5 w-3.5' />
      </button>

      <span
        className='pointer-events-none absolute bottom-6 left-1/2 z-50 hidden w-[min(18rem,calc(100vw-2rem))] -translate-x-1/2 rounded-md border p-2 text-left text-xs text-popover-foreground shadow-md group-hover:block'
        style={{
          background: resolveBackgroundTintWithAlpha(),
          backdropFilter: 'blur(16px)',
        }}
      >
        {description}
      </span>
    </span>
  );
}
