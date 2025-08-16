import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useRef, useState } from 'react';
import { CopyButton } from './CopyButton';
import { Input } from './ui/input';

interface CopyBoxProps {
  title: string;
  value: string;
  displayValue?: string;
  className?: string;
  truncate?: boolean;
  truncateMiddle?: boolean;
  inputRef?: React.RefObject<HTMLInputElement>;
  onCopy?: () => void;
  id?: string;
  'aria-label'?: string;
  'aria-describedby'?: string;
  inputClassName?: string;
}

export function CopyBox(props: CopyBoxProps) {
  const truncate = props.truncate ?? true;
  const truncateMiddle = props.truncateMiddle ?? false;
  const inputId =
    props.id || `copy-box-input-${Math.random().toString(36).substring(2, 9)}`;

  const inputRef = useRef<HTMLInputElement>(null);
  const [displayValue, setDisplayValue] = useState(
    props.displayValue ?? props.value,
  );

  // Helper function to measure text width using canvas
  const getTextWidth = (text: string, element: HTMLElement | null) => {
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context || !element) return 0;

    context.font = getComputedStyle(element).font;
    return context.measureText(text).width;
  };

  // Calculate display value with width-aware middle truncation
  useEffect(() => {
    if (!truncateMiddle || !inputRef.current) {
      setDisplayValue(props.displayValue ?? props.value);
      return;
    }

    const calculateTruncatedValue = () => {
      const input = inputRef.current;
      if (!input) return;

      const fullValue = props.displayValue ?? props.value;
      const ellipsis = '...';
      const inputWidth = input.clientWidth - 24; // Account for padding

      // First check if the full string fits
      const fullWidth = getTextWidth(fullValue, input);
      if (fullWidth <= inputWidth) {
        setDisplayValue(fullValue);
        return;
      }

      // Find the maximum characters that can fit
      let truncatedValue = fullValue;

      // Binary search to find the optimal truncation
      let left = 0;
      let right = Math.floor(fullValue.length / 2);

      while (left <= right) {
        const mid = Math.floor((left + right) / 2);
        const testValue =
          fullValue.substring(0, mid) +
          ellipsis +
          fullValue.substring(fullValue.length - mid);

        const testWidth = getTextWidth(testValue, input);

        if (testWidth <= inputWidth) {
          truncatedValue = testValue;
          left = mid + 1;
        } else {
          right = mid - 1;
        }
      }

      setDisplayValue(truncatedValue);
    };

    calculateTruncatedValue();

    // Recalculate on window resize
    const handleResize = () => calculateTruncatedValue();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [truncateMiddle, props.displayValue, props.value]);

  return (
    <div className={cn('flex rounded-md shadow-sm', props.className)}>
      <Input
        id={inputId}
        ref={truncateMiddle ? inputRef : props.inputRef}
        title={props.title}
        value={displayValue}
        type='text'
        readOnly
        aria-label={props['aria-label'] || props.title}
        aria-describedby={props['aria-describedby']}
        className={cn(
          'rounded-r-none border-r-0 font-mono tracking-tight',
          truncate && !truncateMiddle && 'truncate',
          props.inputClassName,
        )}
      />
      <CopyButton
        value={props.value}
        onCopy={props.onCopy}
        className='relative rounded-none -ml-px inline-flex items-center justify-center h-9 w-9 rounded-r-md border border-input bg-background text-foreground shadow-button hover:bg-accent hover:text-accent-foreground '
        aria-label={t`Copy ${props.value}`}
      />
    </div>
  );
}
