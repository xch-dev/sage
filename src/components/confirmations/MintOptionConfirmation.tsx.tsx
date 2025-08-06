import { Trans } from '@lingui/react/macro';
import { FilePenLine, UserRoundPlus } from 'lucide-react';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

export function MintOptionConfirmation() {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={UserRoundPlus}
        title={<Trans>Option Minting</Trans>}
        variant='info'
      >
        <Trans>
          You are minting a new option contract. This will lock up the
          underlying funds in an asset that you can trade or exercise until the
          expiration.
        </Trans>
      </ConfirmationAlert>

      <ConfirmationCard
        icon={<FilePenLine className='h-8 w-8 text-purple-500' />}
        title={<Trans>Option Contract</Trans>}
      >
        <div className='text-muted-foreground'>
          <Trans>This option contract will be created on the blockchain.</Trans>
        </div>
      </ConfirmationCard>
    </div>
  );
}
