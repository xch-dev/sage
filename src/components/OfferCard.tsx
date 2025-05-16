import { commands, OfferSummary } from '@/bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { fromMojos, formatTimestamp } from '@/lib/utils';
import { useWalletState } from '@/state';
import { Trans } from '@lingui/react/macro';
import { ArrowDownIcon, ArrowUpIcon, InfoIcon } from 'lucide-react';
import { PropsWithChildren, useEffect, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { cn } from '@/lib/utils';
import { Assets, CatPresence } from './Assets';

export interface OfferCardProps {
  summary: OfferSummary;
  status?: string;
  creation_date?: string;
}

export function OfferCard({
  summary,
  status,
  creation_date,
  children,
}: PropsWithChildren<OfferCardProps>) {
  const walletState = useWalletState();
  // State to track which CATs are present in the wallet
  const [catPresence, setCatPresence] = useState<CatPresence>({});

  // Check if CATs in the receiving section are present in the wallet
  useEffect(() => {
    const checkCatPresence = async () => {
      const presence: CatPresence = {};

      // Check each CAT in the maker section (receiving)
      for (const assetId of Object.keys(summary.maker.cats)) {
        try {
          const response = await commands.getCat({ asset_id: assetId });
          presence[assetId] = !!response.cat; // true if cat exists, false otherwise
        } catch (error) {
          console.error(`Error checking CAT presence for ${assetId}:`, error);
          presence[assetId] = false;
        }
      }

      setCatPresence(presence);
    };

    checkCatPresence();
  }, [summary.maker.cats]);

  const getStatusStyles = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active':
        return 'text-green-600 dark:text-green-500';
      case 'completed':
        return 'text-blue-600 dark:text-blue-500';
      case 'cancelled':
        return 'text-amber-600 dark:text-amber-500';
      case 'expired':
        return 'text-red-600 dark:text-red-500';
      default:
        return '';
    }
  };

  const getStatusDotColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active':
        return 'bg-green-500';
      case 'completed':
        return 'bg-blue-500';
      case 'cancelled':
        return 'bg-amber-500';
      case 'expired':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  return (
    <div className='flex flex-col gap-4 max-w-screen-lg'>
      <Card>
        <CardHeader className='pb-2'>
          <CardTitle className='text-lg font-medium flex items-center'>
            <InfoIcon className='mr-2 h-5 w-5' />
            <Trans>Offer Details</Trans>
          </CardTitle>
        </CardHeader>
        <CardContent className='flex flex-col gap-4'>
          <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
            {status && (
              <div className='space-y-1.5'>
                <div className='text-sm font-medium text-muted-foreground'>
                  <Trans>Status</Trans>
                </div>
                <div className='flex items-center gap-2'>
                  <div
                    className={cn(
                      'w-2.5 h-2.5 rounded-full',
                      getStatusDotColor(status),
                    )}
                  />
                  <div className={cn('font-medium', getStatusStyles(status))}>
                    {status === 'active'
                      ? 'Pending'
                      : status === 'completed'
                        ? 'Taken'
                        : status === 'cancelled'
                          ? 'Cancelled'
                          : 'Expired'}
                  </div>
                </div>
              </div>
            )}

            {creation_date && (
              <div className='space-y-1.5'>
                <div className='text-sm font-medium text-muted-foreground'>
                  <Trans>Created</Trans>
                </div>
                <div>{new Date(creation_date).toLocaleString()}</div>
              </div>
            )}

            <div className='space-y-1.5'>
              <div className='text-sm font-medium text-muted-foreground'>
                <Trans>Maker Fee</Trans>
              </div>
              <div>
                <NumberFormat
                  value={fromMojos(summary.fee, walletState.sync.unit.decimals)}
                  minimumFractionDigits={0}
                  maximumFractionDigits={walletState.sync.unit.decimals}
                />{' '}
                {walletState.sync.unit.ticker}
              </div>
            </div>

            {(summary.expiration_timestamp || summary.expiration_height) && (
              <div className='space-y-1.5'>
                <div className='text-sm font-medium text-muted-foreground'>
                  <Trans>Expires</Trans>
                </div>
                {summary.expiration_timestamp && (
                  <div className='text-sm'>
                    {formatTimestamp(summary.expiration_timestamp)}
                  </div>
                )}
                {summary.expiration_height && (
                  <div className='text-sm text-muted-foreground'>
                    <Trans>Block:</Trans> {summary.expiration_height}
                  </div>
                )}
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <div className='grid grid-cols-1 lg:grid-cols-2 gap-4'>
        <Card>
          <CardHeader className='pb-2'>
            <CardTitle className='text-lg font-medium flex items-center'>
              <ArrowUpIcon className='mr-2 h-5 w-5' />
              <Trans>Sending</Trans>
            </CardTitle>
            <p className='text-sm text-muted-foreground'>
              <Trans>The assets you have to pay to fulfill the offer.</Trans>
            </p>
          </CardHeader>
          <CardContent className='flex flex-col gap-3'>
            <Assets assets={summary.taker} />
            {children}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className='pb-2'>
            <CardTitle className='text-lg font-medium flex items-center'>
              <ArrowDownIcon className='mr-2 h-5 w-5' />
              <Trans>Receiving</Trans>
            </CardTitle>
            <p className='text-sm text-muted-foreground'>
              <Trans>The assets being given to you in the offer.</Trans>
            </p>
          </CardHeader>
          <CardContent className='flex flex-col gap-3'>
            <Assets
              assets={
                summary?.maker ?? {
                  xch: { amount: '0', royalty: '0' },
                  cats: {},
                  nfts: {},
                }
              }
              catPresence={catPresence}
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
