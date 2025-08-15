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
        className={`block w-full text-sm rounded-none rounded-l-md border border-input py-1.5 px-2 ${
          truncate ? 'truncate' : ''
        } bg-input text-foreground font-mono tracking-tight shadow-sm sm:leading-6`}
      />
      <CopyButton
        value={props.value}
        onCopy={props.onCopy}
        className='relative rounded-none -ml-px inline-flex items-center gap-x-1.5 rounded-r-md px-3 py-2 text-sm font-semibold border border-input bg-secondary text-secondary-foreground shadow-button hover:bg-accent hover:text-accent-foreground '
        aria-label={`Copy ${props.value}`}
      />
    </div>
  );
}
