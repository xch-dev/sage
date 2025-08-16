import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { CopyCheckIcon, CopyIcon } from 'lucide-react';
import { useState } from 'react';
import { Button } from './ui/button';

interface CopyButtonProps {
  value: string;
  className?: string;
  onCopy?: () => void;
  'aria-label'?: string;
}

export function CopyButton({
  value,
  className,
  onCopy,
  'aria-label': ariaLabel,
}: CopyButtonProps) {
  const [copied, setCopied] = useState(false);

  const copyAddress = () => {
    writeText(value);

    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
    onCopy?.();
  };

  return (
    <Button
      size='icon'
      variant='ghost'
      onClick={copyAddress}
      className={className}
      aria-label={ariaLabel || (copied ? 'Copied!' : `Copy ${value}`)}
      title={copied ? 'Copied!' : `Copy ${value}`}
    >
      {copied ? (
        <CopyCheckIcon
          className='h-5 w-5 text-emerald-500'
          aria-hidden='true'
        />
      ) : (
        <CopyIcon
          className='h-5 w-5 text-muted-foreground'
          aria-hidden='true'
        />
      )}
    </Button>
  );
}
