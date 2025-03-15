import { Trans } from '@lingui/react/macro';
import { ArrowUpIcon, ArrowDownIcon, HandshakeIcon } from 'lucide-react';
import { OfferSummary } from '@/bindings';
import { Assets } from '@/components/OfferCard';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface TakeOfferConfirmationProps {
  offer: OfferSummary;
}

export function TakeOfferConfirmation({ offer }: TakeOfferConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={HandshakeIcon}
        title={<Trans>Taking Offer</Trans>}
        variant='info'
      >
        <Trans>
          Taking this offer will send the assets you are paying to the
          recipient.
        </Trans>
      </ConfirmationAlert>

      <div className='grid grid-cols-1 gap-2'>
        <ConfirmationCard>
          <div className='flex items-center mb-2'>
            <ArrowUpIcon className='mr-2 h-3 w-3' />
            <span className='font-medium'>
              <Trans>Sending</Trans>
            </span>
          </div>
          <div className='text-[10px] text-muted-foreground mb-2'>
            <Trans>The assets you are paying.</Trans>
          </div>
          <Assets assets={offer.taker} />
        </ConfirmationCard>

        <ConfirmationCard>
          <div className='flex items-center mb-2'>
            <ArrowDownIcon className='mr-2 h-3 w-3' />
            <span className='font-medium'>
              <Trans>Receiving</Trans>
            </span>
          </div>
          <div className='text-[10px] text-muted-foreground mb-2'>
            <Trans>The assets you will receive.</Trans>
          </div>
          <Assets assets={offer.maker} />
        </ConfirmationCard>
      </div>
    </div>
  );
}
