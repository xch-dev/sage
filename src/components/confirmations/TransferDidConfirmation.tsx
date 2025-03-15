import { DidRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { SendIcon, UserRoundIcon } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface TransferDidConfirmationProps {
  dids: DidRecord[];
  address: string;
}

export function TransferDidConfirmation({
  dids,
  address,
}: TransferDidConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={SendIcon}
        title={<Trans>Transfer Details</Trans>}
        variant='warning'
      >
        {dids.length > 1 ? (
          <Trans>
            These profiles will be transferred to the address below.
          </Trans>
        ) : (
          <Trans>This profile will be transferred to the address below.</Trans>
        )}
      </ConfirmationAlert>

      <CopyBox
        title={t`Recipient Address`}
        value={address}
        onCopy={() => toast.success(t`Address copied to clipboard`)}
      />

      {dids.map((did) => (
        <ConfirmationCard
          key={did.launcher_id}
          icon={<UserRoundIcon className='h-8 w-8 text-purple-500' />}
          title={did.name}
        >
          <CopyBox
            title={t`DID ID`}
            value={did.launcher_id}
            onCopy={() => toast.success(t`DID ID copied to clipboard`)}
          />
        </ConfirmationCard>
      ))}
    </div>
  );
}
