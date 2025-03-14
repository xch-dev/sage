import { DidRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ActivityIcon, UserRoundIcon } from 'lucide-react';
import { toast } from 'react-toastify';

interface DidNormalizeConfirmationProps {
  dids: DidRecord[];
}

export function DidNormalizeConfirmation({
  dids,
}: DidNormalizeConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-green-50 dark:bg-green-950 border border-green-200 dark:border-green-800 rounded-md text-green-800 dark:text-green-300'>
        <div className='font-medium mb-1 flex items-center'>
          <ActivityIcon className='h-3 w-3 mr-1' />
          <Trans>Normalization</Trans>
        </div>
        <div>
          {dids.length > 1 ? (
            <Trans>
              These profiles will be normalized to be compatible with the Chia
              reference wallet.
            </Trans>
          ) : (
            <Trans>
              This profile will be normalized to be compatible with the Chia
              reference wallet.
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
