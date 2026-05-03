import { CircleHelp } from 'lucide-react';

export function CapabilityHelp({
  description,
}: {
  description: string | null;
}) {
  if (!description) return null;

  return (
    <span className='relative inline-flex group'>
      <button
        type='button'
        className='shrink-0 rounded-sm p-0.5 text-muted-foreground transition-colors hover:text-foreground'
        onClick={(event) => event.preventDefault()}
        aria-label='Permission details'
      >
        <CircleHelp className='h-4 w-4' />
      </button>

      <span className='pointer-events-none absolute right-0 top-6 z-50 hidden w-72 rounded-md border bg-popover p-2 text-left text-xs text-popover-foreground shadow-md group-hover:block'>
        {description}
      </span>
    </span>
  );
}
