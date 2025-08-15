import { Button } from '@/components/ui/button';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import data from '@emoji-mart/data';
import Picker from '@emoji-mart/react';
import { Smile, X } from 'lucide-react';
import React, { useCallback, useState } from 'react';

interface EmojiData {
  native: string;
}

export interface EmojiPickerProps {
  value?: string | null;
  onChange: (emoji: string | null) => void;
  disabled?: boolean;
  placeholder?: string;
  className?: string;
  // New props for external trigger control
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children?: React.ReactNode;
}

export function EmojiPicker({
  value,
  onChange,
  disabled = false,
  placeholder = 'Choose emoji',
  className = '',
  open,
  onOpenChange,
  children,
}: EmojiPickerProps) {
  const [internalOpen, setInternalOpen] = useState(false);

  // Use external control if provided, otherwise use internal state
  const isOpen = open !== undefined ? open : internalOpen;
  const setIsOpen = onOpenChange || setInternalOpen;

  const handleEmojiSelect = useCallback(
    (emoji: EmojiData) => {
      onChange(emoji.native);
      setIsOpen(false);
    },
    [onChange, setIsOpen],
  );

  const handleClear = (e: React.MouseEvent) => {
    e.stopPropagation();
    onChange(null);
  };

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger asChild>
        {children ? (
          children
        ) : (
          <Button
            variant='outline'
            className={`w-[200px] justify-start text-left font-normal ${className}`}
            disabled={disabled}
          >
            {value ? (
              <div className='flex items-center justify-between w-full'>
                <span className='flex items-center gap-2'>
                  <span className='text-lg'>{value}</span>
                  <span className='text-sm text-muted-foreground'>
                    {placeholder}
                  </span>
                </span>
                <X
                  className='h-4 w-4 opacity-50 hover:opacity-100'
                  onClick={handleClear}
                />
              </div>
            ) : (
              <div className='flex items-center gap-2 text-muted-foreground'>
                <Smile className='h-4 w-4' />
                <span>{placeholder}</span>
              </div>
            )}
          </Button>
        )}
      </PopoverTrigger>
      <PopoverContent
        className='w-auto p-0'
        align='start'
        onClick={(e) => e.stopPropagation()}
      >
        {isOpen && (
          <Picker
            data={data}
            onEmojiSelect={handleEmojiSelect}
            theme='auto'
            previewPosition='none'
            searchPosition='top'
            navPosition='bottom'
            perLine={8}
            maxFrequentRows={2}
          />
        )}
      </PopoverContent>
    </Popover>
  );
}
