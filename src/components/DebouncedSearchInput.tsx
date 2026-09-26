import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { SearchIcon, XIcon } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';

export interface DebouncedSearchInputProps {
  value: string | null;
  onChange: (value: string | null) => void;
  placeholder: string;
  disabled?: boolean;
  delay?: number;
  className?: string;
}

export function DebouncedSearchInput({
  value,
  onChange,
  placeholder,
  disabled,
  delay = 400,
  className,
}: DebouncedSearchInputProps) {
  const [text, setText] = useState(value ?? '');
  const timerRef = useRef<ReturnType<typeof setTimeout>>();
  const valueRef = useRef(value);
  const onChangeRef = useRef(onChange);
  // The last value this input reported, so its echo back through `value`
  // is not mistaken for an external change.
  const lastSentRef = useRef(value);

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    valueRef.current = value;
    if (value === lastSentRef.current) return;
    // External change (e.g. "Clear filters"): drop any pending keystrokes.
    clearTimeout(timerRef.current);
    lastSentRef.current = value;
    setText(value ?? '');
  }, [value]);

  useEffect(() => () => clearTimeout(timerRef.current), []);

  const update = (next: string) => {
    setText(next);
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      const normalized = next.trim() === '' ? null : next;
      if (normalized !== (valueRef.current || null)) {
        lastSentRef.current = normalized;
        onChangeRef.current(normalized);
      }
    }, delay);
  };

  return (
    <div className={cn('relative flex-1', className)} role='search'>
      <div className='relative'>
        <SearchIcon
          className='absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground'
          aria-hidden='true'
        />
        <Input
          value={text}
          aria-label={placeholder}
          title={placeholder}
          placeholder={placeholder}
          onChange={(e) => update(e.target.value)}
          className='w-full pl-8 pr-8'
          disabled={disabled}
          aria-disabled={disabled}
        />
      </div>
      {text && (
        <Button
          variant='ghost'
          size='icon'
          title={t`Clear search`}
          aria-label={t`Clear search`}
          className='absolute right-0 top-0 h-full px-2 hover:bg-transparent'
          onClick={() => update('')}
          disabled={disabled}
        >
          <XIcon className='h-4 w-4' aria-hidden='true' />
        </Button>
      )}
    </div>
  );
}
