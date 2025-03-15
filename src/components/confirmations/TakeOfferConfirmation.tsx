import { Trans } from '@lingui/react/macro';
import { ArrowUpIcon, ArrowDownIcon, HandshakeIcon } from 'lucide-react';
import { OfferSummary } from '@/bindings';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Assets } from '@/components/OfferCard';

interface TakeOfferConfirmationProps {
  offer: OfferSummary;
}

export function TakeOfferConfirmation({ offer }: TakeOfferConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-800 rounded-md text-amber-800 dark:text-amber-300'>
        <div className='font-medium mb-1 flex items-center'>
          <HandshakeIcon className='h-3 w-3 mr-1' />
          <Trans>Taking Offer</Trans>
        </div>
        <div>
          <Trans>
            Taking this offer will send the assets you are paying to the
            recipient.
          </Trans>
        </div>
      </div>

      <div className='grid grid-cols-1 gap-2'>
        <Card>
          <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
            <CardTitle className='text-xs font-medium truncate flex items-center'>
              <ArrowUpIcon className='mr-2 h-3 w-3' />
              <Trans>Sending</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='text-[10px] text-muted-foreground mb-2'>
              <Trans>The assets you are paying.</Trans>
            </div>
            <Assets assets={offer.taker} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
            <CardTitle className='text-xs font-medium truncate flex items-center'>
              <ArrowDownIcon className='mr-2 h-3 w-3' />
              <Trans>Receiving</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='text-[10px] text-muted-foreground mb-2'>
              <Trans>The assets you will receive.</Trans>
            </div>
            <Assets assets={offer.maker} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
