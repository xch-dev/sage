import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useWalletState } from '@/state';
import { Trans } from '@lingui/react/macro';
import { CircleDollarSign, Network, FlameIcon } from 'lucide-react';
import { FormattedAddressBox } from './FormattedAddressBox';
import { t } from '@lingui/core/macro';

export function SyncCard() {
  const walletState = useWalletState();
  const sync = walletState.sync;

  return (
    <Card>
      <CardHeader>
        <CardTitle><Trans>Sync Status</Trans></CardTitle>
      </CardHeader>
      <CardContent className='grid gap-4'>

        <div className='flex items-center gap-4'>
          <CircleDollarSign className='h-5 w-5 text-muted-foreground' />
          <div className='flex-1'>
            <div className='text-sm font-medium'><Trans>Coins</Trans></div>
            <div className='text-sm text-muted-foreground'>
              <Trans>{sync.synced_coins} of {sync.total_coins} synced</Trans>
            </div>
          </div>
        </div>

        <div className='flex items-center gap-4'>
          <Network className='h-5 w-5 text-muted-foreground' />
          <div className='flex-1'>
            <div className='text-sm font-medium'><Trans>Peer Max Height</Trans></div>
            <div className='text-sm text-muted-foreground'>
              {sync.peer_max_height}
            </div>
          </div>
        </div>
        <div className='flex items-center gap-4'>
          <FlameIcon className='h-5 w-5 text-muted-foreground' />
          <div className='flex-1'>
            <div className='text-sm font-medium'><Trans>Burn Address</Trans></div>
            <FormattedAddressBox 
              className='text-sm text-muted-foreground'
              address={sync.burn_address}
              title={t`Burn Address`}
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );
} 