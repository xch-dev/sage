import { Trans } from '@lingui/react/macro';
import { UserRoundPlus } from 'lucide-react';

interface CreateProfileConfirmationProps {
  name: string;
}

export function CreateProfileConfirmation({
  name,
}: CreateProfileConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-purple-50 dark:bg-purple-950 border border-purple-200 dark:border-purple-800 rounded-md text-purple-800 dark:text-purple-300'>
        <div className='font-medium mb-1 flex items-center'>
          <UserRoundPlus className='h-3 w-3 mr-1' />
          <Trans>Profile Creation</Trans>
        </div>
        <div>
          <Trans>
            You are creating a new profile. This will generate a decentralized
            identifier (DID) that can be used to associate NFTs and other
            digital assets with your identity.
          </Trans>
        </div>
      </div>

      <div className='flex items-start gap-3 border border-neutral-200 dark:border-neutral-800 rounded-md p-3'>
        <div className='overflow-hidden rounded-md flex-shrink-0 w-16 h-16 border border-neutral-200 dark:border-neutral-800 flex items-center justify-center bg-neutral-50 dark:bg-neutral-900'>
          <UserRoundPlus className='h-8 w-8 text-purple-500' />
        </div>
        <div className='break-words whitespace-pre-wrap flex-1'>
          <div className='font-medium mb-2'>{name}</div>
          <div className='text-muted-foreground'>
            <Trans>This profile will be created on the blockchain.</Trans>
          </div>
        </div>
      </div>
    </div>
  );
}
