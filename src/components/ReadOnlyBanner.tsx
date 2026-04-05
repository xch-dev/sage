import { useWallet } from '@/contexts/WalletContext';
import { Trans } from '@lingui/react/macro';
import { EyeIcon } from 'lucide-react';

export function ReadOnlyBanner() {
  const { isReadOnly } = useWallet();

  if (!isReadOnly) return null;

  return (
    <div
      className='flex items-center gap-2 px-4 py-1.5 text-xs bg-muted border-b text-muted-foreground'
      role='status'
      aria-label='Read-only wallet'
      aria-live='polite'
      aria-atomic='true'
    >
      <EyeIcon className='h-3 w-3 flex-shrink-0' aria-hidden='true' />
      <span>
        <Trans>Read-only wallet — transactions require private keys</Trans>
      </span>
    </div>
  );
}
