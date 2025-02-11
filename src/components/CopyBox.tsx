import { CopyButton } from './CopyButton';

interface CopyBoxProps {
  title: string;
  value: string;
  displayValue?: string;
  className?: string;
  truncate?: boolean;
  inputRef?: React.RefObject<HTMLInputElement>;
  onCopy?: () => void;
}

export function CopyBox(props: CopyBoxProps) {
  const truncate = props.truncate ?? true;
  return (
    <div className={`flex rounded-md shadow-sm max-w-x ${props.className}`}>
      <input
        ref={props.inputRef}
        title={props.title}
        type='text'
        value={props.displayValue ?? props.value}
        readOnly
        className={`block w-full text-sm rounded-none rounded-l-md border-0 py-1.5 px-2 ${
          truncate ? 'truncate' : ''
        } text-muted-foreground bg-white text-neutral-950 dark:bg-neutral-900 dark:text-neutral-50 font-mono tracking-tight ring-1 ring-inset ring-neutral-200 dark:ring-neutral-800 sm:leading-6`}
      />
      <CopyButton
        value={props.value}
        onCopy={props.onCopy}
        className='relative rounded-none -ml-px inline-flex items-center gap-x-1.5 rounded-r-md px-3 py-2 text-sm font-semibold ring-1 ring-inset ring-neutral-200 dark:ring-neutral-800 hover:bg-gray-50 bg-white text-neutral-950 dark:bg-neutral-900 dark:text-neutral-50 '
      />
    </div>
  );
}
