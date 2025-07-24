import { OfferRecord, OfferSummary } from '@/bindings';
import { Assets } from '@/components/Assets';
import { Trans } from '@lingui/react/macro';
import { ArrowDownIcon, ArrowUpIcon, CircleOff } from 'lucide-react';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface CancelOfferConfirmationProps {
  offers: (OfferSummary | OfferRecord)[];
  fee?: string;
}

export function CancelOfferConfirmation({
  offers,
  fee,
}: CancelOfferConfirmationProps) {
  const offerCount = offers.length;
  const isMultiple = offerCount > 1;

  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={CircleOff}
        title={<Trans>Cancel Details</Trans>}
        variant='warning'
      >
        {isMultiple ? (
          <Trans>
            You are canceling {offerCount} offers on-chain. This will prevent
            them from being taken even if someone has the original offer files.
            {fee && (
              <>
                {' '}
                The transaction fee of {fee} applies to each offer being
                canceled.
              </>
            )}
          </Trans>
        ) : (
          <Trans>
            You are canceling this offer on-chain. This will prevent it from
            being taken even if someone has the original offer file.
          </Trans>
        )}
      </ConfirmationAlert>

      <div className='space-y-4'>
        {offers.map((offer, index) => {
          const summary = 'summary' in offer ? offer.summary : offer;
          const hasOfferId = 'offer_id' in offer;

          return (
            // eslint-disable-next-line react/no-array-index-key
            <div key={index} className='space-y-2'>
              {isMultiple && (
                <div className='text-xs font-medium text-muted-foreground sticky top-0 bg-background py-1'>
                  <Trans>Offer {index + 1}</Trans>
                </div>
              )}

              {hasOfferId && (
                <ConfirmationCard title={<Trans>Offer Details</Trans>}>
                  <div className='space-y-2'>
                    <div className='flex justify-between items-center border-b border-neutral-100 dark:border-neutral-800 pb-1'>
                      <div className='text-muted-foreground'>
                        <Trans>Offer ID</Trans>
                      </div>
                      <div className='font-mono'>
                        {offer.offer_id.substring(0, 8)}...
                        {offer.offer_id.substring(offer.offer_id.length - 8)}
                      </div>
                    </div>
                    <div className='flex justify-between items-center'>
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
                      <Trans>Offering</Trans>
                    </span>
                  </div>
                  <div className='text-[10px] text-muted-foreground mb-2'>
                    <Trans>The assets you are offering.</Trans>
                  </div>
                  <Assets assets={summary.maker} />
                </ConfirmationCard>

                <ConfirmationCard>
                  <div className='flex items-center mb-2'>
                    <ArrowDownIcon className='mr-2 h-3 w-3' />
                    <span className='font-medium'>
                      <Trans>Requesting</Trans>
                    </span>
                  </div>
                  <div className='text-[10px] text-muted-foreground mb-2'>
                    <Trans>The assets being requested.</Trans>
                  </div>
                  <Assets assets={summary.taker} />
                </ConfirmationCard>
              </div>

              {index < offers.length - 1 && (
                <div className='border-t border-neutral-200 dark:border-neutral-700 pt-2 mt-4' />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
