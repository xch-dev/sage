import { Trans } from '@lingui/react/macro';
import { CircleOff, ArrowUpIcon, ArrowDownIcon } from 'lucide-react';
import { OfferRecord } from '@/bindings';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Assets } from '@/components/OfferCard';

interface CancelOfferConfirmationProps {
  offer: OfferRecord;
}

export function CancelOfferConfirmation({
  offer,
}: CancelOfferConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-800 rounded-md text-amber-800 dark:text-amber-300'>
        <div className='font-medium mb-1 flex items-center'>
          <CircleOff className='h-3 w-3 mr-1' />
          <Trans>Cancel Details</Trans>
        </div>
        <div>
          <Trans>
            You are canceling this offer on-chain. This will prevent it from
            being taken even if someone has the original offer file.
          </Trans>
        </div>
      </div>

      <div className='border border-neutral-200 dark:border-neutral-800 rounded-md p-3'>
        <div className='font-medium mb-2'>
          <Trans>Offer Details</Trans>
        </div>
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
              <Trans>The assets you are offering.</Trans>
            </div>
            <Assets assets={offer.summary.maker} />
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
              <Trans>The assets being requested.</Trans>
            </div>
            <Assets assets={offer.summary.taker} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
