import { DidRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Flame, UserRoundIcon } from 'lucide-react';
import { toast } from 'react-toastify';

interface DidBurnConfirmationProps {
  dids: DidRecord[];
}

export function DidBurnConfirmation({ dids }: DidBurnConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-800 rounded-md text-red-800 dark:text-red-300'>
        <div className='font-medium mb-1 flex items-center'>
          <Flame className='h-3 w-3 mr-1' />
          <Trans>Warning</Trans>
        </div>
        <div>
          {dids.length > 1 ? (
            <Trans>
              This will permanently delete these profiles by sending them to the
              burn address.
            </Trans>
          ) : (
            <Trans>
              This will permanently delete this profile by sending it to the
              burn address.
            </Trans>
          )}
        </div>
      </div>

      {dids.map((did) => {
        const didName = did.name ?? t`Unnamed Profile`;

        return (
          <div
            key={did.launcher_id}
            className='flex items-start gap-3 border border-neutral-200 dark:border-neutral-800 rounded-md p-3'
          >
            <div className='overflow-hidden rounded-md flex-shrink-0 w-16 h-16 border border-neutral-200 dark:border-neutral-800 flex items-center justify-center bg-neutral-50 dark:bg-neutral-900'>
              <UserRoundIcon className='h-8 w-8 text-purple-500' />
            </div>
            <div className='break-words whitespace-pre-wrap flex-1'>
              <div className='font-medium'>{didName}</div>
              <CopyBox
                title={t`Profile ID`}
                value={did?.launcher_id ?? ''}
                onCopy={() => toast.success(t`Profile ID copied to clipboard`)}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}
