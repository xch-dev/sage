import { OfferRecord, OfferSummary } from '@/bindings';
import { Assets } from '@/components/OfferCard';
import { Trans } from '@lingui/react/macro';
import {
  ArrowUpIcon,
  ArrowDownIcon,
  HandshakeIcon,
  CircleOff,
} from 'lucide-react';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

type OfferOperationType = 'take' | 'cancel';

interface OfferConfirmationProps {
  type: OfferOperationType;
  offer: OfferSummary | OfferRecord;
}

export function OfferConfirmation({ type, offer }: OfferConfirmationProps) {
  const config = {
    take: {
      icon: HandshakeIcon,
      title: <Trans>Taking Offer</Trans>,
      variant: 'info' as const,
      message: (
        <Trans>
          Taking this offer will send the assets you are paying to the
          recipient.
        </Trans>
      ),
    },
    cancel: {
      icon: CircleOff,
      title: <Trans>Cancel Details</Trans>,
      variant: 'warning' as const,
      message: (
        <Trans>
          You are canceling this offer on-chain. This will prevent it from being
          taken even if someone has the original offer file.
        </Trans>
      ),
    },
  };

  const { icon: Icon, title, variant, message } = config[type];
  const summary = 'summary' in offer ? offer.summary : offer;

  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert icon={Icon} title={title} variant={variant}>
        {message}
      </ConfirmationAlert>

      {type === 'cancel' && 'offer_id' in offer && (
        <ConfirmationCard title={<Trans>Offer Details</Trans>}>
          <div className='space-y-2'>
            <div className='flex justify-between items-center border-b border-neutral-100 dark:border-neutral-800 pb-1 last:border-0 last:pb-0'>
              <div className='text-muted-foreground'>
                <Trans>Offer ID</Trans>
              </div>
              <div className='font-mono'>
                {offer.offer_id.substring(0, 8)}...
                {offer.offer_id.substring(offer.offer_id.length - 8)}
              </div>
            </div>
            <div className='flex justify-between items-center border-b border-neutral-100 dark:border-neutral-800 pb-1 last:border-0 last:pb-0'>
              <div className='text-muted-foreground'>
                <Trans>Status</Trans>
              </div>
              <div className='capitalize'>{offer.status}</div>
            </div>
          </div>
        </ConfirmationCard>
      )}

      <div className='grid grid-cols-1 gap-2'>
        <ConfirmationCard>
          <div className='flex items-center mb-2'>
            <ArrowUpIcon className='mr-2 h-3 w-3' />
            <span className='font-medium'>
              <Trans>Sending</Trans>
            </span>
          </div>
          <div className='text-[10px] text-muted-foreground mb-2'>
            {type === 'take' ? (
              <Trans>The assets you are paying.</Trans>
            ) : (
              <Trans>The assets you are offering.</Trans>
            )}
          </div>
          <Assets assets={type === 'take' ? summary.taker : summary.maker} />
        </ConfirmationCard>

        <ConfirmationCard>
          <div className='flex items-center mb-2'>
            <ArrowDownIcon className='mr-2 h-3 w-3' />
            <span className='font-medium'>
              <Trans>Receiving</Trans>
            </span>
          </div>
          <div className='text-[10px] text-muted-foreground mb-2'>
            {type === 'take' ? (
              <Trans>The assets you will receive.</Trans>
            ) : (
              <Trans>The assets being requested.</Trans>
            )}
          </div>
          <Assets assets={type === 'take' ? summary.maker : summary.taker} />
        </ConfirmationCard>
      </div>
    </div>
  );
}
