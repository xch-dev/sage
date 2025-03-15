import { DidRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ActivityIcon, Flame, SendIcon, UserRoundIcon } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface DidConfirmationProps {
  type: 'burn' | 'transfer' | 'normalize';
  dids: DidRecord[];
  address?: string; // Only required for transfer
}

export function DidConfirmation({ type, dids, address }: DidConfirmationProps) {
  const config = {
    burn: {
      icon: Flame,
      title: <Trans>Warning</Trans>,
      variant: 'danger' as const,
      message:
        dids.length > 1 ? (
          <Trans>
            These profiles will be permanently deleted by sending them to the
            burn address.
          </Trans>
        ) : (
          <Trans>
            This profile will be permanently deleted by sending it to the burn
            address.
          </Trans>
        ),
    },
    transfer: {
      icon: SendIcon,
      title: <Trans>Transfer Details</Trans>,
      variant: 'warning' as const,
      message:
        dids.length > 1 ? (
          <Trans>
            These profiles will be transferred to the address below.
          </Trans>
        ) : (
          <Trans>This profile will be transferred to the address below.</Trans>
        ),
    },
    normalize: {
      icon: ActivityIcon,
      title: <Trans>Normalize Profiles</Trans>,
      variant: 'info' as const,
      message:
        dids.length > 1 ? (
          <Trans>
            These profiles will be normalized to ensure they are in a consistent
            state.
          </Trans>
        ) : (
          <Trans>
            This profile will be normalized to ensure it is in a consistent
            state.
          </Trans>
        ),
    },
  };

  const { icon, title, variant, message } = config[type];

  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert icon={icon} title={title} variant={variant}>
        {message}
      </ConfirmationAlert>

      {type === 'transfer' && address && (
        <CopyBox
          title={t`Recipient Address`}
          value={address}
          onCopy={() => toast.success(t`Address copied to clipboard`)}
        />
      )}

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
