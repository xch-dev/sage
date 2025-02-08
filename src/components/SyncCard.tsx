import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useWalletState } from '@/state';
import { Trans } from '@lingui/react/macro';
import { CircleDollarSign, Network, FlameIcon } from 'lucide-react';
import { FormattedAddressBox } from './FormattedAddressBox';
import { t } from '@lingui/core/macro';
import { NumberFormat } from './NumberFormat';

export function SyncCard() {
  const walletState = useWalletState();
  const sync = walletState.sync;

  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}><Trans>Status</Trans></CardTitle>
      </CardHeader>
      <CardContent className='grid gap-4'>

        <div className='flex items-center gap-4' role="status">
          <CircleDollarSign className='h-5 w-5 text-muted-foreground' aria-hidden="true"/>
          <div className='flex-1'>
            <div className='text-sm font-medium' id="coins-label"><Trans>Coins</Trans></div>
            <div className='text-sm text-muted-foreground' aria-labelledby="coins-label">
              <Trans>{sync.synced_coins} of {sync.total_coins} synced</Trans>
            </div>
          </div>
        </div>

        <div className='flex items-center gap-4' role="status">
          <Network className='h-5 w-5 text-muted-foreground' aria-hidden="true" />
          <div className='flex-1'>
            <div className='text-sm font-medium' id="height-label"><Trans>Peer Max Height</Trans></div>
            <div className='text-sm text-muted-foreground' aria-labelledby="height-label">
              <NumberFormat value={sync.peer_max_height} minimumFractionDigits={0} maximumFractionDigits={0} />
            </div>
          </div>
        </div>
        <div className='flex items-center gap-4' role="status">
          <FlameIcon className='h-5 w-5 text-muted-foreground' aria-hidden="true" />
          <div className='flex-1'>
            <div className='text-sm font-medium' id="burn-address-label"><Trans>Burn Address</Trans></div>
            <FormattedAddressBox 
              className='text-sm text-muted-foreground'
              address={sync.burn_address}
              title={t`Burn Address`}
              labelId="burn-address-label"
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );
} 