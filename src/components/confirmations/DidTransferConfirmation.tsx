import { DidRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { SendIcon, UserRoundIcon } from 'lucide-react';
import { toast } from 'react-toastify';

interface DidTransferConfirmationProps {
  dids: DidRecord[];
  address: string;
}

export function DidTransferConfirmation({
  dids,
  address,
}: DidTransferConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-800 rounded-md text-amber-800 dark:text-amber-300'>
        <div className='font-medium mb-1 flex items-center'>
          <SendIcon className='h-3 w-3 mr-1' />
          <Trans>Transfer Details</Trans>
        </div>
        <div>
          {dids.length > 1 ? (
            <Trans>
              These profiles will be transferred to the address below.
            </Trans>
          ) : (
            <Trans>
              This profile will be transferred to the address below.
            </Trans>
          )}
        </div>
      </div>

      <CopyBox
        title={t`Recipient Address`}
        value={address}
        onCopy={() => toast.success(t`Address copied to clipboard`)}
      />

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
