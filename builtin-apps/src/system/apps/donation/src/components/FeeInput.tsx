import { ChevronDown } from 'lucide-react';
import { useState } from 'react';

interface Props {
  value: string;
  onChange: (value: string) => void;
}

export function FeeInput({ value, onChange }: Props) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div>
      <button
        type='button'
        onClick={() => setExpanded((v) => !v)}
        className='group flex items-center gap-1.5 rounded-md px-1 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground'
      >
        <ChevronDown
          className={[
            'h-3.5 w-3.5 transition-transform',
            expanded ? 'rotate-180' : '',
          ].join(' ')}
        />

        <span>Network fee: {value || '0'} XCH</span>
      </button>

      {expanded ? (
        <label className='mt-2 block'>
          <div className='mb-1 text-sm font-medium'>Network fee</div>

          <div className='flex items-center gap-2 rounded-lg border border-border px-3 py-2'>
            <input
              value={value}
              onChange={(event) => onChange(event.target.value)}
              placeholder='0.0001'
              inputMode='decimal'
              className='min-w-0 flex-1 bg-transparent text-right text-sm outline-none'
            />

            <span className='text-xs font-medium text-muted-foreground'>
              XCH
            </span>
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            Higher fee may help the transaction confirm faster.
          </div>
        </label>
      ) : null}
    </div>
  );
}
