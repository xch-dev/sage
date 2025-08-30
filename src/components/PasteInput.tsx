import { cn } from '@/lib/utils';
import { platform } from '@tauri-apps/plugin-os';
import { ClipboardPasteIcon, ScanIcon } from 'lucide-react';
import { forwardRef } from 'react';
import { Input, InputProps } from './ui/input';

export interface PasteInputProps extends InputProps {
  className?: string;
  truncate?: boolean;
  value?: string;
  onEndIconClick?: () => void;
}

export const PasteInput = forwardRef<HTMLInputElement, PasteInputProps>(
  (
    {
      className = '',
      truncate = true,
      placeholder,
      onChange,
      value = '',
      onEndIconClick,
      ...props
    },
    ref,
  ) => {
    const isMobile = platform() === 'ios' || platform() === 'android';

    return (
      <div
        className={cn(
          'dynamic-input flex h-9 w-full items-center relative',
          className,
        )}
      >
        <Input
          ref={ref}
          type='text'
          placeholder={placeholder}
          className={cn(
            'pr-10 w-full focus:outline-none select-none',
            truncate && 'truncate',
          )}
          onChange={onChange}
          value={value}
          {...props}
        />
        <div
          className='absolute right-0 flex items-center pr-3 pointer-events-auto'
          onClick={onEndIconClick}
        >
          {isMobile ? (
            <ScanIcon className='h-4 w-4 text-muted-foreground hover:text-foreground cursor-pointer shrink-0' />
          ) : (
            <ClipboardPasteIcon className='h-4 w-4 text-muted-foreground hover:text-foreground cursor-pointer shrink-0' />
          )}
        </div>
      </div>
    );
  },
);

PasteInput.displayName = 'PasteInput';
