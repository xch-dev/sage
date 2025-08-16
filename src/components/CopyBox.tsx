import { cn } from '@/lib/utils';
import { CopyButton } from './CopyButton';
import { Input } from './ui/input';

interface CopyBoxProps {
  title: string;
  value: string;
  displayValue?: string;
  className?: string;
  truncate?: boolean;
  inputRef?: React.RefObject<HTMLInputElement>;
  onCopy?: () => void;
  id?: string;
  'aria-label'?: string;
  'aria-describedby'?: string;
}

export function CopyBox(props: CopyBoxProps) {
  const truncate = props.truncate ?? true;
  const inputId =
    props.id || `copy-box-input-${Math.random().toString(36).substring(2, 9)}`;

  return (
    <div className={cn('flex rounded-md shadow-sm', props.className)}>
      <Input
        id={inputId}
        ref={props.inputRef}
        title={props.title}
        value={props.displayValue ?? props.value}
        type='text'
        readOnly
        aria-label={props['aria-label'] || props.title}
        aria-describedby={props['aria-describedby']}
        className={cn(
          'rounded-r-none border-r-0 font-mono tracking-tight',
          truncate && 'truncate',
        )}
      />
      <CopyButton
        value={props.value}
        onCopy={props.onCopy}
        className='relative rounded-none -ml-px inline-flex items-center justify-center h-9 w-9 rounded-r-md border border-input bg-background text-foreground shadow-button hover:bg-accent hover:text-accent-foreground '
        aria-label={`Copy ${props.value}`}
      />
    </div>
  );
}
