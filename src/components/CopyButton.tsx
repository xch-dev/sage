import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { CopyCheckIcon, CopyIcon } from 'lucide-react';
import { useState } from 'react';
import { Button } from './ui/button';

export function CopyButton(props: { value: string; className?: string }) {
  const [copied, setCopied] = useState(false);

  const copyAddress = () => {
    writeText(props.value);

    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Button
      size='icon'
      variant='ghost'
      onClick={copyAddress}
      className={props.className}
    >
      {copied ? (
        <CopyCheckIcon className='h-5 w-5 text-emerald-500' />
      ) : (
        <CopyIcon className='h-5 w-5 text-muted-foreground' />
      )}
    </Button>
  );
}
