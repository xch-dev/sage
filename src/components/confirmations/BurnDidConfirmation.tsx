import { DidRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Flame, UserRoundIcon } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface BurnDidConfirmationProps {
  dids: DidRecord[];
}

export function BurnDidConfirmation({ dids }: BurnDidConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={Flame}
        title={<Trans>Warning</Trans>}
        variant='danger'
      >
        {dids.length > 1 ? (
          <Trans>
            These profiles will be permanently deleted by sending them to the
            burn address.
          </Trans>
        ) : (
          <Trans>
            This profile will be permanently deleted by sending it to the burn
            address.
          </Trans>
        )}
      </ConfirmationAlert>

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
