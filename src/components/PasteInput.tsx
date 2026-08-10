import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { platform } from '@tauri-apps/plugin-os';
import { ClipboardPasteIcon, ImageIcon, ScanIcon } from 'lucide-react';
import { forwardRef, useRef } from 'react';
import { Input, InputProps } from './ui/input';

export interface PasteInputProps extends InputProps {
  className?: string;
  truncate?: boolean;
  value?: string;
  onEndIconClick?: () => void;
  onScanImage?: (file: File) => void;
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
      onScanImage,
      ...props
    },
    ref,
  ) => {
    const isMobile = platform() === 'ios' || platform() === 'android';

    const fileInputRef = useRef<HTMLInputElement>(null);

    const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      // Clear first so picking the same file twice still fires a change event.
      event.target.value = '';
      if (file) {
        onScanImage?.(file);
      }
    };

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
            onScanImage ? 'pr-16' : 'pr-10',
            'w-full focus:outline-none select-none',
            truncate && 'truncate',
          )}
          onChange={onChange}
          value={value}
          {...props}
        />
        <div className='absolute right-0 flex items-center gap-2 pr-3 pointer-events-auto'>
          {onScanImage && (
            <>
              <input
                ref={fileInputRef}
                type='file'
                accept='image/*'
                className='hidden'
                onChange={handleFileChange}
              />
              <ImageIcon
                aria-label={t`Scan QR code from an image`}
                role='button'
                className='h-4 w-4 text-muted-foreground hover:text-foreground cursor-pointer shrink-0'
                onClick={() => fileInputRef.current?.click()}
              />
            </>
          )}
          <div onClick={onEndIconClick}>
            {isMobile ? (
              <ScanIcon
                aria-label={t`Scan QR code with the camera`}
                role='button'
                className='h-4 w-4 text-muted-foreground hover:text-foreground cursor-pointer shrink-0'
              />
            ) : (
              <ClipboardPasteIcon
                aria-label={t`Paste from clipboard`}
                role='button'
                className='h-4 w-4 text-muted-foreground hover:text-foreground cursor-pointer shrink-0'
              />
            )}
          </div>
        </div>
      </div>
    );
  },
);

PasteInput.displayName = 'PasteInput';
