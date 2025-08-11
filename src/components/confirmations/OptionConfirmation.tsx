import { OptionRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { FilePenLine, Flame, HandCoins, SendIcon } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface OptionConfirmationProps {
  type: 'burn' | 'transfer' | 'exercise';
  options: OptionRecord[];
  address?: string; // Only required for transfer
}

export function OptionConfirmation({
  type,
  options,
  address,
}: OptionConfirmationProps) {
  const config = {
    burn: {
      icon: Flame,
      title: <Trans>Warning</Trans>,
      variant: 'danger' as const,
      message:
        options.length > 1 ? (
          <Trans>
            These options will be permanently deleted by sending them to the
            burn address. The underlying asset will still be returned to the
            minter, but only after the option expires.
          </Trans>
        ) : (
          <Trans>
            This option will be permanently deleted by sending it to the burn
            address. The underlying asset will still be returned to the minter,
            but only after the option expires.
          </Trans>
        ),
    },
    transfer: {
      icon: SendIcon,
      title: <Trans>Transfer Details</Trans>,
      variant: 'warning' as const,
      message:
        options.length > 1 ? (
          <Trans>
            These options will be transferred to the address below. Please note
            that only Sage supports option contracts at this time.
          </Trans>
        ) : (
          <Trans>
            This option will be transferred to the address below. Please note
            that only Sage supports option contracts at this time.
          </Trans>
        ),
    },
    exercise: {
      icon: HandCoins,
      title: <Trans>Exercise Details</Trans>,
      variant: 'success' as const,
      message:
        options.length > 1 ? (
          <Trans>These options will be exercised.</Trans>
        ) : (
          <Trans>This option will be exercised.</Trans>
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

      {options.map((option) => (
        <ConfirmationCard
          key={option.launcher_id}
          icon={<FilePenLine className='h-8 w-8 text-purple-500' />}
          title={option.name}
        >
          <CopyBox
            title={t`Option ID`}
            value={option.launcher_id}
            onCopy={() => toast.success(t`Option ID copied to clipboard`)}
          />
        </ConfirmationCard>
      ))}
    </div>
  );
}
