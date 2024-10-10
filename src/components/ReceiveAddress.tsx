import { useWalletState } from '@/state';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useState } from 'react';
import { Button } from './ui/button';
import { CopyCheckIcon, CopyIcon } from 'lucide-react';

export function ReceiveAddress(props: { className?: string }) {
  const walletState = useWalletState();
  const [addressCopied, setAddressCopied] = useState(false);

  const copyAddress = () => {
    writeText(walletState.sync.receive_address);

    setAddressCopied(true);
    setTimeout(() => setAddressCopied(false), 2000);
  };

  return (
    <div className={`flex rounded-md shadow-sm max-w-lg ${props.className}`}>
      <input
        title='Receive Address'
        type='text'
        value={walletState.sync.receive_address}
        readOnly
        className='block w-full text-sm rounded-none rounded-l-md border-0 py-1.5 px-2 truncate text-muted-foreground bg-background font-mono tracking-tight ring-1 ring-inset ring-neutral-200 dark:ring-neutral-800 sm:leading-6'
      />
      <Button
        size='icon'
        variant='ghost'
        className='relative rounded-none -ml-px inline-flex items-center gap-x-1.5 rounded-r-md px-3 py-2 text-sm font-semibold ring-1 ring-inset ring-neutral-200 dark:ring-neutral-800 hover:bg-gray-50'
        onClick={copyAddress}
      >
        {addressCopied ? (
          <CopyCheckIcon className='-ml-0.5 h-5 w-5 text-emerald-500' />
        ) : (
          <CopyIcon className='-ml-0.5 h-5 w-5 text-muted-foreground' />
        )}
      </Button>
    </div>
  );
}
