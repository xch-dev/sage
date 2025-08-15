import { CopyButton } from './CopyButton';

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
    <div className={`flex rounded-md shadow-sm max-w-x ${props.className}`}>
      <input
        id={inputId}
        ref={props.inputRef}
        title={props.title}
        type='text'
        value={props.displayValue ?? props.value}
        readOnly
        aria-label={props['aria-label'] || props.title}
        aria-describedby={props['aria-describedby']}
        className={`block w-full text-sm rounded-none rounded-l-md border-0 py-1.5 px-2 ${
          truncate ? 'truncate' : ''
        } text-muted-foreground bg-background text-foreground font-mono tracking-tight ring-1 ring-inset ring-border sm:leading-6`}
      />
      <CopyButton
        value={props.value}
        onCopy={props.onCopy}
        className='relative rounded-none -ml-px inline-flex items-center gap-x-1.5 rounded-r-md px-3 py-2 text-sm font-semibold ring-1 ring-inset ring-border hover:bg-accent bg-background text-foreground '
        aria-label={`Copy ${props.value}`}
      />
    </div>
  );
}
