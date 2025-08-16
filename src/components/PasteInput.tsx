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
          'dynamic-input flex h-9 w-full items-center rounded-md border border-input shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-within:outline-none focus-within:ring-1 focus-within:ring-ring disabled:cursor-not-allowed disabled:opacity-50',
          className,
        )}
      >
        <Input
          ref={ref}
          type='text'
          placeholder={placeholder}
          className={cn(
            'border-0 ring-0 focus-visible:ring-0 rounded-none shadow-none px-3 w-full bg-transparent focus:outline-none select-none truncate',
            truncate && 'truncate',
          )}
          onChange={onChange}
          value={value}
          {...props}
        />
        <div className='flex items-center pr-3' onClick={onEndIconClick}>
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
