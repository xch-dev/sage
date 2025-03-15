import { Trans } from '@lingui/react/macro';
import { UserRoundPlus } from 'lucide-react';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface CreateProfileConfirmationProps {
  name: string;
}

export function CreateProfileConfirmation({
  name,
}: CreateProfileConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={UserRoundPlus}
        title={<Trans>Profile Creation</Trans>}
        variant='info'
      >
        <Trans>
          You are creating a new profile. This will generate a decentralized
          identifier (DID) that can be used to associate NFTs and other digital
          assets with your identity.
        </Trans>
      </ConfirmationAlert>

      <ConfirmationCard
        icon={<UserRoundPlus className='h-8 w-8 text-purple-500' />}
        title={name}
      >
        <div className='text-muted-foreground'>
          <Trans>This profile will be created on the blockchain.</Trans>
        </div>
      </ConfirmationCard>
    </div>
  );
}
