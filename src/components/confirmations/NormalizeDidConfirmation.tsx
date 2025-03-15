import { DidRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ActivityIcon, UserRoundIcon } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface NormalizeDidConfirmationProps {
  dids: DidRecord[];
}

export function NormalizeDidConfirmation({
  dids,
}: NormalizeDidConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={ActivityIcon}
        title={<Trans>Normalize Profiles</Trans>}
        variant='info'
      >
        {dids.length > 1 ? (
          <Trans>
            These profiles will be normalized to ensure they are in a consistent
            state.
          </Trans>
        ) : (
          <Trans>
            This profile will be normalized to ensure it is in a consistent
            state.
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
